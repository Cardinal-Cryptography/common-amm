use amm_helpers::{ensure, math::casted_mul};
use ink::{contract_ref, env::DefaultEnvironment, primitives::AccountId};
use traits::{MathError, Pair, RouterV2Error, StablePool};

const PAIR_TRADING_FEE_DENOM: u128 = 1000;

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub enum Pool {
    Pair(AccountId, u8),
    StablePool(AccountId),
}

impl Pool {
    pub fn get_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
    ) -> Result<u128, RouterV2Error> {
        match self {
            Pool::Pair(pair, fee) => {
                let (reserve_in, reserve_out) = get_pair_reserves(*pair, token_in, token_out);
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
                    PAIR_TRADING_FEE_DENOM - (*fee as u128),
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
                match stable_pool_ref(*pool).get_swap_amount_in(token_in, token_out, amount_out) {
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
            Pool::Pair(pair, fee) => {
                let (reserve_in, reserve_out) = get_pair_reserves(*pair, token_in, token_out);
                ensure!(amount_in > 0, RouterV2Error::InsufficientAmount);
                ensure!(
                    reserve_in > 0 && reserve_out > 0,
                    RouterV2Error::InsufficientLiquidity
                );

                // Adjusts for fees paid in the `token_in`.
                let amount_in_with_fee =
                    casted_mul(amount_in, PAIR_TRADING_FEE_DENOM - (*fee as u128));

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
                let mut pool_contract: contract_ref!(StablePool, DefaultEnvironment) =
                    (*pool).into();
                match pool_contract.get_swap_amount_out(token_in, token_out, amount_in) {
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
        ensure!(token_in != token_out, RouterV2Error::IdenticalAddresses);
        match self {
            Pool::Pair(pair, _) => {
                let (amount_0_out, amount_1_out) = if token_in < token_out {
                    (0, amount_out)
                } else {
                    (amount_out, 0)
                };

                pair_ref(*pair).swap(amount_0_out, amount_1_out, to, None)?
            }
            Pool::StablePool(pool) => {
                stable_pool_ref(*pool).swap_received(token_in, token_out, amount_out, to)?;
            }
        }
        Ok(())
    }
}

/// Calculates `X` so `X / reserve_1 == amount_0 / reserve_0`
fn get_propotional_amount(
    amount_0: u128,
    reserve_0: u128,
    reserve_1: u128,
) -> Result<u128, RouterV2Error> {
    let amount_1: u128 = casted_mul(amount_0, reserve_1)
        .checked_div(reserve_0.into())
        .ok_or(MathError::DivByZero(6))?
        .try_into()
        .map_err(|_| MathError::CastOverflow(3))?;

    Ok(amount_1)
}

/// Makes a cross-contract call to fetch `pair`'s reserves.
/// Returns reserves `(reserve_0, reserve_1)` in order of `token_0` and `token_1`
fn get_pair_reserves(pair: AccountId, token_0: AccountId, token_1: AccountId) -> (u128, u128) {
    let (reserve_0, reserve_1, _) = pair_ref(pair).get_reserves();
    if token_0 < token_1 {
        (reserve_0, reserve_1)
    } else {
        (reserve_1, reserve_0)
    }
}

/// Calculates optimal amounts for `Pair` liquidity deposit.
pub fn calculate_pair_liquidity(
    pair: AccountId,
    token_0: AccountId,
    token_1: AccountId,
    amount_0_desired: u128,
    amount_1_desired: u128,
    amount_0_min: u128,
    amount_1_min: u128,
) -> Result<(u128, u128), RouterV2Error> {
    let (reserve_0, reserve_1) = get_pair_reserves(pair, token_0, token_1);

    if reserve_0 == 0 && reserve_1 == 0 {
        return Ok((amount_0_desired, amount_1_desired));
    }

    ensure!(
        reserve_0 > 0 && reserve_1 > 0,
        RouterV2Error::InsufficientLiquidity
    );
    ensure!(
        amount_0_desired > 0 && amount_1_desired > 0,
        RouterV2Error::InsufficientAmount
    );

    let amount_1_optimal = get_propotional_amount(amount_0_desired, reserve_0, reserve_1)?;
    if amount_1_optimal <= amount_1_desired {
        ensure!(
            amount_1_optimal >= amount_1_min,
            RouterV2Error::InsufficientAmountB
        );
        Ok((amount_0_desired, amount_1_optimal))
    } else {
        let amount_0_optimal = get_propotional_amount(amount_1_desired, reserve_1, reserve_0)?;
        // amount_0_optimal <= amount_0_desired holds as amount_1_optimal > amount_1_desired
        ensure!(
            amount_0_optimal >= amount_0_min,
            RouterV2Error::InsufficientAmountA
        );
        Ok((amount_0_optimal, amount_1_desired))
    }
}

#[inline]
pub fn pair_ref(pair: AccountId) -> contract_ref!(Pair, DefaultEnvironment) {
    pair.into()
}

#[inline]
fn stable_pool_ref(pool: AccountId) -> contract_ref!(StablePool, DefaultEnvironment) {
    pool.into()
}
