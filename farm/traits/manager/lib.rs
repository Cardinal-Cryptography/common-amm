#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    prelude::vec::Vec,
    primitives::AccountId,
};

use farm_instance_trait::FarmStartError;
use psp22_traits::PSP22Error;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmManagerError {
    FarmStartError(FarmStartError),
    PSP22Error(PSP22Error),
    FarmAlreadyRunning(AccountId),
    FarmInstantiationFailed,
    CallerNotOwner,
    /// Unknown farm address registered under id.
    FarmNotFound(u32),
    /// Address not registered as a farm.
    FarmUnknown(AccountId),
}

impl From<FarmStartError> for FarmManagerError {
    fn from(error: FarmStartError) -> Self {
        FarmManagerError::FarmStartError(error)
    }
}

impl From<PSP22Error> for FarmManagerError {
    fn from(error: PSP22Error) -> Self {
        FarmManagerError::PSP22Error(error)
    }
}

#[ink::trait_definition]
pub trait FarmManager {
    /// Returns address of the token pool for which this farm is created.
    #[ink(message)]
    fn pool_id(&self) -> AccountId;

    /// Returns total supply of LP tokens deposited for this farm.
    #[ink(message)]
    fn total_supply(&self) -> u128;

    /// Returns share of LP tokens deposited by the `owner` in this farm.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> u128;

    /// Returns an address of the latest farm instance.
    #[ink(message)]
    fn latest_farm_id(&self) -> Option<AccountId>;

    /// Returns an address of the farm registered under `farm_id`.
    #[ink(message)]
    fn get_farm_address(&self, farm_id: u32) -> Option<AccountId>;

    /// Withdraws `amount` of shares from `owner`.
    ///
    /// NOTE: Must be called only be farm instances, never directly,
    /// at correct moments. Otherwise LP providers will miss some of the rewards.
    /// Implementation should return error if `caller != known farm instance`.
    #[ink(message)]
    fn withdraw_shares(&mut self, account: AccountId, amount: u128)
        -> Result<(), FarmManagerError>;

    /// Deposits `amount` of shares under `owner` account.
    ///
    /// NOTE: Must be called only be farm instances, never directly,
    /// at correct moments. Otherwise LP providers will miss some of the rewards.
    /// Implementation should return error if `caller != known farm instance`.
    #[ink(message)]
    fn deposit_shares(&mut self, account: AccountId, amount: u128) -> Result<(), FarmManagerError>;

    /// Returns a vector of token addresses which are rewarded for participating in this farm.
    #[ink(message)]
    fn reward_tokens(&self) -> Vec<AccountId>;

    /// Creates an instance of the farm that ends at `end`` with `rewards` for participating.
    /// Returns address of the created farm instance.
    /// Should return error if caller is not the owner of the manager contract.
    #[ink(message)]
    fn instantiate_farm(
        &mut self,
        end: u64,
        rewards: Vec<u128>,
    ) -> Result<AccountId, FarmManagerError>;
}
