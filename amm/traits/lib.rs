#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod factory;
mod pair;
mod router;
mod swap_callee;

pub type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

pub use factory::{Factory, FactoryError};
pub use pair::{Pair, PairError};
pub use router::{Router, RouterError};
pub use swap_callee::SwapCallee;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MathError {
    AddOverflow(u8),
    CastOverflow(u8),
    DivByZero(u8),
    MulOverflow(u8),
    SubUnderflow(u8),
}
