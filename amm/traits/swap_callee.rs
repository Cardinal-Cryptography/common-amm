use ink::prelude::vec::Vec;
use ink::primitives::AccountId;

#[ink::trait_definition]
pub trait SwapCallee {
    #[ink(message)]
    fn swap_call(&mut self, sender: AccountId, amount0: u128, amount1: u128, data: Vec<u8>);
}
