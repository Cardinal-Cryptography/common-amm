#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmStartError {
    StillRunning,
    FarmAlreadyStarted,
    CallerNotOwner,
    InvalidInitParams,
    FarmEndBeforeStart,
    FarmTooLong,
    FarmAlreadyFinished,
    TooManyRewardTokens,
    ZeroRewardAmount,
    ZeroRewardRate,
    InsufficientRewardAmount,
    ArithmeticError,
}

use ink::primitives::{
    AccountId,
    Hash,
};

#[ink::trait_definition]
pub trait Farm {
    /// Starts farm instance.
    #[ink(message)]
    fn start(&mut self, end: u64, reward_tokens: Vec<AccountId>) -> Result<(), FarmStartError>;

    /// Returns address of the token pool for which this farm is created.
    #[ink(message)]
    fn pool_id(&self) -> AccountId;

    /// Returns whether this farm instance is currently running.
    #[ink(message)]
    fn is_running(&self) -> bool;

    /// Returns who is a manager of this farm (created its code instance).
    #[ink(message)]
    fn farm_manager(&self) -> AccountId;

    /// Returns who's the owner of this farm.
    #[ink(message)]
    fn farm_owner(&self) -> AccountId;

    /// Returns farm's code hash.
    #[ink(message)]
    fn code_hash(&self) -> Hash;
}
