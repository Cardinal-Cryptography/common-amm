pub mod fees;

use crate::{
    constants::stable_pool::RATE_PRECISION,
    math::{casted_mul, MathError},
};
use ink::prelude::vec::Vec;
use primitive_types::U256;

use fees::Fees;

/// Max number of iterations performed in Newton–Raphson method
const MAX_ITERATIONS: u8 = 255;

fn amount_to_rated(amount: u128, scaled_rate: u128) -> Result<u128, MathError> {
    casted_mul(amount, scaled_rate)
        .checked_div(U256::from(RATE_PRECISION))
        .unwrap()
        .try_into()
        .map_err(|_| MathError::CastOverflow(120))
}

fn amounts_to_rated(amounts: &[u128], scaled_rates: &[u128]) -> Result<Vec<u128>, MathError> {
    amounts
        .iter()
        .zip(scaled_rates.iter())
        .map(|(amount, &rate)| amount_to_rated(*amount, rate))
        .collect()
}

fn amount_from_rated(amount: u128, scaled_rate: u128) -> Result<u128, MathError> {
    casted_mul(amount, RATE_PRECISION)
        .checked_div(U256::from(scaled_rate))
        .unwrap()
        .try_into()
        .map_err(|_| MathError::CastOverflow(121))
}

/// Computes stable swap invariant (D)
fn compute_d(amounts: &Vec<u128>, amp_coef: u128) -> Result<U256, MathError> {
    // SUM{x_i}
    let amount_sum = amounts.iter().try_fold(U256::from(0), |acc, &amount| {
        acc.checked_add(amount.into())
            .ok_or(MathError::AddOverflow(1))
    })?;
    if amount_sum == 0.into() {
        Ok(0.into())
    } else {
        let n = amounts.len() as u32;
        // n^n
        let nn = n.checked_pow(n).ok_or(MathError::MulOverflow(1))?;
        // A * n^n
        let ann: U256 = casted_mul(amp_coef, nn.into());
        // A * n^n * SUM{x_i}
        let ann_sum = ann
            .checked_mul(amount_sum)
            .ok_or(MathError::MulOverflow(2))?;
        // A * n^n - 1
        let ann_sub_one = ann
            .checked_sub(1.into())
            .ok_or(MathError::SubUnderflow(1))?;
        // n + 1
        let n_add_one = n.checked_add(1).ok_or(MathError::AddOverflow(2))?;
        let mut d = amount_sum;
        // Computes next D unitl satisfying precision is reached
        for _ in 0..MAX_ITERATIONS {
            let d_next = compute_d_next(d, n, nn, amounts, ann_sum, ann_sub_one, n_add_one)?;
            if d_next.abs_diff(d) <= 1.into() {
                return Ok(d);
            }
            d = d_next;
        }
        Err(MathError::Precision(1))
    }
}

/// Computes next step's approximation of D in Newton-Raphson method.
/// Returns d_next, or error if any math error occurred.
fn compute_d_next(
    d_prev: U256,
    n: u32,
    nn: u32,
    amounts: &Vec<u128>,
    ann_sum: U256,
    ann_sub_one: U256,
    n_add_one: u32,
) -> Result<U256, MathError> {
    let mut d_prod = d_prev;
    // d_prod = d_prev^(n+1) / (n^n * Prod{amounts_i})
    for &amount in amounts {
        d_prod = d_prod
            .checked_mul(d_prev)
            .ok_or(MathError::MulOverflow(3))?
            .checked_div(amount.into())
            .ok_or(MathError::DivByZero(1))?;
    }
    d_prod = d_prod.checked_div(nn.into()).unwrap();
    let numerator = d_prev
        .checked_mul(
            d_prod
                .checked_mul(n.into())
                .ok_or(MathError::MulOverflow(5))?
                .checked_add(ann_sum)
                .ok_or(MathError::AddOverflow(3))?,
        )
        .ok_or(MathError::MulOverflow(6))?;
    let denominator = d_prev
        .checked_mul(ann_sub_one)
        .ok_or(MathError::MulOverflow(7))?
        .checked_add(
            d_prod
                .checked_mul(n_add_one.into())
                .ok_or(MathError::MulOverflow(8))?,
        )
        .ok_or(MathError::AddOverflow(4))?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::DivByZero(2))
}

