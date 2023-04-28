#![cfg(feature = "testutils")]

use crate::arbitrary::SorobanArbitrary;
use crate::xdr::ScVal;
use crate::Env;
use crate::RawVal;
use crate::{FromVal, TryFromVal};
use crate::{Map, Vec};
use core::cmp::Ordering;
use proptest::prelude::*;
use proptest_arbitrary_interop::arb;
use soroban_env_host::{Compare, Tag};

fn scval_conversion_might_fail(env: &Env, rawval: RawVal) -> bool {
    if rawval.get_tag() == Tag::Status {
        // Some statuses can't be serialized.
        true
    } else {
        if let Ok(v) = Vec::<RawVal>::try_from_val(env, &rawval) {
            for rawval in v {
                if let Ok(rawval) = rawval {
                    if scval_conversion_might_fail(env, rawval) {
                        return true;
                    }
                }
            }
            false
        } else if let Ok(m) = Map::<RawVal, RawVal>::try_from_val(env, &rawval) {
            // NB: MapIter will not iterate over Status keys!
            for key in m.keys() {
                if let Ok(key) = key {
                    if scval_conversion_might_fail(env, key) {
                        return true;
                    }
                }
            }
            for value in m.values() {
                if let Ok(value) = value {
                    if scval_conversion_might_fail(env, value) {
                        return true;
                    }
                }
            }
            false
        } else {
            false
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100000))]

    #[test]
    fn test(
        rawval_proto_1 in arb::<<RawVal as SorobanArbitrary>::Prototype>(),
        rawval_proto_2 in arb::<<RawVal as SorobanArbitrary>::Prototype>(),
    ) {
        let env = &Env::default();
        let budget = env.budget().0;

        let rawval_1 = RawVal::from_val(env, &rawval_proto_1);
        let rawval_2 = RawVal::from_val(env, &rawval_proto_2);

        let (scval_1, scval_2) = {
            let scval_1 = ScVal::try_from_val(env, &rawval_1);
            let scval_2 = ScVal::try_from_val(env, &rawval_2);

            let scval_1 = match scval_1 {
                Ok(scval_1) => scval_1,
                Err(e) => {
                    if scval_conversion_might_fail(env, rawval_1) {
                        return Ok(());
                    }

                    panic!(
                        "couldn't convert rawval to scval:\n\
                         {rawval_1:?},\n\
                         {e:#?}"
                    );
                }
            };

            let scval_2 = match scval_2 {
                Ok(scval_2) => scval_2,
                Err(e) => {
                    if scval_conversion_might_fail(env, rawval_2) {
                        return Ok(());
                    }

                    panic!(
                        "couldn't convert rawval to scval:\n\
                         {rawval_2:?},\n\
                         {e:#?}"
                    );
                }
            };

            (scval_1, scval_2)
        };

        // Check the comparison functions
        {
            let rawval_cmp = env.compare(&rawval_1, &rawval_2);
            let rawval_cmp = rawval_cmp.expect("cmp");
            let scval_cmp = Ord::cmp(&scval_1, &scval_2);

            let rawval_cmp_is_eq = rawval_cmp == Ordering::Equal;

            if rawval_cmp != scval_cmp {
                panic!(
                    "rawval and scval don't compare the same:\n\
                     {rawval_1:#?}\n\
                     {rawval_2:#?}\n\
                     {rawval_cmp:#?}\n\
                     {scval_1:#?}\n\
                     {scval_2:#?}\n\
                     {scval_cmp:#?}"
                );
            }

            let scval_cmp_partial = PartialOrd::partial_cmp(&scval_1, &scval_2);

            prop_assert_eq!(Some(scval_cmp), scval_cmp_partial);

            let scval_partial_eq = PartialEq::eq(&scval_1, &scval_2);
            prop_assert_eq!(rawval_cmp_is_eq, scval_partial_eq);

            // Compare<ScVal> for Budget
            let scval_budget_cmp = budget.compare(&scval_1, &scval_2);
            let scval_budget_cmp = scval_budget_cmp.expect("cmp");
            if rawval_cmp != scval_budget_cmp {
                panic!(
                    "rawval and scval (budget) don't compare the same:\n\
                     {rawval_1:#?}\n\
                     {rawval_2:#?}\n\
                     {rawval_cmp:#?}\n\
                     {scval_1:#?}\n\
                     {scval_2:#?}\n\
                     {scval_budget_cmp:#?}"
                );
            }
        }

        // Roundtrip checks
        {
            let rawval_after_1 = RawVal::try_from_val(env, &scval_1);
            let rawval_after_1 = match rawval_after_1 {
                Ok(rawval_after_1) => rawval_after_1,
                Err(e) => {
                    panic!(
                        "couldn't convert scval to rawval:\n\
                         {scval_1:?},\n\
                         {e:#?}"
                    );
                }
            };

            let rawval_cmp_before_after_1 = env.compare(&rawval_1, &rawval_after_1).expect("compare");

            prop_assert_eq!(rawval_cmp_before_after_1, Ordering::Equal);

            let rawval_after_2 = RawVal::try_from_val(env, &scval_2);
            let rawval_after_2 = match rawval_after_2 {
                Ok(rawval_after_2) => rawval_after_2,
                Err(e) => {
                    panic!(
                        "couldn't convert scval to rawval:\n\
                         {scval_2:?},\n\
                         {e:#?}"
                    );
                }
            };

            let rawval_cmp_before_after_2 = env.compare(&rawval_2, &rawval_after_2).expect("compare");

            prop_assert_eq!(rawval_cmp_before_after_2, Ordering::Equal);
        }
    }
}
