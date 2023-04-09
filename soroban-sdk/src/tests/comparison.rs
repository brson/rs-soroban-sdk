// todo convert to proptests

use crate::{Env, RawVal};
use crate::Vec;
use crate::{FromVal, TryFromVal};
use crate::xdr::ScVal;
use soroban_env_host::Compare;

#[test]
fn vec_unequal_lengths() {
    let env = &Env::default();
    let budget = env.budget().0;

    let v1_vec: Vec<u32> = Vec::from_slice(env, &[0]);
    let v2_vec: Vec<u32> = Vec::from_slice(env, &[0, 1]);

    let v1_rawval = RawVal::from_val(env, &v1_vec);
    let v2_rawval = RawVal::from_val(env, &v2_vec);

    let v1_scval = ScVal::try_from_val(env, &v1_rawval).unwrap();
    let v2_scval = ScVal::try_from_val(env, &v2_rawval).unwrap();

    let cmp_env = env.compare(&v1_rawval, &v2_rawval).unwrap();
    let cmp_budget = budget.compare(&v1_scval, &v2_scval).unwrap();

    assert_eq!(cmp_env, cmp_budget);
}
