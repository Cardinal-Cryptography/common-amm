#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod errors;
mod factory;
mod pair;
mod router;
mod wnative;

pub type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

pub use errors::DexError;
pub use factory::Factory;
pub use pair::Pair;
pub use router::Router;
pub use wnative::Wnative;
