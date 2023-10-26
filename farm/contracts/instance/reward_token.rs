use scale::{
    Decode,
    Encode,
};

use crate::TokenId;
use amm_helpers::types::WrappedU256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RewardTokenInfo {
    /// Tokens deposited as rewards for providing LP to the farm.
    pub token_id: TokenId,
    /// How many rewards to pay out for the smallest unit of time.
    pub reward_rate: u128,
    /// Totals of unclaimed rewards.
    // Necessary for not letting owner, re-claim more than allowed to once the farm is stopped.
    pub unclaimed_rewards_total: u128,
    /// Reward counter at the last farm change.
    pub cumulative_reward_per_share: WrappedU256,
}
