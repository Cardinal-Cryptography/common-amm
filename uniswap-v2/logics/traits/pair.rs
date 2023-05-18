use ink::LangError;
use openbrush::{
    contracts::{
        reentrancy_guard::*,
        traits::{
            ownable::*,
            psp22::PSP22Error,
        },
    },
    traits::{
        AccountId,
        Balance,
        Timestamp,
    },
};

use super::types::WrappedU256;

#[openbrush::wrapper]
pub type PairRef = dyn Pair;

#[openbrush::trait_definition]
pub trait Pair {
    /// Returns amounts of tokens this pair holds at `Timestamp`.
    ///
    /// NOTE: This does not include the tokens that were transferred to the contract
    /// as part of the _current_ transaction.
    #[ink(message)]
    fn get_reserves(&self) -> (Balance, Balance, Timestamp);

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

    /// Initializes the pair with given token IDs.
    ///
    /// NOTE: Why do we need it at all? Why not put in the constructor?
    /// Potentialy dangerous in case of a hack where initial caller/owner
    /// of the contract can re-initialize with a different pair.
    #[ink(message)]
    fn initialize(&mut self, token_0: AccountId, token_1: AccountId) -> Result<(), PairError>;

    /// Mints liquidity tokens `to` account.
    /// The amount minted is equivalent to the excess of contract's balance and reserves.
    #[ink(message)]
    fn mint(&mut self, to: AccountId) -> Result<Balance, PairError>;

    #[ink(message)]
    fn burn(&mut self, to: AccountId) -> Result<(Balance, Balance), PairError>;

    #[ink(message)]
    fn swap(
        &mut self,
        amount_0_out: Balance,
        amount_1_out: Balance,
        to: AccountId,
    ) -> Result<(), PairError>;

    #[ink(message)]
    fn skim(&mut self, to: AccountId) -> Result<(), PairError>;

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
    OwnableError(OwnableError),
    ReentrancyGuardError(ReentrancyGuardError),
    LangError(LangError),
    TransferError,
    K,
    InsufficientLiquidityMinted,
    InsufficientLiquidityBurned,
    InsufficientOutputAmount,
    InsufficientLiquidity,
    InsufficientInputAmount,
    SafeTransferFailed,
    InvalidTo,
    Overflow,
    Locked,
    SubUnderFlow1,
    SubUnderFlow2,
    SubUnderFlow3,
    SubUnderFlow4,
    SubUnderFlow5,
    SubUnderFlow6,
    SubUnderFlow7,
    SubUnderFlow8,
    SubUnderFlow9,
    SubUnderFlow10,
    SubUnderFlow11,
    SubUnderFlow12,
    SubUnderFlow13,
    SubUnderFlow14,
    MulOverFlow1,
    MulOverFlow2,
    MulOverFlow3,
    MulOverFlow4,
    MulOverFlow5,
    MulOverFlow6,
    MulOverFlow7,
    MulOverFlow8,
    MulOverFlow9,
    MulOverFlow10,
    MulOverFlow11,
    MulOverFlow12,
    MulOverFlow13,
    MulOverFlow14,
    DivByZero1,
    DivByZero2,
    DivByZero3,
    DivByZero4,
    DivByZero5,
    AddOverflow1,
    CastOverflow1,
    CastOverflow2,
}

impl From<OwnableError> for PairError {
    fn from(error: OwnableError) -> Self {
        PairError::OwnableError(error)
    }
}

impl From<PSP22Error> for PairError {
    fn from(error: PSP22Error) -> Self {
        PairError::PSP22Error(error)
    }
}

impl From<ReentrancyGuardError> for PairError {
    fn from(error: ReentrancyGuardError) -> Self {
        PairError::ReentrancyGuardError(error)
    }
}

impl From<LangError> for PairError {
    fn from(error: LangError) -> Self {
        PairError::LangError(error)
    }
}
