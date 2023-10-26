#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod factory;
mod pair;
mod router;
mod wnative;

pub use ink::env::DefaultEnvironment as Env;
pub type Balance = <Env as ink::env::Environment>::Balance;
pub type Timestamp = <Env as ink::env::Environment>::Timestamp;

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
