use ink::prelude::vec::Vec;
use ink::primitives::AccountId;

/// An interface to implement by the caller of `Pair::swap` call.
/// 
/// If caller wishes to receive the callback, it must implement this interface.
/// 
/// Note to the implementors:
/// Implementation should ensure that the caller is actually a `Pair` contract instance.
/// For example by calling the caller using `Pair` contract interface and checking that
/// it's an instance of expected token pair in the Factory.
/// 
/// Example:
/// 
/// ```rust
/// fn swap_call(&mut self, sender: AccountId, amount0: u128, amount1: u128, data: Vec<u8>) {
///   let pair: contract_ref!(Pair) = self.env().caller().into();
///   let token_0 = pair.get_token_0();
///   let token_1 = pair.get_token_1();
///   let factory: contract_ref!(Factory) = self.factory.into(); // Needs to know the factory address.
///   assert!(factory.get_pair(token_0, token_1) == self.env().caller());
/// 
///   // rest of the code
/// }`
#[ink::trait_definition]
pub trait SwapCallee {
    #[ink(message)]
    fn swap_call(&mut self, sender: AccountId, amount0: u128, amount1: u128, data: Vec<u8>);
}
