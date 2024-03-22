#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{prelude::vec::Vec, primitives::AccountId};

use amm_helpers::math::MathError;
use psp22::PSP22Error;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmError {
    PSP22Error(PSP22Error),
    FarmIsRunning,
    FarmAlreadyStopped,
    CallerNotOwner,
    ArithmeticError(MathError),
    AllRewardRatesZero,
    FarmStartInThePast,
    FarmEndInThePast,
    FarmDuration,
    InsufficientShares,
    RewardsVecLengthMismatch,
    TooManyRewardTokens,
    RewardTokenIsPoolToken,
    TokenTransferFailed(AccountId, PSP22Error),
    DuplicateRewardTokens,
}

/// Summary of the farm's details.
///
/// Useful for display purposes.
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct FarmDetails {
    /// Address of a DEX pair for which this farm is created.
    pub pool_id: AccountId,
    /// Flag indicating whether the farm is active (currently running or planned for the future).
    pub is_active: bool,
    /// Start timestamp of the latest farm instance.
    pub start: u64,
    /// End timestamp of the latest farm instance.
    pub end: u64,
    /// Vector of PSP22 token addresses that are paid out as rewards.
    pub reward_tokens: Vec<AccountId>,
    /// Vector of rewards rates paid out for locking LP tokens per smallest unit of time.
    pub reward_rates: Vec<u128>,
}

impl From<PSP22Error> for FarmError {
    fn from(e: PSP22Error) -> Self {
        FarmError::PSP22Error(e)
    }
}

impl From<MathError> for FarmError {
    fn from(e: MathError) -> Self {
        FarmError::ArithmeticError(e)
    }
}

#[ink::trait_definition]
pub trait Farm {
    /// Returns address of the token pool for which this farm is created.
    #[ink(message)]
    fn pool_id(&self) -> AccountId;

    /// Returns total supply of LP tokens deposited for this farm.
    #[ink(message)]
    fn total_shares(&self) -> u128;

    /// Returns share of LP tokens deposited by the `account` in this farm.
    #[ink(message)]
    fn shares_of(&self, account: AccountId) -> u128;

    /// Withdraws `amount` of shares from caller's stake in the farm.
    #[ink(message)]
    fn withdraw(&mut self, amount: u128) -> Result<(), FarmError>;

    /// Deposits `amount` of LP tokens (shares) under caller's account in the farm.
    #[ink(message)]
    fn deposit(&mut self, amount: u128) -> Result<(), FarmError>;

    /// Deposits all transferred LP tokens under caller's account.
    #[ink(message)]
    fn deposit_all(&mut self) -> Result<(), FarmError>;

    /// Returns a vector of token addresses which are rewarded for participating in this farm.
    #[ink(message)]
    fn reward_tokens(&self) -> Vec<AccountId>;

    /// Sets the parameters of the farm (`start`, `end`, `rewards`).
    ///
    /// NOTE: Implementation should make sure that it's callable only by an authorized account (owner of the farm).
    #[ink(message)]
    fn owner_start_new_farm(
        &mut self,
        start: u64,
        end: u64,
        rewards: Vec<u128>,
    ) -> Result<(), FarmError>;

    /// Generic method that allows for stopping (a running) farm.
    /// Details are implementation-dependent (Common AMM will set the farm's `end` timestamp to current blocktime).
    ///
    /// NOTE: Implementation should make sure that it's callable only by an authorized account (owner of the farm).
    #[ink(message)]
    fn owner_stop_farm(&mut self) -> Result<(), FarmError>;

    /// NOTE: Implementation should make sure that it's callable only by an authorized account (owner of the farm).
    #[ink(message)]
    fn owner_withdraw_token(&mut self, token: AccountId) -> Result<u128, FarmError>;

    /// Requests farming rewards that have been accumulated to the caller of this method.
    ///
    /// Arguments:
    /// `tokens_indices` - vector of tokens' indices to be claimed.
    ///
    /// NOTE: To acquire token indices, one can query the `view_farm_details`
    ///       and use `reward_tokens` information for that.
    ///       It may happen that one of the reward tokens is malicious and fails during the operation,
    ///       in such case it's advised to filter out that token from the `tokens` list.
    #[ink(message)]
    fn claim_rewards(&mut self, tokens_indices: Vec<u8>) -> Result<Vec<u128>, FarmError>;

    /// Returns information about the current farm instance.
    #[ink(message)]
    fn view_farm_details(&self) -> FarmDetails;
}
