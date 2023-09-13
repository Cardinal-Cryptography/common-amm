#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod error;

// TODO:
// Add upper bound on farm length.
// Tests.
// ? Refactor to make staking logic reusable in different contracts.

#[ink::contract]
mod farm {
    use crate::error::{
        FarmError,
        FarmStartError,
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

    use amm_helpers::{
        math::{
            casted_mul,
            MathError,
        },
        types::WrappedU256,
    };

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
        pool: contract_ref!(PSP22),
        /// Address of the farm creator.
        owner: AccountId,
        /// Whether the farm is running now.
        is_running: bool,
        /// Farm state.
        state: Lazy<State, ManualKey<0x4641524d>>,
    }

    const SCALING_FACTOR: u128 = u128::MAX;
    const MAX_REWARD_TOKENS: u32 = 10;

    impl Farm {
        #[ink(constructor)]
        pub fn new(pair_address: AccountId) -> Self {
            Farm {
                pool: pair_address.into(),
                owner: Self::env().caller(),
                is_running: false,
                state: Lazy::new(),
            }
        }

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
        #[ink(message)]
        pub fn start(
            &mut self,
            end: Timestamp,
            reward_amounts: Vec<u128>,
            reward_tokens: Vec<AccountId>,
        ) -> Result<(), FarmStartError> {
            if self.is_running {
                return Err(FarmStartError::StillRunning)
            }
            // (For now) we don't allow for "restarting" the farm.
            if self.state.get().is_some() {
                return Err(FarmStartError::FarmAlreadyFinished)
            }

            if reward_tokens.len() > MAX_REWARD_TOKENS as usize {
                return Err(FarmStartError::TooManyRewardTokens)
            }

            let farm_owner = self.owner;
            if Self::env().caller() != farm_owner {
                return Err(FarmStartError::CallerNotOwner)
            }

            let now = Self::env().block_timestamp();

            if now >= end {
                return Err(FarmStartError::FarmEndBeforeStart)
            }

            let duration = end as u128 - now as u128;

            if reward_amounts.len() != reward_tokens.len() {
                return Err(FarmStartError::RewardAmountsAndTokenLengthDiffer)
            }

            let tokens_len = reward_tokens.len();

            let mut reward_rates = Vec::with_capacity(tokens_len);

            for i in 0..tokens_len {
                let reward_amount = reward_amounts[i];

                if reward_amount == 0 {
                    return Err(FarmStartError::ZeroRewardAmount)
                }
                let rate = reward_amount
                    .checked_div(duration)
                    .ok_or(FarmStartError::ArithmeticError)?;

                if rate == 0 {
                    return Err(FarmStartError::ZeroRewardRate)
                }

                // Double-check we have enough to cover the whole farm.
                if duration * rate < reward_amount {
                    return Err(FarmStartError::InsufficientRewardAmount)
                }

                let mut psp22_ref: ink::contract_ref!(PSP22) = reward_tokens[i].into();

                psp22_ref.transfer_from(
                    farm_owner,
                    Self::env().account_id(),
                    reward_amount,
                    vec![],
                )?;

                reward_rates.push(rate);
            }

            let state = State {
                owner: farm_owner,
                start: now,
                end,
                reward_rates,
                reward_tokens,
                reward_per_token_stored: vec![WrappedU256::ZERO; tokens_len],
                timestamp_at_last_update: now,
                total_shares: 0,
                shares: Mapping::new(),
                user_reward_per_token_paid: Mapping::new(),
                user_unclaimed_rewards: Mapping::new(),
                total_unclaimed_rewards: vec![0; tokens_len],
            };

            self.state.set(&state);
            self.is_running = true;

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

            // We allow owner of the farm to stop it prematurely
            // while anyone else can change the farm's status only when it's finished.
            if self.env().caller() == self.owner {
                running.end = self.env().block_timestamp();
            } else if self.env().block_timestamp() < running.end {
                return Err(FarmError::StillRunning)
            }

            self.is_running = false;
            self.state.set(&running);

            // Send remaining rewards to the farm owner.
            for (idx, reward_token) in running.reward_tokens.iter().enumerate() {
                let mut psp22_ref: ink::contract_ref!(PSP22) = (*reward_token).into();
                let balance: Balance = safe_balance_of(&psp22_ref, self.env().account_id());
                let reserved = running.total_unclaimed_rewards[idx];
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

            let contract = self.env().account_id();
            let caller = self.env().caller();

            let mut running_state = self.get_state()?;

            let prev_share = running_state.shares.get(caller).unwrap_or(0);
            running_state.shares.insert(caller, &(prev_share + amount));
            running_state.total_shares += amount;

            self.state.set(&running_state);

            self.pool.transfer_from(caller, contract, amount, vec![])?;

            self.env().emit_event(Deposited {
                account: caller,
                amount,
            });
            Ok(())
        }

        /// Withdraws the given amount of shares from the farm.
        #[ink(message)]
        #[modifiers(non_zero_amount(amount))]
        pub fn withdraw(&mut self, amount: u128) -> Result<(), FarmError> {
            let caller = self.env().caller();

            let mut state = self.get_state()?;

            let shares = state.shares.get(caller).ok_or(FarmError::CallerNotFarmer)?;

            if shares < amount {
                return Err(FarmError::InvalidWithdrawAmount)
            }

            self.update_reward_index()?;

            state.shares.insert(caller, &(shares - amount));
            state.total_shares -= amount;

            self.state.set(&state);

            self.pool.transfer(caller, amount, vec![])?;

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
                .get(caller)
                .ok_or(FarmError::CallerNotFarmer)?;

            // Reset state before calling PSP22 methods.
            // Reentrancy protection.
            state
                .user_unclaimed_rewards
                .insert(caller, &vec![0; user_rewards.len()]);

            self.state.set(&state);

            for (user_reward, reward_token) in user_rewards
                .clone()
                .into_iter()
                .zip(state.reward_tokens.iter())
            {
                if user_reward > 0 {
                    let mut psp22_ref: ink::contract_ref!(PSP22) = (*reward_token).into();
                    safe_transfer(&mut psp22_ref, caller, user_reward)?;
                }
            }
            for (idx, reward) in user_rewards.iter().enumerate() {
                state.total_unclaimed_rewards[idx] =
                    state.total_unclaimed_rewards[idx].saturating_sub(*reward);
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
        #[ink(message)]
        pub fn claimable(&self, account: AccountId) -> Result<Vec<u128>, FarmError> {
            let state = self.get_state()?;
            let rewards_per_token = state.rewards_per_token(self.env().block_timestamp())?;
            let user_rewards = state.rewards_earned(account, &rewards_per_token)?;

            Ok(user_rewards)
        }

        /// Returns the amount of rewards per token that have been accumulated for the given account.
        fn update_reward_index(&mut self) -> Result<Vec<u128>, FarmError> {
            let account = self.env().caller();

            let mut state = self.get_state()?;

            let rewards_per_token = state.rewards_per_token(self.env().block_timestamp())?;
            let user_rewards = state.rewards_earned(account, &rewards_per_token)?;

            state.total_unclaimed_rewards =
                state.rewards_distributable(self.env().block_timestamp());
            state.user_reward_per_token_paid.insert(
                account,
                &rewards_per_token
                    .clone()
                    .into_iter()
                    .map(WrappedU256::from)
                    .collect::<Vec<_>>(),
            );
            state.user_unclaimed_rewards.insert(account, &user_rewards);
            state.reward_per_token_stored = rewards_per_token.into_iter().map(Into::into).collect();

            self.state.set(&state);

            Ok(user_rewards)
        }

        fn get_state(&self) -> Result<State, FarmError> {
            self.state.get().ok_or(FarmError::StateMissing)
        }
    }

    type TokenId = AccountId;
    type UserId = AccountId;

    #[ink::storage_item]
    pub struct State {
        /// Creator(owner) of the farm.
        pub owner: UserId,
        /// The timestamp when the farm was created.
        pub start: Timestamp,
        /// The timestamp when the farm will stop.
        pub end: Timestamp,
        /// How many rewards to pay out for the smallest unit of time.
        pub reward_rates: Vec<u128>,
        /// Tokens deposited as rewards for providing LP to the farm.
        pub reward_tokens: Vec<TokenId>,
        /// Reward counter at the last farm change.
        pub reward_per_token_stored: Vec<WrappedU256>,
        /// Timestamp of the last farm change.
        pub timestamp_at_last_update: Timestamp,
        /// Total shares in the farm after the last action.
        pub total_shares: u128,
        /// How many shares each user has in the farm.
        pub shares: Mapping<UserId, u128>,
        /// Reward per token paid to the user for each reward token.
        // We need to track this separately for each reward token as each can have different reward rate.
        // Vectors should be relatively small (probably < 5).
        pub user_reward_per_token_paid: Mapping<UserId, Vec<WrappedU256>>,
        /// Rewards that have not been claimed (withdrawn) by the user yet.
        pub user_unclaimed_rewards: Mapping<UserId, Vec<u128>>,
        /// Totals of unclaimed rewards.
        // Necessary for not letting owner, re-claim more than allowed to once the farm is stopped.
        pub total_unclaimed_rewards: Vec<u128>,
    }

    impl State {
        /// Calculates rewards per token due for providing liquidity to the farm
        /// since the last update until `last_timestamp_applicable`.
        ///
        /// Returned value is a vector of numbers, one for each reward token in the farm.
        pub fn rewards_per_token(
            &self,
            current_timestamp: Timestamp,
        ) -> Result<Vec<U256>, FarmError> {
            let mut rewards_per_token: Vec<U256> = Vec::with_capacity(self.reward_tokens.len());

            for i in 0..self.reward_tokens.len() {
                let reward_rate: u128 = self.reward_rates[i];
                let rpr = reward_per_token(
                    self.reward_per_token_stored[i].0,
                    reward_rate,
                    self.total_shares,
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
            rewards_per_token: &[U256],
        ) -> Result<Vec<u128>, FarmError> {
            let shares = self.shares.get(account).ok_or(FarmError::CallerNotFarmer)?;

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

        /// Returns rewards distribuatble to all farmers for the period.
        pub fn rewards_distributable(&self, current_timestamp: Timestamp) -> Vec<u128> {
            let last_time_reward_applicable = core::cmp::min(current_timestamp, self.end) as u128;
            self.reward_rates
                .iter()
                .map(|reward_rate| {
                    reward_rate
                        .checked_mul(
                            last_time_reward_applicable - self.timestamp_at_last_update as u128,
                        )
                        .unwrap_or(0)
                })
                .collect()
        }
    }

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
    fn reward_per_token(
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
    fn rewards_earned(
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
        if !should_be_running && instance.is_running {
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

    #[cfg(test)]
    mod tests {
        use ink::env::DefaultEnvironment;

        use super::*;

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<Environment>(sender);
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts<Environment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        #[cfg(test)]
        mod reward_calculation {
            use crate::farm::SCALING_FACTOR;
            use amm_helpers::math::casted_mul;
            use primitive_types::U256;

            // 100 reward tokens for every t=1.
            const REWARD_RATE: u128 = 100;

            // Handy wrappers to use in tests.
            fn reward_per_token(
                reward_per_token_stored: U256,
                reward_rate: u128,
                total_supply: u128,
                last_update_time: u128,
                last_time_reward_applicable: u128,
            ) -> U256 {
                super::reward_per_token(
                    reward_per_token_stored,
                    reward_rate,
                    total_supply,
                    last_update_time,
                    last_time_reward_applicable,
                )
                .expect("to calculate reward per token")
            }

            fn rewards_earned(
                shares: u128,
                rewards_per_token: U256,
                paid_reward_per_token: U256,
            ) -> u128 {
                super::rewards_earned(shares, rewards_per_token, paid_reward_per_token)
                    .expect("to calculate rewards earned")
            }

            /// Case when there's a single farmer,
            /// staking 100 tokens, from t=3 until t=5.
            /// shares:
            //       ▲
            //       │
            //   100 │    ┌─────┐
            //       └────┴─────┴──►
            //            3     5    t
            #[test]
            fn single_farmer_simple() {
                let shares = 100;
                let total_supply = shares;

                let rewards_per_token =
                    reward_per_token(U256::zero(), REWARD_RATE, total_supply, 3, 5);
                // = r_j0 + R/T(t_j - t_j0)
                // = 0 + 100/100 * 2
                // = 2
                assert_eq!(rewards_per_token, casted_mul(2, super::SCALING_FACTOR));
                let reward_earned = rewards_earned(shares, rewards_per_token, U256::zero());
                assert_eq!(reward_earned, 200);
            }

            /// Case when there's a single farmer,
            /// staking 100 tokens, from t=3 until t=5,
            /// then topping up with 200 tokens more,
            /// and exiting at t=8.
            /// For t=3 until t=5, the farmer should get 200 tokens.
            /// For t=5 until t=8, the farmer should get 300 tokens.
            /// Total: 500 tokens.
            ///
            ///    ▲
            //     │
            //     │
            // 300 │          ┌─────┐
            //     │          │     │
            // 100 │    ┌─────┘     │
            //     └────┴───────────┴───►
            //          3     5     8     t
            #[test]
            fn single_farmer_top_up() {
                let shares = 100;
                let total_supply = shares;
                let rewards_per_token_from0_till3 = U256::zero();

                let rewards_per_token_from0_till5 = reward_per_token(
                    rewards_per_token_from0_till3,
                    REWARD_RATE,
                    total_supply,
                    3,
                    5,
                );
                // expected value is
                // = r_j0 + R/T(t_j-t_j0)
                // = 0 + REWARD_RATE / TOTAL_SUPPLY * (5 - 3)
                // = 0 + 100/100 * 2
                // = 2
                assert_eq!(rewards_per_token_from0_till5, casted_mul(2, SCALING_FACTOR));
                let reward_earned =
                    rewards_earned(shares, rewards_per_token_from0_till5, U256::zero());
                assert_eq!(reward_earned, 200);

                let shares: u128 = 300;
                let total_supply = shares;
                let rewards_per_token_from0_till8 = reward_per_token(
                    rewards_per_token_from0_till5,
                    REWARD_RATE,
                    total_supply,
                    5,
                    8,
                );
                // Reminder: expected value is:
                // = r_j0 + R/T(t_j-t_j0)
                // = r_j0 + REWARD_RATE / TOTAL_SUPPLY * (8 - 5)
                // = r_j0 + 100/300 * 3
                // = r_j0 + 1
                let expected_second = rewards_per_token_from0_till5 + 1 * super::SCALING_FACTOR;
                assert_eq!(rewards_per_token_from0_till8, expected_second);
                let reward_earned = rewards_earned(
                    shares,
                    rewards_per_token_from0_till8,
                    rewards_per_token_from0_till5,
                );
                assert_eq!(reward_earned, 300);
            }

            //     ▲
            //     │
            //     │
            // 300 │    ┌─────┐
            //     │    │     │
            // 100 │    │     └─────┐
            //     └────┴───────────┴───►
            //          3     5     8     t
            #[test]
            fn single_farmer_withdraw_partial() {
                let shares = 300;
                let total_supply = shares;
                let rewards_per_token_from0_till3 = U256::zero();

                let rewards_per_token_from0_till5 = reward_per_token(
                    rewards_per_token_from0_till3,
                    REWARD_RATE,
                    total_supply,
                    3,
                    5,
                );
                // expected value is
                // = r_j0 + R/T(t_j-t_j0)
                // = 0 + (100 * 2s)/300 * SCALING_FACTOR
                // = 200/300 * SCALING_FACTOR
                // = 2/3 * SCALING_FACTOR
                let expected = casted_mul(2, SCALING_FACTOR) / 3;
                assert_eq!(rewards_per_token_from0_till5, expected);
                let reward_earned =
                    rewards_earned(shares, rewards_per_token_from0_till5, U256::zero());
                // expected value is 200:
                // = reward_per_token * shares / SCALING_FACTOR
                // = (2/300 * SCALING_FACTOR) * 300 / SCALING_FACTOR
                // = 200
                assert_eq!(reward_earned, 200);

                let shares: u128 = 100;
                let total_supply = shares;
                let rewards_per_token = reward_per_token(
                    rewards_per_token_from0_till5,
                    REWARD_RATE,
                    total_supply,
                    5,
                    8,
                );
                // Expected value is:
                // = r_j0 + R/T(t_j-t_j0)
                // = 200/300  + 100/100 * 3
                // = (3 + 2/3)
                // modulo SCALING_FACTOR
                let expected = expected + casted_mul(3, SCALING_FACTOR);
                assert_eq!(rewards_per_token, expected);
                let reward_earned =
                    rewards_earned(shares, rewards_per_token, rewards_per_token_from0_till5);
                assert_eq!(reward_earned, 300);
            }

            //     ▲
            // 300 │      ┌────┐
            //     │      │    └─────┐ 200
            //     │      │   BOB    │
            // 100 │   ┌──┴────┐     │
            //     │   │ ALICE │     │
            //     └───┴───────┴─────┴──────►
            //         3  5    7    10       t
            #[test]
            fn two_farmers_overlap() {
                let alice = 100;
                let bob = 200;
                let reward_per_token_from0_till3 = U256::zero();

                // Alice deposits 100 at t=3;
                // = r_j0 + R/T(t_j-t_j0)
                // = 100/100 * 2
                // = 2
                let rewards_per_token_from0_till5 =
                    reward_per_token(reward_per_token_from0_till3, REWARD_RATE, alice, 3, 5);
                let expected = casted_mul(2, SCALING_FACTOR);
                assert_eq!(rewards_per_token_from0_till5, expected);

                // Bob deposits 200 at t=5;
                let reward_per_token_from0_till7 = reward_per_token(
                    rewards_per_token_from0_till5,
                    REWARD_RATE,
                    alice + bob,
                    5,
                    7,
                );
                // = r_j0 + R/T(t_j-t_j0)
                // = 2 + 100/300 * 2
                // = 2 + 2/3
                let expected = expected + casted_mul(2, SCALING_FACTOR) / 3;
                assert_eq!(reward_per_token_from0_till7, expected);

                // Alice withdraws 100 at t=7;
                let alice_reward_earned =
                    rewards_earned(alice, reward_per_token_from0_till7, U256::zero());

                // Expected value is:
                // 2 full rewards for 2 units of time when she's the only farmer.
                // 1/3 * 2 worth of reward for 2 units of time when she has 1/3 of shares.
                // Scaled with SCALING_FACTOR for fixed point arithmetic.
                // reward_rate(ALICE) = 8/3R = 2 2/3 R, where R=reward_rate
                // rewards_earned(ALICE) = reward_rate(ALICE) * shares(ALICE) / SCALING_FACTOR
                let alice_expected: u128 = (expected * U256::from(alice))
                    .checked_div(U256::from(SCALING_FACTOR))
                    .expect("to calculate alice_expected")
                    .try_into()
                    .expect("to cast alice_expected to u128");
                assert_eq!(alice_expected, alice_reward_earned);

                // Bob withdraws 200 at t=10;
                // = r_j0 + R/T(t_j-t_j0)
                // = r_j0 + REWARD_RATE/200 * 3
                // = r_j0 + 100*3/200
                // = r_j0 + 3/2
                // = 2 + 2/3 + 3/2
                // = 4 + 1/6
                let reward_per_token_from0_till10 =
                    reward_per_token(reward_per_token_from0_till7, REWARD_RATE, bob, 7, 10);
                let expected_rate = casted_mul(4, SCALING_FACTOR) + U256::from(SCALING_FACTOR) / 6;
                assert_eq!(reward_per_token_from0_till10, expected_rate);
                let bob_rewards_earned =
                    rewards_earned(bob, reward_per_token_from0_till10, U256::zero());
                // 2/3 * 2 worth of reward for 3 units of time when he has 2/3 of shares.
                // 3 full rewards for 3 units of time when he's the only farmer.
                // rewards_earned(BOB) = reward_rate(BOB) * shares(BOB) / SCALING_FACTOR
                let bob_expected: u128 = (expected_rate * U256::from(bob))
                    .checked_div(U256::from(SCALING_FACTOR))
                    .expect("to calculate bob_expected")
                    .try_into()
                    .expect("to cast bob_expected to u128");
                assert_eq!(bob_expected, bob_rewards_earned);
            }

            //     ▲
            //  400│            ┌──────┐  ┌─┐
            //     │            │CAROL │  │ │
            //  300│          ┌─┴──────┼──┘ │
            //     │          │        │ C  │
            //  200│          │        └────┼───┐
            //     │          │ BOB         │   │
            //     │    ┌─────┴────────┐    │ C │
            //  100│    │    ALICE     │    │   │
            //     └────┴──────────────┴────┴───┴──────►
            //         3     5 6       9 10 11 13     t
            //
            // t=3: Alice deposits 100
            // t=5: Bob deposits 200
            // t=6: Carol deposits 100
            // t=9: Alice withdraws 100
            // t=10: Carol deposits 100
            // t=11: Bob withdraws 200
            // t=13: Carol withdraws 200
            //
            #[test]
            fn three_farmers_overlap_topup() {
                let alice = 100;
                let bob = 200;
                let carol = 100;

                let reward_per_token_from0_till5 =
                    reward_per_token(U256::zero(), REWARD_RATE, alice, 3, 5);

                let reward_per_token_from0_till6 =
                    reward_per_token(reward_per_token_from0_till5, REWARD_RATE, alice + bob, 5, 6);

                let reward_per_token_from0_till9 = reward_per_token(
                    reward_per_token_from0_till6,
                    REWARD_RATE,
                    alice + bob + carol,
                    6,
                    9,
                );

                let alice_reward_earned =
                    rewards_earned(alice, reward_per_token_from0_till9, U256::zero());
                assert_eq!(alice_reward_earned, 37 * alice / 12);

                let reward_rate_from0_till10 = reward_per_token(
                    reward_per_token_from0_till9,
                    REWARD_RATE,
                    bob + carol,
                    9,
                    10,
                );
                let expected_reward_rate =
                    casted_mul(3, SCALING_FACTOR) + casted_mul(5, SCALING_FACTOR) / 12;
                assert_eq!(reward_rate_from0_till10, expected_reward_rate);
                let new_carol = carol + 100;
                let reward_rate_from0_till11 = reward_per_token(
                    reward_rate_from0_till10,
                    REWARD_RATE,
                    bob + new_carol,
                    10,
                    11,
                );
                let expected_reward_rate = expected_reward_rate + (SCALING_FACTOR / 4);
                assert_eq!(reward_rate_from0_till11, expected_reward_rate,);
                let bob_earned = rewards_earned(bob, reward_rate_from0_till11, U256::zero());
                let bob_expected: u128 = (expected_reward_rate * U256::from(bob))
                    .checked_div(U256::from(SCALING_FACTOR))
                    .expect("to calculate bob_expected")
                    .try_into()
                    .expect("to cast bob_expected to u128");
                assert_eq!(bob_earned, bob_expected);
            }
        }

        #[cfg(test)]
        mod farm_start {
            use super::*;

            use ink::env::test::set_block_timestamp;

            fn pool_id() -> AccountId {
                AccountId::from([0x01; 32])
            }

            fn farm() -> Farm {
                Farm::new(pool_id())
            }

            fn single_reward_token() -> Vec<AccountId> {
                vec![AccountId::from([0x02; 32])]
            }

            #[ink::test]
            fn new_creates_stopped_farm() {
                let farm = farm();
                assert_eq!(farm.is_running, false);
            }

            #[ink::test]
            fn non_owner_cannot_start_farm() {
                set_sender(alice());

                let mut farm = farm();
                set_block_timestamp::<DefaultEnvironment>(1);
                let reward_tokens = single_reward_token();
                set_sender(bob());
                assert_eq!(
                    farm.start(5, vec![300], reward_tokens),
                    Err(FarmStartError::CallerNotOwner)
                );
            }

            #[ink::test]
            fn farm_end_before_start() {
                let mut farm = farm();
                set_block_timestamp::<DefaultEnvironment>(5);
                let reward_tokens = single_reward_token();
                let reward_amounts = vec![300];
                assert_eq!(
                    farm.start(2, reward_amounts, reward_tokens),
                    Err(FarmStartError::FarmEndBeforeStart)
                );
            }

            #[ink::test]
            fn farm_too_many_tokens_fails() {
                let mut farm = farm();
                set_block_timestamp::<DefaultEnvironment>(1);
                let reward_tokens = (0..super::MAX_REWARD_TOKENS + 10)
                    .into_iter()
                    .map(|i| AccountId::from([i as u8; 32]))
                    .collect::<Vec<_>>();
                let reward_amounts = vec![300];
                assert_eq!(
                    farm.start(1000, reward_amounts, reward_tokens),
                    Err(FarmStartError::TooManyRewardTokens)
                );
            }

            #[ink::test]
            fn reward_amounts_and_tokens_mismatch() {
                let mut farm = farm();
                let reward_tokens = single_reward_token();
                let reward_amounts = vec![300, 400];
                assert_eq!(
                    farm.start(10, reward_amounts, reward_tokens),
                    Err(FarmStartError::RewardAmountsAndTokenLengthDiffer)
                );
            }

            #[ink::test]
            fn fail_on_zero_reward_amount() {
                let mut farm = farm();
                let reward_tokens = single_reward_token();
                let reward_amounts = vec![0];
                assert_eq!(
                    farm.start(10, reward_amounts, reward_tokens),
                    Err(FarmStartError::ZeroRewardAmount)
                );
            }

            #[ink::test]
            fn fail_on_insufficient_rewards() {
                let mut farm = farm();
                let reward_tokens = single_reward_token();
                let reward_amounts = vec![10];
                // reward_rate = reward / duration
                // rr = 10 / 100 == 0;
                assert_eq!(
                    farm.start(100, reward_amounts, reward_tokens),
                    Err(FarmStartError::ZeroRewardRate)
                );
            }
        }

        // Tests:
        // Deposit:
        // - deposit with non-zero balance succeeds
        // - deposit as first farmer takes all shares
        // - deposit triggers claim
        // - deposit as second farmer splits shares and updates reward counter properly
        // - multiple, repeated deposits by the same farmer update reward counter properly
        //
        // Withdraw:
        // Stop:
        // Create & Start farm:
    }
}
