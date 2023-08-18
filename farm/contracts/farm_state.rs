use ink::env::Environment;

use ink::prelude::vec::Vec;

type AccountId = <ink::env::DefaultEnvironment as Environment>::AccountId;
type Timestamp = <ink::env::DefaultEnvironment as Environment>::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct RunningState {
    // Creator(owner) of the farm.
    pub owner: AccountId,
    // The timestamp when the farm was created.
    pub start: Timestamp,
    // The timestamp when the farm will stop.
    pub end: Timestamp,
    // How many rewards to pay out for the smallest unit of time.
    pub reward_rates: Vec<u128>,
    // Tokens deposited as rewards for providing LP to the farm.
    pub reward_tokens: Vec<AccountId>,
    // Reward counter at the last farm change.
    pub reward_per_token_stored: Vec<u128>,
    // Timestamp of the last farm change.
    pub timestamp_at_last_update: Timestamp,
}
