use core::convert::TryInto;

#[cfg(target_family = "wasm")]
pub mod internal {
    pub use soroban_env_guest::*;
    pub type EnvImpl = Guest;
}

#[cfg(not(target_family = "wasm"))]
pub mod internal {
    pub use soroban_env_host::*;
    pub type EnvImpl = Host;

    #[doc(hidden)]
    impl<F, T> TryConvert<F, T> for super::Env
    where
        EnvImpl: TryConvert<F, T>,
    {
        type Error = <EnvImpl as TryConvert<F, T>>::Error;
        fn convert(&self, f: F) -> Result<T, Self::Error> {
            self.env_impl.convert(f)
        }
    }
}

// Testutils from the environmen are pub here, and then pub re-exported out of
// the SDK in the crate::testutils mod.
#[cfg(any(test, feature = "testutils"))]
pub mod testutils {
    pub use super::internal::LedgerInfo;
}

pub use internal::meta;
pub use internal::xdr;
pub use internal::BitSet;
pub use internal::ConversionError;
pub use internal::EnvBase;
pub use internal::FromVal;
pub use internal::IntoVal;
use internal::InvokerType;
pub use internal::Object;
pub use internal::RawVal;
pub use internal::RawValConvertible;
pub use internal::Status;
pub use internal::Symbol;
pub use internal::TryFromVal;
pub use internal::TryIntoVal;
pub use internal::Val;

pub type EnvVal = internal::EnvVal<Env, RawVal>;
pub type EnvObj = internal::EnvVal<Env, Object>;

use crate::{
    accounts::Accounts, address::Address, data::Data, deploy::Deployer, events::Events,
    ledger::Ledger, logging::Logger, AccountId, Bytes, BytesN, Vec,
};

/// The [Env] type provides access to the environment the contract is executing
/// within.
///
/// The [Env] provides access to information about the currently executing
/// contract, who invoked it, contract data, functions for signing, hashing,
/// etc.
///
/// Most types require access to an [Env] to be constructed or converted.
#[derive(Clone)]
pub struct Env {
    env_impl: internal::EnvImpl,
}

impl Default for Env {
    #[cfg(not(any(test, feature = "testutils")))]
    fn default() -> Self {
        Self {
            env_impl: Default::default(),
        }
    }

    #[cfg(any(test, feature = "testutils"))]
    fn default() -> Self {
        Self::default_with_testutils()
    }
}

impl Env {
    /// Panic with the given error.
    ///
    /// Equivalent to `panic!`, but with an error value instead of a string.
    #[doc(hidden)]
    pub fn panic_with_error(&self, error: impl Into<Status>) {
        _ = internal::Env::fail_with_status(self, error.into());
        unreachable!()
    }

    /// Get the invoking [Address] of the current executing contract.
    pub fn invoker(&self) -> Address {
        let invoker_type: InvokerType = internal::Env::get_invoker_type(self)
            .try_into()
            .expect("unrecognized invoker type");
        match invoker_type {
            InvokerType::Account => Address::Account(unsafe {
                AccountId::unchecked_new(internal::Env::get_invoking_account(self).in_env(self))
            }),
            InvokerType::Contract => Address::Contract(unsafe {
                BytesN::unchecked_new(internal::Env::get_invoking_contract(self).in_env(self))
            }),
        }
    }

    /// Get a [Data] for accessing and update contract data that has been stored
    /// by the currently executing contract.
    #[inline(always)]
    pub fn data(&self) -> Data {
        Data::new(self)
    }

    /// Get [Events] for publishing events associated with the
    /// currently executing contract.
    #[inline(always)]
    pub fn events(&self) -> Events {
        Events::new(self)
    }

    /// Get a [Ledger] for accessing the current ledger.
    #[inline(always)]
    pub fn ledger(&self) -> Ledger {
        Ledger::new(self)
    }

    /// Get an [Accounts] for accessing accounts in the current ledger.
    #[inline(always)]
    pub fn accounts(&self) -> Accounts {
        Accounts::new(self)
    }

    /// Get a deployer for deploying contracts.
    #[inline(always)]
    pub fn deployer(&self) -> Deployer {
        Deployer::new(self)
    }

