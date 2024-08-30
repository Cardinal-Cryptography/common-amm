use crate::{Balance, FactoryError, MathError, PairError, StablePoolError};
use ink::{
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
    LangError,
};
use psp22::PSP22Error;

/// Specifies the pool for the trade and the input token of this trade (token to sell).
#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct Step {
    pub token_in: AccountId,
    pub pool_id: AccountId,
}

#[ink::trait_definition]
pub trait RouterV2 {
    /// Returns address of the `Factory` contract for this `RouterV2` instance.
    #[ink(message)]
    fn pair_factory(&self) -> AccountId;

    /// Returns address of the `WrappedNative` contract for this `RouterV2` instance.
    #[ink(message)]
    fn wnative(&self) -> AccountId;

    /// Adds liquidity to `(token_0, token_1)` pair.
    ///
    /// If `(token_0, token_1)` pair does not exist, creates a new pair.
    ///
    /// Will add at least `*_min` amount of tokens and up to `*_desired`
    /// while still maintaining the constant `k` ratio of the pair.
    ///
    ///
    /// If successful, liquidity tokens will be minted for `to` account.
    #[ink(message)]
    fn add_pair_liquidity(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        amount_0_desired: u128,
        amount_b_desired: u128,
        amount_0_min: u128,
        amount_b_min: u128,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, u128, u128), RouterV2Error>;

    /// Removes `liquidity` amount of tokens from `(token_0, token_1)`
    /// pair and transfers tokens `to` account.
    ///
    /// Fails if any of the balances is lower than respective `*_min` amount.
    ///
    /// Returns withdrawn balances of both tokens.
    #[ink(message)]
    fn remove_pair_liquidity(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
        liquidity: u128,
        amount_0_min: u128,
        amount_b_min: u128,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, u128), RouterV2Error>;

    /// Adds liquidity to `(token, native token)` pair.
    ///
    /// If `(token_0, token_1)` pair does not exist, creates a new pair.
    ///
    /// Will add at least `*_min` amount of tokens and up to `*_desired`
    /// while still maintaining the constant `k` ratio of the pair.
    ///
    /// If successful, liquidity tokens will be minted for `to` account.
    #[ink(message, payable)]
    fn add_pair_liquidity_native(
        &mut self,
        token: AccountId,
        amount_token_desired: u128,
        amount_token_min: u128,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, Balance, u128), RouterV2Error>;

    /// Removes `liquidity` amount of tokens from `(token, wrapped_native)`
    /// pair and transfers tokens `to` account.
    ///
    /// Fails if any of the balances is lower than respective `*_min` amount.
    ///
    /// Returns withdrawn balances of both tokens.
    #[ink(message)]
    fn remove_pair_liquidity_native(
        &mut self,
        token: AccountId,
        liquidity: u128,
        amount_token_min: u128,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, Balance), RouterV2Error>;

    /// Exchanges tokens along the `path` to `token_out`.
    ///
    /// Starts with `amount_in` and token address under `path[0].token_in`.
    ///
    /// If `path[i].pool_id` is `None`, it attemps the swap through
    /// `(path[i].token_in, path[i + 1].token_in)` Pair,
    /// or `(path[path.len() - 1].token_in, token_out)` Pair
    /// if this is the last Step in the `path`
    ///
    /// Fails if output amount is less than `amount_out_min`.
    /// Transfers tokens to account under `to` address.
    #[ink(message)]
    fn swap_exact_tokens_for_tokens(
        &mut self,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<Step>,
        token_out: AccountId,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Exchanges tokens along `path` to `token_out`
    /// so that at the end caller receives `amount_out`
    /// worth of tokens and pays no more than `amount_in_max`
    /// of the starting token. Fails if any of these conditions
    /// is not satisfied.
    /// Transfers tokens to account under `to` address.
    #[ink(message)]
    fn swap_tokens_for_exact_tokens(
        &mut self,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<Step>,
        token_out: AccountId,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Exchanges exact amount of native token,
    /// along the `path` to `token_out`, and expects
    /// to receive at least `amount_out_min` of tokens
    /// at the end of execution. Fails if the output
    /// amount is less than `amount_out_min`.
    /// Transfers tokens to account under `to` address.
    #[ink(message, payable)]
    fn swap_exact_native_for_tokens(
        &mut self,
        amount_out_min: u128,
        path: Vec<Step>,
        token_out: AccountId,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Exchanges tokens along `path`
    /// so that at the end caller receives `amount_out`
    /// worth of native tokens and pays no more than `amount_in_max`
    /// of the starting token. Fails if any of these conditions
    /// is not satisfied.
    /// Transfers tokens to account under `to` address.
    #[ink(message)]
    fn swap_tokens_for_exact_native(
        &mut self,
        amount_out: Balance,
        amount_in_max: u128,
        path: Vec<Step>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Exchanges exact amount of token,
    /// along the `path`, and expects
    /// to receive at least `amount_out_min` of native tokens
    /// at the end of execution. Fails if the output
    /// amount is less than `amount_out_min`.
    /// Transfers tokens to account under `to` address.
    #[ink(message)]
    fn swap_exact_tokens_for_native(
        &mut self,
        amount_in: u128,
        amount_out_min: Balance,
        path: Vec<Step>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Exchanges tokens along `path` to `token_out`
    /// so that at the end caller receives `amount_out`
    /// worth of tokens and pays no more than `amount_in_max`
    /// of the native token. Fails if any of these conditions
    /// is not satisfied.
    /// Transfers tokens to account under `to` address.
    #[ink(message, payable)]
    fn swap_native_for_exact_tokens(
        &mut self,
        amount_out: u128,
        path: Vec<Step>,
        token_out: AccountId,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Returns amounts of tokens received for `amount_in`.
    #[ink(message)]
    fn get_amounts_out(
        &self,
        amount_in: u128,
        path: Vec<Step>,
        token_out: AccountId,
    ) -> Result<Vec<u128>, RouterV2Error>;

    /// Returns amounts of tokens user has to supply.
    #[ink(message)]
    fn get_amounts_in(
        &self,
        amount_out: u128,
        path: Vec<Step>,
        token_out: AccountId,
    ) -> Result<Vec<u128>, RouterV2Error>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum RouterV2Error {
    PSP22Error(PSP22Error),
    FactoryError(FactoryError),
    PairError(PairError),
    LangError(LangError),
    MathError(MathError),
    StablePoolError(StablePoolError),

    CrossContractCallFailed(String),
    Expired,
    IdenticalAddresses,
    InvalidPath,
    PairNotFound,
    PoolNotFound,
    TransferError,

    ExcessiveInputAmount,
    InsufficientAmount,
    InsufficientOutputAmount,
    InsufficientAmountA,
    InsufficientAmountB,
    InsufficientLiquidity,
}

macro_rules! impl_froms {
    ( $( $error:ident ),* ) => {
        $(
            impl From<$error> for RouterV2Error {
                fn from(error: $error) -> Self {
                    RouterV2Error::$error(error)
                }
            }
        )*
    };
}

impl_froms!(
    PSP22Error,
    FactoryError,
    PairError,
    LangError,
    MathError,
    StablePoolError
);
