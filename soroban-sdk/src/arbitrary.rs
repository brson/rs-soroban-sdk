//! Support for fuzzing Soroban contracts with [`cargo-fuzz`].
//!
//! This module provides a pattern for generating Soroban contract types for the
//! purpose of fuzzing Soroban contracts. It is focused on implementing the
//! [`Arbitrary`] trait that `cargo-fuzz` relies on to feed fuzzers with
//! generated Rust values.
//!
//! [`cargo-fuzz`]: https://github.com/rust-fuzz/cargo-fuzz/
//! [`Arbitrary`]: ::arbitrary::Arbitrary
//!
//! This module
//!
//! - defines the [`SorobanArbitrary`] trait,
//! - defines the [`fuzz_catch_panic`] helper,
//! - reexports the [`arbitrary`] crate and the [`Arbitrary`] type.
//!
//! This module is only available when the "testutils" Cargo feature is defined.
//!
//!
//! ## About `cargo-fuzz` and `Arbitrary`
//!
//! In its basic operation `cargo-fuzz` fuzz generates raw bytes and feeds them
//! to a program-dependent fuzzer designed to exercise a program, in our case a
//! Soroban contract.
//!
//! `cargo-fuzz` programs declare their entry points with a macro:
//!
//! ```
//! # macro_rules! fuzz_target {
//! #     (|$data:ident: $dty: ty| $body:block) => { };
//! # }
//! fuzz_target!(|input: &[u8]| {
//!     // feed bytes to the program
//! });
//! ```
//!
//! More sophisticated fuzzers accept not bytes but Rust types:
//!
//! ```
//! # use arbitrary::Arbitrary;
//! # macro_rules! fuzz_target {
//! #     (|$data:ident: $dty: ty| $body:block) => { };
//! # }
//! #[derive(Arbitrary, Debug)]
//! struct FuzzDeposit {
//!     deposit_amount: i128,
//! }
//!
//! fuzz_target!(|input: FuzzDeposit| {
//!     // fuzz the program based on the input
//! });
//! ```
//!
//! Types accepted as input to `fuzz_target` must implement the `Arbitrary` trait,
//! which transforms bytes to Rust types.
//!
//!
//! ## The `SorobanArbitrary` trait
//!
//! Soroban types are managed by the host environment, and so must be created
//! from an [`Env`] value; `Arbitrary` values though must be created from
//! nothing but bytes. The [`SorobanArbitrary`] trait, implemented for all
//! Soroban contract types, exists to bridge this gap: it defines a _prototype_
//! pattern whereby the `fuzz_target` macro creates prototype values that the
//! fuzz program can convert to contract values with the standard soroban
//! conversion traits, [`FromVal`] or [`IntoVal`].
//!
//! [`Env`]: crate::Env
//! [`FromVal`]: crate::FromVal
//! [`IntoVal`]: crate::IntoVal
//!
//! The types of prototypes are identified by the associated type,
//! [`SorobanArbitrary::Prototype`]:
//!
//! ```
//! # use soroban_sdk::arbitrary::Arbitrary;
//! # use soroban_sdk::{TryFromVal, IntoVal, RawVal, Env};
//! pub trait SorobanArbitrary:
//!     TryFromVal<Env, Self::Prototype> + IntoVal<Env, RawVal> + TryFromVal<Env, RawVal>
//! {
//!     type Prototype: for <'a> Arbitrary<'a>;
//! }
//! ```
//!
//! Types that implement `SorobanArbitrary` include:
//!
//! - `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `()`, and `bool`,
//! - [`U256Val`], [`I256Val`],
//! - [`Error`],
//! - [`Bytes`], [`BytesN`], [`Vec`], [`Map`],
//! - [`Address`], [`Symbol`], [`TimepointVal`], [`DurationVal`],
//! - [`RawVal`],
//!
//! [`U256Val`]: soroban_env_host::U256Val
//! [`I256Val`]: soroban_env_host::I256Val
//! [`Error`]: crate::Error
//! [`Bytes`]: crate::Bytes
//! [`BytesN`]: crate::BytesN
//! [`Vec`]: crate::Vec
//! [`Map`]: crate::Map
//! [`Address`]: crate::Address
//! [`Symbol`]: crate::Symbol
//! [`TimepointVal`]: soroban_env_host::TimepointVal
//! [`DurationVal`]: soroban_env_host::DurationVal
//! [`RawVal`]: crate::RawVal
//!
//! All user-defined contract types, those with the [`contracttype`] attribute,
//! automatically derive `SorobanArbitrary`. Note that `SorobanArbitrary` is
//! only derived when the "testutils" Cargo feature is active. This implies
//! that, in general, to make a Soroban contract fuzzable, the contract crate
//! must define a "testutils" Cargo feature, that feature should turn on the
//! "soroban-sdk/testutils" feature, and the fuzz test, which is its own crate,
//! must turn that feature on.
//!
//! [`contracttype`]: crate::contracttype
//!
//!
//! ## Example: take a Soroban `Vec` of `Address` as fuzzer input
//!
//! ```
//! # macro_rules! fuzz_target {
//! #     (|$data:ident: $dty: ty| $body:block) => { };
//! # }
//! use soroban_sdk::{Address, Env, Vec};
//! use soroban_sdk::arbitrary::SorobanArbitrary;
//!
//! fuzz_target!(|input: <Vec<Address> as SorobanArbitrary>::Prototype| {
//!     let env = Env::default();
//!     let addresses: Vec<Address> = input.into_val(&env);
//!     // fuzz the program based on the input
//! });
//! ```
//!
//!
//! ## Example: take a custom contract type as fuzzer input
//!
//! ```
//! # macro_rules! fuzz_target {
//! #     (|$data:ident: $dty: ty| $body:block) => { };
//! # }
//! use soroban_sdk::{Address, Env, Vec};
//! use soroban_sdk::contracttype;
//! use soroban_sdk::arbitrary::{Arbitrary, SorobanArbitrary};
//! use std::vec::Vec as RustVec;
//!
//! #[derive(Arbitrary, Debug)]
//! struct TestInput {
//!     deposit_amount: i128,
//!     claim_address: <Address as SorobanArbitrary>::Prototype,
//!     time_bound: <TimeBound as SorobanArbitrary>::Prototype,
//! }
//!
//! #[contracttype]
//! pub struct TimeBound {
//!     pub kind: TimeBoundKind,
//!     pub timestamp: u64,
//! }
//!
//! #[contracttype]
//! pub enum TimeBoundKind {
//!     Before,
//!     After,
//! }
//!
//! fuzz_target!(|input: TestInput| {
//!     let env = Env::default();
//!     let claim_address: Address = input.claim_address.into_val(&env);
//!     let time_bound: TimeBound = input.time_bound.into_val(&env);
//!     // fuzz the program based on the input
//! });
//! ```

