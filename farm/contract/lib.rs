#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod farm {
    type TokenId = AccountId;
    type UserId = AccountId;
    use amm_helpers::{ensure, math::casted_mul, types::WrappedU256};
    use farm_trait::{Farm, FarmDetails, FarmError};
    use ink::{codegen::EmitEvent, contract_ref, reflect::ContractEventBase, storage::Mapping};

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
        rewards_claimed: Vec<u128>,
    }

    #[ink(event)]
    pub struct FarmStopped {
        end: u64,
    }

    #[ink(event)]
    pub struct FarmStarted {
        start: u64,
        end: u64,
        reward_rates: Vec<u128>,
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
        pub farm_reward_rates: Vec<WrappedU256>,

        /// cumulative_per_share at the last update for each user.
        pub user_cumulative_reward_last_update: Mapping<UserId, Vec<WrappedU256>>,

        /// Reward rates per user of unclaimed, accumulated users' rewards.
        pub user_claimable_rewards: Mapping<UserId, Vec<u128>>,

        /// Flag indicating whether farm is active.
        /// Farm is active when:
        /// * farm is running
        /// * farm is planned for the future
        pub is_active: bool,
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
            if !no_duplicates(&reward_tokens) {
                return Err(FarmError::DuplicateRewardTokens);
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
                farm_reward_rates: vec![WrappedU256::ZERO; n_reward_tokens],
                user_cumulative_reward_last_update: Mapping::default(),
                user_claimable_rewards: Mapping::default(),
                is_active: false,
            })
        }

        // Guarantee: after calling update() it holds that self.timestamp_at_last_update = self.env().block_timestamp()
        fn update(&mut self) -> Result<(), FarmError> {
            let current_timestamp = self.env().block_timestamp();
            // Update reward rates just once per block.
            if self.timestamp_at_last_update >= current_timestamp {
                return Ok(());
            };

            let prev = core::cmp::max(self.timestamp_at_last_update, self.start);
            let now = core::cmp::min(current_timestamp, self.end);
            if prev >= now || !self.is_active {
                self.timestamp_at_last_update = current_timestamp;
                return Ok(());
            }

            // At this point we know [prev, now] is the intersection of [self.start, self.end] and [self.timestamp_at_last_update, current_timestamp]
            // It is non-empty because of the checks above and self.start <= now <= self.end

            for idx in 0..self.reward_tokens.len() {
                let delta_reward_per_share = rewards_per_share_in_time_interval(
                    self.farm_reward_rates[idx].0,
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
        ) -> Result<Vec<WrappedU256>, FarmError> {
            let now = Self::env().block_timestamp();
            let tokens_len = self.reward_tokens.len();

            if rewards.len() != tokens_len {
                return Err(FarmError::RewardsVecLengthMismatch);
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

                let reward_rate = casted_mul(*reward_amount, SCALING_FACTOR)
                    .checked_div(U256::from(duration))
                    .ok_or(FarmError::ArithmeticError(MathError::DivByZero(3)))?;

                reward_rates.push(WrappedU256::from(reward_rate));
            }

            if reward_rates.iter().all(|rr| *rr == WrappedU256::ZERO) {
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

        fn reward_rates_to_u128(&self) -> Result<Vec<u128>, FarmError> {
            let mut rates = Vec::with_capacity(self.farm_reward_rates.len());
            for rr in &self.farm_reward_rates {
                rates.push(
                    rr.0.checked_div(U256::from(SCALING_FACTOR))
                        .ok_or(MathError::DivByZero(4))?
                        .try_into()
                        .map_err(|_| MathError::Overflow(3))?,
                );
            }
            Ok(rates)
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
            ensure!(self.env().caller() == self.owner, FarmError::CallerNotOwner);
            self.update()?;
            ensure!(!self.is_active, FarmError::FarmIsRunning);
            self.farm_reward_rates = self.assert_start_params(start, end, rewards.clone())?;
            self.start = start;
            self.end = end;
            self.is_active = true;
            FarmContract::emit_event(
                self.env(),
                Event::FarmStarted(FarmStarted {
                    start,
                    end,
                    reward_rates: self.reward_rates_to_u128()?,
                }),
            );
            Ok(())
        }

        #[ink(message)]
        fn owner_stop_farm(&mut self) -> Result<(), FarmError> {
            ensure!(self.env().caller() == self.owner, FarmError::CallerNotOwner);
            ensure!(self.is_active, FarmError::FarmAlreadyStopped);
            self.update()?;
            let current_timestamp = self.env().block_timestamp();
            // If owner deactivates the farm before it even starts,
            // We set the end timestamp to self.start to make it clear there's no farm.
            if current_timestamp < self.start {
                self.end = self.start
            } else if current_timestamp < self.end {
                // End farm prematurely.
                self.end = self.env().block_timestamp();
            } else if current_timestamp >= self.end {
                // No-op after farm's end.
            }
            self.is_active = false;
            FarmContract::emit_event(
                self.env(),
                Event::FarmStopped(FarmStopped { end: self.end }),
            );
            Ok(())
        }

        #[ink(message)]
        fn owner_withdraw_token(&mut self, token: TokenId) -> Result<u128, FarmError> {
            ensure!(self.env().caller() == self.owner, FarmError::CallerNotOwner);
            ensure!(!self.is_active, FarmError::FarmIsRunning);
            self.update()?;
            let mut token_ref: contract_ref!(PSP22) = token.into();
            let total_balance = token_ref.balance_of(self.env().account_id());
            let mut undistributed_balance = if let Some(token_index) =
                self.reward_tokens.iter().position(|&t| t == token)
            {
                total_balance.saturating_sub(self.farm_distributed_unclaimed_rewards[token_index])
            } else {
                total_balance
            };
            if token == self.pool_id {
                undistributed_balance -= self.total_shares;
            }
            token_ref.transfer(self.owner, undistributed_balance, vec![])?;
            Ok(undistributed_balance)
        }

        // To learn how much rewards the user has, it's best to dry-run claim_rewards.
        #[ink(message)]
        fn claim_rewards(&mut self, tokens_indices: Vec<u8>) -> Result<Vec<u128>, FarmError> {
            self.update()?;
            let account = self.env().caller();
            self.update_account(account);

            let mut user_rewards = match self.user_claimable_rewards.get(account) {
                Some(user_rewards) => user_rewards,
                None => return Ok(vec![0u128; self.reward_tokens.len()]),
            };

            let mut rewards_claimed: Vec<u128> = vec![0u128; self.reward_tokens.len()];

            for token_idx in tokens_indices {
                let idx = token_idx as usize;
                let token = self.reward_tokens[idx];
                let user_reward = user_rewards[idx];
                if user_reward > 0 {
                    user_rewards[idx] = 0;
                    let mut psp22_ref: ink::contract_ref!(PSP22) = token.into();
                    self.farm_distributed_unclaimed_rewards[idx] -= user_reward;
                    rewards_claimed[idx] = user_reward;
                    psp22_ref
                        .transfer(account, user_reward, vec![])
                        .map_err(|e| FarmError::TokenTransferFailed(token, e))?;
                }
            }

            if user_rewards.iter().all(|r| *r == 0) {
                self.user_claimable_rewards.remove(account);
            } else {
                self.user_claimable_rewards.insert(account, &user_rewards);
            }

            FarmContract::emit_event(
                self.env(),
                Event::RewardsClaimed(RewardsClaimed {
                    account,
                    rewards_claimed: rewards_claimed.clone(),
                }),
            );
            Ok(rewards_claimed)
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
                is_active: self.is_active,
                start: self.start,
                end: self.end,
                reward_tokens: self.reward_tokens.clone(),
                reward_rates: self.reward_rates_to_u128().unwrap(),
            }
        }
    }

    pub fn rewards_per_share_in_time_interval(
        reward_rate: U256,
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

        reward_rate
            .checked_mul(U256::from(time_delta))
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

    pub fn no_duplicates<A: Eq + PartialEq>(v: &Vec<A>) -> bool {
        for (idx, el) in v.iter().enumerate() {
            // Add 1 since the first `idx=0` and we would
            // start iterating from the beginning (rather than the next element).
            for other in v.iter().skip(idx + 1) {
                if el == other {
                    return false;
                }
            }
        }
        true
    }

    #[cfg(test)]
    mod tests {
        use farm_trait::{Farm, FarmError};
        use ink::{env::DefaultEnvironment, primitives::AccountId};

        #[ink::test]
        fn new_farm_works() {
            let pool_id = AccountId::from([0u8; 32]);
            let reward_tokens = vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])];

            let farm =
                super::FarmContract::new(pool_id, reward_tokens.clone()).expect("farm::new works");

            let farm_details = farm.view_farm_details();
            assert_eq!(farm_details.pool_id, pool_id);
            assert_eq!(farm_details.start, 0);
            assert_eq!(farm_details.end, 0);
            assert_eq!(farm_details.reward_tokens, reward_tokens);
            assert_eq!(farm_details.reward_rates, vec![0, 0]);

            assert_eq!(farm.is_active, false);

            assert_eq!(farm.total_shares(), 0);
        }

        #[ink::test]
        fn new_farm_fails() {
            let pool_id = AccountId::from([0u8; 32]);
            let reward_tokens = vec![AccountId::from([1u8; 32]), pool_id.clone()];

            assert_eq!(
                super::FarmContract::new(pool_id, reward_tokens.clone())
                    .err()
                    .unwrap(),
                FarmError::RewardTokenIsPoolToken
            );

            let too_many_tokens = (0..=super::MAX_REWARD_TOKENS)
                .into_iter()
                .map(|i| AccountId::from([i as u8; 32]))
                .collect();
            assert_eq!(
                super::FarmContract::new(pool_id, too_many_tokens)
                    .err()
                    .unwrap(),
                FarmError::TooManyRewardTokens
            )
        }

        #[ink::test]
        fn deposit_zero_fails() {
            let pool_id = AccountId::from([0u8; 32]);
            let reward_tokens = vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])];

            let mut farm =
                super::FarmContract::new(pool_id, reward_tokens.clone()).expect("farm::new works");

            assert_eq!(
                Farm::deposit(&mut farm, 0).err().unwrap(),
                FarmError::InsufficientShares
            );
        }

        #[ink::test]
        fn withdraw_too_much_fails() {
            let pool_id = AccountId::from([0u8; 32]);
            let reward_tokens = vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])];

            let mut farm =
                super::FarmContract::new(pool_id, reward_tokens.clone()).expect("farm::new works");

            assert_eq!(
                Farm::withdraw(&mut farm, 100).err().unwrap(),
                FarmError::InsufficientShares
            );
        }

        #[ink::test]
        fn fail_for_nonowner() {
            use ink::env::test::*;

            let acc = default_accounts::<DefaultEnvironment>();

            let pool_id = AccountId::from([0u8; 32]);
            let reward_tokens = vec![AccountId::from([1u8; 32]), AccountId::from([2u8; 32])];

            set_caller::<DefaultEnvironment>(acc.alice);
            let mut farm =
                super::FarmContract::new(pool_id, reward_tokens).expect("farm::new works");
            set_caller::<DefaultEnvironment>(acc.bob);
            assert_eq!(
                Farm::owner_start_new_farm(&mut farm, 100, 110, vec![1, 2])
                    .err()
                    .unwrap(),
                FarmError::CallerNotOwner
            );
            assert_eq!(
                Farm::owner_stop_farm(&mut farm).err().unwrap(),
                FarmError::CallerNotOwner
            );
            assert_eq!(
                Farm::owner_withdraw_token(&mut farm, AccountId::from([1u8; 32]))
                    .err()
                    .unwrap(),
                FarmError::CallerNotOwner
            );
        }

        #[ink::test]
        fn duplicate_reward_tokens_not_allowed() {
            let pool_id = AccountId::from([0u8; 32]);
            let reward_tokens = vec![AccountId::from([1u8; 32]), AccountId::from([1u8; 32])];

            assert_eq!(
                super::FarmContract::new(pool_id, reward_tokens).unwrap_err(),
                FarmError::DuplicateRewardTokens,
            );
        }

        #[test]
        fn test_no_duplicates() {
            use crate::farm::no_duplicates;

            assert!(no_duplicates::<u32>(&vec![]));
            assert!(no_duplicates(&vec![1]));
            assert!(no_duplicates(&vec![1, 2, 3, 4]));

            assert!(!no_duplicates(&vec![2, 2]));
            assert!(!no_duplicates(&vec![1, 2, 2]));
            assert!(!no_duplicates(&vec![1, 2, 3, 2]));
            assert!(!no_duplicates(&vec![1, 2, 3, 4, 1]));
        }
    }
}
