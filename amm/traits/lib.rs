#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod factory;
mod pair;
mod router;
mod wnative;

pub type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

pub use factory::{
    Factory,
    FactoryError,
};
pub use pair::{
    Pair,
    PairError,
};
pub use router::{
    Router,
    RouterError,
};
pub use wnative::Wnative;