/// Returns new reserve of `token_y_id`
/// given new reserve of `token_x_id`.
///
/// NOTE: it does not check if `token_x_id` != `token_y_id` and if tokens' `id`s are out of bounds
fn compute_y(
    new_reserve_x: u128,
    reserves: &Vec<u128>,
    token_x_id: usize,
    token_y_id: usize,
    amp_coef: u128,
) -> Result<u128, MathError> {
    let n = reserves.len() as u32;
    let ann: U256 = casted_mul(
        amp_coef,
        n.checked_pow(n).ok_or(MathError::MulOverflow(9))?.into(),
    );
    let d: U256 = compute_d(reserves, amp_coef)?;

    let mut c = d
        .checked_mul(d)
        .ok_or(MathError::MulOverflow(10))?
        .checked_div(new_reserve_x.into())
        .ok_or(MathError::DivByZero(3))?;
    let mut reserves_sum: U256 = new_reserve_x.into();
    // reserves_sum = ... + x_(i') + ...
    // c1 = ... * d / x_(i') * ... * d
    // where  i' in (0,n) AND i' != token_y_id
    for (idx, &reserve) in reserves.iter().enumerate() {
        if idx != token_x_id && idx != token_y_id {
            reserves_sum = reserves_sum
                .checked_add(reserve.into())
                .ok_or(MathError::AddOverflow(5))?;
            c = c
                .checked_mul(d)
                .ok_or(MathError::MulOverflow(11))?
                .checked_div(reserve.into())
                .ok_or(MathError::DivByZero(4))?;
        }
    }
    // c = c_1 * d / (A * n^2n)
    c = c
        .checked_mul(d)
        .ok_or(MathError::MulOverflow(12))?
        .checked_div(
            ann.checked_mul((n).checked_pow(n).ok_or(MathError::MulOverflow(13))?.into())
                .ok_or(MathError::MulOverflow(14))?,
        )
        .ok_or(MathError::DivByZero(5))?;
    // reserves_sum + d / ( A * n^n)
    let b: U256 = d
        .checked_div(ann)
        .ok_or(MathError::DivByZero(6))?
        .checked_add(reserves_sum)
        .ok_or(MathError::AddOverflow(6))?; // d will be subtracted later

    let mut y_prev = d;
    for _ in 0..MAX_ITERATIONS {
        let y = compute_y_next(y_prev, b, c, d)?;
        if y.abs_diff(y_prev) <= 1.into() {
            return y.try_into().map_err(|_| MathError::CastOverflow(11));
        }
        y_prev = y;
    }
    Err(MathError::Precision(2))
}

fn compute_y_next(y_prev: U256, b: U256, c: U256, d: U256) -> Result<U256, MathError> {
    let numerator = y_prev
        .checked_pow(2.into())
        .ok_or(MathError::MulOverflow(15))?
        .checked_add(c)
        .ok_or(MathError::AddOverflow(7))?;
    let denominator = y_prev
        .checked_mul(2.into())
        .ok_or(MathError::MulOverflow(16))?
        .checked_add(b)
        .ok_or(MathError::AddOverflow(8))?
        .checked_sub(d)
        .ok_or(MathError::SubUnderflow(6))?;
    numerator
        .checked_div(denominator)
        .ok_or(MathError::DivByZero(7))
}

