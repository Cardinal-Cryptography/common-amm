use ink::primitives::AccountId;
use traits::RouterV2Error;

pub use crate::pair::*;
pub use crate::stable_pool::*;

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Pool {
    Pair(Pair),
    StablePool(StablePool),
}

impl Pool {
    pub fn try_new(pool_id: AccountId) -> Option<Self> {
        Pair::try_new(pool_id)
            .map(Self::Pair)
            .or(StablePool::try_new(pool_id).map(Self::StablePool))
    }

    pub fn get_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
    ) -> Result<u128, RouterV2Error> {
        match self {
            Pool::Pair(pool) => pool.get_amount_in(token_in, token_out, amount_out),
            Pool::StablePool(pool) => pool.get_amount_in(token_in, token_out, amount_out),
        }
    }

    pub fn get_amount_out(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
    ) -> Result<u128, RouterV2Error> {
        match self {
            Pool::Pair(pool) => pool.get_amount_out(token_in, token_out, amount_in),
            Pool::StablePool(pool) => pool.get_amount_out(token_in, token_out, amount_in),
        }
    }

    pub fn swap(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
        to: AccountId,
    ) -> Result<(), RouterV2Error> {
        match self {
            Pool::Pair(pool) => pool.swap(token_in, token_out, amount_out, to),
            Pool::StablePool(pool) => pool.swap(token_in, token_out, amount_out, to),
        }
    }
}
