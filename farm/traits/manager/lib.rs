#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    prelude::vec::Vec,
    primitives::AccountId,
};


use psp22_traits::PSP22Error;
use amm_helpers::math::MathError;
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmError {
    PSP22Error(PSP22Error),
    FarmAlreadyRunning(AccountId),
    FarmInstantiationFailed,
    CallerNotOwner,
    /// Unknown farm address registered under id.
    FarmNotFound(u32),
    /// Address not registered as a farm.
    FarmUnknown(AccountId),
    ArithmeticError(MathError),
    CallerNotFarmer,
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
    fn total_supply(&self) -> u128;

    /// Returns share of LP tokens deposited by the `account` in this farm.
    #[ink(message)]
    fn balance_of(&self, account: AccountId) -> u128;


    /// Withdraws `amount` of shares from caller.
    #[ink(message)]
    fn withdraw_shares(&mut self, amount: u128)
        -> Result<(), FarmError>;

    /// Deposits `amount` of shares under caller's account.
    #[ink(message)]
    fn deposit_shares(&mut self,  amount: u128) -> Result<(), FarmError>;

    /// Returns a vector of token addresses which are rewarded for participating in this farm.
    #[ink(message)]
    fn reward_tokens(&self) -> Vec<AccountId>;

    // TODO: u64 -> Timestamp (need suitable import)
    #[ink(message)]
    fn owner_start_new_farm(&mut self, start: u64, end: u64, rewards: Vec<u128>) -> Result<(), FarmError>; 

    #[ink(message)]
    fn owner_stop_farm(&mut self) -> Result<(), FarmError>;

    // TODO: AccountId -> TokenId (need suitable import)
    #[ink(message)]
    fn owner_withdraw_token(&mut self, token: AccountId) -> Result<(), FarmError>;


    #[ink(message)]
    fn claim_rewards(&mut self) -> Result<Vec<u128>, FarmError>; 


}