/// Compute swap result after an exchange given `token_amount_in` of the `token_in_id`.
/// panics if token ids are out of bounds.
/// Returns a tuple of (amount out, fee amount)
/// NOTE: it does not check if `token_in_id` != `token_out_id`.
fn swap_to(
    token_in_id: usize,
    token_in_amount: u128,
    token_out_id: usize,
    current_reserves: &Vec<u128>,
    fees: &Fees,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let y = compute_y(
        token_in_amount
            .checked_add(current_reserves[token_in_id])
            .ok_or(MathError::AddOverflow(9))?,
        current_reserves,
        token_in_id,
        token_out_id,
        amp_coef,
    )?;
    // sub 1 in case there are any rounding errors
    // https://github.com/curvefi/curve-contract/blob/b0bbf77f8f93c9c5f4e415bce9cd71f0cdee960e/contracts/pool-templates/base/SwapTemplateBase.vy#L466
    let dy = current_reserves[token_out_id]
        .checked_sub(y)
        .ok_or(MathError::SubUnderflow(7))?
        .checked_sub(1)
        .ok_or(MathError::SubUnderflow(8))?;
    // fees are applied to "token_out" amount
    let fee = fees.trade_fee_from_gross(dy)?;
    let amount_swapped = dy.checked_sub(fee).ok_or(MathError::SubUnderflow(9))?;

    Ok((amount_swapped, fee))
}

pub fn rated_swap_to(
    rates: &[u128],
    token_in_id: usize,
    token_in_amount: u128,
    token_out_id: usize,
    current_reserves: &[u128],
    fees: &Fees,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let r_token_in_amount = amount_to_rated(token_in_amount, rates[token_in_id])?;
    let r_current_reserves = amounts_to_rated(current_reserves, rates)?;

    let (r_amount_swapped, r_fee) = swap_to(
        token_in_id,
        r_token_in_amount,
        token_out_id,
        &r_current_reserves,
        fees,
        amp_coef,
    )?;

    let amount_swapped = amount_from_rated(r_amount_swapped, rates[token_out_id])?;
    let fee = amount_from_rated(r_fee, rates[token_out_id])?;
    Ok((amount_swapped, fee))
}

/// Compute swap result after an exchange given `token_amount_out` of the `token_out_id`
/// panics if token ids are out of bounds
/// Returns a tuple (amount in, fee amount)
/// NOTE: it does not check if `token_in_id` != `token_out_id`
fn swap_from(
    token_in_id: usize,
    token_out_amount: u128, // Net amount (w/o fee)
    token_out_id: usize,
    current_reserves: &Vec<u128>,
    fees: &Fees,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    // fees are applied to "token_out" amount
    let fee = fees.trade_fee_from_net(token_out_amount)?;
    let token_out_amount_plus_fee = token_out_amount
        .checked_add(fee)
        .ok_or(MathError::AddOverflow(11))?;

    let y = compute_y(
        current_reserves[token_out_id]
            .checked_sub(token_out_amount_plus_fee)
            .ok_or(MathError::SubUnderflow(12))?,
        current_reserves,
        token_out_id,
        token_in_id,
        amp_coef,
    )?;
    let dy: u128 = y
        .checked_sub(current_reserves[token_in_id])
        .ok_or(MathError::SubUnderflow(13))?;

    Ok((dy, fee))
}

pub fn rated_swap_from(
    rates: &[u128],
    token_in_id: usize,
    token_out_amount: u128,
    token_out_id: usize,
    current_reserves: &[u128],
    fees: &Fees,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let r_token_out_amount = amount_to_rated(token_out_amount, rates[token_out_id])?;
    let r_current_reserves = amounts_to_rated(current_reserves, rates)?;
    let (r_dy, r_fee) = swap_from(
        token_in_id,
        r_token_out_amount,
        token_out_id,
        &r_current_reserves,
        fees,
        amp_coef,
    )?;
    // add one in case of rounding error, for the protocol advantage
    let dy = amount_from_rated(r_dy, rates[token_in_id])?
        .checked_add(1)
        .ok_or(MathError::AddOverflow(12))?;
    let fee = amount_from_rated(r_fee, rates[token_out_id])?;
    Ok((dy, fee))
}

