#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod farm {
    type TokenId = AccountId;
    type UserId = AccountId;
    use amm_helpers::{ensure, math::casted_mul, types::WrappedU256};
    use farm_trait::{Farm, FarmDetails, FarmError};
    use ink::{
        codegen::{EmitEvent, TraitCallBuilder},
        contract_ref,
        reflect::ContractEventBase,
        storage::Mapping,
    };

    use ink::prelude::{vec, vec::Vec};
    use primitive_types::U256;

    use psp22::PSP22;

    pub const SCALING_FACTOR: u128 = u128::MAX;
    pub const MAX_REWARD_TOKENS: u32 = 10;

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

    use amm_helpers::math::MathError;

    pub type Event = <FarmContract as ContractEventBase>::Type;

    #[ink(storage)]
    pub struct FarmContract {
        /// Address of the token pool for which this farm is created.
        pub pool_id: AccountId,
        /// Address of the farm creator.
        owner: AccountId,
        /// How many shares each user has in the farm.
        shares: Mapping<UserId, u128>,
        /// Total shares in the farm after the last action.
        total_shares: u128,
        /// Reward tokens.
        pub reward_tokens: Vec<TokenId>,

        /// The timestamp when the farm should start.
        pub start: Timestamp,
        /// The timestamp when the farm will stop.
        pub end: Timestamp,
        /// The timestamp at the last call to update().
        pub timestamp_at_last_update: Timestamp,

        /// Total rewards that have been distributed but not yet claimed by users.
        pub farm_distributed_unclaimed_rewards: Vec<u128>,
        /// Cumulative rewards distributed per share since `start` and until `timestamp_at_last_update`.
        pub farm_cumulative_reward_per_share: Vec<WrappedU256>,
        /// Rewards rate - how many rewards per smallest unit of time are distributed.
        pub farm_reward_rates: Vec<u128>,

        /// cumulative_per_share at the last update for each user.
        pub user_cumulative_reward_last_update: Mapping<UserId, Vec<WrappedU256>>,

        /// Reward rates per user of unclaimed, accumulated users' rewards.
        pub user_claimable_rewards: Mapping<UserId, Vec<u128>>,
    }

    impl FarmContract {
        #[ink(constructor)]
        pub fn new(pool_id: AccountId, reward_tokens: Vec<TokenId>) -> Result<Self, FarmError> {
            let n_reward_tokens = reward_tokens.len();
            if n_reward_tokens > MAX_REWARD_TOKENS as usize {
                return Err(FarmError::TooManyRewardTokens);
            }
            if reward_tokens.contains(&pool_id) {
                return Err(FarmError::RewardTokenIsPoolToken);
            }
            Ok(FarmContract {
                pool_id,
                owner: Self::env().caller(),
                shares: Mapping::default(),
                total_shares: 0,
                reward_tokens,
                start: 0,
                end: 0,
                timestamp_at_last_update: 0,
                farm_distributed_unclaimed_rewards: vec![0; n_reward_tokens],
                farm_cumulative_reward_per_share: vec![WrappedU256::ZERO; n_reward_tokens],
                farm_reward_rates: vec![0; n_reward_tokens],
                user_cumulative_reward_last_update: Mapping::default(),
                user_claimable_rewards: Mapping::default(),
            })
        }

        fn is_active(&self) -> bool {
            let current_timestamp = self.env().block_timestamp();
            current_timestamp >= self.start && current_timestamp < self.end
        }

        // Guarantee: after calling update() it holds that self.timestamp_at_last_update = self.env().block_timestamp()
        fn update(&mut self) -> Result<(), FarmError> {
            let current_timestamp = self.env().block_timestamp();
            if self.timestamp_at_last_update >= current_timestamp {
                return Ok(());
            };

            let prev = core::cmp::max(self.timestamp_at_last_update, self.start);
            let now = core::cmp::min(current_timestamp, self.end);
            if prev >= now {
                self.timestamp_at_last_update = current_timestamp;
                return Ok(());
            }

            // At this point we know [prev, now] is the intersection of [self.start, self.end] and [self.timestamp_at_last_update, current_timestamp]
            // It is non-empty because of the checks above and self.start <= now <= self.end

            for idx in 0..self.reward_tokens.len() {
                let delta_reward_per_share = rewards_per_share_in_time_interval(
                    self.farm_reward_rates[idx],
                    self.total_shares,
                    prev as u128,
                    now as u128,
                )?;
                let delta_reward_distributed =
                    rewards_earned_by_shares(self.total_shares, delta_reward_per_share)?;
                self.farm_distributed_unclaimed_rewards[idx] = self
                    .farm_distributed_unclaimed_rewards[idx]
                    .saturating_add(delta_reward_distributed);
                self.farm_cumulative_reward_per_share[idx] = self.farm_cumulative_reward_per_share
                    [idx]
                    .0
                    .saturating_add(delta_reward_per_share)
                    .into();
            }

            self.timestamp_at_last_update = current_timestamp;
            Ok(())
        }

        // Guarantee: after calling update_account(acc) it holds that
        // 1) both self.user_cumulative_last_update[acc] and self.user_claimable_rewards[acc] exist
        // 2) self.user_cumulative_last_update[acc][i] = self.farm_cumulative[i] for all i
        fn update_account(&mut self, account: AccountId) {
            let user_shares = self.shares.get(account).unwrap_or(0);
            let new_reward_vector = match self.user_cumulative_reward_last_update.take(account) {
                Some(user_cumulative_reward_last_update) => {
                    let mut user_claimable_rewards = self
                        .user_claimable_rewards
                        .take(account)
                        .unwrap_or(vec![0; self.reward_tokens.len()]);
                    for (idx, user_cumulative) in
                        user_cumulative_reward_last_update.into_iter().enumerate()
                    {
                        let user_reward = rewards_earned_by_shares(
                            user_shares,
                            self.farm_cumulative_reward_per_share[idx]
                                .0
                                .saturating_sub(user_cumulative.0),
                        )
                        .unwrap_or(0);
                        user_claimable_rewards[idx] =
                            user_claimable_rewards[idx].saturating_add(user_reward);
                    }
                    user_claimable_rewards
                }
                None => {
                    vec![0; self.reward_tokens.len()]
                }
            };
            self.user_claimable_rewards
                .insert(account, &new_reward_vector);
            self.user_cumulative_reward_last_update
                .insert(account, &self.farm_cumulative_reward_per_share);
        }

        fn assert_start_params(
            &self,
            start: Timestamp,
            end: Timestamp,
            rewards: Vec<u128>,
        ) -> Result<Vec<u128>, FarmError> {
            let now = Self::env().block_timestamp();
            let tokens_len = self.reward_tokens.len();

            if rewards.len() != tokens_len {
                return Err(FarmError::RewardsTokensMismatch);
            }

            // NOTE: `timestamp_at_last_update == now` in `self.update()` called before this.

            if start < now {
                return Err(FarmError::FarmStartInThePast);
            }

            if end <= now {
                return Err(FarmError::FarmEndInThePast);
            }

            let duration = if let Some(duration) = end.checked_sub(start) {
                duration as u128
            } else {
                return Err(FarmError::FarmDuration);
            };

            let mut reward_rates = Vec::with_capacity(tokens_len);

            for (token_id, reward_amount) in self.reward_tokens.iter().zip(rewards.iter()) {
                let mut psp22_ref: ink::contract_ref!(PSP22) = (*token_id).into();

                psp22_ref.transfer_from(
                    self.owner,
                    self.env().account_id(),
                    *reward_amount,
                    vec![],
                )?;

                let reward_rate = reward_amount
                    .checked_div(duration)
                    .ok_or(FarmError::ArithmeticError(MathError::DivByZero(3)))?;

                reward_rates.push(reward_rate);
            }

            if reward_rates.iter().all(|rr| *rr == 0) {
                return Err(FarmError::AllRewardRatesZero);
            }

            Ok(reward_rates)
        }

        fn deposit(&mut self, account: AccountId, amount: u128) -> Result<(), FarmError> {
            if amount == 0 {
                return Err(FarmError::InsufficientShares);
            }
            self.update()?;
            self.update_account(account);
            let mut pool: contract_ref!(PSP22) = self.pool_id.into();
            pool.transfer_from(account, self.env().account_id(), amount, vec![])?;
            let shares = self.shares.get(account).unwrap_or(0);
            self.shares.insert(account, &(shares + amount));
            self.total_shares += amount;
            Ok(())
        }

        fn emit_event<EE: EmitEvent<Self>>(emitter: EE, event: Event) {
            emitter.emit_event(event);
        }
    }

    impl Farm for FarmContract {
        #[ink(message)]
        fn pool_id(&self) -> AccountId {
            self.pool_id
        }

        // wouldn't just "total_shares" be a better name, "total_supply" might suggest that farm has its own token?
        #[ink(message)]
        fn total_shares(&self) -> u128 {
            self.total_shares
        }

        #[ink(message)]
        fn shares_of(&self, owner: AccountId) -> u128 {
            self.shares.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        fn reward_tokens(&self) -> Vec<AccountId> {
            self.reward_tokens.clone()
        }

        #[ink(message)]
        fn owner_start_new_farm(
            &mut self,
            start: Timestamp,
            end: Timestamp,
            rewards: Vec<u128>,
        ) -> Result<(), FarmError> {
            if self.env().caller() != self.owner {
                return Err(FarmError::CallerNotOwner);
            }
            self.update()?;
            if self.is_active() {
                return Err(FarmError::FarmAlreadyRunning);
            }
            self.farm_reward_rates = self.assert_start_params(start, end, rewards.clone())?;
            self.start = start;
            self.end = end;
            Ok(())
        }

        #[ink(message)]
        fn owner_stop_farm(&mut self) -> Result<(), FarmError> {
            if self.env().caller() != self.owner {
                return Err(FarmError::CallerNotOwner);
            }
            self.update()?;
            self.end = self.env().block_timestamp();
            Ok(())
        }

        #[ink(message)]
        fn owner_withdraw_token(&mut self, token: TokenId) -> Result<(), FarmError> {
            ensure!(self.env().caller() == self.owner, FarmError::CallerNotOwner);
            self.update()?;
            ensure!(!self.is_active(), FarmError::FarmAlreadyRunning);

            // Owner should be able to withdraw every token except the pool token.
            ensure!(self.pool_id != token, FarmError::RewardTokenIsPoolToken);
            let mut token_ref = token.into();

            // To me it seems that both "safe" calls in this functions should fail when the error arises.
            // Effect is actually the same, but returning that the call was succesfull might be misleading
            let balance: Balance = safe_balance_of(&token_ref, self.env().account_id());
            let balance =
                if let Some(token_index) = self.reward_tokens.iter().position(|&t| t == token) {
                    balance.saturating_sub(self.farm_distributed_unclaimed_rewards[token_index])
                } else {
                    balance
                };
            safe_transfer(&mut token_ref, self.owner, balance);
            Ok(())
        }

        // To learn how much rewards the user has, it's best to dry-run claim_rewards
        #[ink(message)]
        fn claim_rewards(&mut self) -> Result<Vec<u128>, FarmError> {
            self.update()?;
            let account = self.env().caller();
            self.update_account(account);

            let user_rewards = self
                .user_claimable_rewards
                .take(account)
                .ok_or(FarmError::CallerNotFarmer)?;

            for (idx, user_reward) in user_rewards.clone().into_iter().enumerate() {
                if user_reward > 0 {
                    let mut psp22_ref: ink::contract_ref!(PSP22) = self.reward_tokens[idx].into();
                    self.farm_distributed_unclaimed_rewards[idx] -= user_reward;
                    // Here we have changed farm_distributed_unchanged_rewards, so we shouldn't just ignore the result of the call
                    // I see two alternatives:
                    // - match on the result of the call and don't change storage if the call fails, however that would require us to some data back to "user_claimable_rewards"
                    // - replace this call with "claim_reward" which would only transfer the reward for the given token
                    safe_transfer(&mut psp22_ref, account, user_reward);
                }
            }

            FarmContract::emit_event(
                self.env(),
                Event::RewardsClaimed(RewardsClaimed {
                    account,
                    amounts: user_rewards.clone(),
                }),
            );
            Ok(user_rewards)
        }

        #[ink(message)]
        fn deposit(&mut self, amount: u128) -> Result<(), FarmError> {
            let account = self.env().caller();
            self.deposit(account, amount)?;
            FarmContract::emit_event(self.env(), Event::Deposited(Deposited { account, amount }));
            Ok(())
        }

        #[ink(message)]
        fn deposit_all(&mut self) -> Result<(), FarmError> {
            let account = self.env().caller();
            let pool: contract_ref!(PSP22) = self.pool_id.into();
            let amount = pool.balance_of(account);
            self.deposit(account, amount)?;
            FarmContract::emit_event(self.env(), Event::Deposited(Deposited { account, amount }));
            Ok(())
        }

        #[ink(message)]
        fn withdraw(&mut self, amount: u128) -> Result<(), FarmError> {
            self.update()?;
            let account = self.env().caller();
            self.update_account(account);

            let shares = self.shares.get(account).unwrap_or(0);

            if let Some(new_shares) = shares.checked_sub(amount) {
                self.shares.insert(account, &new_shares);
                self.total_shares -= amount;
            } else {
                return Err(FarmError::InsufficientShares);
            }

            let mut pool: contract_ref!(PSP22) = self.pool_id.into();
            pool.transfer(account, amount, vec![])?;

            FarmContract::emit_event(self.env(), Event::Withdrawn(Withdrawn { account, amount }));
            Ok(())
        }

        #[ink(message)]
        fn view_farm_details(&self) -> FarmDetails {
            FarmDetails {
                pool_id: self.pool_id,
                start: self.start,
                end: self.end,
                reward_tokens: self.reward_tokens.clone(),
                reward_rates: self.farm_reward_rates.clone(),
            }
        }
    }

    // Would suggest adding some explanation why we want this.
    pub fn safe_transfer(psp22: &mut contract_ref!(PSP22), recipient: AccountId, amount: u128) {
        match psp22
            .call_mut()
            .transfer(recipient, amount, vec![])
            .try_invoke()
        {
            Err(ink_env_err) => {
                ink::env::debug_println!("ink env error: {:?}", ink_env_err);
            }
            Ok(Err(ink_lang_err)) => {
                ink::env::debug_println!("ink lang error: {:?}", ink_lang_err);
            }
            Ok(Ok(Err(psp22_error))) => {
                ink::env::debug_println!("psp22 error: {:?}", psp22_error);
            }
            Ok(Ok(Ok(_))) => {}
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

    pub fn rewards_per_share_in_time_interval(
        reward_rate: u128,
        total_shares: u128,
        from_timestamp: u128,
        to_timestamp: u128,
    ) -> Result<U256, MathError> {
        if total_shares == 0 || from_timestamp >= to_timestamp {
            return Ok(0.into());
        }

        let time_delta = to_timestamp
            .checked_sub(from_timestamp)
            .ok_or(MathError::Underflow)?;

        casted_mul(reward_rate, time_delta)
            .checked_mul(U256::from(SCALING_FACTOR))
            .ok_or(MathError::Overflow(1))?
            .checked_div(U256::from(total_shares))
            .ok_or(MathError::DivByZero(1))
    }

    /// The formula is:
    /// rewards_per_share * shares / SCALING_FACTOR
    pub fn rewards_earned_by_shares(
        shares: u128,
        rewards_per_share: U256,
    ) -> Result<u128, MathError> {
        rewards_per_share
            .checked_mul(U256::from(shares))
            .ok_or(MathError::Overflow(2))?
            .checked_div(U256::from(SCALING_FACTOR))
            .ok_or(MathError::DivByZero(2))?
            .try_into()
            .map_err(|_| MathError::CastOverflow)
    }
}
