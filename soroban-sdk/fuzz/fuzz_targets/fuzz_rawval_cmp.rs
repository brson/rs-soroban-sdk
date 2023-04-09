// todo
//
// - expand arbitrary impls for RawVal
// - add missing SorobanArbitrary impls for other types
// - make fuzz_scval_cmp - like this but start from ScVal
// - decide what to do about i128 - scval comparison doesn't treat i128 as numbers
//   - Compare<u128/i128> for Budget uses ScVal comparison
// - Tags are not ordered the same as ScVal variants
//
// tests needed
//
// - object comparisions where object types are not the same
//   - proptest?
//   - maybe not needed
// - Compare<ScVal> vs Compare<RawVal> for vecs of unequal length
// - Compare<ScVal> vs Compare<RawVal> for maps of unequal length

#![allow(unreachable_code)]
#![allow(unused)]
#![no_main]

use arbitrary::Arbitrary;
use core::cmp::Ordering;
use libfuzzer_sys::fuzz_target;
use soroban_sdk::arbitrary::{arbitrary, SorobanArbitrary};
use soroban_sdk::testutils::Compare;
use soroban_sdk::xdr::ScVal;
use soroban_sdk::Env;
use soroban_sdk::RawVal;
use soroban_sdk::{FromVal, TryFromVal};

#[derive(Arbitrary, Debug)]
struct Test {
    rawval_proto_1: <RawVal as SorobanArbitrary>::Prototype,
    rawval_proto_2: <RawVal as SorobanArbitrary>::Prototype,
}

fuzz_target!(|input: Test| {
    let Test {
        rawval_proto_1,
        rawval_proto_2,
    } = input;

    let env = &Env::default();
    let budget = env.budget().0;

    let rawval_1 = RawVal::from_val(env, &rawval_proto_1);
    let rawval_2 = RawVal::from_val(env, &rawval_proto_2);

    let exception_i128;
    let exception_u128;

    // Handle some exceptions
    {
        use soroban_sdk::testutils::I128Val;
        match (I128Val::try_from(rawval_1), I128Val::try_from(rawval_2)) {
            (Ok(_), Ok(_)) => {
                // todo I128 ScVal comparison is not numeric
                exception_i128 = true;
            }
            _ => {
                exception_i128 = false
            }
        }
        use soroban_sdk::testutils::U128Val;
        match (U128Val::try_from(rawval_1), U128Val::try_from(rawval_2)) {
            (Ok(_), Ok(_)) => {
                // todo U128 ScVal comparison is not numeric
                exception_u128 = true;
            }
            _ => {
                exception_u128 = false
            }
        }
    }

    let (scval_1, scval_2) = {
        let scval_1 = ScVal::try_from_val(env, &rawval_1);
        let scval_2 = ScVal::try_from_val(env, &rawval_2);

        let scval_1 = match scval_1 {
            Ok(scval_1) => scval_1,
            Err(e) => {
                return; // todo
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
                return; // todo
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

        // We only care about whether the values are equal
        //let rawval_cmp_is_eq = rawval_cmp == Ordering::Equal;
        let scval_cmp_is_eq = scval_cmp == Ordering::Equal;

        //if rawval_cmp_is_eq != scval_cmp_is_eq {
        if rawval_cmp != scval_cmp {
            if !(exception_i128 || exception_u128) {
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
        }

        let scval_cmp_partial = PartialOrd::partial_cmp(&scval_1, &scval_2);
        let scval_cmp_partial_is_eq = scval_cmp_partial == Some(Ordering::Equal);

        //assert_eq!(scval_cmp_is_eq, scval_cmp_partial_is_eq);
        assert_eq!(Some(scval_cmp), scval_cmp_partial);

        let scval_partial_eq = PartialEq::eq(&scval_1, &scval_2);
        assert_eq!(scval_cmp_is_eq, scval_partial_eq);

        // Compare<ScVal> for Budget
        let scval_budget_cmp = budget.compare(&scval_1, &scval_2);
        let scval_budget_cmp = scval_budget_cmp.expect("cmp");
        //let scval_budget_cmp_is_eq = scval_budget_cmp == Ordering::Equal;
        //if rawval_cmp_is_eq != scval_budget_cmp_is_eq {
        if rawval_cmp != scval_budget_cmp {
            if !(exception_i128 || exception_u128) {
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

        let rawval_before_after_1 = env.compare(&rawval_1, &rawval_after_1);

        let rawval_before_after_1 = match rawval_before_after_1 {
            Ok(rawval_before_after_1) => rawval_before_after_1,
            Err(e) => {
                panic!(
                    "couldn't compare rawvals:\n\
                     {rawval_1:#?},\n\
                     {rawval_after_1:#?},\n\
                     {e:#?}"
                );
            }
        };

        assert_eq!(rawval_before_after_1, Ordering::Equal);

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

        let rawval_before_after_2 = env.compare(&rawval_2, &rawval_after_2);

        let rawval_before_after_2 = match rawval_before_after_2 {
            Ok(rawval_before_after_2) => rawval_before_after_2,
            Err(e) => {
                panic!(
                    "couldn't compare rawvals:\n\
                     {rawval_2:#?},\n\
                     {rawval_after_2:#?},\n\
                     {e:#?}"
                );
            }
        };

        assert_eq!(rawval_before_after_2, Ordering::Equal);
    }
});