/// Given `deposit_amounts` user want deposit, calculates how many lpt
/// are required to be minted.
/// Returns a tuple of (lpt to mint, fee)
fn compute_lp_amount_for_deposit(
    deposit_amounts: &Vec<u128>,
    old_reserves: &Vec<u128>,
    pool_token_supply: u128,
    fees: Option<&Fees>,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    if pool_token_supply == 0 {
        if deposit_amounts.contains(&0) {
            return Err(MathError::DivByZero(8));
        }
        Ok((
            compute_d(deposit_amounts, amp_coef)?
                .try_into()
                .map_err(|_| MathError::CastOverflow(1))?,
            0,
        ))
    } else {
        // Initial invariant
        let d_0 = compute_d(old_reserves, amp_coef)?;
        let n_coins = old_reserves.len() as u32;
        let mut new_reserves = old_reserves
            .iter()
            .zip(deposit_amounts.iter())
            .map(|(reserve, &amount)| {
                reserve
                    .checked_add(amount)
                    .ok_or(MathError::AddOverflow(14))
            })
            .collect::<Result<Vec<u128>, MathError>>()?;
        // Invariant after change
        let d_1 = compute_d(&new_reserves, amp_coef)?;
        if let Some(_fees) = fees {
            // Recalculate the invariant accounting for fees
            for i in 0..new_reserves.len() {
                let ideal_reserve: u128 = d_1
                    .checked_mul(old_reserves[i].into())
                    .ok_or(MathError::MulOverflow(17))?
                    .checked_div(d_0)
                    .ok_or(MathError::DivByZero(9))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(2))?;
                let difference = ideal_reserve.abs_diff(new_reserves[i]);
                let fee = _fees.normalized_trade_fee(n_coins, difference)?;
                new_reserves[i] = new_reserves[i]
                    .checked_sub(fee)
                    .ok_or(MathError::SubUnderflow(18))?;
            }
            let d_2: U256 = compute_d(&new_reserves, amp_coef)?;
            let mint_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_2.checked_sub(d_0).ok_or(MathError::SubUnderflow(19))?)
                .ok_or(MathError::MulOverflow(18))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(10))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(3))?;

            let diff_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_1.checked_sub(d_0).ok_or(MathError::SubUnderflow(20))?)
                .ok_or(MathError::MulOverflow(19))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(11))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(4))?;
            // d1 > d2 > d0,
            // (d2-d0) => mint_shares (charged fee),
            // (d1-d0) => diff_shares (without fee),
            // (d1-d2) => fee part,
            // diff_shares = mint_shares + fee part
            Ok((
                mint_shares,
                diff_shares
                    .checked_sub(mint_shares)
                    .ok_or(MathError::SubUnderflow(21))?,
            ))
        } else {
            // Calc without fees
            let mint_shares: u128 = U256::from(pool_token_supply)
                .checked_mul(d_1.checked_sub(d_0).ok_or(MathError::SubUnderflow(22))?)
                .ok_or(MathError::MulOverflow(20))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(12))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(5))?;
            // d1 > d0,
            // (d1-d0) => mint_shares
            Ok((mint_shares, 0))
        }
    }
}

pub fn rated_compute_lp_amount_for_deposit(
    rates: &[u128],
    deposit_amounts: &[u128],
    old_reserves: &[u128],
    pool_token_supply: u128,
    fees: Option<&Fees>,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let r_deposit_amounts = amounts_to_rated(deposit_amounts, rates)?;
    let r_old_reserves = amounts_to_rated(old_reserves, rates)?;

    compute_lp_amount_for_deposit(
        &r_deposit_amounts,
        &r_old_reserves,
        pool_token_supply,
        fees,
        amp_coef,
    )
}

/// Computes proportional token amounts to the given `lpt_amount`.
pub fn compute_amounts_given_lp(
    lpt_amount: u128,
    reserves: &Vec<u128>,
    pool_token_supply: u128,
) -> Result<Vec<u128>, MathError> {
    let mut amounts = Vec::with_capacity(reserves.len());
    for &reserve in reserves {
        amounts.push(
            casted_mul(reserve, lpt_amount)
                .checked_div(pool_token_supply.into())
                .ok_or(MathError::DivByZero(13))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(6))?,
        );
    }
    Ok(amounts)
}

