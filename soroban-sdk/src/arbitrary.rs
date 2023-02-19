//! Support for implementing the fuzzing `Arbitrary` crate for Soroban
//! contracts.
//!
//! todo
//!
//! This module is perhaps over-organized in a way that is convenient for
//! current development.
//!
//! # TODO
//!
//! - remove arbitrary feature
//! - BitSet, Static, Status, Symbol shouldn't be reexported here
//! - make sure both into_val and from_val work as expected
//! - implement TryFromVal instead of IntoVal
//!   - also in sdk-macros
//!   - https://github.com/stellar/rs-soroban-env/pull/628
//!   - https://github.com/stellar/rs-soroban-sdk/pull/824
//! - Object - does this type appear in contracts?
//! - invalid types
//!   - vecs/maps with incorrect elements
//!   - statuses with incorrect codes
//!   - unallocated objects
//!   - rust types backed by mistyped rawvals?
//! - reused objects
//! - vecs/maps that contain themselves

#![cfg(feature = "arbitrary")]

// These two rexports are used by #[contracttype] to derive `SorobanArbitrary`.
pub use arbitrary;
pub use std;

pub use api::*;
pub use fuzz_test_helpers::*;
pub use objects::*;
pub use scalars::*;
pub use simple::*;

/// The traits that must be implemented on Soroban types to support fuzzing.
///
/// These allow for ergonomic conversion from a randomly-generated "prototype"
/// that implements `Arbitrary` into `Env`-"hosted" values that are paired with an
/// `Env`.
///
/// These traits are intended to be easy to automatically derive.
mod api {
    use crate::Env;
    use crate::RawVal;
    use crate::{IntoVal, TryFromVal};

    /// An `Env`-hosted contract value that can be randomly generated.
    ///
    /// Types that implement `SorabanArbitrary` have an associated "prototype"
    /// type that implements `Arbitrary`.
    ///
    /// This exists partly that the prototype can be named like
    ///
    /// ```ignore
    /// fuzz_target!(|input: <Bytes as SorobanArbitrary>::Arbitrary| {
    ///   ...
    /// });
    /// ```
    ///
    /// This also makes derivation of `SorobanArbitrary` for custom types easier
    /// since we depend on all fields also implementing `SorobanArbitrary`.
    pub trait SorobanArbitrary:
        TryFromVal<Env, Self::Prototype> + IntoVal<Env, RawVal> + TryFromVal<Env, RawVal>
    {
        type Prototype;
    }
}

/// New implementations of `IntoVal` on existing types.
///
/// These are only needed to support `soroban_sdk::arbitrary`.
/// They are trivial and could be seen as a wart.
///
/// FIXME: This should be enabled even when the "arbitrary" feature is not
/// enabled to avoid type inference inconsistencies.
mod extra_into_vals {
    use crate::env::internal::BitSet;
    use crate::env::internal::Static;
    use crate::ConversionError;
    use crate::{Env, TryFromVal};
    use crate::{Status, Symbol};

    impl TryFromVal<Env, u32> for u32 {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &u32) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, i32> for i32 {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &i32) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, u64> for u64 {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &u64) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, i64> for i64 {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &i64) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, u128> for u128 {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &u128) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, i128> for i128 {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &i128) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, Symbol> for Symbol {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &Symbol) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, BitSet> for BitSet {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &BitSet) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, Static> for Static {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &Static) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }

    impl TryFromVal<Env, Status> for Status {
        type Error = ConversionError;
        fn try_from_val(_env: &Env, v: &Status) -> Result<Self, Self::Error> {
            Ok(*v)
        }
    }
}

/// Implementations of `soroban_sdk::arbitrary::api` for Rust scalar types.
///
/// These types
///
/// - do not have a distinct `Arbitrary` prototype,
///   i.e. they use themselves as the `SorobanArbitrary::Prototype` type,
/// - implement `Arbitrary` in the `arbitrary` crate,
/// - trivially implement `IntoVal<Env, SorobanArbitraryPrototype::Into>`,
///
/// Examples:
///
/// - `u32`
mod scalars {
    use crate::arbitrary::api::*;

