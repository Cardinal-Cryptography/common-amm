use crate::MathError;
use amm_helpers::types::WrappedU256;
use ink::prelude::vec::Vec;
use ink::{primitives::AccountId, LangError};
use psp22::PSP22Error;

#[ink::trait_definition]
pub trait Pair {
    /// Returns address of the `Factory` contract that created this pair.
    #[ink(message)]
    fn get_factory(&self) -> AccountId;

    /// Returns the minimum liquidity that must be minted for the add_liquidity to be valid.
    #[ink(message)]
    fn get_minimum_liquidity(&self) -> u128;

    /// Returns amounts of tokens this pair holds and a timestamp.
    ///
    /// NOTE: This does not include the tokens that were transferred to the contract
    /// as part of the _current_ transaction.
    #[ink(message)]
    fn get_reserves(&self) -> (u128, u128, u32);

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
    fn mint(&mut self, to: AccountId) -> Result<u128, PairError>;

    /// Burns liquidity transferred to the contract prior to calling this method.
    /// Tokens resulting from the burning of this liquidity tokens are transferred to
    /// an address controlled by `to` account.
    #[ink(message)]
    fn burn(&mut self, to: AccountId) -> Result<(u128, u128), PairError>;

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
        data: Option<Vec<u8>>,
    ) -> Result<(), PairError>;

    /// Skims the excess of tokens (difference between balance and reserves) and
    /// sends them to an address controlled by `to` account.
    /// This situation happens if, for example, someone sends tokens to the contract
    /// (by mistake). If enough tokens were sent to the contract to trigger overflows,
    /// the `swap` methods could start to fail.
    #[ink(message)]
    fn skim(&mut self, to: AccountId) -> Result<(), PairError>;

    /// Sets the reserves of the contract to its balances providing a graceful recover
    /// in the case that a token asynchronously deflates the balance of a pair.
    // In this case, trades will receive sub-optimal rates.
    #[ink(message)]
    fn sync(&mut self) -> Result<(), PairError>;

    /// Returns address of the first token.
    #[ink(message)]
    fn get_token_0(&self) -> AccountId;

    /// Returns address of the second token.
    #[ink(message)]
    fn get_token_1(&self) -> AccountId;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PairError {
    PSP22Error(PSP22Error),
    LangError(LangError),
    MathError(MathError),
    KInvariantChanged,
    InsufficientLiquidityMinted,
    InsufficientLiquidityBurned,
    InsufficientOutputAmount,
    InsufficientLiquidity,
    InsufficientInputAmount,
    InvalidTo,
    ReservesOverflow,
}

impl From<PSP22Error> for PairError {
    fn from(error: PSP22Error) -> Self {
        PairError::PSP22Error(error)
    }
}

impl From<LangError> for PairError {
    fn from(error: LangError) -> Self {
        PairError::LangError(error)
    }
}

impl From<MathError> for PairError {
    fn from(error: MathError) -> Self {
        PairError::MathError(error)
    }
}