    /// Get the 32-byte hash identifier of the current executing contract.
    pub fn current_contract(&self) -> BytesN<32> {
        let id = internal::Env::get_current_contract(self);
        unsafe { BytesN::<32>::unchecked_new(id.in_env(self)) }
    }

    /// Get the 32-byte hash identifier of the current executing contract.
    #[doc(hidden)]
    pub fn get_current_contract(&self) -> BytesN<32> {
        self.current_contract()
    }

    /// Returns the contract call stack as a [`Vec`]
    /// of `(contract_id, function_name)`.
    ///
    /// ### Examples
    /// ```
    /// use soroban_sdk::{contractimpl, BytesN, Env, Symbol, symbol};
    ///
    /// pub struct Contract;
    ///
    /// #[contractimpl]
    /// impl Contract {
    ///     pub fn hello(env: Env) {
    ///         let stack = env.call_stack();
    ///         assert_eq!(stack.len(), 1);
    ///
    ///         let outer = stack.get(0).unwrap().unwrap();
    ///         assert_eq!(outer.0, BytesN::from_array(&env, &[0; 32]));
    ///         assert_eq!(outer.1, symbol!("hello"));
    ///     }
    /// }
    /// # #[cfg(feature = "testutils")]
    /// # fn main() {
    /// let env = Env::default();
    /// let contract_id = BytesN::from_array(&env, &[0; 32]);
    /// env.register_contract(&contract_id, Contract);
    /// let client = ContractClient::new(&env, &contract_id);
    /// client.hello();
    /// # }
    /// # #[cfg(not(feature = "testutils"))]
    /// # fn main() { }
    /// ```
    pub fn call_stack(&self) -> Vec<(BytesN<32>, Symbol)> {
        let stack = internal::Env::get_current_call_stack(self);
        unsafe { Vec::unchecked_new(stack.in_env(self)) }
    }

    /// Computes a SHA-256 hash.
    pub fn compute_hash_sha256(&self, msg: &Bytes) -> BytesN<32> {
        let bin_obj = internal::Env::compute_hash_sha256(self, msg.into());
        unsafe { BytesN::unchecked_new(bin_obj.in_env(self)) }
    }

    /// Verifies an ed25519 signature.
    ///
    /// The ed25519 signature (`sig`) is verified as a valid signature of the
    /// message (`msg`) by the ed25519 public key (`pk`).
    ///
    /// ### Panics
    ///
    /// Will panic if the signature verification fails.
    ///
    /// ### TODO
    ///
    /// Return a [Result] instead of panicking.
    pub fn verify_sig_ed25519(&self, pk: &BytesN<32>, msg: &Bytes, sig: &BytesN<64>) {
        let _ = internal::Env::verify_sig_ed25519(
            self,
            msg.to_object(),
            pk.to_object(),
            sig.to_object(),
        );
    }

    /// Invokes a function of a contract that is registered in the [Env].
    ///
    /// # Panics
    ///
    /// Will panic if the `contract_id` does not match a registered contract,
    /// `func` does not match a function of the referenced contract, or the
    /// number of `args` do not match the argument count of the referenced
    /// contract function.
    ///
    /// Will panic if the contract that is invoked fails or aborts in anyway.
    ///
    /// Will panic if the value returned from the contract cannot be converted
    /// into the type `T`.
    pub fn invoke_contract<T>(
        &self,
        contract_id: &BytesN<32>,
        func: &Symbol,
        args: Vec<RawVal>,
    ) -> T
    where
        T: TryFromVal<Env, RawVal>,
    {
        let rv = internal::Env::call(self, contract_id.to_object(), *func, args.to_object());
        T::try_from_val(self, rv.clone())
            .map_err(|_| ConversionError)
            .unwrap()
    }

    /// Invokes a function of a contract that is registered in the [Env],
    /// returns an error if the invocation fails for any reason.
    pub fn try_invoke_contract<T, E>(
        &self,
        contract_id: &BytesN<32>,
        func: &Symbol,
        args: Vec<RawVal>,
    ) -> Result<Result<T, T::Error>, Result<E, E::Error>>
    where
        T: TryFromVal<Env, RawVal>,
        E: TryFrom<Status>,
    {
        let rv = internal::Env::try_call(self, contract_id.to_object(), *func, args.to_object());
        match Status::try_from_val(self, rv) {
            Ok(status) => Err(E::try_from(status)),
            Err(ConversionError) => Ok(T::try_from_val(self, rv)),
        }
    }

