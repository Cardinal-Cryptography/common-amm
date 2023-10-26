use crate::{
    error::FarmError,
    math::*,
    reward_token::RewardTokenInfo,
    Timestamp,
    UserId,
};
use amm_helpers::types::WrappedU256;
use ink::{
    prelude::{
        vec,
        vec::Vec,
    },
    storage::Mapping,
};

#[ink::storage_item]
pub struct State {
    /// Creator(owner) of the farm.
    pub owner: UserId,
    /// The timestamp when the farm was created.
    pub start: Timestamp,
    /// The timestamp when the farm will stop.
    pub end: Timestamp,
    /// Reward tokens.
    pub reward_tokens_info: Vec<RewardTokenInfo>,
    /// Timestamp of the last farm change.
    pub timestamp_at_last_update: Timestamp,
    /// Reward per token paid to the user for each reward token.
    // We need to track this separately for each reward token as each can have different reward rate.
    // Vectors should be relatively small (probably < 5).
    pub user_reward_per_share_paid: Mapping<UserId, Vec<WrappedU256>>,
    /// Rewards that have not been claimed (withdrawn) by the user yet.
    pub user_claimable_rewards: Mapping<UserId, Vec<u128>>,
}

impl State {
    /// Updates the rewards that should be paid out to liquidity providers since the last update.
    /// Returns an error if there was an issue calculating the rewards.
    pub fn update_rewards(
        &mut self,
        total_shares: u128,
        current_timestamp: Timestamp,
    ) -> Result<(), FarmError> {
        let past = core::cmp::max(self.timestamp_at_last_update, self.start) as u128;
        let now = core::cmp::min(current_timestamp, self.end) as u128;
        if past >= now || self.timestamp_at_last_update == current_timestamp {
            return Ok(())
        }

        for reward_token in self.reward_tokens_info.iter_mut() {
            let reward_rate = reward_token.reward_rate;
            let reward_delta =
                rewards_per_share_in_time_interval(reward_rate, total_shares, past, now)?;
            let new_cumulative_reward_rate =
                reward_token.cumulative_reward_per_share.0 + reward_delta;
            reward_token.cumulative_reward_per_share = new_cumulative_reward_rate.into();
            reward_token.unclaimed_rewards_total +=
                rewards_earned_by_shares(total_shares, reward_delta)?;
        }

        self.timestamp_at_last_update = current_timestamp;

        Ok(())
    }

    /// Computes how much rewards have been earned by the user since the last update
    /// and updates the user's unclaimed rewards.
    pub fn move_unclaimed_rewards_to_claimable(
        &mut self,
        user_shares: u128,
        account: UserId,
    ) -> Result<Vec<u128>, FarmError> {
        if user_shares == 0 {
            return Ok(vec![0; self.reward_tokens_info.len()])
        }

        let mut reward_per_share_paid_so_far = self
            .user_reward_per_share_paid
            .get(account)
            .unwrap_or(vec![WrappedU256::ZERO; self.reward_tokens_info.len()]);

        let mut uncollected_user_rewards = self
            .user_claimable_rewards
            .get(account)
            .unwrap_or(vec![0; self.reward_tokens_info.len()]);

        let mut new_rewards = vec![0u128; self.reward_tokens_info.len()];

        for (idx, token) in self.reward_tokens_info.iter().enumerate() {
            new_rewards[idx] = calculate_rewards_earned(
                user_shares,
                token.cumulative_reward_per_share.0,
                reward_per_share_paid_so_far[idx].0,
            )?;
            uncollected_user_rewards[idx] =
                uncollected_user_rewards[idx].saturating_add(new_rewards[idx]);
            reward_per_share_paid_so_far[idx] = token.cumulative_reward_per_share;
        }

        self.user_claimable_rewards
            .insert(account, &uncollected_user_rewards);
        self.user_reward_per_share_paid
            .insert(account, &reward_per_share_paid_so_far);

        Ok(new_rewards)
    }

    /// Returns the unclaimed rewards for the specified account.
    pub fn unclaimed_rewards(&self, account: UserId) -> Result<Vec<u128>, FarmError> {
        self.user_claimable_rewards
            .get(account)
            .ok_or(FarmError::CallerNotFarmer)
    }

    /// Returns rewards distributable to all farmers for the period between now (or farm end)
    /// and last farm change.
    pub fn rewards_distributable(&self, current_timestamp: Timestamp) -> Vec<u128> {
        let last_time_reward_applicable = core::cmp::min(current_timestamp, self.end) as u128;
        self.reward_tokens_info
            .iter()
            .map(|info| {
                info.reward_rate
                    .checked_mul(
                        last_time_reward_applicable - self.timestamp_at_last_update as u128,
                    )
                    .unwrap_or(0)
            })
            .collect()
    }
}
