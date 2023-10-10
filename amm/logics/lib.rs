#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

pub mod helpers;
pub mod traits;

pub use ink::env::DefaultEnvironment as Env;
pub type Balance = <Env as ink::env::Environment>::Balance;
pub type Timestamp = <Env as ink::env::Environment>::Timestamp;