    /// Get the [Logger] for logging debug events.
    #[inline(always)]
    pub fn logger(&self) -> Logger {
        Logger::new(self)
    }

    #[doc(hidden)]
    pub fn log_value<V: IntoVal<Env, RawVal>>(&self, v: V) {
        internal::Env::log_value(self, v.into_val(self));
    }
}

#[cfg(any(test, feature = "testutils"))]
use crate::testutils::{Accounts as _, ContractFunctionSet, Ledger as _};
#[cfg(any(test, feature = "testutils"))]
use rand::RngCore;
#[cfg(any(test, feature = "testutils"))]
use std::rc::Rc;
#[cfg(any(test, feature = "testutils"))]
#[cfg_attr(feature = "docs", doc(cfg(feature = "testutils")))]
impl Env {
    pub(crate) fn host(&self) -> &internal::Host {
        &self.env_impl
    }

    fn default_with_testutils() -> Env {
        struct EmptySnapshotSource();

        impl internal::storage::SnapshotSource for EmptySnapshotSource {
            fn get(
                &self,
                _key: &xdr::LedgerKey,
            ) -> Result<xdr::LedgerEntry, soroban_env_host::HostError> {
                use xdr::{ScHostStorageErrorCode, ScStatus};
                let status: internal::Status =
                    ScStatus::HostStorageError(ScHostStorageErrorCode::UnknownError).into();
                Err(status.into())
            }

            fn has(&self, _key: &xdr::LedgerKey) -> Result<bool, soroban_env_host::HostError> {
                Ok(false)
            }
        }

        let rf = Rc::new(EmptySnapshotSource());
        let storage = internal::storage::Storage::with_recording_footprint(rf);
        let env_impl = internal::EnvImpl::with_storage_and_budget(
            storage,
            internal::budget::Budget::default(),
        );

        let env = Env { env_impl };

        env.set_source_account(&env.accounts().generate());

        env.ledger().set(internal::LedgerInfo {
            protocol_version: 0,
            sequence_number: 0,
            timestamp: 0,
            network_passphrase: vec![0u8],
            base_reserve: 0,
        });

        env
    }

    /// Sets the source account in the [Env].
    ///
    /// The source account will be accessible via [Env::invoker] when a contract
    /// is directly invoked.
    pub fn set_source_account(&self, account_id: &AccountId) {
        self.accounts().create(account_id);
        self.env_impl
            .set_source_account(account_id.try_into().unwrap());
    }

    /// Gets the source account set in the [Env].
    pub fn source_account(&self) -> AccountId {
        self.env_impl
            .source_account()
            .unwrap()
            .try_into_val(self)
            .unwrap()
    }

    /// Run the closure as if executed by the given contract ID.
    ///
    /// Used to write or read contract data, or take other actions in tests for
    /// setting up tests or asserting on internal state.
    pub fn as_contract<T>(&self, id: &BytesN<32>, f: impl FnOnce() -> T) -> T {
        let id: [u8; 32] = id.into();
        let func = Symbol::from_str("");
        let mut t: Option<T> = None;
        self.env_impl
            .with_test_contract_frame(id.into(), func, || {
                t = Some(f());
                Ok(().into())
            })
            .unwrap();
        t.unwrap()
    }