/// Given `withdraw_amounts` user want get, calculates how many lpt
/// are required to be burnt
/// Returns a tuple of (lpt to burn, fee part)
fn compute_lp_amount_for_withdraw(
    withdraw_amounts: &[u128],
    old_reserves: &Vec<u128>,
    pool_token_supply: u128,
    fees: Option<&Fees>,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let n_coins = old_reserves.len() as u32;
    // Initial invariant, D0
    let d_0 = compute_d(old_reserves, amp_coef)?;

    // real invariant after withdraw, D1
    let mut new_reserves = old_reserves
        .iter()
        .zip(withdraw_amounts.iter())
        .map(|(reserve, &amount)| {
            reserve
                .checked_sub(amount)
                .ok_or(MathError::SubUnderflow(14))
        })
        .collect::<Result<Vec<u128>, MathError>>()?;
    let d_1 = compute_d(&new_reserves, amp_coef)?;

    // Recalculate the invariant accounting for fees
    if let Some(_fees) = fees {
        for i in 0..new_reserves.len() {
            let ideal_reserve: u128 = d_1
                .checked_mul(old_reserves[i].into())
                .ok_or(MathError::MulOverflow(22))?
                .checked_div(d_0)
                .ok_or(MathError::DivByZero(14))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(7))?;
            let difference = ideal_reserve.abs_diff(new_reserves[i]);
            let fee = _fees.normalized_trade_fee(n_coins, difference)?;
            // new_reserves is for calculation D2, the one with fee charged
            new_reserves[i] = new_reserves[i]
                .checked_sub(fee)
                .ok_or(MathError::SubUnderflow(27))?;
        }
        let d_2 = compute_d(&new_reserves, amp_coef)?;
        // d0 > d1 > d2,
        // (d0-d2) => burn_shares (plus fee),
        // (d0-d1) => diff_shares (without fee),
        // (d1-d2) => fee part,
        // burn_shares = diff_shares + fee part

        let burn_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_2).ok_or(MathError::SubUnderflow(28))?)
            .ok_or(MathError::MulOverflow(23))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(15))?
            .try_into()
            .map_err(|_| MathError::CastOverflow(8))?;
        let diff_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_1).ok_or(MathError::SubUnderflow(29))?)
            .ok_or(MathError::MulOverflow(24))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(16))?
            .try_into()
            .map_err(|_| MathError::CastOverflow(9))?;
        Ok((
            burn_shares,
            burn_shares
                .checked_sub(diff_shares)
                .ok_or(MathError::SubUnderflow(30))?,
        ))
    } else {
        let burn_shares = U256::from(pool_token_supply)
            .checked_mul(d_0.checked_sub(d_1).ok_or(MathError::SubUnderflow(31))?)
            .ok_or(MathError::MulOverflow(25))?
            .checked_div(d_0)
            .ok_or(MathError::DivByZero(17))?
            .try_into()
            .map_err(|_| MathError::CastOverflow(10))?;
        Ok((burn_shares, 0))
    }
}

