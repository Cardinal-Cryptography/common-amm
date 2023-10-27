use crate::{
    Balance,
    DexError,
};
use ink::{
    prelude::vec::Vec,
    primitives::AccountId,
};

#[ink::trait_definition]
pub trait Router {
    /// Returns address of the `Factory` contract for this `Router` instance.
    #[ink(message)]
    fn factory(&self) -> AccountId;

    /// Returns address of the `WrappedNative` contract for this `Router` instance.
    #[ink(message)]
    fn wnative(&self) -> AccountId;

    /// Adds liquidity to `(token_a, token_b)` pair.
    ///
    /// Will add at least `*_min` amount of tokens and up to `*_desired`
    /// while still maintaining the constant `k` product of the pair.
    ///
    /// If successful, liquidity tokens will be minted for `to` account.
    #[ink(message)]
    fn add_liquidity(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, u128, u128), DexError>;

    /// Removes `liquidity` amount of tokens from `(token_a, token_b)`
    /// pair and transfers tokens `to` account.
    ///
    /// Fails if any of the balances is lower than respective `*_min` amount.
    ///
    /// Returns withdrawn balances of both tokens.
    #[ink(message)]
    fn remove_liquidity(
        &mut self,
        token_a: AccountId,
        token_b: AccountId,
        liquidity: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, u128), DexError>;

    /// Adds liquidity to `(token, native token)` pair.
    ///
    /// Will add at least `*_min` amount of tokens and up to `*_desired`
    /// while still maintaining the constant `k` product of the pair.
    ///
    /// If successful, liquidity tokens will be minted for `to` account.
    #[ink(message, payable)]
    fn add_liquidity_native(
        &mut self,
        token: AccountId,
        amount_token_desired: u128,
        amount_token_min: u128,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, Balance, u128), DexError>;

    /// Removes `liquidity` amount of tokens from `(token, wrapped_native)`
    /// pair and transfers tokens `to` account.
    ///
    /// Fails if any of the balances is lower than respective `*_min` amount.
    ///
    /// Returns withdrawn balances of both tokens.
    #[ink(message)]
    fn remove_liquidity_native(
        &mut self,
        token: AccountId,
        liquidity: u128,
        amount_token_min: u128,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, Balance), DexError>;

    /// Exchanges tokens along `path` tokens.
    /// Starts with `amount_in` and pair under `(path[0], path[1])` address.
    /// Fails if output amount is less than `amount_out_min`.
    /// Transfers tokens to account under `to` address.
    #[ink(message)]
    fn swap_exact_tokens_for_tokens(
        &mut self,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, DexError>;

    /// Exchanges tokens along `path` token pairs
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
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, DexError>;

    /// Exchanges exact amount of native token,
    /// along the `path` token pairs, and expects
    /// to receive at least `amount_out_min` of tokens
    /// at the end of execution. Fails if the output
    /// amount is less than `amount_out_min`.
    /// Transfers tokens to account under `to` address.
    #[ink(message, payable)]
    fn swap_exact_native_for_tokens(
        &mut self,
        amount_out_min: u128,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, DexError>;

    /// Exchanges tokens along `path` token pairs
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
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, DexError>;

    /// Exchanges exact amount of token,
    /// along the `path` token pairs, and expects
    /// to receive at least `amount_out_min` of native tokens
    /// at the end of execution. Fails if the output
    /// amount is less than `amount_out_min`.
    /// Transfers tokens to account under `to` address.
    #[ink(message)]
    fn swap_exact_tokens_for_native(
        &mut self,
        amount_in: u128,
        amount_out_min: Balance,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, DexError>;

    /// Exchanges tokens along `path` token pairs
    /// so that at the end caller receives `amount_out`
    /// worth of tokens and pays no more than `amount_in_max`
    /// of the native token. Fails if any of these conditions
    /// is not satisfied.
    /// Transfers tokens to account under `to` address.
    #[ink(message, payable)]
    fn swap_native_for_exact_tokens(
        &mut self,
        amount_out: u128,
        path: Vec<AccountId>,
        to: AccountId,
        deadline: u64,
    ) -> Result<Vec<u128>, DexError>;

    /// Returns amount of `B` tokens that have to be supplied
    /// , with the `amount_a` amount of tokens `A, to maintain
    /// constant `k` product of `(A, B)` token pair.
    #[ink(message)]
    fn quote(&self, amount_a: u128, reserve_a: u128, reserve_b: u128) -> Result<u128, DexError>;

    /// Returns amount of `B` tokens received
    /// for `amount_in` of `A` tokens that maintains
    /// the constant product of `reserve_a * reserve_b`.
    #[ink(message)]
    fn get_amount_out(
        &self,
        amount_in: u128,
        reserve_a: u128,
        reserve_b: u128,
    ) -> Result<u128, DexError>;

    /// Returns amount of `A` tokens user has to supply
    /// to get exactly `amount_out` of `B` token while maintaining
    /// pool's constant product.
    #[ink(message)]
    fn get_amount_in(
        &self,
        amount_out: u128,
        reserve_a: u128,
        reserve_b: u128,
    ) -> Result<u128, DexError>;

    /// Returns amounts of tokens received for `amount_in`.
    #[ink(message)]
    fn get_amounts_out(&self, amount_in: u128, path: Vec<AccountId>)
        -> Result<Vec<u128>, DexError>;

    /// Returns amounts of tokens user has to supply.
    #[ink(message)]
    fn get_amounts_in(&self, amount_out: u128, path: Vec<AccountId>)
        -> Result<Vec<u128>, DexError>;
}
