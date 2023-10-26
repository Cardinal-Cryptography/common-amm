//! Types used for the views of the contract.
//! Views are used to return data from the contract to the frontend.
//! They are not used internally in the contract.
//! Note that the usage of these can be expensive for the node, so they should be used sparingly.
//! Instead, use the indexer (if one exists).

use ink::{
    prelude::vec::Vec,
    primitives::AccountId,
};

use crate::reward_token::RewardTokenInfo;
use scale::{
    Decode,
    Encode,
};

/// View of the farm.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct FarmDetailsView {
    /// Address of the pool this farm is for.
    pub pair: AccountId,
    /// Timestamp (in milliseconds) of the farm start.
    pub start: u64,
    /// Timestamp (in milliseconds) of the farm end.
    pub end: u64,
    /// Amount of LP tokens locked in the farm.
    pub total_shares: u128,
    /// Reward tokens info.
    pub reward_tokens: Vec<RewardTokenInfoView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct RewardTokenInfoView {
    // Address of the reward token.
    token_id: AccountId,
    // Amount of reward tokens per second.
    reward_rate: u128,
}

impl From<&RewardTokenInfo> for RewardTokenInfoView {
    fn from(info: &RewardTokenInfo) -> Self {
        Self {
            token_id: info.token_id,
            reward_rate: info.reward_rate,
        }
    }
}

/// View of the user's position in the farm.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct UserPositionView {
    // Amount of LP tokens locked in the farm by the user.
    pub shares: u128,
    // Amount of unclaimed rewards.
    pub unclaimed_rewards: Vec<u128>,
}