    /// Register a contract with the [Env] for testing.
    ///
    /// Passing a contract ID for the first arguments registers the contract
    /// with that contract ID. Providing `None` causes a random ID to be
    /// assigned to the contract.
    ///
    /// Returns the contract ID of the registered contract.
    ///
    /// ### Examples
    /// ```
    /// use soroban_sdk::{contractimpl, BytesN, Env, Symbol};
    ///
    /// pub struct HelloContract;
    ///
    /// #[contractimpl]
    /// impl HelloContract {
    ///     pub fn hello(env: Env, recipient: soroban_sdk::Symbol) -> soroban_sdk::Symbol {
    ///         todo!()
    ///     }
    /// }
    ///
    /// # fn main() {
    /// let env = Env::default();
    /// let contract_id = BytesN::from_array(&env, &[0; 32]);
    /// env.register_contract(&contract_id, HelloContract);
    /// # }
    /// ```
    pub fn register_contract<'a, T: ContractFunctionSet + 'static>(
        &self,
        contract_id: impl Into<Option<&'a BytesN<32>>>,
        contract: T,
    ) -> BytesN<32> {
        struct InternalContractFunctionSet<T: ContractFunctionSet>(pub(crate) T);
        impl<T: ContractFunctionSet> internal::ContractFunctionSet for InternalContractFunctionSet<T> {
            fn call(
                &self,
                func: &Symbol,
                env_impl: &internal::EnvImpl,
                args: &[RawVal],
            ) -> Option<RawVal> {
                self.0.call(func, Env::with_impl(env_impl.clone()), args)
            }
        }

        let contract_id = if let Some(contract_id) = contract_id.into() {
            contract_id.clone()
        } else {
            let mut contract_id = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut contract_id);
            BytesN::from_array(self, &contract_id)
        };
        self.env_impl
            .register_test_contract(
                contract_id.to_object(),
                Rc::new(InternalContractFunctionSet(contract)),
            )
            .unwrap();
        contract_id
    }

    /// Register a contract in a WASM file with the [Env] for testing.
    ///
    /// Passing a contract ID for the first arguments registers the contract
    /// with that contract ID. Providing `None` causes a random ID to be
    /// assigned to the contract.
    ///
    /// Returns the contract ID of the registered contract.
    ///
    /// ### Examples
    /// ```
    /// use soroban_sdk::{BytesN, Env};
    ///
    /// const WASM: &[u8] = include_bytes!("../doctest_fixtures/contract.wasm");
    ///
    /// # fn main() {
    /// let env = Env::default();
    /// let contract_id = BytesN::from_array(&env, &[0; 32]);
    /// env.register_contract_wasm(&contract_id, WASM);
    /// # }
    /// ```
    pub fn register_contract_wasm<'a>(
        &self,
        contract_id: impl Into<Option<&'a BytesN<32>>>,
        contract_wasm: &[u8],
    ) -> BytesN<32> {
        let contract_id = if let Some(contract_id) = contract_id.into() {
            contract_id.clone()
        } else {
            let mut contract_id = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut contract_id);
            BytesN::from_array(self, &contract_id)
        };
        self.env_impl
            .register_test_contract_wasm(contract_id.to_object(), contract_wasm)
            .unwrap();
        contract_id
    }

    /// Register the built-in token contract with the [Env] for testing.
    ///
    /// Passing a contract ID for the first arguments registers the contract
    /// with that contract ID. Providing `None` causes a random ID to be
    /// assigned to the contract.
    ///
    /// Returns the contract ID of the registered contract.
    ///
    /// ### Examples
    /// ```
    /// use soroban_sdk::{BytesN, Env};
    ///
    /// # fn main() {
    /// let env = Env::default();
    /// let contract_id = BytesN::from_array(&env, &[0; 32]);
    /// env.register_contract_token(&contract_id);
    /// # }
    /// ```
    pub fn register_contract_token<'a>(
        &self,
        contract_id: impl Into<Option<&'a BytesN<32>>>,
    ) -> BytesN<32> {
        let contract_id = if let Some(contract_id) = contract_id.into() {
            contract_id.clone()
        } else {
            let mut contract_id = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut contract_id);
            BytesN::from_array(self, &contract_id)
        };
        self.env_impl
            .register_test_contract_token(contract_id.to_object())
            .unwrap();
        contract_id
    }
}

#[doc(hidden)]
impl Env {
    pub fn with_impl(env_impl: internal::EnvImpl) -> Env {
        Env { env_impl }
    }
}

#[doc(hidden)]
impl internal::EnvBase for Env {
    fn as_mut_any(&mut self) -> &mut dyn core::any::Any {
        self
    }

    fn check_same_env(&self, other: &Self) {
        self.env_impl.check_same_env(&other.env_impl);
    }

    fn deep_clone(&self) -> Self {
        Env {
            env_impl: self.env_impl.deep_clone(),
        }
    }

    fn bytes_copy_from_slice(
        &self,
        b: Object,
        b_pos: RawVal,
        mem: &[u8],
    ) -> Result<Object, Status> {
        self.env_impl.bytes_copy_from_slice(b, b_pos, mem)
    }