pub fn rated_compute_lp_amount_for_withdraw(
    rates: &[u128],
    withdraw_amounts: &[u128],
    old_reserves: &[u128],
    pool_token_supply: u128,
    fees: Option<&Fees>,
    amp_coef: u128,
) -> Result<(u128, u128), MathError> {
    let r_withdraw_amounts = amounts_to_rated(withdraw_amounts, rates)?;
    let r_old_reserves = amounts_to_rated(old_reserves, rates)?;

    compute_lp_amount_for_withdraw(
        &r_withdraw_amounts,
        &r_old_reserves,
        pool_token_supply,
        fees,
        amp_coef,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d_computation_high_amp_coef() {
        let amp_coef: u128 = 1_000_000_000_000;
        let reserve_0: u128 = 400_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let d = compute_d(&Vec::from([reserve_0, reserve_1]), amp_coef).expect("Should compute D");
        assert_eq!(
            d,
            (reserve_0 + reserve_1).into(),
            "Invariant should be equal constant sum invariant"
        )
    }

    #[test]
    fn d_computation_low_amp_coef() {
        let amp_coef: u128 = 1;
        let reserve_0: u128 = 400_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let d = compute_d(&Vec::from([reserve_0, reserve_1]), amp_coef).expect("Should compute D");
        assert!(
            d < (reserve_0 + reserve_1).into(),
            "Invariant should be less than const sum invariant"
        );
        let prod_d = casted_mul(reserve_0, reserve_1).integer_sqrt() * 2;
        assert!(
            d > prod_d,
            "Invariant should be greater than const prod invariant"
        );
    }

    #[test]
    fn y_computation_high_amp_coef() {
        let amp_coef: u128 = 1_000_000_000_000;
        let reserve_0: u128 = 500_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let reserve_delta: u128 = 40_000_000_000;
        let reserve_0_after = reserve_0 - reserve_delta;
        let reserve_1_after = compute_y(
            reserve_0_after,
            &Vec::from([reserve_0, reserve_1]),
            0,
            1,
            amp_coef,
        )
        .expect("Should compute y.");
        assert_eq!(
            reserve_1_after,
            reserve_1 + reserve_delta,
            "Reserve change should be linear"
        );
    }

    #[test]
    fn y_computation_low_amp_coef() {
        let amp_coef: u128 = 1;
        let reserve_0: u128 = 400_000_000_000;
        let reserve_1: u128 = 500_000_000_000;
        let reserve_delta: u128 = 40_000_000_000;
        let reserve_0_after = reserve_0 - reserve_delta;
        let reserve_1_after = compute_y(
            reserve_0_after,
            &Vec::from([reserve_0, reserve_1]),
            0,
            1,
            amp_coef,
        )
        .expect("Should compute y.");
        assert!(
            reserve_1_after > reserve_1 + reserve_delta,
            "Destination reserve change should be greater than in const sum swap"
        );
        let const_prod_y = (reserve_1 * (reserve_0 + reserve_delta)) / reserve_0;
        assert!(
            const_prod_y > reserve_1_after,
            "Destination reserve change should be less than in const prod swap"
        );
    }

    #[test]
    fn swap_to_computation_no_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::zero();
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_in = 10000000000;
        let rates: [u128; 2] = [RATE_PRECISION, RATE_PRECISION];
        // ref https://github.com/ref-finance/ref-contracts/blob/be5c0e33465c13a05dab6e5e9ff9f8af414e16a7/ref-exchange/src/stable_swap/mod.rs#L744
        let expect_token_out = 9999495232;
        let (amount_out, fee) = rated_swap_to(&rates, 0, token_in, 1, &reserves, &fees, amp_coef)
            .expect("Should return swap result");
        assert_eq!(amount_out, expect_token_out, "Incorrect swap ammount");
        assert_eq!(fee, 0, "Fee should nbe 0");
    }

    #[test]
    fn swap_from_computation_no_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::zero();
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_out = 9999495232;
        let expect_token_in = 10000000000;
        let rates: [u128; 2] = [RATE_PRECISION, RATE_PRECISION];

        let (amount_in, fee) = rated_swap_from(&rates, 0, token_out, 1, &reserves, &fees, amp_coef)
            .expect("Should return swap result");
        assert_eq!(amount_in, expect_token_in, "Incorrect swap ammount");
        assert_eq!(fee, 0, "Fee should nbe 0");
    }

    #[test]
    fn swap_to_computation_with_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(10000000, 0).unwrap(); // 1% fee
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_in = 10000000000;
        let expect_token_out = 9999495232;
        let expect_fee = expect_token_out / 100;
        let expect_token_out_minus_fee = expect_token_out - expect_fee;
        let rates: [u128; 2] = [RATE_PRECISION, RATE_PRECISION];

        let (amount_out, fee) = rated_swap_to(&rates, 0, token_in, 1, &reserves, &fees, amp_coef)
            .expect("Should return swap result");
        assert_eq!(
            amount_out, expect_token_out_minus_fee,
            "Incorrect swap ammount"
        );
        assert_eq!(fee, expect_fee, "Incorrect total fee ammount");
    }

    #[test]
    fn swap_from_computation_with_fees() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(10000000, 0).unwrap(); // 1% fee
        let reserves: Vec<u128> = vec![100000000000, 100000000000];
        let token_out = 9999495232;
        let expect_fee: u128 = 9999495232 / 100;
        let token_out_minus_expect_fee = token_out - expect_fee;
        let expect_token_in = 10000000000;
        let rates: [u128; 2] = [RATE_PRECISION, RATE_PRECISION];

        let (amount_in, fee) = rated_swap_from(
            &rates,
            0,
            token_out_minus_expect_fee,
            1,
            &reserves,
            &fees,
            amp_coef,
        )
        .expect("Should return swap result");
        assert_eq!(amount_in, expect_token_in, "Incorrect swap ammount");
        assert_eq!(fee, expect_fee, "Incorrect total fee ammount");
    }

    #[test]
    fn swap_to_from_computation() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(2137, 0).unwrap();
        let reserves: Vec<u128> = vec![12341234123412341234, 5343245543253432435];
        let token_0_in: u128 = 62463425433;
        let rates: [u128; 2] = [RATE_PRECISION, RATE_PRECISION];

        let (amount_out, fee_out) =
            rated_swap_to(&rates, 0, token_0_in, 1, &reserves, &fees, amp_coef)
                .expect("Should return swap result");
        let (amount_in, fee_in) =
            rated_swap_from(&rates, 0, amount_out, 1, &reserves, &fees, amp_coef)
                .expect("Should return swap result");
        assert_eq!(amount_in, token_0_in, "Incorrect swap amount");
        assert_eq!(fee_out, fee_in, "Incorrect fee amount");
    }

    #[test]
    fn swap_from_to_computation() {
        let amp_coef: u128 = 1000;
        let fees = Fees::new(2137, 0).unwrap();
        let reserves: Vec<u128> = vec![12341234123412341234, 5343245543253432435];
        let token_0_out: u128 = 62463425433;
        let rates: [u128; 2] = [RATE_PRECISION, RATE_PRECISION];

        let (amount_in, fee_in) =
            rated_swap_from(&rates, 0, token_0_out, 1, &reserves, &fees, amp_coef)
                .expect("Should return swap result");
        let (amount_out, fee_out) =
            rated_swap_to(&rates, 0, amount_in, 1, &reserves, &fees, amp_coef)
                .expect("Should return swap result");
        assert_eq!(amount_out, token_0_out, "Incorrect swap amount");
        assert_eq!(fee_in, fee_out, "Incorrect fee amount");
    }

    #[test]
    fn withdraw_liquidity_by_share_and_by_amounts_equality_1() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(2137, 0).unwrap();
        let reserves: Vec<u128> = Vec::from([500_000_000_000, 500_000_000_000]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 20; // 5%
        let withdraw_amounts_by_share =
            compute_amounts_given_lp(share, &reserves, token_supply).expect("Compute LPT failed");
        let (share_by_withdraw_amounts, fee_part) = compute_lp_amount_for_withdraw(
            &withdraw_amounts_by_share,
            &reserves,
            token_supply,
            Some(&fees),
            amp_coef,
        )
        .expect("Compute LPT failed");
        assert_eq!(fee_part, 0, "Fee should be 0");
        assert_eq!(
            share_by_withdraw_amounts, share,
            "Share amounts should match"
        );
    }

    #[test]
    fn deposit_liquidity_by_share_and_by_amounts_equality_1() {
        let amp_coef: u128 = 85;
        let fees = Fees::new(2137, 0).unwrap();
        let reserves: Vec<u128> = Vec::from([500_000_000_000, 500_000_000_000]);
        let token_supply = compute_d(&reserves, amp_coef).unwrap().as_u128();
        let share = token_supply / 20; // 5%
        let deposit_amounts = compute_amounts_given_lp(share, &reserves, token_supply)
            .expect("Should mint liquidity");
        let (share_by_deposit, fee_part) = compute_lp_amount_for_deposit(
            &deposit_amounts,
            &reserves,
            token_supply,
            Some(&fees),
            amp_coef,
        )
        .expect("Should mint liquidity");
        assert_eq!(fee_part, 0, "Fee should be 0");
        assert_eq!(share, share_by_deposit, "Deposit amounts differ.");
    }
}
