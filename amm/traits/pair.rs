use crate::DexError;
use amm_helpers::types::WrappedU256;
use ink::primitives::AccountId;

#[ink::trait_definition]
pub trait Pair {
    /// Returns amounts of tokens this pair holds and a timestamp.
    ///
    /// NOTE: This does not include the tokens that were transferred to the contract
    /// as part of the _current_ transaction.
    #[ink(message)]
    fn get_reserves(&self) -> (u128, u128, u64);

    /// Returns cumulative prive of the first token.
    ///
    /// NOTE: Cumulative price is the sum of token price,
    /// recorded at the end of the block (in the last transaction),
    /// since the beginning of the token pair.
    #[ink(message)]
    fn price_0_cumulative_last(&self) -> WrappedU256;

    /// Returns cumulative prive of the second token.
    ///
    /// NOTE: Cumulative price is the sum of token price,
    /// recorded at the end of the block (in the last transaction),
    /// since the beginning of the token pair.
    #[ink(message)]
    fn price_1_cumulative_last(&self) -> WrappedU256;

    /// Mints liquidity tokens `to` account.
    /// The amount minted is equivalent to the excess of contract's balance and reserves.
    #[ink(message)]
    fn mint(&mut self, to: AccountId) -> Result<u128, DexError>;

    /// Burns liquidity transferred to the contract prior to calling this method.
    /// Tokens resulting from the burning of this liquidity tokens are transferred to
    /// an address controlled by `to` account.
    #[ink(message)]
    fn burn(&mut self, to: AccountId) -> Result<(u128, u128), DexError>;

    /// Requests a swap on the token pair, with the outcome amounts equal to
    /// `amount_0_out` and `amount_1_out`. Assumes enough tokens have been transferred
    /// to the contract before calling the method. Tokens are sent to address controlled
    /// by `to` account.
    #[ink(message)]
    fn swap(
        &mut self,
        amount_0_out: u128,
        amount_1_out: u128,
        to: AccountId,
    ) -> Result<(), DexError>;

    /// Skims the excess of tokens (difference between balance and reserves) and
    /// sends them to an address controlled by `to` account.
    /// This situation happens if, for example, someone sends tokens to the contract
    /// (by mistake). If enough tokens were sent to the contract to trigger overflows,
    /// the `swap` methods could start to fail.
    #[ink(message)]
    fn skim(&mut self, to: AccountId) -> Result<(), DexError>;

    /// Sets the reserves of the contract to its balances providing a graceful recover
    /// in the case that a token asynchronously deflates the balance of a pair.
    // In this case, trades will receive sub-optimal rates.
    #[ink(message)]
    fn sync(&mut self) -> Result<(), DexError>;

    /// Returns address of the first token.
    #[ink(message)]
    fn get_token_0(&self) -> AccountId;

    /// Returns address of the second token.
    #[ink(message)]
    fn get_token_1(&self) -> AccountId;
}
