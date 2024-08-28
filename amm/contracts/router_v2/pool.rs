use amm_helpers::{ensure, math::casted_mul};
use ink::primitives::AccountId;
use traits::{MathError, Pair as PairTrait, RouterV2Error, StablePool as StablePoolTrait};

const PAIR_TRADING_FEE_DENOM: u128 = 1000;

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
    pub fn get_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
    ) -> Result<u128, RouterV2Error> {
        match self {
            Pool::Pair(pool) => {
                let (reserve_in, reserve_out) = pool.get_reserves(token_in, token_out);
                ensure!(amount_out > 0, RouterV2Error::InsufficientAmount);
                ensure!(
                    reserve_in > 0 && reserve_out > 0,
                    RouterV2Error::InsufficientLiquidity
                );

                let numerator = casted_mul(reserve_in, amount_out)
                    .checked_mul(PAIR_TRADING_FEE_DENOM.into())
                    .ok_or(MathError::MulOverflow(14))?;

                let denominator = casted_mul(
                    reserve_out
                        .checked_sub(amount_out)
                        .ok_or(MathError::SubUnderflow(15))?,
                    PAIR_TRADING_FEE_DENOM - (pool.fee() as u128),
                );

                let amount_in: u128 = numerator
                    .checked_div(denominator)
                    .ok_or(MathError::DivByZero(8))?
                    .checked_add(1.into())
                    .ok_or(MathError::AddOverflow(3))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(5))?;

                Ok(amount_in)
            }
            Pool::StablePool(pool) => {
                match pool
                    .contract_ref()
                    .get_swap_amount_in(token_in, token_out, amount_out)
                {
                    Ok((amount_in, _)) => Ok(amount_in),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn get_amount_out(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
    ) -> Result<u128, RouterV2Error> {
        match self {
            Pool::Pair(pool) => {
                let (reserve_in, reserve_out) = pool.get_reserves(token_in, token_out);
                ensure!(amount_in > 0, RouterV2Error::InsufficientAmount);
                ensure!(
                    reserve_in > 0 && reserve_out > 0,
                    RouterV2Error::InsufficientLiquidity
                );

                // Adjusts for fees paid in the `token_in`.
                let amount_in_with_fee =
                    casted_mul(amount_in, PAIR_TRADING_FEE_DENOM - (pool.fee() as u128));

                let numerator = amount_in_with_fee
                    .checked_mul(reserve_out.into())
                    .ok_or(MathError::MulOverflow(13))?;

                let denominator = casted_mul(reserve_in, PAIR_TRADING_FEE_DENOM)
                    .checked_add(amount_in_with_fee)
                    .ok_or(MathError::AddOverflow(2))?;

                let amount_out: u128 = numerator
                    .checked_div(denominator)
                    .ok_or(MathError::DivByZero(7))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(4))?;

                Ok(amount_out)
            }
            Pool::StablePool(pool) => {
                match pool
                    .contract_ref()
                    .get_swap_amount_out(token_in, token_out, amount_in)
                {
                    Ok((amount_out, _)) => Ok(amount_out),
                    Err(err) => Err(err.into()),
                }
            }
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
            Pool::Pair(pool) => {
                let (amount_0_out, amount_1_out) = if token_in < token_out {
                    (0, amount_out)
                } else {
                    (amount_out, 0)
                };
                pool.contract_ref()
                    .swap(amount_0_out, amount_1_out, to, None)?;
            }
            Pool::StablePool(pool) => {
                pool.contract_ref()
                    .swap_received(token_in, token_out, amount_out, to)?;
            }
        }
        Ok(())
    }

    pub fn pool_id(&self) -> AccountId {
        match self {
            Pool::Pair(pool) => pool.pool_id(),
            Pool::StablePool(pool) => pool.pool_id(),
        }
    }
}
