#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod error;
mod views;

#[cfg(test)]
mod tests;

// TODO:
// Add upper bound on farm length.
// Tests.
// ? Refactor to make staking logic reusable in different contracts.

pub use farm::FarmRef;

#[ink::contract]
mod farm {
    use crate::error::{
        FarmError,
        FarmStartError,
    };

    use crate::views::{
        FarmDetailsView,
        UserPositionView,
    };

    use crate::{
        reward_per_token,
        rewards_earned,
    };

    use openbrush::modifiers;

    use primitive_types::U256;
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

    use farm_manager_trait::FarmManager as FarmManagerT;

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
        /// Starts the farm with the given parameters.
        ///
        /// Arguments:
        /// * end - timestamp when the farm will stop.
        /// * reward_tokens - vector of account ids of reward tokens.
        /// * reward_amounts - vector of unsigned integers, specifying how many rewards
        ///   will be paid out throughout the whole farm of each reward token.
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

        #[ink(message)]
        pub fn start(
            &mut self,
            end: Timestamp,
            reward_tokens: Vec<AccountId>,
        ) -> Result<(), FarmStartError> {
            let farm_owner = self.owner;
            if Self::env().caller() != farm_owner {
                return Err(FarmStartError::CallerNotOwner)
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

                let total_unclaimed_rewards = 0;
                let reward_per_token_stored = WrappedU256::ZERO;

                let info = RewardTokenInfo {
                    token_id,
                    reward_rate,
                    total_unclaimed_rewards,
                    reward_per_token_stored,
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
                user_reward_per_token_paid: Mapping::new(),
                user_unclaimed_rewards: Mapping::new(),
            };

            self.state.set(&state);
            Ok(())
        }

        /// Stops the farm and sends all remaining rewards to the farm owner.
        ///
        /// Returns errors in the following cases:
        /// 1. Farm is not in `Running` state.
        /// 2. Farm's `end` timestamp is still in the future.
        #[ink(message)]
        #[modifiers(ensure_running(true))]
        pub fn stop(&mut self) -> Result<(), FarmError> {
            let mut running = self.get_state()?;

            // TODO: consider extracting to `set_end` function.
            // We allow owner of the farm to stop it prematurely
            // while anyone else can change the farm's status only when it's finished.
            if self.env().caller() == self.owner {
                running.end = self.env().block_timestamp();
            } else if self.is_running()? {
                return Err(FarmError::StillRunning)
            }

            self.state.set(&running);

            // Send remaining rewards to the farm owner.
            for reward_token in running.reward_tokens_info.iter() {
                let mut psp22_ref: ink::contract_ref!(PSP22) = reward_token.token_id.into();
                let balance: Balance = safe_balance_of(&psp22_ref, self.env().account_id());
                let reserved = reward_token.total_unclaimed_rewards;
                let to_refund = balance.saturating_sub(reserved);
                if to_refund > 0 {
                    safe_transfer(&mut psp22_ref, running.owner, to_refund)?;
                }
            }

            Ok(())
        }

        /// Deposits the given amount of tokens into the farm.
        #[ink(message)]
        #[modifiers(ensure_running(true), non_zero_amount(amount))]
        pub fn deposit(&mut self, amount: u128) -> Result<(), FarmError> {
            self.update_reward_index()?;
            self.add_shares(amount)
        }

        /// Deposits all of the LP tokens the caller has.
        /// NOTE: Requires that the caller has approved the farm to spend their tokens.
        #[ink(message)]
        #[modifiers(ensure_running(true))]
        pub fn deposit_all(&mut self) -> Result<(), FarmError> {
            self.update_reward_index()?;
            let token_balance = safe_balance_of(&self.pool.into(), self.env().caller());
            self.add_shares(token_balance)
        }

        /// Withdraws the given amount of shares from the farm.
        #[ink(message)]
        #[modifiers(non_zero_amount(amount))]
        pub fn withdraw(&mut self, amount: u128) -> Result<(), FarmError> {
            self.update_reward_index()?;
            let caller = self.env().caller();

            let mut manager: contract_ref!(FarmManagerT) = self.manager.into();

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
                .user_unclaimed_rewards
                .take(caller)
                .ok_or(FarmError::CallerNotFarmer)?;

            if !self.is_running()? {
                // We can remove the user from the map only when the farm is already finished.
                // That's b/c it won't be earning any more rewards for this particular farm.
                state.user_reward_per_token_paid.remove(caller);
            }

            for (idx, user_reward) in user_rewards.clone().into_iter().enumerate() {
                if user_reward > 0 {
                    let mut psp22_ref: ink::contract_ref!(PSP22) =
                        state.reward_tokens_info[idx].token_id.into();
                    state.reward_tokens_info[idx].total_unclaimed_rewards = state
                        .reward_tokens_info[idx]
                        .total_unclaimed_rewards
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

        /// Returns how much reward tokens the caller account has accumulated.
        // We're using the `account` as an argument, instead of `&self.env().caller()`,
        // for easier frontend integration.
        // TODO: Rename to `view_claimable`.
        #[ink(message)]
        pub fn claimable(&self, account: AccountId) -> Result<Vec<u128>, FarmError> {
            let manager: contract_ref!(FarmManagerT) = self.manager.into();

            let shares = manager.balance_of(account);
            if shares == 0 {
                return Err(FarmError::CallerNotFarmer)
            }

            let state = self.get_state()?;
            let rewards_per_token =
                state.rewards_per_token(manager.total_supply(), self.env().block_timestamp())?;

            let user_rewards = state.rewards_earned(account, shares, &rewards_per_token)?;

            Ok(user_rewards)
        }

        /// Returns view structure with details about the currently active farm.
        #[ink(message)]
        pub fn view_farm_details(&self) -> FarmDetailsView {
            let manager: contract_ref!(FarmManagerT) = self.manager.into();

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
            let manager: contract_ref!(FarmManagerT) = self.manager.into();

            Some(UserPositionView {
                shares: manager.balance_of(account),
                unclaimed_rewards: self.claimable(account).ok()?,
            })
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
            let mut manager: contract_ref!(FarmManagerT) = self.manager.into();

            manager.deposit_shares(caller, amount);

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
            let manager: contract_ref!(FarmManagerT) = self.manager.into();

            let account = self.env().caller();
            let shares = manager.balance_of(account);
            if shares == 0 {
                return Err(FarmError::CallerNotFarmer)
            }
            let total_shares = manager.total_supply();

            let mut state = self.get_state()?;

            let rewards_per_token =
                state.rewards_per_token(total_shares, self.env().block_timestamp())?;
            let unclaimed_rewards = state.rewards_earned(account, shares, &rewards_per_token)?;

            for (idx, rewards_distributable) in state
                .rewards_distributable(self.env().block_timestamp())
                .iter()
                .enumerate()
            {
                state.reward_tokens_info[idx].total_unclaimed_rewards += *rewards_distributable;
                state.reward_tokens_info[idx].reward_per_token_stored =
                    rewards_per_token[idx].into();
            }

            state.user_reward_per_token_paid.insert(
                account,
                &rewards_per_token
                    .clone()
                    .into_iter()
                    .map(WrappedU256::from)
                    .collect::<Vec<_>>(),
            );
            state
                .user_unclaimed_rewards
                .insert(account, &unclaimed_rewards);

            self.state.set(&state);

            Ok(unclaimed_rewards)
        }

        fn get_state(&self) -> Result<State, FarmError> {
            self.state.get().ok_or(FarmError::StateMissing)
        }
    }

    impl farm_instance_trait::Farm for Farm {
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

    type TokenId = AccountId;
    type UserId = AccountId;

    use scale::{
        Decode,
        Encode,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RewardTokenInfo {
        /// Tokens deposited as rewards for providing LP to the farm.
        pub token_id: TokenId,
        /// How many rewards to pay out for the smallest unit of time.
        pub reward_rate: u128,
        /// Totals of unclaimed rewards.
        // Necessary for not letting owner, re-claim more than allowed to once the farm is stopped.
        pub total_unclaimed_rewards: u128,
        /// Reward counter at the last farm change.
        pub reward_per_token_stored: WrappedU256,
    }

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
        pub user_reward_per_token_paid: Mapping<UserId, Vec<WrappedU256>>,
        /// Rewards that have not been claimed (withdrawn) by the user yet.
        pub user_unclaimed_rewards: Mapping<UserId, Vec<u128>>,
    }

    impl State {
        /// Calculates rewards per token due for providing liquidity to the farm
        /// since the last update until `last_timestamp_applicable`.
        ///
        /// Returned value is a vector of numbers, one for each reward token in the farm.
        pub fn rewards_per_token(
            &self,
            total_shares: u128,
            current_timestamp: Timestamp,
        ) -> Result<Vec<U256>, FarmError> {
            let mut rewards_per_token: Vec<U256> =
                Vec::with_capacity(self.reward_tokens_info.len());

            for reward_token in self.reward_tokens_info.iter() {
                let reward_rate = reward_token.reward_rate;
                let rpr = reward_per_token(
                    reward_token.reward_per_token_stored.0,
                    reward_rate,
                    total_shares,
                    self.timestamp_at_last_update as u128,
                    core::cmp::min(current_timestamp, self.end) as u128,
                )?;
                rewards_per_token.push(rpr);
            }

            Ok(rewards_per_token)
        }

        /// Returns the amount of rewards earned by the given account.
        pub fn rewards_earned(
            &self,
            account: AccountId,
            shares: u128,
            rewards_per_token: &[U256],
        ) -> Result<Vec<u128>, FarmError> {
            let rewards_per_token_paid_so_far = self
                .user_reward_per_token_paid
                .get(account)
                .unwrap_or(vec![WrappedU256::ZERO; rewards_per_token.len()]);

            let uncollected_user_rewards = self
                .user_unclaimed_rewards
                .get(account)
                .unwrap_or(vec![0; rewards_per_token.len()]);

            let mut unclaimed_user_rewards = vec![];

            for i in 0..rewards_per_token.len() {
                let rewards_earned = rewards_earned(
                    shares,
                    rewards_per_token[i],
                    rewards_per_token_paid_so_far[i].0,
                )?;
                unclaimed_user_rewards.push(rewards_earned + uncollected_user_rewards[i]);
            }

            Ok(unclaimed_user_rewards)
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

    use openbrush::modifier_definition;

    #[modifier_definition]
    pub fn ensure_running<F, T>(
        instance: &mut Farm,
        body: F,
        should_be_running: bool,
    ) -> Result<T, FarmError>
    where
        F: FnOnce(&mut Farm) -> Result<T, FarmError>,
    {
        if !should_be_running && instance.is_running()? {
            return Err(FarmError::StillRunning)
        }
        body(instance)
    }

    #[modifier_definition]
    pub fn non_zero_amount<F, T>(instance: &mut Farm, body: F, amount: u128) -> Result<T, FarmError>
    where
        F: FnOnce(&mut Farm) -> Result<T, FarmError>,
    {
        if amount == 0 {
            return Err(FarmError::InvalidAmountArgument)
        }
        body(instance)
    }

    use ink::codegen::TraitCallBuilder;

    // We're making a concious choice here that we don't want to fail the whole transaction
    // if a PSP22::transfer fails with a panic.
    // This is to ensure that funds are not locked in the farm if someone uses malicious
    // PSP22 token impl for rewards.
    pub fn safe_transfer(
        psp22: &mut contract_ref!(PSP22),
        recipient: AccountId,
        amount: u128,
    ) -> Result<(), psp22_traits::PSP22Error> {
        match psp22
            .call_mut()
            .transfer(recipient, amount, vec![])
            .try_invoke()
        {
            Err(ink_env_err) => {
                ink::env::debug_println!("ink env error: {:?}", ink_env_err);
                Ok(())
            }
            Ok(Err(ink_lang_err)) => {
                ink::env::debug_println!("ink lang error: {:?}", ink_lang_err);
                Ok(())
            }
            Ok(Ok(res)) => res,
        }
    }

    // We don't want to fail the whole transaction if PSP22::balance_of fails with a panic either.
    // We choose to use `0` to denote the "panic" scenarios b/c it's a noop for the farm.
    pub fn safe_balance_of(psp22: &contract_ref!(PSP22), account: AccountId) -> u128 {
        match psp22.call().balance_of(account).try_invoke() {
            Err(ink_env_err) => {
                ink::env::debug_println!("ink env error: {:?}", ink_env_err);
                0
            }
            Ok(Err(ink_lang_err)) => {
                ink::env::debug_println!("ink lang error: {:?}", ink_lang_err);
                0
            }
            Ok(Ok(res)) => res,
        }
    }
}

use amm_helpers::math::{
    casted_mul,
    MathError,
};
use farm::SCALING_FACTOR;
use primitive_types::U256;

/// Returns rewards due for providing liquidity from `last_update_time` to `last_time_reward_applicable`.
///
/// r_j = r_j0 + R/T(t_j - t_j0)
///
/// where:
/// - r_j0 - reward per token stored at the last time any user interacted with the farm
/// - R - total amount of rewards available for distribution
/// - T - total shares in the farm
/// - t_j - last time user interacted with the farm, usually _now_.
/// - t_j0 - last time user "claimed" rewards.
/// - r_j - rewards due to user for providing liquidity from t_j0 to t_j
///
/// See https://github.com/stakewithus/notes/blob/main/excalidraw/staking-rewards.png for more.
pub fn reward_per_token(
    reward_per_token_stored: U256,
    reward_rate: u128,
    total_supply: u128,
    last_update_time: u128,
    last_time_reward_applicable: u128,
) -> Result<U256, MathError> {
    if total_supply == 0 {
        return Ok(reward_per_token_stored)
    }

    casted_mul(reward_rate, last_time_reward_applicable - last_update_time)
        .checked_mul(U256::from(SCALING_FACTOR))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(total_supply))
        .ok_or(MathError::DivByZero)?
        .checked_add(reward_per_token_stored)
        .ok_or(MathError::Overflow)
}

/// Returns rewards earned by the user given `rewards_per_token` for some period of time.
pub fn rewards_earned(
    shares: u128,
    rewards_per_token: U256,
    paid_reward_per_token: U256,
) -> Result<u128, MathError> {
    let r = rewards_per_token
        .checked_sub(paid_reward_per_token)
        .ok_or(MathError::Underflow)?;

    r.checked_mul(U256::from(shares))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(SCALING_FACTOR))
        .ok_or(MathError::DivByZero)?
        .try_into()
        .map_err(|_| MathError::CastOverflow)
}
