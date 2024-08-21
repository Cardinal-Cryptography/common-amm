use amm_helpers::{ensure, math::casted_mul};
use ink::{
    contract_ref,
    env::DefaultEnvironment,
    prelude::{vec, vec::Vec},
    primitives::AccountId,
};
use traits::{MathError, Pair, RouterError, StablePool};

const TRADING_FEE_DENOM: u128 = 1000;

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Pool {
    Pair((AccountId, AccountId), AccountId, u8),
    StablePool(Vec<AccountId>, AccountId),
}

impl Pool {
    pub fn get_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
    ) -> Result<u128, RouterError> {
        match self {
            Pool::Pair(_, _, _) => {
                let (reserves, fee) = self.get_reserves_with_fee();
                let reserves = if token_in < token_out {
                    (reserves[0], reserves[1])
                } else {
                    (reserves[1], reserves[0])
                };
                ensure!(amount_out > 0, RouterError::InsufficientAmount);
                ensure!(
                    reserves.0 > 0 && reserves.1 > 0,
                    RouterError::InsufficientLiquidity
                );

                let numerator = casted_mul(reserves.0, amount_out)
                    .checked_mul(TRADING_FEE_DENOM.into())
                    .ok_or(MathError::MulOverflow(14))?;

                let denominator = casted_mul(
                    reserves
                        .1
                        .checked_sub(amount_out)
                        .ok_or(MathError::SubUnderflow(15))?,
                    TRADING_FEE_DENOM - (fee as u128),
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
            Pool::StablePool(_, pool) => {
                let mut pool_contract: contract_ref!(StablePool, DefaultEnvironment) =
                    (*pool).into();
                match pool_contract.get_swap_amount_in(token_in, token_out, amount_out) {
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
    ) -> Result<u128, RouterError> {
        match self {
            Pool::Pair(_, _, _) => {
                let (reserves, fee) = self.get_reserves_with_fee();
                let reserves = if token_in < token_out {
                    (reserves[0], reserves[1])
                } else {
                    (reserves[1], reserves[0])
                };
                ensure!(amount_in > 0, RouterError::InsufficientAmount);
                ensure!(
                    reserves.0 > 0 && reserves.1 > 0,
                    RouterError::InsufficientLiquidity
                );

                // Adjusts for fees paid in the `token_in`.
                let amount_in_with_fee = casted_mul(amount_in, TRADING_FEE_DENOM - (fee as u128));

                let numerator = amount_in_with_fee
                    .checked_mul(reserves.1.into())
                    .ok_or(MathError::MulOverflow(13))?;

                let denominator = casted_mul(reserves.0, TRADING_FEE_DENOM)
                    .checked_add(amount_in_with_fee)
                    .ok_or(MathError::AddOverflow(2))?;

                let amount_out: u128 = numerator
                    .checked_div(denominator)
                    .ok_or(MathError::DivByZero(7))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(4))?;

                Ok(amount_out)
            }
            Pool::StablePool(_, pool) => {
                let mut pool_contract: contract_ref!(StablePool, DefaultEnvironment) =
                    (*pool).into();
                match pool_contract.get_swap_amount_out(token_in, token_out, amount_in) {
                    Ok((amount_out, _)) => Ok(amount_out),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    pub fn get_reserves_with_fee(&self) -> (Vec<u128>, u8) {
        match self {
            Pool::Pair(_, pool, fee) => {
                let pair: contract_ref!(Pair, DefaultEnvironment) = (*pool).into();
                let (reserve_0, reserve_1, _) = pair.get_reserves();
                (vec![reserve_0, reserve_1], *fee)
            }
            Pool::StablePool(_, _) => unimplemented!(),
        }
    }

    pub fn swap(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
        to: AccountId,
    ) -> Result<(), RouterError> {
        ensure!(token_in != token_out, RouterError::IdenticalAddresses);
        match self {
            Pool::Pair(_, pair, _) => {
                let (amount_0_out, amount_1_out) = if token_in < token_out {
                    (0, amount_out)
                } else {
                    (amount_out, 0)
                };

                let mut pair_contract: contract_ref!(Pair, DefaultEnvironment) = (*pair).into();
                pair_contract.swap(amount_0_out, amount_1_out, to, None)?
            }
            Pool::StablePool(_, pool) => {
                let mut pool_contract: contract_ref!(StablePool, DefaultEnvironment) =
                    (*pool).into();
                pool_contract.swap_received(token_in, token_out, amount_out, to)?;
            }
        }
        Ok(())
    }
}
