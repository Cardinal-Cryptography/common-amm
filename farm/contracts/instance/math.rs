use crate::farm::SCALING_FACTOR;
use amm_helpers::math::{
    casted_mul,
    MathError,
};
use primitive_types::U256;

/// Calculates the reward per share in a given time interval.
///
/// Covers the `R/T(t_j - t_j0)` in the formula below.
///
/// r_j = r_j0 + R/T(t_j - t_j0)
///
/// where:
/// - R - reward rate (rewards distributed per unit of time)
/// - T - total shares in the farm
/// - t_j - last time user interacted with the farm, usually _now_.
/// - t_j0 - last time user "claimed" rewards.
/// - r_j - rewards due to user for providing liquidity from t_j0 to t_j
///
/// See https://github.com/stakewithus/notes/blob/main/excalidraw/staking-rewards.png for more.
///
/// # Arguments
///
/// * `reward_rate` - The rate at which rewards are distributed.
/// * `total_shares` - The total number of shares.
/// * `from_timestamp` - The starting timestamp of the interval.
/// * `to_timestamp` - The ending timestamp of the interval.
///
/// # Errors
///
/// Returns a `MathError::Overflow` if the multiplication overflows, or a `MathError::DivByZero`
/// if `total_shares` is zero.
///
/// # Returns
///
/// Returns the reward per share as a `U256` value.
pub fn rewards_per_share_in_time_interval(
    reward_rate: u128,
    total_shares: u128,
    from_timestamp: u128,
    to_timestamp: u128,
) -> Result<U256, MathError> {
    if total_shares == 0 || from_timestamp > to_timestamp {
        return Ok(0.into())
    }

    casted_mul(reward_rate, to_timestamp - from_timestamp)
        .checked_mul(U256::from(SCALING_FACTOR))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(total_shares))
        .ok_or(MathError::DivByZero)
}

/// Returns rewards earned by the user given `rewards_per_share` for some period of time.
/// Calculates the rewards earned based on the number of shares, rewards per share, and paid reward per share.
///
/// # Arguments
///
/// * `shares` - The number of shares.
/// * `rewards_per_share` - The rewards per share.
/// * `paid_reward_per_share` - The paid reward per share.
///
/// # Errors
///
/// Returns a `MathError::Underflow` if the subtraction of `paid_reward_per_share` from `rewards_per_share` results in an underflow.
///
/// # Returns
///
/// The rewards earned as a `u128` value.
pub fn calculate_rewards_earned(
    shares: u128,
    rewards_per_share: U256,
    paid_reward_per_share: U256,
) -> Result<u128, MathError> {
    let r = rewards_per_share
        .checked_sub(paid_reward_per_share)
        .ok_or(MathError::Underflow)?;

    rewards_earned_by_shares(shares, r)
}

/// Calculates the amount of rewards earned by a given number of shares, based on the rewards per share.
///
/// # Arguments
///
/// * `shares` - The number of shares for which to calculate the rewards earned.
/// * `rewards_per_share` - The rewards per share value used to calculate the rewards earned.
///
/// # Errors
///
/// Returns a `MathError::Overflow` if the multiplication of `rewards_per_share` and `shares` overflows.
/// Returns a `MathError::DivByZero` if the division of the multiplication result by the scaling factor overflows.
/// Returns a `MathError::CastOverflow` if the result of the division cannot be cast to `u128`.
///
/// # Returns
///
/// The amount of rewards earned by the given number of shares, as a `u128`.
pub fn rewards_earned_by_shares(shares: u128, rewards_per_share: U256) -> Result<u128, MathError> {
    rewards_per_share
        .checked_mul(U256::from(shares))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(SCALING_FACTOR))
        .ok_or(MathError::DivByZero)?
        .try_into()
        .map_err(|_| MathError::CastOverflow)
}