    impl SorobanArbitrary for u32 {
        type Prototype = u32;
    }

    impl SorobanArbitrary for i32 {
        type Prototype = i32;
    }

    impl SorobanArbitrary for u64 {
        type Prototype = u64;
    }

    impl SorobanArbitrary for i64 {
        type Prototype = i64;
    }

    impl SorobanArbitrary for u128 {
        type Prototype = u128;
    }

    impl SorobanArbitrary for i128 {
        type Prototype = i128;
    }
}

/// Implementations of `soroban_sdk::arbitrary::api` for Soroban types that do not
/// need access to the Soroban host environment.
///
/// These types
///
/// - do not have a distinct `Arbitrary` prototype,
///   i.e. they use themselves as the `SorobanArbitrary::Prototype` type,
/// - implement `Arbitrary` in the `soroban-env-common` crate,
/// - trivially implement `IntoVal<Env, SorobanArbitraryPrototype::Into>`,
///
/// Examples:
///
/// - `Symbol`
mod simple {
    use crate::arbitrary::api::*;
    pub use crate::env::internal::BitSet;
    pub use crate::env::internal::Static;
    pub use crate::{Status, Symbol};

    impl SorobanArbitrary for Symbol {
        type Prototype = Symbol;
    }

    impl SorobanArbitrary for BitSet {
        type Prototype = BitSet;
    }

    impl SorobanArbitrary for Static {
        type Prototype = Static;
    }

    impl SorobanArbitrary for Status {
        type Prototype = Status;
    }
}

/// Implementations of `soroban_sdk::arbitrary::api` for Soroban types that do
/// need access to the Soroban host environment.
///
/// These types
///
/// - have a distinct `Arbitrary` prototype that derives `Arbitrary`,
/// - require an `Env` to implement `IntoVal<Env, SorobanArbitraryPrototype::Into>`,
///
/// Examples:
///
/// - `Vec`
mod objects {
    use arbitrary::Arbitrary;

    use crate::arbitrary::api::*;
    use crate::ConversionError;
    use crate::{Env, IntoVal, TryFromVal};

    use crate::{Address, Bytes, BytesN, Map, Vec};

    use std::vec::Vec as RustVec;

    //////////////////////////////////

    #[derive(Arbitrary, Debug)]
    pub struct ArbitraryBytes {
        vec: RustVec<u8>,
    }

    impl SorobanArbitrary for Bytes {
        type Prototype = ArbitraryBytes;
    }

    impl TryFromVal<Env, ArbitraryBytes> for Bytes {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryBytes) -> Result<Self, Self::Error> {
            Self::try_from_val(env, &v.vec.as_slice())
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug)]
    pub struct ArbitraryBytesN<const N: usize> {
        array: [u8; N],
    }

    impl<const N: usize> SorobanArbitrary for BytesN<N> {
        type Prototype = ArbitraryBytesN<N>;
    }

    impl<const N: usize> TryFromVal<Env, ArbitraryBytesN<N>> for BytesN<N> {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryBytesN<N>) -> Result<Self, Self::Error> {
            Self::try_from_val(env, &v.array)
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug)]
    pub struct ArbitraryVec<T> {
        vec: RustVec<T>,
    }

    impl<T> SorobanArbitrary for Vec<T>
    where
        T: SorobanArbitrary,
    {
        type Prototype = ArbitraryVec<T::Prototype>;
    }

    impl<T> TryFromVal<Env, ArbitraryVec<T::Prototype>> for Vec<T>
    where
        T: SorobanArbitrary,
    {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryVec<T::Prototype>) -> Result<Self, Self::Error> {
            let mut buf: Vec<T> = Vec::new(env);
            for item in v.vec.iter() {
                buf.push_back(item.into_val(env));
            }
            Ok(buf)
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug)]
    pub struct ArbitraryMap<K, V> {
        map: RustVec<(K, V)>,
    }

    impl<K, V> SorobanArbitrary for Map<K, V>
    where
        K: SorobanArbitrary,
        V: SorobanArbitrary,
    {
        type Prototype = ArbitraryMap<K::Prototype, V::Prototype>;
    }

    impl<K, V> TryFromVal<Env, ArbitraryMap<K::Prototype, V::Prototype>> for Map<K, V>
    where
        K: SorobanArbitrary,
        V: SorobanArbitrary,
    {
        type Error = ConversionError;
        fn try_from_val(
            env: &Env,
            v: &ArbitraryMap<K::Prototype, V::Prototype>,
        ) -> Result<Self, Self::Error> {
            let mut map: Map<K, V> = Map::new(env);
            for (k, v) in v.map.iter() {
                map.set(k.into_val(env), v.into_val(env));
            }
            Ok(map)
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug)]
    pub struct ArbitraryAddress {
        inner: [u8; 32],
    }

    impl SorobanArbitrary for Address {
        type Prototype = ArbitraryAddress;
    }

    impl TryFromVal<Env, ArbitraryAddress> for Address {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryAddress) -> Result<Self, Self::Error> {
            use crate::env::xdr::{Hash, ScAddress, ScObject, ScVal};

            let sc_addr =
                ScVal::Object(Some(ScObject::Address(ScAddress::Contract(Hash(v.inner)))));
            Ok(sc_addr.into_val(env))
        }
    }
}