// ## TODO: Adverserial values
//
// - objects
//   - invalid references
// - SorobanArbitrary for Vec<T>
//   - elements that are not T
//   - generate the same object again
//   - elements that are Self - doesn't seem possible
// - SorobanArbitrary for Map<K, V>
//   - keys that are not K
//   - values that are not V
//   - generate the same object again
//   - keys that are Self - doesn't seem possible
//   - values that are Self - doesn't seem possible
// - SorobanArbitrary for RawVal
//   - bogus tags
//   - recursive vecs - doesn't seem possible
//   - recursive maps - doesn't seem possible
//   - deeply nested vecs
//   - deeply nested maps

#![cfg(feature = "testutils")]

/// A reexport of the `arbitrary` crate.
///
/// Used by the `contracttype` macro to derive `Arbitrary`.
pub use arbitrary;

// Used often enough in fuzz tests to want direct access to it.
pub use arbitrary::Arbitrary;

// Used by `contracttype`
#[doc(hidden)]
pub use std;

pub use api::*;
pub use fuzz_test_helpers::*;

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
    use arbitrary::Arbitrary;

    /// An `Env`-hosted contract value that can be randomly generated.
    ///
    /// Types that implement `SorabanArbitrary` have an associated "prototype"
    /// type that implements [`Arbitrary`].
    ///
    /// This exists partly that the prototype can be named like
    ///
    /// ```ignore
    /// fuzz_target!(|input: <Bytes as SorobanArbitrary>::Arbitrary| {
    ///   ...
    /// });
    /// ```
    // This also makes derivation of `SorobanArbitrary` for custom types easier
    // since we depend on all fields also implementing `SorobanArbitrary`.
    //
    // The IntoVal<Env, RawVal> + TryFromVal<Env, RawVal> bounds are to satisfy
    // the bounds of Vec and Map, so that collections of prototypes can be
    // converted to contract types.
    pub trait SorobanArbitrary:
        TryFromVal<Env, Self::Prototype> + IntoVal<Env, RawVal> + TryFromVal<Env, RawVal>
    {
        /// A type that implements [`Arbitrary`] and can be converted to this
        /// [`SorobanArbitrary`] type.
        // NB: The `Arbitrary` bound here is not necessary for the correct use of
        // `SorobanArbitrary`, but it makes the purpose clear.
        type Prototype: for<'a> Arbitrary<'a>;
    }
}