    fn bytes_copy_to_slice(&self, b: Object, b_pos: RawVal, mem: &mut [u8]) -> Result<(), Status> {
        self.env_impl.bytes_copy_to_slice(b, b_pos, mem)
    }

    fn bytes_new_from_slice(&self, mem: &[u8]) -> Result<Object, Status> {
        self.env_impl.bytes_new_from_slice(mem)
    }

    fn log_static_fmt_val(&self, fmt: &'static str, v: RawVal) -> Result<(), Status> {
        self.env_impl.log_static_fmt_val(fmt, v)
    }

    fn log_static_fmt_static_str(&self, fmt: &'static str, s: &'static str) -> Result<(), Status> {
        self.env_impl.log_static_fmt_static_str(fmt, s)
    }

    fn log_static_fmt_val_static_str(
        &self,
        fmt: &'static str,
        v: RawVal,
        s: &'static str,
    ) -> Result<(), Status> {
        self.env_impl.log_static_fmt_val_static_str(fmt, v, s)
    }

    fn log_static_fmt_general(
        &self,
        fmt: &'static str,
        v: &[RawVal],
        s: &[&'static str],
    ) -> Result<(), Status> {
        self.env_impl.log_static_fmt_general(fmt, v, s)
    }
}

///////////////////////////////////////////////////////////////////////////////
/// X-macro use: impl Env for SDK's Env
///////////////////////////////////////////////////////////////////////////////

// This is a helper macro used only by impl_env_for_sdk below. It consumes a
// token-tree of the form:
//
//  {fn $fn_id:ident $args:tt -> $ret:ty}
//
// and produces the the corresponding method definition to be used in the
// SDK's Env implementation of the Env (calling through to the corresponding
// guest or host implementation).
macro_rules! sdk_function_helper {
    {$mod_id:ident, fn $fn_id:ident($($arg:ident:$type:ty),*) -> $ret:ty}
    =>
    {
        fn $fn_id(&self, $($arg:$type),*) -> $ret {
            self.env_impl.$fn_id($($arg),*)
        }
    };
}

// This is a callback macro that pattern-matches the token-tree passed by the
// x-macro (call_macro_with_all_host_functions) and produces a suite of
// forwarding-method definitions, which it places in the body of the declaration
// of the implementation of Env for the SDK's Env.
macro_rules! impl_env_for_sdk {
    {
        $(
            // This outer pattern matches a single 'mod' block of the token-tree
            // passed from the x-macro to this macro. It is embedded in a `$()*`
            // pattern-repetition matcher so that it will match all provided
            // 'mod' blocks provided.
            $(#[$mod_attr:meta])*
            mod $mod_id:ident $mod_str:literal
            {
                $(
                    // This inner pattern matches a single function description
                    // inside a 'mod' block in the token-tree passed from the
                    // x-macro to this macro. It is embedded in a `$()*`
                    // pattern-repetition matcher so that it will match all such
                    // descriptions.
                    $(#[$fn_attr:meta])*
                    { $fn_str:literal, fn $fn_id:ident $args:tt -> $ret:ty }
                )*
            }
        )*
    }

    =>  // The part of the macro above this line is a matcher; below is its expansion.

    {
        // This macro expands to a single item: the implementation of Env for
        // the SDK's Env struct used by client contract code running in a WASM VM.
        #[doc(hidden)]
        impl internal::Env for Env
        {
            $(
                $(
                    // This invokes the guest_function_helper! macro above
                    // passing only the relevant parts of the declaration
                    // matched by the inner pattern above. It is embedded in two
                    // nested `$()*` pattern-repetition expanders that
                    // correspond to the pattern-repetition matchers in the
                    // match section, but we ignore the structure of the 'mod'
                    // block repetition-level from the outer pattern in the
                    // expansion, flattening all functions from all 'mod' blocks
                    // into the implementation of Env for Guest.
                    sdk_function_helper!{$mod_id, fn $fn_id $args -> $ret}
                )*
            )*
        }
    };
}

// Here we invoke the x-macro passing generate_env_trait as its callback macro.
internal::call_macro_with_all_host_functions! { impl_env_for_sdk }
