#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod error;
mod math;
mod reward_token;
mod state;
#[cfg(test)]
mod tests;
mod utils;
mod views;

pub use farm::FarmRef;

use ink::env::Environment;

pub type TokenId = <ink::env::DefaultEnvironment as Environment>::AccountId;
pub type UserId = <ink::env::DefaultEnvironment as Environment>::AccountId;
pub type Timestamp = <ink::env::DefaultEnvironment as ink::env::Environment>::Timestamp;

#[ink::contract]
mod farm {
    use crate::{
        error::FarmError,
        reward_token::RewardTokenInfo,
        state::State,
        utils::{
            safe_balance_of,
            safe_transfer,
        },
        views::{
            FarmDetailsView,
            UserPositionView,
        },
    };

    use psp22_traits::PSP22;

    use ink::{
        contract_ref,
        storage::{
            traits::ManualKey,
            Lazy,
            Mapping,
        },
    };

    use ink::prelude::{
        vec,
        vec::Vec,
    };

    use amm_helpers::types::WrappedU256;

    use farm_instance_trait::{
        Farm as FarmT,
        FarmStartError,
    };

    use farm_manager_trait::FarmManager;

    #[ink(event)]
    pub struct Deposited {
        #[ink(topic)]
        account: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Withdrawn {
        #[ink(topic)]
        account: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct RewardsClaimed {
        #[ink(topic)]
        account: AccountId,
        amounts: Vec<u128>,
    }

    #[ink(storage)]
    pub struct Farm {
        /// Address of the token pool for which this farm is created.
        pub pool: AccountId,
        /// Address of the farm creator.
        pub owner: AccountId,
        /// Farm manager that created this instance.
        pub manager: AccountId,
        /// Farm state.
        pub state: Lazy<State, ManualKey<0x4641524d>>,
    }

    pub const SCALING_FACTOR: u128 = u128::MAX;
    pub const MAX_REWARD_TOKENS: u32 = 10;

    impl Farm {
        /// Creates the farm instance, without starting it yet.
        ///
        /// NOTE:
        /// Current block's timestamp is used as the start time.
        /// Farm can be started only if it's in `Stopped` state.
        #[ink(constructor)]
        pub fn new(pair_address: AccountId, manager: AccountId, owner: AccountId) -> Self {
            Farm {
                pool: pair_address,
                owner,
                manager,
                state: Lazy::new(),
            }
        }

        /// Stops the farm in the current block.
        ///  
        /// Callable only by the owner.
        ///
        /// Returns errors in the following cases:
        /// 1. Farm is not in `Running` state.
        /// 2. Caller is not owner.
        #[ink(message)]
        pub fn stop(&mut self) -> Result<(), FarmError> {
            self.ensure_running(true)?;
            if self.env().caller() != self.owner {
                return Err(FarmError::CallerNotOwner)
            }
            let mut running = self.get_state()?;
            running.end = self.env().block_timestamp();
            self.state.set(&running);
            Ok(())
        }

        /// Transfers remaining reward tokens to the farm's owner.
        ///
        /// Returns errors in the following cases:
        /// 1. Farm is still `Running`.
        #[ink(message)]
        pub fn withdraw_reward_tokens(&mut self) -> Result<(), FarmError> {
            self.ensure_running(false)?;
            self.update_reward_index()?;
            let mut running = self.get_state()?;

            let mut to_refund = Vec::with_capacity(running.reward_tokens_info.len());

            for reward_token in running.reward_tokens_info.iter_mut() {
                let token_ref = reward_token.token_id.into();

                let reserved_for_rewards = reward_token.unclaimed_rewards_total;
                if reserved_for_rewards == 0 {
                    // If there are no yet-unclaimed rewards, nothing is reserved
                    // and we can skip this reward token (most probably all iterations will be skipped).
                    to_refund.push((token_ref, 0));
                    continue
                }
                let balance: Balance =
                    safe_balance_of::<Environment>(&token_ref, self.env().account_id());
                let refund_amount = balance.saturating_sub(reserved_for_rewards);
                reward_token.unclaimed_rewards_total = 0;
                to_refund.push((token_ref, refund_amount));
            }

            self.state.set(&running);

            for (mut token_ref, refund_amount) in to_refund {
                if refund_amount > 0 {
                    safe_transfer(&mut token_ref, running.owner, refund_amount)?;
                }
            }

            Ok(())
        }

        /// Deposits the given amount of tokens into the farm.
        #[ink(message)]
        pub fn deposit(&mut self, amount: u128) -> Result<(), FarmError> {
            self.ensure_running(true)?;
            Self::ensure_non_zero_amount(amount)?;
            self.update_reward_index()?;
            self.add_shares(amount)
        }

        /// Deposits all of the LP tokens the caller has.
        /// NOTE: Requires that the caller has approved the farm to spend their tokens.
        #[ink(message)]
        pub fn deposit_all(&mut self) -> Result<(), FarmError> {
            self.ensure_running(true)?;
            self.update_reward_index()?;
            let token_balance =
                safe_balance_of::<Environment>(&self.pool.into(), self.env().caller());
            Self::ensure_non_zero_amount(token_balance)?;
            self.add_shares(token_balance)
        }

        /// Withdraws the given amount of shares from the farm.
        #[ink(message)]
        pub fn withdraw(&mut self, amount: u128) -> Result<(), FarmError> {
            Self::ensure_non_zero_amount(amount)?;
            self.update_reward_index()?;
            let caller = self.env().caller();

            let mut manager: contract_ref!(FarmManager) = self.manager.into();

            manager.withdraw_shares(caller, amount)?;

            let mut pool: contract_ref!(PSP22) = self.pool.into();

            pool.transfer(caller, amount, vec![])?;

            self.env().emit_event(Withdrawn {
                account: caller,
                amount,
            });

            Ok(())
        }

        /// Claim all rewards for the caller.
        #[ink(message)]
        pub fn claim(&mut self) -> Result<(), FarmError> {
            self.update_reward_index()?;

            let caller = Self::env().caller();

            let mut state = self.get_state()?;

            let user_rewards = state
                .user_claimable_rewards
                .take(caller)
                .ok_or(FarmError::CallerNotFarmer)?;

            if !self.is_running()? {
                // We can remove the user from the map only when the farm is already finished.
                // That's b/c it won't be earning any more rewards for this particular farm instance anymore.
                state.user_reward_per_share_paid.remove(caller);
            }

            for (idx, user_reward) in user_rewards.clone().into_iter().enumerate() {
                if user_reward > 0 {
                    let mut psp22_ref: ink::contract_ref!(PSP22) =
                        state.reward_tokens_info[idx].token_id.into();
                    state.reward_tokens_info[idx].unclaimed_rewards_total = state
                        .reward_tokens_info[idx]
                        .unclaimed_rewards_total
                        .saturating_sub(user_reward);
                    safe_transfer(&mut psp22_ref, caller, user_reward)?;
                }
            }

            self.state.set(&state);

            self.env().emit_event(RewardsClaimed {
                account: caller,
                amounts: user_rewards,
            });
            Ok(())
        }

        /// Returns view structure with details about the currently active farm.
        #[ink(message)]
        pub fn view_farm_details(&self) -> FarmDetailsView {
            let manager: contract_ref!(FarmManager) = self.manager.into();

            let state = self.get_state().expect("state to exist");
            FarmDetailsView {
                pair: self.pool,
                start: state.start,
                end: state.end,
                total_shares: manager.total_supply(),
                reward_tokens: state.reward_tokens_info.iter().map(Into::into).collect(),
            }
        }

        /// Returns view structure with details about the caller's position in the farm.
        #[ink(message)]
        pub fn view_user_position(&self, account: AccountId) -> Option<UserPositionView> {
            let manager: contract_ref!(FarmManager) = self.manager.into();

            Some(UserPositionView {
                shares: manager.balance_of(account),
                unclaimed_rewards: self.claimable(account).ok()?,
            })
        }

        // Returns how much reward tokens the caller account has accumulated.
        // We're using the `account` as an argument, instead of `&self.env().caller()`,
        // for easier frontend integration.
        fn claimable(&self, account: AccountId) -> Result<Vec<u128>, FarmError> {
            let manager: contract_ref!(FarmManager) = self.manager.into();
            let user_shares = manager.balance_of(account);
            if user_shares == 0 {
                return Err(FarmError::CallerNotFarmer)
            }
            let total_shares = manager.total_supply();
            let mut state = self.get_state()?;
            state.update_rewards(total_shares, self.env().block_timestamp())?;
            let _newly_earned_rewards =
                state.move_unclaimed_rewards_to_claimable(user_shares, account)?;
            state.unclaimed_rewards(account)
            // note that without state.set() this is still immutable
        }

        /// Check whether farm is currently running - i.e. whether current timestamp
        /// is between "start" and "end" of the farm.
        ///
        /// Returns `FarmError::StateMissing` when farm hasn't been started yet.
        pub fn is_running(&self) -> Result<bool, FarmError> {
            self.get_state().map(|state| {
                self.env().block_timestamp() < state.start
                    || self.env().block_timestamp() >= state.end
            })
        }

        /// Adds the given amount of shares to the farm under `account`.
        fn add_shares(&mut self, amount: u128) -> Result<(), FarmError> {
            let caller = self.env().caller();
            let mut manager: contract_ref!(FarmManager) = self.manager.into();

            manager.deposit_shares(caller, amount)?;

            let mut pool: contract_ref!(PSP22) = self.pool.into();

            pool.transfer_from(caller, self.env().account_id(), amount, vec![])?;

            self.env().emit_event(Deposited {
                account: caller,
                amount,
            });
            Ok(())
        }

        /// Returns the amount of new rewards per token that have been accumulated for the given account.
        fn update_reward_index(&mut self) -> Result<Vec<u128>, FarmError> {
            let account = self.env().caller();
            let manager: contract_ref!(FarmManager) = self.manager.into();
            let user_shares = manager.balance_of(account);
            let total_shares = manager.total_supply();
            let mut state = self.get_state()?;
            state.update_rewards(total_shares, self.env().block_timestamp())?;
            let newly_earned_rewards =
                state.move_unclaimed_rewards_to_claimable(user_shares, account)?;
            self.state.set(&state);
            Ok(newly_earned_rewards)
        }

        fn get_state(&self) -> Result<State, FarmError> {
            self.state.get().ok_or(FarmError::StateMissing)
        }

        fn ensure_running(&self, expected_running: bool) -> Result<(), FarmError> {
            let is_running = self.is_running()?;
            if expected_running && !is_running {
                return Err(FarmError::NotRunning)
            }
            if !expected_running && is_running {
                return Err(FarmError::StillRunning)
            }
            Ok(())
        }

        fn ensure_non_zero_amount(amount: u128) -> Result<(), FarmError> {
            if amount == 0 {
                return Err(FarmError::InvalidAmountArgument)
            }
            Ok(())
        }
    }

    impl FarmT for Farm {
        #[ink(message)]
        fn start(
            &mut self,
            end: Timestamp,
            reward_tokens: Vec<AccountId>,
        ) -> Result<(), FarmStartError> {
            let farm_owner = self.owner;
            if Self::env().caller() != farm_owner {
                return Err(FarmStartError::CallerNotOwner)
            }

            if self.get_state().is_ok() {
                return Err(FarmStartError::FarmAlreadyStarted)
            }

            if reward_tokens.len() > MAX_REWARD_TOKENS as usize {
                return Err(FarmStartError::TooManyRewardTokens)
            }

            let now = Self::env().block_timestamp();

            if now >= end {
                return Err(FarmStartError::FarmEndBeforeStart)
            }

            let duration = end as u128 - now as u128;

            let tokens_len = reward_tokens.len();

            let mut reward_rates = Vec::with_capacity(tokens_len);
            let mut reward_tokens_info = Vec::with_capacity(tokens_len);

            for token_id in reward_tokens {
                let psp22_ref: ink::contract_ref!(PSP22) = token_id.into();

                let reward_amount = psp22_ref.balance_of(self.env().account_id());
                if reward_amount == 0 {
                    return Err(FarmStartError::ZeroRewardAmount)
                }
                let reward_rate = reward_amount
                    .checked_div(duration)
                    .ok_or(FarmStartError::ArithmeticError)?;

                if reward_rate == 0 {
                    return Err(FarmStartError::ZeroRewardRate)
                }

                // Double-check we have enough to cover the whole farm.
                if duration * reward_rate < reward_amount {
                    return Err(FarmStartError::InsufficientRewardAmount)
                }

                let unclaimed_rewards_total = 0;
                let cumulative_reward_per_share = WrappedU256::ZERO;

                let info = RewardTokenInfo {
                    token_id,
                    reward_rate,
                    unclaimed_rewards_total,
                    cumulative_reward_per_share,
                };

                reward_tokens_info.push(info);
                reward_rates.push(reward_rate);
            }

            let state = State {
                owner: farm_owner,
                start: now,
                end,
                reward_tokens_info,
                timestamp_at_last_update: now,
                user_reward_per_share_paid: Mapping::new(),
                user_claimable_rewards: Mapping::new(),
            };

            self.state.set(&state);
            Ok(())
        }

        #[ink(message)]
        fn pool_id(&self) -> AccountId {
            self.pool
        }

        #[ink(message)]
        fn is_running(&self) -> bool {
            self.is_running().unwrap()
        }

        #[ink(message)]
        fn farm_manager(&self) -> AccountId {
            self.manager
        }

        #[ink(message)]
        fn farm_owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        fn code_hash(&self) -> Hash {
            self.env()
                .own_code_hash()
                .expect("to properly deserialize code hash")
        }
    }
}