/// Implementations of `soroban_sdk::arbitrary::api` for Rust scalar types.
///
/// These types
///
/// - do not have a distinct `Arbitrary` prototype,
///   i.e. they use themselves as the `SorobanArbitrary::Prototype` type,
/// - implement `Arbitrary` in the `arbitrary` crate,
/// - trivially implement `TryFromVal<Env, SorobanArbitrary::Prototype>`,
///
/// Examples:
///
/// - `u32`
mod scalars {
    use crate::arbitrary::api::*;

    impl SorobanArbitrary for () {
        type Prototype = ();
    }

    impl SorobanArbitrary for bool {
        type Prototype = bool;
    }

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
/// - trivially implement `TryFromVal<Env, SorobanArbitrary::Prototype>`,
///
/// Examples:
///
/// - `Error`
mod simple {
    use crate::arbitrary::api::*;
    pub use crate::Error;

    impl SorobanArbitrary for Error {
        type Prototype = Error;
    }
}

/// Implementations of `soroban_sdk::arbitrary::api` for Soroban types that do
/// need access to the Soroban host environment.
///
/// These types
///
/// - have a distinct `Arbitrary` prototype that derives `Arbitrary`,
/// - require an `Env` to be converted to their actual contract type.
///
/// Examples:
///
/// - `Vec`
mod objects {
    use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

    use crate::arbitrary::api::*;
    use crate::ConversionError;
    use crate::{Env, IntoVal, TryFromVal};

    use crate::xdr::{Duration, Int256Parts, ScVal, TimePoint, UInt256Parts};
    use crate::{Address, Bytes, BytesN, Map, RawVal, String, Symbol, Vec};
    use soroban_env_host::{DurationVal, I256Val, TimepointVal, TryIntoVal, U256Val};

    use std::string::String as RustString;
    use std::vec::Vec as RustVec;

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitraryTimepoint {
        inner: u64,
    }

    impl SorobanArbitrary for TimepointVal {
        type Prototype = ArbitraryTimepoint;
    }

    impl TryFromVal<Env, ArbitraryTimepoint> for TimepointVal {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryTimepoint) -> Result<Self, Self::Error> {
            let v = ScVal::Timepoint(TimePoint::from(v.inner));
            let v = RawVal::try_from_val(env, &v)?;
            v.try_into_val(env)
        }
    }

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitraryDuration {
        inner: u64,
    }

