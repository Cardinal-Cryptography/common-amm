#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    prelude::vec::Vec,
    primitives::AccountId,
};

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

    /// Withdarws `amount` of shares from `owner`.
    #[ink(message)]
    fn withdraw_shares(
        &mut self,
        account: AccountId,
        amount: u128,
    ) -> Result<(), psp22_traits::PSP22Error>;

    /// Deposits `amount` of shares under `owner` account.
    #[ink(message)]
    fn deposit_shares(&mut self, account: AccountId, amount: u128);

    /// Returns a vector of token addresses which are rewarded for participating in this farm.
    #[ink(message)]
    fn reward_tokens(&self) -> Vec<AccountId>;
}
