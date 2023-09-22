#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    env::{
        DefaultEnvironment,
        Environment,
    },
    primitives::{
        AccountId,
        Hash,
    },
};

pub type Balance = <DefaultEnvironment as Environment>::Balance;

#[ink::trait_definition]
pub trait Farm {
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
