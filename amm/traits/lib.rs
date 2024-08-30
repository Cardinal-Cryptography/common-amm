#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod factory;
mod ownable2step;
mod pair;
mod rate_provider;
mod router;
mod router_v2;
mod stable_pool;
mod swap_callee;

pub type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

pub use amm_helpers::math::MathError;
pub use factory::{Factory, FactoryError};
pub use ownable2step::{Ownable2Step, Ownable2StepData, Ownable2StepError, Ownable2StepResult};
pub use pair::{Pair, PairError};
pub use rate_provider::RateProvider;
pub use router::{Router, RouterError};
pub use router_v2::{RouterV2, RouterV2Error, Step};
pub use stable_pool::{StablePool, StablePoolError};
pub use swap_callee::SwapCallee;
