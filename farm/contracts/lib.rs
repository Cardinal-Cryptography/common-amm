#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod manager {

    type TokenId = AccountId;
    type UserId = AccountId;
    use amm_helpers::{
        math::casted_mul,
        types::WrappedU256,
    };
    use farm_manager_trait::{
        Farm,
        FarmError,
    };
    use ink::{
        codegen::{
            EmitEvent,
            TraitCallBuilder,
        },
        contract_ref,
        reflect::ContractEventBase,
        storage::Mapping,
    };

    use ink::prelude::{
        vec,
        vec::Vec,
    };
    use primitive_types::U256;

    use psp22_traits::{
        PSP22Error,
        PSP22,
    };
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
        pool_id: AccountId,
        /// Address of the farm creator.
        owner: AccountId,
        /// How many shares each user has in the farm.
        shares: Mapping<UserId, u128>,
        /// Total shares in the farm after the last action.
        total_shares: u128,
        /// Reward tokens.
        reward_tokens: Vec<TokenId>,

        /// The timestamp when the farm should start.
        start: Timestamp,
        /// The timestamp when the farm will stop.
        end: Timestamp,
        /// The timestamp at the last call to update().
        pub timestamp_at_last_update: Timestamp,

        /// Farm rewards that are yet to be distributed.
        pub farm_rewards_to_distribute: Vec<u128>,
        /// Total rewards that have been distributed but not yet claimed by users.
        pub farm_distributed_unclaimed_rewards: Vec<u128>,
        /// Cumulative rewards distributed per share since `start` and until `timestmap_at_last_update`.
        pub farm_cumulative_reward_per_share: Vec<WrappedU256>,
        /// Rewards rate - how many rewards per smallest unit of time are distributed.
        pub farm_reward_rates: Vec<u128>,

        /// cumulative_per_share at the last update for each user.
        pub user_cumulative_reward_last_update: Mapping<UserId, Vec<WrappedU256>>,

        pub user_claimable_rewards: Mapping<UserId, Vec<u128>>,
    }

    impl FarmContract {
        #[ink(constructor)]
        pub fn new(pool_id: AccountId, reward_tokens: Vec<TokenId>) -> Result<Self, FarmError> {
            let n_reward_tokens = reward_tokens.len();
            if n_reward_tokens > MAX_REWARD_TOKENS as usize {
                return Err(FarmError::InvalidFarmStartParams)
            }
            if reward_tokens.contains(&pool_id) {
                return Err(FarmError::InvalidFarmStartParams)
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
                farm_rewards_to_distribute: vec![0; n_reward_tokens],
                farm_distributed_unclaimed_rewards: vec![0; n_reward_tokens],
                farm_cumulative_reward_per_share: vec![WrappedU256::default(); n_reward_tokens],
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
                return Ok(())
            };

            let prev = core::cmp::max(self.timestamp_at_last_update, self.start);
            let now = core::cmp::min(current_timestamp, self.end);
            if prev >= now || self.timestamp_at_last_update == current_timestamp {
                self.timestamp_at_last_update = current_timestamp;
                return Ok(())
            }

            // At this point we know [prev, now] is the intersection of [self.start, self.end] and [self.timestamp_at_last_update, current_timestamp]
            // It is non-empty because of the checks above and self.start <= now <= self.end

            for (idx, _) in self.reward_tokens.iter().enumerate() {
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
                self.farm_rewards_to_distribute[idx] =
                    self.farm_rewards_to_distribute[idx].saturating_sub(delta_reward_distributed);
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

            if start <= self.timestamp_at_last_update
                || now >= end
                || rewards.len() != self.reward_tokens.len()
            {
                return Err(FarmError::InvalidFarmStartParams)
            }

            let duration = end as u128 - now as u128;

            let tokens_len = self.reward_tokens.len();
            let mut reward_rates = Vec::with_capacity(tokens_len);

            for (token_id, reward_amount) in self.reward_tokens.iter().zip(rewards.iter()) {
                if *reward_amount == 0 {
                    return Err(FarmError::InvalidFarmStartParams)
                }

                let mut psp22_ref: ink::contract_ref!(PSP22) = (*token_id).into();

                psp22_ref.transfer_from(
                    self.owner,
                    self.env().account_id(),
                    *reward_amount,
                    vec![],
                )?;

                let reward_rate = reward_amount
                    .checked_div(duration)
                    .ok_or(FarmError::InvalidFarmStartParams)?;

                if reward_rate == 0 {
                    return Err(FarmError::InvalidFarmStartParams)
                }

                // Double-check we have enough to cover the whole farm.
                if duration * reward_rate < *reward_amount {
                    return Err(FarmError::InvalidFarmStartParams)
                }
                reward_rates.push(reward_rate);
            }
            Ok(reward_rates)
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

        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.total_shares
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
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
                return Err(FarmError::CallerNotOwner)
            }
            self.update()?;
            if self.is_active() {
                return Err(FarmError::FarmAlreadyRunning)
            }
            self.farm_reward_rates = self.assert_start_params(start, end, rewards.clone())?;
            self.start = start;
            self.end = end;
            self.farm_rewards_to_distribute = rewards;
            Ok(())
        }

        #[ink(message)]
        fn owner_stop_farm(&mut self) -> Result<(), FarmError> {
            if self.env().caller() != self.owner {
                return Err(FarmError::CallerNotOwner)
            }
            self.update()?;
            self.end = self.env().block_timestamp();
            Ok(())
        }

        #[ink(message)]
        fn owner_withdraw_token(&mut self, token: TokenId) -> Result<(), FarmError> {
            if self.env().caller() != self.owner {
                return Err(FarmError::CallerNotOwner)
            }
            self.update()?;
            assert!(!self.is_active());

            // Owner should be able to withdraw every token except the pool token.
            assert!(self.pool_id != token);
            let mut token_ref = token.into();

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
                    // It could happen that user_reward > self.farm_distributed_unclaimed_rewards[idx] because of rounding errors.
                    let user_claim =
                        core::cmp::min(self.farm_distributed_unclaimed_rewards[idx], user_reward);
                    self.farm_distributed_unclaimed_rewards[idx] -= user_claim;
                    safe_transfer(&mut psp22_ref, account, user_claim);
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
        fn deposit_shares(&mut self, amount: u128) -> Result<(), FarmError> {
            self.update()?;
            let account = self.env().caller();
            self.update_account(account);

            let mut pool: contract_ref!(PSP22) = self.pool_id.into();

            pool.transfer_from(account, self.env().account_id(), amount, vec![])?;

            let shares = self.shares.get(account).unwrap_or(0);
            self.shares.insert(account, &(shares + amount));
            self.total_shares += amount;

            FarmContract::emit_event(self.env(), Event::Deposited(Deposited { account, amount }));
            Ok(())
        }

        #[ink(message)]
        fn deposit_all(&mut self) -> Result<(), FarmError> {
            self.update()?;
            let account = self.env().caller();
            self.update_account(account);

            let pool: contract_ref!(PSP22) = self.pool_id.into();
            // Check how much have been transferred to the contract. We assume this was done by the caller.
            let amount = safe_balance_of(&pool, self.env().account_id());

            let shares = self.shares.get(account).unwrap_or(0);
            self.shares.insert(account, &(shares + amount));
            self.total_shares += amount;

            FarmContract::emit_event(self.env(), Event::Deposited(Deposited { account, amount }));
            Ok(())
        }

        #[ink(message)]
        fn withdraw_shares(&mut self, amount: u128) -> Result<(), FarmError> {
            self.update()?;
            let account = self.env().caller();
            self.update_account(account);

            let shares = self.shares.get(account).unwrap_or(0);

            if let Some(new_shares) = shares.checked_sub(amount) {
                self.shares.insert(account, &new_shares);
                self.total_shares -= amount;
            } else {
                return Err(PSP22Error::InsufficientBalance.into())
            }

            let mut pool: contract_ref!(PSP22) = self.pool_id.into();
            pool.transfer(account, amount, vec![])?;

            FarmContract::emit_event(self.env(), Event::Withdrawn(Withdrawn { account, amount }));
            Ok(())
        }

        #[ink(message)]
        fn claimmable(&self, account: AccountId) -> Vec<(TokenId, u128)> {
            self.reward_tokens
                .clone()
                .into_iter()
                .zip(self.user_claimable_rewards.get(account).unwrap_or(vec![
                        0;
                        self.reward_tokens
                            .len()
                    ]))
                .collect()
        }
    }

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
            Ok(Ok(Ok(res))) => {}
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
        if total_shares == 0 || from_timestamp > to_timestamp {
            return Ok(0.into())
        }

        let time_delta = to_timestamp
            .checked_sub(from_timestamp)
            .ok_or(MathError::Underflow)?;

        casted_mul(reward_rate, time_delta)
            .checked_mul(U256::from(SCALING_FACTOR))
            .ok_or(MathError::Overflow)?
            .checked_div(U256::from(total_shares))
            .ok_or(MathError::DivByZero)
    }

    /// The formula is:
    /// rewards_per_share * shares / SCALING_FACTOR
    pub fn rewards_earned_by_shares(
        shares: u128,
        rewards_per_share: U256,
    ) -> Result<u128, MathError> {
        rewards_per_share
            .checked_mul(U256::from(shares))
            .ok_or(MathError::Overflow)?
            .checked_div(U256::from(SCALING_FACTOR))
            .ok_or(MathError::Overflow)?
            .try_into()
            .map_err(|_| MathError::CastOverflow)
    }
}
