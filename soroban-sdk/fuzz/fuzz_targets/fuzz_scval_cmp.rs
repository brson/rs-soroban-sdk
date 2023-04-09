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
    scval_proto_1: <ScVal as SorobanArbitrary>::Prototype,
    scval_proto_2: <ScVal as SorobanArbitrary>::Prototype,
}

fuzz_target!(|input: Test| {
    let Test {
        scval_proto_1,
        scval_proto_2,
    } = input;

    let env = &Env::default();
    let rawval_1 = RawVal::try_from_val(env, &scval_proto_1).expect("RawVal");
    let rawval_2 = RawVal::try_from_val(env, &scval_proto_2).expect("RawVal");

    let scval_cmp = Ord::cmp(&scval_proto_1, &scval_proto_2);

});