    impl SorobanArbitrary for DurationVal {
        type Prototype = ArbitraryDuration;
    }

    impl TryFromVal<Env, ArbitraryDuration> for DurationVal {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryDuration) -> Result<Self, Self::Error> {
            let v = ScVal::Duration(Duration::from(v.inner));
            let v = RawVal::try_from_val(env, &v)?;
            v.try_into_val(env)
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitraryU256 {
        parts: (u64, u64, u64, u64),
    }

    impl SorobanArbitrary for U256Val {
        type Prototype = ArbitraryU256;
    }

    impl TryFromVal<Env, ArbitraryU256> for U256Val {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryU256) -> Result<Self, Self::Error> {
            let v = ScVal::U256(UInt256Parts {
                hi_hi: v.parts.0,
                hi_lo: v.parts.1,
                lo_hi: v.parts.2,
                lo_lo: v.parts.3,
            });
            let v = RawVal::try_from_val(env, &v)?;
            v.try_into_val(env)
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitraryI256 {
        parts: (i64, u64, u64, u64),
    }

    impl SorobanArbitrary for I256Val {
        type Prototype = ArbitraryI256;
    }

    impl TryFromVal<Env, ArbitraryI256> for I256Val {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryI256) -> Result<Self, Self::Error> {
            let v = ScVal::I256(Int256Parts {
                hi_hi: v.parts.0,
                hi_lo: v.parts.1,
                lo_hi: v.parts.2,
                lo_lo: v.parts.3,
            });
            let v = RawVal::try_from_val(env, &v)?;
            v.try_into_val(env)
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitraryString {
        inner: RustString,
    }

    impl SorobanArbitrary for String {
        type Prototype = ArbitraryString;
    }

    impl TryFromVal<Env, ArbitraryString> for String {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryString) -> Result<Self, Self::Error> {
            Self::try_from_val(env, &v.inner.as_str())
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

    #[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitrarySymbol {
        s: RustString,
    }

    impl<'a> Arbitrary<'a> for ArbitrarySymbol {
        fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<ArbitrarySymbol> {
            let valid_chars = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            let valid_chars = valid_chars.as_bytes();
            let mut chars = vec![];
            let len = u.int_in_range(0..=32)?;
            for _ in 0..len {
                let ch = u.choose(valid_chars)?;
                chars.push(*ch);
            }
            Ok(ArbitrarySymbol {
                s: RustString::from_utf8(chars).expect("utf8"),
            })
        }
    }

    impl SorobanArbitrary for Symbol {
        type Prototype = ArbitrarySymbol;
    }

    impl TryFromVal<Env, ArbitrarySymbol> for Symbol {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitrarySymbol) -> Result<Self, Self::Error> {
            Self::try_from_val(env, &v.s.as_str())
        }
    }

    //////////////////////////////////

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ArbitraryAddress {
        inner: [u8; 32],
    }

    impl SorobanArbitrary for Address {
        type Prototype = ArbitraryAddress;
    }

    impl TryFromVal<Env, ArbitraryAddress> for Address {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryAddress) -> Result<Self, Self::Error> {
            use crate::env::xdr::{Hash, ScAddress};

            let sc_addr = ScVal::Address(ScAddress::Contract(Hash(v.inner)));
            Ok(sc_addr.into_val(env))
        }
    }
}

/// Implementations of `soroban_sdk::arbitrary::api` for `RawVal`.
mod composite {
    use arbitrary::Arbitrary;

    use crate::arbitrary::api::*;
    use crate::ConversionError;
    use crate::{Env, IntoVal, TryFromVal};

    use super::objects::*;
    use super::simple::*;
    use crate::{Address, Bytes, Map, RawVal, String, Symbol, Vec};

