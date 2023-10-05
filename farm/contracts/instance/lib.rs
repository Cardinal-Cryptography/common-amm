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

    use crate::error::FarmError;

    use crate::views::{
        FarmDetailsView,
        UserPositionView,
    };

    use crate::{
        reward_per_share_in_time_interval,
        rewards_earned_by_shares,
    };

    use openbrush::modifiers;

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
        #[modifiers(ensure_running(true))]
        pub fn stop(&mut self) -> Result<(), FarmError> {
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
        #[modifiers(ensure_running(false))]
        pub fn withdraw_reward_tokens(&mut self) -> Result<(), FarmError> {
            self.bump()?;
            let mut running = self.get_state()?;

            let mut to_refund: Vec<(AccountId, u128)> =
                Vec::with_capacity(running.reward_tokens_info.len());

            for reward_token in running.reward_tokens_info.iter() {
                let token_id = reward_token.token_id;
                let reserved = reward_token.total_unclaimed_rewards;
                if reserved == 0 {
                    // If there are no yet-unclaimed rewards, nothing is reserved
                    // and we can skip this reward token (most probably all iterations will be skipped).
                    to_refund.push((token_id, 0));
                    continue
                }
                let psp22_ref: ink::contract_ref!(PSP22) = token_id.into();
                let balance: Balance = safe_balance_of(&psp22_ref, self.env().account_id());
                let refund_amount = balance.saturating_sub(reserved);
                to_refund.push((token_id, refund_amount));
            }

            running.reward_tokens_info = running
                .reward_tokens_info
                .clone()
                .into_iter()
                .map(|mut rti| {
                    rti.total_unclaimed_rewards = 0;
                    rti
                })
                .collect();

            self.state.set(&running);

            for (token_id, refund_amount) in to_refund {
                let mut psp22_ref: ink::contract_ref!(PSP22) = token_id.into();

                if refund_amount > 0 {
                    safe_transfer(&mut psp22_ref, running.owner, refund_amount)?;
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
                // That's b/c it won't be earning any more rewards for this particular farm.
                state.user_reward_per_share_paid.remove(caller);
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
            let manager: contract_ref!(FarmManager) = self.manager.into();

            let shares = manager.balance_of(account);
            if shares == 0 {
                return Err(FarmError::CallerNotFarmer)
            }

            let mut state = self.get_state()?;
            let current_timestamp = self.env().block_timestamp();
            let total_shares = manager.total_supply();
            state.update_rewards(total_shares, current_timestamp)?;
            state.move_user_rewards_to_claimable(shares, account)?;
            state.rewards_earned(account)

            // note that without state.set() this is still immutable
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


        fn bump(&mut self) -> Result<(), FarmError> {
            let manager: contract_ref!(FarmManager) = self.manager.into();
            let total_shares = manager.total_supply();
            let mut state = self.get_state()?;
            state.update_rewards(total_shares, self.env().block_timestamp())?;
            self.state.set(&state);
            Ok(())
        }

        fn move_user_rewards_to_claimable(&mut self, user: AccountId) -> Result<Vec<u128>, FarmError> {
            let manager: contract_ref!(FarmManager) = self.manager.into();
            let total_shares = manager.total_supply();
            let mut state = self.get_state()?;
            state.move_user_rewards_to_claimable(total_shares, user)
        } 


        fn update_reward_index(&mut self) -> Result<Vec<u128>, FarmError> {
            self.bump()?;
            self.move_user_rewards_to_claimable(self.env().caller())
        }

        fn get_state(&self) -> Result<State, FarmError> {
            self.state.get().ok_or(FarmError::StateMissing)
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

                let total_unclaimed_rewards = 0;
                let cumulative_reward_per_share = WrappedU256::ZERO;

                let info = RewardTokenInfo {
                    token_id,
                    reward_rate,
                    total_unclaimed_rewards,
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
        pub cumulative_reward_per_share: WrappedU256,
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
        pub user_reward_per_share_paid: Mapping<UserId, Vec<WrappedU256>>,
        /// Rewards that have not been claimed (withdrawn) by the user yet.
        pub user_claimable_rewards: Mapping<UserId, Vec<u128>>,
    }

    impl State {

        pub fn move_user_rewards_to_claimable(&mut self,  user_shares: u128, user: UserId) -> Result<Vec<u128>, FarmError> {
            if user_shares == 0 {
                return Ok(vec![0; self.reward_tokens_info.len()])
            }

            let mut rewards_per_share_paid_so_far = self
                .user_reward_per_share_paid
                .get(user)
                .unwrap_or(vec![WrappedU256::ZERO; self.reward_tokens_info.len()]);

            let mut uncollected_user_rewards = self
                .user_claimable_rewards
                .get(user)
                .unwrap_or(vec![0; self.reward_tokens_info.len()]);

            let mut new_rewards = vec![0u128; self.reward_tokens_info.len()];
            for (idx, reward_token) in self.reward_tokens_info.iter().enumerate() {
                let rewards_per_share_delta = reward_token.cumulative_reward_per_share.0.checked_sub(rewards_per_share_paid_so_far[idx].0).unwrap().into();
                new_rewards[idx] = rewards_earned_by_shares(
                    user_shares,
                    rewards_per_share_delta,
                )?;
                uncollected_user_rewards[idx] = uncollected_user_rewards[idx].saturating_add(new_rewards[idx]);
                rewards_per_share_paid_so_far[idx] = reward_token.cumulative_reward_per_share;
            }

            self.user_reward_per_share_paid.insert(user, &rewards_per_share_paid_so_far);
            self.user_claimable_rewards.insert(user, &uncollected_user_rewards);

            Ok(new_rewards)
        }

        pub fn update_rewards(
            &mut self,
            total_shares: u128,
            current_timestamp: Timestamp,
        ) -> Result<(), FarmError> {
            
            let past = core::cmp::max(self.timestamp_at_last_update, self.start) as u128;
            let now = core::cmp::min(current_timestamp, self.end) as u128;
            if past > now {
                return Ok(());
            }

            for reward_token in self.reward_tokens_info.iter_mut() {
                let reward_rate = reward_token.reward_rate;
                let reward_delta = reward_per_share_in_time_interval(
                    reward_rate,
                    total_shares,
                    past,
                    now,
                )?;
                let new_cumulative = reward_token.cumulative_reward_per_share.0 + reward_delta;
                reward_token.cumulative_reward_per_share = new_cumulative.into();
                reward_token.total_unclaimed_rewards += rewards_earned_by_shares(
                    total_shares,
                    reward_delta,
                )?;
            }
            self.timestamp_at_last_update = current_timestamp;
            Ok(())
        }

        /// Returns the amount of rewards earned by the given account.
        pub fn rewards_earned(
            &self,
            account: AccountId,
        ) -> Result<Vec<u128>, FarmError> {

            let uncollected_user_rewards = self
                .user_claimable_rewards
                .get(account)
                .unwrap_or(vec![0; self.reward_tokens_info.len()]);


            Ok(uncollected_user_rewards)
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
            Ok(Ok(Err(psp22_error))) => {
                ink::env::debug_println!("psp22 error: {:?}", psp22_error);
                Ok(())
            }
            Ok(Ok(Ok(res))) => Ok(res),
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
    cumulative_reward_per_share: U256,
    reward_rate: u128,
    total_supply: u128,
    last_update_time: u128,
    last_time_reward_applicable: u128,
) -> Result<U256, MathError> {
    if total_supply == 0 {
        return Ok(cumulative_reward_per_share)
    }

    casted_mul(reward_rate, last_time_reward_applicable - last_update_time)
        .checked_mul(U256::from(SCALING_FACTOR))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(total_supply))
        .ok_or(MathError::DivByZero)?
        .checked_add(cumulative_reward_per_share)
        .ok_or(MathError::Overflow)
}

pub fn reward_per_share_in_time_interval(
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

/// Returns rewards earned by the user given `rewards_per_token` for some period of time.
pub fn rewards_earned(
    shares: u128,
    rewards_per_token: U256,
    paid_reward_per_token: U256,
) -> Result<u128, MathError> {
    let r = rewards_per_token
        .checked_sub(paid_reward_per_token)
        .ok_or(MathError::Underflow)?;

    rewards_earned_by_shares(shares, r)
}

pub fn rewards_earned_by_shares(
    shares: u128,
    rewards_per_share: U256,
) -> Result<u128, MathError> {

        rewards_per_share.checked_mul(U256::from(shares))
        .ok_or(MathError::Overflow)?
        .checked_div(U256::from(SCALING_FACTOR))
        .ok_or(MathError::DivByZero)?
        .try_into()
        .map_err(|_| MathError::CastOverflow)
}
