#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod farm {
    use openbrush::modifiers;

    use ink::{
        env::DefaultEnvironment,
        storage::{
            traits::StorageLayout,
            Mapping,
        },
    };

    use crate::{
        ensure_state,
        FarmError,
        FarmState,
    };

    /// Computes the payout for certain period.
    ///
    /// The payout is computed as the number of rewards per unit of time (smallest period for which we pay out rewards)
    /// multiplied by the time elapsed since the last payout.
    fn payout(
        start_timestamp: Timestamp,
        now: Timestamp,
        resolution: u64,
        rewards_per_unit: u64,
    ) -> u64 {
        let time_elapsed = now - start_timestamp;
        if time_elapsed < resolution {
            return 0
        }
        let time_elapsed_in_resolution = time_elapsed / resolution;
        time_elapsed_in_resolution * rewards_per_unit
    }

    /// total_payout / total_shares = payout_per_share
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, StorageLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RewardCounter {
        total_payout: u64,
        total_shares: u64,
    }

    impl RewardCounter {
        pub fn for_period(
            start: Timestamp,
            end: Timestamp,
            resolution: u64,
            rewards_per_unit: u64,
            total_shares: u64,
        ) -> Self {
            let total_payout = payout(start, end, resolution, rewards_per_unit);
            return RewardCounter {
                total_payout,
                total_shares,
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode, StorageLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Farmer {
        last_claim_timestamp: u64,
        reward_counter_at_last_payout: RewardCounter,
    }

    #[ink(storage)]
    pub struct Farm {
        // Address of the token pool for which this farm is created.
        pool: AccountId,
        farmers: Mapping<AccountId, Farmer>,
        rewards_per_unit: u64,
        rewards_resolution: u64,
        start: Timestamp,
        end: Timestamp,
        pub state: FarmState,
    }

    impl Farm {
        #[ink(constructor)]
        pub fn new(pair_address: AccountId) -> Self {
            // TODO: check if end-start is not too large
            return Farm {
                pool: pair_address,
                farmers: Mapping::new(),
                // TODO
                rewards_per_unit: 0,
                // TODO
                rewards_resolution: 0,
                // TODO
                start: 0,
                // TODO
                end: 0,
                state: FarmState::Running,
            }
        }

        #[modifiers(ensure_state(FarmState::Running))]
        #[ink(message)]
        pub fn deposit(&mut self, _amount: u64) -> Result<(), FarmError> {
            // Deposit forces claim before depositing.
            // Mint new PSP22 tokens for the caller.
            // TODO
            Ok(())
        }

        #[ink(message)]
        pub fn withdraw(&mut self, _amount: u64) -> Result<(), FarmError> {
            // Withdraw forces claim.
            // Burn PSP22 tokens from the caller.
            // TODO
            Ok(())
        }

        #[ink(message)]
        pub fn claim_rewards(&mut self) -> Result<(), FarmError> {
            let caller = Self::env().caller();

            Ok(())
        }

        #[modifiers(ensure_state(FarmState::Running))]
        // TODO #[modifiers(only_owner)]
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), FarmError> {
            if ink::env::block_timestamp::<DefaultEnvironment>() < self.end {
                Err(FarmError::StillRunning)
            } else {
                self.state = FarmState::Stopped;
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode, StorageLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmError {
    InvalidFarmState(FarmState),
    StillRunning,
    CallerIsNotOwner,
}

use crate::farm::Farm;
use openbrush::modifier_definition;

#[modifier_definition]
pub fn ensure_state<F, T, E>(
    instance: &mut Farm,
    body: F,
    expected_state: FarmState,
) -> Result<T, E>
where
    F: FnOnce(&mut Farm) -> Result<T, E>,
    E: From<FarmError>,
{
    if instance.state != expected_state {
        return Err(FarmError::InvalidFarmState(instance.state.clone()).into())
    }
    body(instance)
}

// TODO
/// Throws if called by any account other than the owner.
// #[modifier_definition]
// pub fn only_owner<T, F, E, AccountId>(instance: &mut Farm, body: F) -> Result<T, E>
// where
//     Farm: Ownable<AccountId>,
//     F: FnOnce(&mut Farm) -> Result<T, E>,
//     E: From<FarmError>,
// {
//     if instance.owner() != DefaultEnvironment::caller() {
//         return Err(From::from(FarmError::CallerIsNotOwner))
//     }
//     body(instance)
// }
use ink::{
    env::DefaultEnvironment,
    storage::traits::StorageLayout,
};

#[derive(Clone, Debug, PartialEq, Eq, scale::Encode, scale::Decode, StorageLayout)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmState {
    Running,
    Stopped,
}

impl std::fmt::Display for FarmState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FarmState::Running => write!(f, "Running"),
            FarmState::Stopped => write!(f, "Stopped"),
        }
    }
}