    use soroban_env_host::{DurationVal, I256Val, TimepointVal, U256Val};

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub enum ArbitraryRawVal {
        Void,
        Bool(bool),
        Error(Error),
        U32(u32),
        I32(i32),
        U64(u64),
        I64(i64),
        Timepoint(ArbitraryTimepoint),
        Duration(ArbitraryDuration),
        U128(u128),
        I128(i128),
        U256(ArbitraryU256),
        I256(ArbitraryI256),
        Bytes(ArbitraryBytes),
        String(ArbitraryString),
        Symbol(ArbitrarySymbol),
        Vec(ArbitraryRawValVec),
        Map(ArbitraryRawValMap),
        Address(<Address as SorobanArbitrary>::Prototype),
        //BodyAndTag(u64, u8),
    }

    impl SorobanArbitrary for RawVal {
        type Prototype = ArbitraryRawVal;
    }

    impl TryFromVal<Env, ArbitraryRawVal> for RawVal {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryRawVal) -> Result<Self, Self::Error> {
            Ok(match v {
                ArbitraryRawVal::Void => RawVal::VOID.into(),
                ArbitraryRawVal::Bool(v) => v.into_val(env),
                ArbitraryRawVal::Error(v) => v.into_val(env),
                ArbitraryRawVal::U32(v) => v.into_val(env),
                ArbitraryRawVal::I32(v) => v.into_val(env),
                ArbitraryRawVal::U64(v) => v.into_val(env),
                ArbitraryRawVal::I64(v) => v.into_val(env),
                ArbitraryRawVal::Timepoint(v) => {
                    let v: TimepointVal = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::Duration(v) => {
                    let v: DurationVal = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::U256(v) => {
                    let v: U256Val = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::I256(v) => {
                    let v: I256Val = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::U128(v) => v.into_val(env),
                ArbitraryRawVal::I128(v) => v.into_val(env),
                ArbitraryRawVal::Bytes(v) => {
                    let v: Bytes = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::String(v) => {
                    let v: String = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::Symbol(v) => {
                    let v: Symbol = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawVal::Vec(v) => v.into_val(env),
                ArbitraryRawVal::Map(v) => v.into_val(env),
                ArbitraryRawVal::Address(v) => {
                    let v: Address = v.into_val(env);
                    v.into_val(env)
                }
                /*ArbitraryRawVal::BodyAndTag(body, tag) => {
                    const TAG_BITS: usize = 8;
                    let payload = (body << TAG_BITS) | *tag as u64;
                    RawVal::from_payload(payload)
                }*/
            })
        }
    }

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub enum ArbitraryRawValVec {
        Void(<Vec<()> as SorobanArbitrary>::Prototype),
        Bool(<Vec<bool> as SorobanArbitrary>::Prototype),
        Error(<Vec<Error> as SorobanArbitrary>::Prototype),
        U32(<Vec<u32> as SorobanArbitrary>::Prototype),
        I32(<Vec<i32> as SorobanArbitrary>::Prototype),
        U64(<Vec<u64> as SorobanArbitrary>::Prototype),
        I64(<Vec<i64> as SorobanArbitrary>::Prototype),
        Timepoint(<Vec<TimepointVal> as SorobanArbitrary>::Prototype),
        Duration(<Vec<DurationVal> as SorobanArbitrary>::Prototype),
        U128(<Vec<u128> as SorobanArbitrary>::Prototype),
        I128(<Vec<i128> as SorobanArbitrary>::Prototype),
        U256(<Vec<U256Val> as SorobanArbitrary>::Prototype),
        I256(<Vec<I256Val> as SorobanArbitrary>::Prototype),
        Bytes(<Vec<Bytes> as SorobanArbitrary>::Prototype),
        String(<Vec<String> as SorobanArbitrary>::Prototype),
        Symbol(<Vec<Symbol> as SorobanArbitrary>::Prototype),
        Vec(<Vec<Vec<u32>> as SorobanArbitrary>::Prototype),
        Map(<Map<u32, u32> as SorobanArbitrary>::Prototype),
        Address(<Vec<Address> as SorobanArbitrary>::Prototype),
        RawVal(<Vec<RawVal> as SorobanArbitrary>::Prototype),
    }

    impl TryFromVal<Env, ArbitraryRawValVec> for RawVal {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryRawValVec) -> Result<Self, Self::Error> {
            Ok(match v {
                ArbitraryRawValVec::Void(v) => {
                    let v: Vec<()> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Bool(v) => {
                    let v: Vec<bool> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Error(v) => {
                    let v: Vec<Error> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::U32(v) => {
                    let v: Vec<u32> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::I32(v) => {
                    let v: Vec<i32> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::U64(v) => {
                    let v: Vec<u64> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::I64(v) => {
                    let v: Vec<i64> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Timepoint(v) => {
                    let v: Vec<TimepointVal> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Duration(v) => {
                    let v: Vec<DurationVal> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::U128(v) => {
                    let v: Vec<u128> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::I128(v) => {
                    let v: Vec<i128> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::U256(v) => {
                    let v: Vec<U256Val> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::I256(v) => {
                    let v: Vec<I256Val> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Bytes(v) => {
                    let v: Vec<Bytes> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::String(v) => {
                    let v: Vec<String> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Symbol(v) => {
                    let v: Vec<Symbol> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Vec(v) => {
                    let v: Vec<Vec<u32>> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Map(v) => {
                    let v: Map<u32, u32> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::Address(v) => {
                    let v: Vec<Address> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValVec::RawVal(v) => {
                    let v: Vec<RawVal> = v.into_val(env);
                    v.into_val(env)
                }
            })
        }
    }

    #[derive(Arbitrary, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub enum ArbitraryRawValMap {
        VoidToVoid(<Map<(), ()> as SorobanArbitrary>::Prototype),
        BoolToBool(<Map<bool, bool> as SorobanArbitrary>::Prototype),
        ErrorToError(<Map<Error, Error> as SorobanArbitrary>::Prototype),
        U32ToU32(<Map<u32, u32> as SorobanArbitrary>::Prototype),
        I32ToI32(<Map<i32, i32> as SorobanArbitrary>::Prototype),
        U64ToU64(<Map<u64, u64> as SorobanArbitrary>::Prototype),
        I64ToI64(<Map<i64, i64> as SorobanArbitrary>::Prototype),
        TimepointToTimepoint(<Map<TimepointVal, TimepointVal> as SorobanArbitrary>::Prototype),
        DurationToDuration(<Map<DurationVal, DurationVal> as SorobanArbitrary>::Prototype),
        U128ToU128(<Map<u128, u128> as SorobanArbitrary>::Prototype),
        I128ToI128(<Map<i128, i128> as SorobanArbitrary>::Prototype),
        U256ToU256(<Map<U256Val, U256Val> as SorobanArbitrary>::Prototype),
        I256ToI256(<Map<I256Val, I256Val> as SorobanArbitrary>::Prototype),
        BytesToBytes(<Map<Bytes, Bytes> as SorobanArbitrary>::Prototype),
        StringToString(<Map<String, String> as SorobanArbitrary>::Prototype),
        SymbolToSymbol(<Map<Symbol, Symbol> as SorobanArbitrary>::Prototype),
        VecToVec(<Map<Vec<u32>, Vec<u32>> as SorobanArbitrary>::Prototype),
        MapToMap(<Map<Map<u32, u32>, Map<u32, u32>> as SorobanArbitrary>::Prototype),
        AddressToAddress(<Map<Address, Address> as SorobanArbitrary>::Prototype),
        RawValToRawVal(<Map<RawVal, RawVal> as SorobanArbitrary>::Prototype),
    }

    impl TryFromVal<Env, ArbitraryRawValMap> for RawVal {
        type Error = ConversionError;
        fn try_from_val(env: &Env, v: &ArbitraryRawValMap) -> Result<Self, Self::Error> {
            Ok(match v {
                ArbitraryRawValMap::VoidToVoid(v) => {
                    let v: Map<(), ()> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::BoolToBool(v) => {
                    let v: Map<bool, bool> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::ErrorToError(v) => {
                    let v: Map<Error, Error> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::U32ToU32(v) => {
                    let v: Map<u32, u32> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::I32ToI32(v) => {
                    let v: Map<i32, i32> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::U64ToU64(v) => {
                    let v: Map<u64, u64> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::I64ToI64(v) => {
                    let v: Map<i64, i64> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::TimepointToTimepoint(v) => {
                    let v: Map<TimepointVal, TimepointVal> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::DurationToDuration(v) => {
                    let v: Map<DurationVal, DurationVal> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::U128ToU128(v) => {
                    let v: Map<u128, u128> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::I128ToI128(v) => {
                    let v: Map<i128, i128> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::U256ToU256(v) => {
                    let v: Map<U256Val, U256Val> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::I256ToI256(v) => {
                    let v: Map<I256Val, I256Val> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::BytesToBytes(v) => {
                    let v: Map<Bytes, Bytes> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::StringToString(v) => {
                    let v: Map<String, String> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::SymbolToSymbol(v) => {
                    let v: Map<Symbol, Symbol> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::VecToVec(v) => {
                    let v: Map<Vec<u32>, Vec<u32>> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::MapToMap(v) => {
                    let v: Map<Map<u32, u32>, Map<u32, u32>> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::AddressToAddress(v) => {
                    let v: Map<Address, Address> = v.into_val(env);
                    v.into_val(env)
                }
                ArbitraryRawValMap::RawValToRawVal(v) => {
                    let v: Map<RawVal, RawVal> = v.into_val(env);
                    v.into_val(env)
                }
            })
        }
    }
}

/// Additional tools for writing fuzz tests.
mod fuzz_test_helpers {
    use soroban_env_host::call_with_suppressed_panic_hook;

    /// Catch panics within a fuzz test.
    ///
    /// The `cargo-fuzz` test harness turns panics into test failures,
    /// immediately aborting the process.
    ///
    /// This function catches panics while temporarily disabling the
    /// `cargo-fuzz` panic handler.
    ///
    /// # Example
    ///
    /// ```
    /// # macro_rules! fuzz_target {
    /// #     (|$data:ident: $dty: ty| $body:block) => { };
    /// # }
    /// # struct ExampleContract;
    /// # impl ExampleContract {
    /// #   fn new(e: &soroban_sdk::Env, b: &soroban_sdk::BytesN<32>) { }
    /// #   fn deposit(&self, a: soroban_sdk::Address, n: i128) { }
    /// # }
    /// use soroban_sdk::{Address, Env};
    /// use soroban_sdk::arbitrary::{Arbitrary, SorobanArbitrary};
    ///
    /// #[derive(Arbitrary, Debug)]
    /// struct FuzzDeposit {
    ///     deposit_amount: i128,
    ///     deposit_address: <Address as SorobanArbitrary>::Prototype,
    /// }
    ///
    /// fuzz_target!(|input: FuzzDeposit| {
    ///     let env = Env::default();
    ///
    ///     let contract = ExampleContract::new(env, &env.register_contract(None, ExampleContract {}));
    ///
    ///     let addresses: Address = input.deposit_address.into_val(&env);
    ///     let r = fuzz_catch_panic(|| {
    ///         contract.deposit(deposit_address, input.deposit_amount);
    ///     });
    /// });
    /// ```
    pub fn fuzz_catch_panic<F, R>(f: F) -> std::thread::Result<R>
    where
        F: FnOnce() -> R,
    {
        call_with_suppressed_panic_hook(std::panic::AssertUnwindSafe(f))
    }
}

#[cfg(test)]
mod tests {
    use crate::arbitrary::*;
    use crate::{Bytes, BytesN, Map, RawVal, String, Symbol, Vec};
    use crate::{Env, IntoVal};
    use arbitrary::{Arbitrary, Unstructured};
    use rand::RngCore;
    use soroban_env_host::{DurationVal, I256Val, TimepointVal, U256Val};

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
    fn test_timepoint() {
        run_test::<TimepointVal>()
    }

    #[test]
    fn test_duration() {
        run_test::<DurationVal>()
    }

    #[test]
    fn test_u256() {
        run_test::<U256Val>()
    }

    #[test]
    fn test_i256() {
        run_test::<I256Val>()
    }

    #[test]
    fn test_bytes() {
        run_test::<Bytes>()
    }

    #[test]
    fn test_string() {
        run_test::<String>()
    }

    #[test]
    fn test_symbol() {
        run_test::<Symbol>()
    }

    #[test]
    fn test_bytes_n() {
        run_test::<BytesN<64>>()
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
    fn test_raw_val() {
        run_test::<RawVal>()
    }

    mod user_defined_types {
        use crate as soroban_sdk;
        use crate::arbitrary::tests::run_test;
        use crate::{Bytes, BytesN, Map, Vec};
        use soroban_sdk::contracttype;

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivStruct {
            count_u: u32,
            count_i: i32,
            bytes_n: BytesN<32>,
            vec: Vec<Bytes>,
            map: Map<Bytes, Vec<i32>>,
        }

        #[test]
        fn test_user_defined_priv_struct() {
            run_test::<PrivStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivStructPubFields {
            pub count_u: u32,
            pub count_i: i32,
            pub bytes_n: BytesN<32>,
            pub vec: Vec<Bytes>,
            pub map: Map<Bytes, Vec<i32>>,
        }

        #[test]
        fn test_user_defined_priv_struct_pub_fields() {
            run_test::<PrivStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubStruct {
            count_u: u32,
            count_i: i32,
            bytes_n: BytesN<32>,
            vec: Vec<Bytes>,
            map: Map<Bytes, Vec<i32>>,
        }

        #[test]
        fn test_user_defined_pub_struct() {
            run_test::<PubStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubStructPubFields {
            pub count_u: u32,
            pub count_i: i32,
            pub bytes_n: BytesN<32>,
            pub vec: Vec<Bytes>,
            pub map: Map<Bytes, Vec<i32>>,
        }

        #[test]
        fn test_user_defined_pubstruct_pub_fields() {
            run_test::<PubStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivTupleStruct(u32, i32, BytesN<32>, Vec<Bytes>, Map<Bytes, Vec<i32>>);

        #[test]
        fn test_user_defined_priv_tuple_struct() {
            run_test::<PrivTupleStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        struct PrivTupleStructPubFields(
            pub u32,
            pub i32,
            pub BytesN<32>,
            pub Vec<Bytes>,
            pub Map<Bytes, Vec<i32>>,
        );

        #[test]
        fn test_user_defined_priv_tuple_struct_pub_fields() {
            run_test::<PrivTupleStructPubFields>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubTupleStruct(u32, i32, BytesN<32>, Vec<Bytes>, Map<Bytes, Vec<i32>>);

        #[test]
        fn test_user_defined_pub_tuple_struct() {
            run_test::<PubTupleStruct>();
        }

        #[contracttype]
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct PubTupleStructPubFields(
            pub u32,
            pub i32,
            pub BytesN<32>,
            pub Vec<Bytes>,
            pub Map<Bytes, Vec<i32>>,
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
            Aa(u32, u32),
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

        #[test]
        fn test_declared_inside_a_fn() {
            #[contracttype]
            struct Foo {
                a: u32,
            }

            #[contracttype]
            enum Bar {
                Baz,
                Qux,
            }

            run_test::<Foo>();
            run_test::<Bar>();
        }
    }
}