mod composite {
    use arbitrary::Arbitrary;

    use crate::arbitrary::api::*;
    use crate::ConversionError;
    use crate::{Env, IntoVal, TryFromVal};

    use super::objects::*;
    use super::simple::*;
    use crate::{Bytes, RawVal};

    #[derive(Arbitrary, Debug)]
    pub enum ArbitraryRawVal {
        U32(u32),
        I32(i32),
        U64(u64),
        I64(i64),
        U128(u128),
        I128(i128),
        Symbol(Symbol),
        BitSet(BitSet),
        Static(Static),
        Status(Status),
        Bytes(ArbitraryBytes),
        // Vec, todo
        // Map, todo
        // todo AccountId(ArbitraryAccountId),
        //Address(<Address as SorobanArbitrary>::Prototype),
    }

    impl SorobanArbitrary for RawVal {
        type Prototype = ArbitraryRawVal;
    }

    impl TryFromVal<Env, ArbitraryRawVal> for RawVal {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryRawVal) -> Result<Self, Self::Error> {
            Ok(match v {
                ArbitraryRawVal::U32(v) => v.into_val(env),
                ArbitraryRawVal::I32(v) => v.into_val(env),
                ArbitraryRawVal::U64(v) => v.into_val(env),
                ArbitraryRawVal::I64(v) => v.into_val(env),
                ArbitraryRawVal::U128(v) => v.into_val(env),
                ArbitraryRawVal::I128(v) => v.into_val(env),
                ArbitraryRawVal::Symbol(v) => v.into_val(env),
                ArbitraryRawVal::BitSet(v) => v.into_val(env),
                ArbitraryRawVal::Static(v) => v.into_val(env),
                ArbitraryRawVal::Status(v) => v.into_val(env),
                ArbitraryRawVal::Bytes(v) => {
                    let v: Bytes = v.into_val(env);
                    v.into_val(env)
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arbitrary::*;
    use crate::{Bytes, BytesN, Map, RawVal, Symbol, Vec};
    use crate::{Env, IntoVal};
    use arbitrary::{Arbitrary, Unstructured};
    use rand::RngCore;

    fn run_test<T>()
    where
        T: SorobanArbitrary,
        T::Prototype: for<'a> Arbitrary<'a>,
    {
        let env = Env::default();
        let mut rng = rand::thread_rng();
        let mut rng_data = [0u8; 64];

        for _ in 0..100 {
            rng.fill_bytes(&mut rng_data);
            let mut unstructured = Unstructured::new(&rng_data);
            let input = T::Prototype::arbitrary(&mut unstructured).expect("SorobanArbitrary");
            let _val: T = input.into_val(&env);
        }
    }

    #[test]
    fn test_u32() {
        run_test::<u32>()
    }

    #[test]
    fn test_i32() {
        run_test::<i32>()
    }

    #[test]
    fn test_u64() {
        run_test::<u64>()
    }

    #[test]
    fn test_i64() {
        run_test::<i64>()
    }

    #[test]
    fn test_u128() {
        run_test::<u128>()
    }

    #[test]
    fn test_i128() {
        run_test::<i128>()
    }

    #[test]
    fn test_bytes() {
        run_test::<Bytes>()
    }

    #[test]
    fn test_bytes_n() {
        run_test::<BytesN<64>>()
    }

    #[test]
    fn test_symbol() {
        run_test::<Symbol>()
    }

    #[test]
    fn test_vec_u32() {
        run_test::<Vec<u32>>()
    }

    #[test]
    fn test_vec_i32() {
        run_test::<Vec<i32>>()
    }

    #[test]
    fn test_vec_bytes() {
        run_test::<Vec<Bytes>>()
    }

    #[test]
    fn test_vec_bytes_n() {
        run_test::<Vec<BytesN<32>>>()
    }

    #[test]
    fn test_vec_vec_bytes() {
        run_test::<Vec<Vec<Bytes>>>()
    }

    #[test]
    fn test_vec_symbol() {
        run_test::<Vec<Symbol>>()
    }

    #[test]
    fn test_map_u32() {
        run_test::<Map<u32, Vec<u32>>>()
    }

    #[test]
    fn test_map_i32() {
        run_test::<Map<i32, Vec<i32>>>()
    }

    #[test]
    fn test_map_bytes() {
        run_test::<Map<Bytes, Bytes>>()
    }

    #[test]
    fn test_map_bytes_n() {
        run_test::<Map<BytesN<32>, Bytes>>()
    }

    #[test]
    fn test_map_vec() {
        run_test::<Map<Vec<Bytes>, Vec<Bytes>>>()
    }

    #[test]
    fn test_map_symbol() {
        run_test::<Map<Symbol, Bytes>>()
    }

    #[test]
    fn test_raw_val() {
        run_test::<RawVal>()
    }

    mod user_defined_types {
        use crate as soroban_sdk;
        use crate::arbitrary::tests::run_test;
        use crate::{Bytes, BytesN, Map, Symbol, Vec};
        use soroban_sdk::contracttype;

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivStruct {
            symbol: Symbol,
            count_u: u32,
            count_i: i32,
            bytes_n: BytesN<32>,
            vec: Vec<Bytes>,
            map: Map<Bytes, Vec<Symbol>>,
        }

        #[test]
        fn test_user_defined_priv_struct() {
            run_test::<PrivStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivStructPubFields {
            pub symbol: Symbol,
            pub count_u: u32,
            pub count_i: i32,
            pub bytes_n: BytesN<32>,
            pub vec: Vec<Bytes>,
            pub map: Map<Bytes, Vec<Symbol>>,
        }

        #[test]
        fn test_user_defined_priv_struct_pub_fields() {
            run_test::<PrivStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubStruct {
            symbol: Symbol,
            count_u: u32,
            count_i: i32,
            bytes_n: BytesN<32>,
            vec: Vec<Bytes>,
            map: Map<Bytes, Vec<Symbol>>,
        }

        #[test]
        fn test_user_defined_pub_struct() {
            run_test::<PubStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubStructPubFields {
            pub symbol: Symbol,
            pub count_u: u32,
            pub count_i: i32,
            pub bytes_n: BytesN<32>,
            pub vec: Vec<Bytes>,
            pub map: Map<Bytes, Vec<Symbol>>,
        }

        #[test]
        fn test_user_defined_pubstruct_pub_fields() {
            run_test::<PubStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivTupleStruct(
            Symbol,
            u32,
            i32,
            BytesN<32>,
            Vec<Bytes>,
            Map<Bytes, Vec<Symbol>>,
        );

        #[test]
        fn test_user_defined_priv_tuple_struct() {
            run_test::<PrivTupleStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivTupleStructPubFields(
            pub Symbol,
            pub u32,
            pub i32,
            pub BytesN<32>,
            pub Vec<Bytes>,
            pub Map<Bytes, Vec<Symbol>>,
        );

        #[test]
        fn test_user_defined_priv_tuple_struct_pub_fields() {
            run_test::<PrivTupleStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubTupleStruct(
            Symbol,
            u32,
            i32,
            BytesN<32>,
            Vec<Bytes>,
            Map<Bytes, Vec<Symbol>>,
        );

        #[test]
        fn test_user_defined_pub_tuple_struct() {
            run_test::<PubTupleStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubTupleStructPubFields(
            pub Symbol,
            pub u32,
            pub i32,
            pub BytesN<32>,
            pub Vec<Bytes>,
            pub Map<Bytes, Vec<Symbol>>,
        );

        #[test]
        fn test_user_defined_pub_tuple_struct_pub_fields() {
            run_test::<PubTupleStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) struct PubCrateStruct(u32);

        #[test]
        fn test_user_defined_pub_crate_struct() {
            run_test::<PubCrateStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        enum PrivEnum {
            A(u32),
            // todo not supported by contracttype
            // Aa(u32, u32),
            // todo not supported by contracttype
            /*B {
                a: BytesN<32>,
                b: Vec<Bytes>,
            },*/
            C,
            D,
        }

        #[test]
        fn test_user_defined_priv_enum() {
            run_test::<PrivEnum>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum PubEnum {
            A(u32),
            C,
            D,
        }

        #[test]
        fn test_user_defined_pub_enum() {
            run_test::<PubEnum>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub(crate) enum PubCrateEnum {
            A(u32),
            C,
            D,
        }

        #[test]
        fn test_user_defined_pub_crate_enum() {
            run_test::<PubCrateEnum>();
        }

        #[contracttype]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        enum PrivEnumInt {
            A = 1,
            C = 2,
            D = 3,
        }

        #[test]
        fn test_user_defined_priv_enum_int() {
            run_test::<PrivEnumInt>();
        }

        #[contracttype]
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub enum PubEnumInt {
            A = 1,
            C = 2,
            D = 3,
        }

        #[test]
        fn test_user_defined_pub_enum_int() {
            run_test::<PubEnumInt>();
        }
    }
}

mod fuzz_test_helpers {
    use super::testutils::call_with_suppressed_panic_hook;

    pub fn fuzz_catch_panic<F, R>(f: F) -> std::thread::Result<R>
    where
        F: FnOnce() -> R,
    {
        call_with_suppressed_panic_hook(std::panic::AssertUnwindSafe(f))
    }
}

// FIXME duplicated from soroban-env-host
mod testutils {
    use std::cell::Cell;
    use std::panic::{catch_unwind, set_hook, take_hook, UnwindSafe};
    use std::sync::Once;

    /// Catch panics while suppressing the default panic hook that prints to the
    /// console.
    ///
    /// For the purposes of test reporting we don't want every panicking (but
    /// caught) contract call to print to the console. This requires overriding
    /// the panic hook, a global resource. This is an awkward thing to do with
    /// tests running in parallel.
    ///
    /// This function lazily performs a one-time wrapping of the existing panic
    /// hook. It then uses a thread local variable to track contract call depth.
    /// If a panick occurs during a contract call the original hook is not
    /// called, otherwise it is called.
    pub fn call_with_suppressed_panic_hook<C, R>(closure: C) -> std::thread::Result<R>
    where
        C: FnOnce() -> R + UnwindSafe,
    {
        thread_local! {
            static TEST_CONTRACT_CALL_COUNT: Cell<u64> = Cell::new(0);
        }

        static WRAP_PANIC_HOOK: Once = Once::new();

        WRAP_PANIC_HOOK.call_once(|| {
            let existing_panic_hook = take_hook();
            set_hook(Box::new(move |info| {
                let calling_test_contract = TEST_CONTRACT_CALL_COUNT.with(|c| c.get() != 0);
                if !calling_test_contract {
                    existing_panic_hook(info)
                }
            }))
        });

        TEST_CONTRACT_CALL_COUNT.with(|c| {
            let old_count = c.get();
            let new_count = old_count.checked_add(1).expect("overflow");
            c.set(new_count);
        });

        let res = catch_unwind(closure);

        TEST_CONTRACT_CALL_COUNT.with(|c| {
            let old_count = c.get();
            let new_count = old_count.checked_sub(1).expect("overflow");
            c.set(new_count);
        });

        res
    }
}
