#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod manager {

    type TokenId = AccountId;
    type UserId = AccountId;
    use amm_helpers::types::WrappedU256;
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

        // TODO: maybe bundle the 3 below into one struct
        pub farm_rewards_to_distribute: Vec<u128>,
        pub farm_distributed_unclaimed_rewards: Vec<u128>,
        pub farm_cumulative: Vec<WrappedU256>,

        pub user_cumulative_last_update: Mapping<UserId, Vec<WrappedU256>>,
        pub user_claimable_rewards: Mapping<UserId, Vec<u128>>,
    }

    impl FarmContract {
        #[ink(constructor)]
        pub fn new(pool_id: AccountId, reward_tokens: Vec<TokenId>) -> Self {
            let n_reward_tokens = reward_tokens.len();
            assert!(!reward_tokens.contains(&pool_id));
            FarmContract {
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
                farm_cumulative: vec![WrappedU256::default(); n_reward_tokens],
                user_cumulative_last_update: Mapping::default(),
                user_claimable_rewards: Mapping::default(),
            }
        }

        fn is_active(&self) -> bool {
            self.timestamp_at_last_update < self.end
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
                let reward_till_end = self.farm_rewards_to_distribute[idx];
                let delta_reward_per_share = rewards_per_share_to_distribute(
                    self.total_shares,
                    reward_till_end,
                    prev,
                    now,
                    self.end,
                )?;
                let delta_reward_distributed =
                    per_share_to_amount(self.total_shares, delta_reward_per_share)?;
                self.farm_cumulative[idx] = self.farm_cumulative[idx]
                    .0
                    .saturating_add(delta_reward_per_share)
                    .into();
                self.farm_distributed_unclaimed_rewards[idx] = self
                    .farm_distributed_unclaimed_rewards[idx]
                    .saturating_add(delta_reward_distributed);
                self.farm_rewards_to_distribute[idx] =
                    self.farm_rewards_to_distribute[idx].saturating_sub(delta_reward_distributed);
            }

            self.timestamp_at_last_update = current_timestamp;

            Ok(())
        }

        // Guarantee: after calling update_account(acc) it holds that
        // 1) both self.user_cumulative_last_update[acc] and self.user_claimable_rewards[acc] exist
        // 2) self.user_cumulative_last_update[acc][i] = self.farm_cumulative[i] for all i
        fn update_account(&mut self, account: AccountId) {
            let user_shares = self.shares.get(account).unwrap_or(0);
            let new_reward_vector = match self.user_cumulative_last_update.take(account) {
                Some(user_cumulative_last_update) => {
                    let mut user_claimable_rewards = self
                        .user_claimable_rewards
                        .take(account)
                        .unwrap_or(vec![0; self.reward_tokens.len()]);
                    for (idx, user_cumulative) in
                        user_cumulative_last_update.into_iter().enumerate()
                    {
                        let user_reward = per_share_to_amount(
                            user_shares,
                            self.farm_cumulative[idx]
                                .0
                                .saturating_sub(user_cumulative.0)
                                .into(),
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
            self.user_cumulative_last_update
                .insert(account, &self.farm_cumulative);
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
            assert!(!self.is_active());
            // At this point self.timestamp_at_last_update = self.env().block_timestamp();
            assert!(start > self.timestamp_at_last_update);
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

            self.start = 0;
            self.end = 0;
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
            safe_transfer(&mut token_ref, self.owner, balance)?;
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
                    // TODO: we should not Err here
                    safe_transfer(&mut psp22_ref, account, user_claim)?;
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
    }

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

    pub fn rewards_per_share_to_distribute(
        total_shares: u128,
        reward_till_end: u128,
        prev: Timestamp,
        current: Timestamp,
        end: Timestamp,
    ) -> Result<U256, MathError> {
        // The formula is:
        // SCALING_FACTOR * (reward_till_end * ((current - prev)/(end - prev)) / total_shares
        let total_time = end.checked_sub(prev).ok_or(MathError::Underflow)?;
        let time_used = current.checked_sub(prev).ok_or(MathError::Underflow)?;
        if total_time == 0 || total_shares == 0 {
            return Ok(U256::from(0))
        }
        let fraction =
            (U256::from(SCALING_FACTOR) * U256::from(time_used)) / (U256::from(total_time));

        fraction
            .checked_mul(U256::from(reward_till_end))
            .ok_or(MathError::Overflow)?
            .checked_div(U256::from(total_shares))
            .ok_or(MathError::DivByZero)
    }

    pub fn per_share_to_amount(total_shares: u128, per_share: U256) -> Result<u128, MathError> {
        // The formula is:
        // per_share * total_shares / SCALING_FACTOR
        per_share
            .checked_mul(U256::from(total_shares))
            .ok_or(MathError::Overflow)?
            .checked_div(U256::from(SCALING_FACTOR))
            .ok_or(MathError::Overflow)?
            .try_into()
            .map_err(|_| MathError::CastOverflow)
    }
}
