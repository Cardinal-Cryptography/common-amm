#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod error;
mod farm_state;
mod reentrancy_guard;

// TODO:
// Add upper bound on farm length.
// ? Consider separating constructor's errors from the rest
// ? Refactor to make staking logic reusable in different contracts.

// Tests:
// Deposit:
// - deposit with 0 balance fails
// - deposit with 0 amount fails
// - deposit with non-zero balance succeeds
// - deposit as first farmer takes all shares
// - deposit triggers claim
// - deposit as second farmer splits shares and updates reward counter properly
// - multiple, repeated deposits by the same farmer update reward counter properly
//
// Withdraw:
// Stop:
// Create & Start farm:

#[ink::contract]
mod farm {
    use crate::{
        error::FarmError,
        farm_state::RunningState,
        reentrancy_guard::*,
    };

    use openbrush::modifiers;

    use psp22_traits::PSP22;

    use ink::{
        contract_ref,
        storage::{
            Lazy,
            Mapping,
        },
    };

    use ink::prelude::{
        vec,
        vec::Vec,
    };

    #[ink(event)]
    pub struct Deposit {
        #[ink(topic)]
        account: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Withdraw {
        #[ink(topic)]
        account: AccountId,
        amount: u128,
    }

    #[ink(storage)]
    pub struct Farm {
        /// Address of the token pool for which this farm is created.
        pool: contract_ref!(PSP22),
        /// Address of the farm creator.
        owner: AccountId,
        /// Whether the farm is stopped.
        is_stopped: bool,
        /// How many shares each user has in the farm.
        shares: Mapping<AccountId, u128>,
        /// Total shares in the farm after the last action.
        total_shares: u128,
        /// Reward per token paid to the user for each reward token.
        // We need to track this separately for each reward token as each can have different reward rate.
        // Vectors should be relatively small (probably < 5).
        user_reward_per_token_paid: Mapping<AccountId, Vec<u128>>,
        /// Rewards that have not been collected (withdrawn) by the user yet.
        user_uncollected_rewards: Mapping<AccountId, Vec<u128>>,
        /// Farm state.
        state: Lazy<Option<RunningState>>,
        /// Flag to prevent reentrancy attacks.
        reentrancy_guard: u8,
    }

    const REENTRANCY_GUARD_LOCKED: u8 = 1u8;
    const REENTRANCY_GUARD_FREE: u8 = 0u8;

    impl Farm {
        #[ink(constructor)]
        pub fn new(pair_address: AccountId) -> Self {
            Farm {
                pool: pair_address.into(),
                owner: Self::env().caller(),
                is_stopped: true,
                total_shares: 0,
                shares: Mapping::new(),
                user_reward_per_token_paid: Mapping::new(),
                user_uncollected_rewards: Mapping::new(),
                state: Lazy::new(),
                reentrancy_guard: REENTRANCY_GUARD_FREE,
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
        #[modifiers(ensure_running(false))]
        #[ink(message)]
        pub fn start(
            &mut self,
            end: Timestamp,
            reward_amounts: Vec<u128>,
            reward_tokens: Vec<AccountId>,
        ) -> Result<(), FarmError> {
            let farm_owner = self.owner;
            if Self::env().caller() != farm_owner {
                return Err(FarmError::CallerNotOwner)
            }

            let now = Self::env().block_timestamp();

            if now >= end {
                return Err(FarmError::InvalidInitParams)
            }

            let duration = end as u128 - now as u128;
            // TODO: check that farm lenght is not too long. Like that it doesn't last a year.

            if reward_amounts.len() != reward_tokens.len() {
                return Err(FarmError::InvalidInitParams)
            }

            let tokens_len = reward_tokens.len();

            let mut reward_rates = Vec::with_capacity(tokens_len);

            for i in 0..tokens_len {
                if reward_amounts[i] == 0 {
                    return Err(FarmError::InvalidInitParams)
                }
                let rate = reward_amounts[i]
                    .checked_div(duration)
                    .ok_or(FarmError::ArithmeticError)?;

                if rate == 0 {
                    return Err(FarmError::InvalidInitParams)
                }

                let psp22_ref: ink::contract_ref!(PSP22) = reward_tokens[i].into();

                // Alternatively, we could call psp22_ref.transfer_from here.
                let balance: Balance = psp22_ref.balance_of(Self::env().account_id());
                // A reward of 0 is a spam.
                if balance == 0 {
                    return Err(FarmError::InvalidInitParams)
                }

                // Validate assumptions made earlier.
                if balance != reward_amounts[i] {
                    return Err(FarmError::InvalidInitParams)
                }
                // Double-check we have enough to cover the whole farm.
                if duration * rate <= reward_amounts[i] {
                    return Err(FarmError::InvalidInitParams)
                }

                reward_rates.push(rate);
            }

            let state = RunningState {
                owner: farm_owner,
                start: now,
                end,
                reward_rates,
                reward_tokens,
                reward_per_token_stored: vec![0; tokens_len],
                timestamp_at_last_update: now,
            };

            self.state.set(&Some(state));

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
            // Q: should anyone be able to stop the farm? Or only the owner?
            let running = self.get_state()?;

            if self.env().block_timestamp() < running.end {
                return Err(FarmError::StillRunning)
            }

            self.is_stopped = true;

            // Send remaining rewards to the farm owner.
            for reward_token in running.reward_tokens.iter() {
                let mut psp22_ref: ink::contract_ref!(PSP22) = (*reward_token).into();
                let balance: Balance = psp22_ref.balance_of(Self::env().account_id());
                // TODO: What if `psp22_ref.transfer` fails?
                if balance > 0 {
                    psp22_ref.transfer(running.owner, balance, vec![])?;
                }
            }

            Ok(())
        }

        /// Deposits the given amount of tokens into the farm.
        #[ink(message)]
        #[modifiers(ensure_running(true), non_zero_amount(amount), non_reentrant)]
        pub fn deposit(&mut self, amount: u128) -> Result<(), FarmError> {
            self.update_reward_index()?;

            let contract = self.env().account_id();
            let caller = self.env().caller();

            let prev_share = self.shares.get(caller).unwrap_or(0);
            self.shares.insert(caller, &(prev_share + amount));
            self.total_shares += amount;

            self.pool.transfer_from(caller, contract, amount, vec![])?;

            self.env().emit_event(Deposit {
                account: caller,
                amount,
            });
            Ok(())
        }

        /// Withdraws the given amount of shares from the farm.
        #[ink(message)]
        #[modifiers(non_zero_amount(amount), non_reentrant)]
        pub fn withdraw(&mut self, amount: u128) -> Result<(), FarmError> {
            let caller = self.env().caller();

            let shares = self.shares.get(caller).ok_or(FarmError::CallerNotFarmer)?;

            if shares < amount {
                return Err(FarmError::InvalidWithdrawAmount)
            }

            self.update_reward_index()?;

            self.shares.insert(caller, &(shares - amount));
            self.total_shares -= amount;

            self.pool.transfer(caller, amount, vec![])?;

            self.env().emit_event(Withdraw {
                account: caller,
                amount,
            });

            Ok(())
        }

        /// Claim all rewards for the caller.
        #[ink(message)]
        #[modifiers(non_reentrant)]
        pub fn claim(&mut self) -> Result<(), FarmError> {
            self.update_reward_index()?;

            let caller = Self::env().caller();

            let user_rewards = self
                .user_uncollected_rewards
                .get(caller)
                .ok_or(FarmError::CallerNotFarmer)?;

            // Reset state before calling PSP22 methods.
            // Reentrancy protection.
            self.user_uncollected_rewards
                .insert(caller, &vec![0; user_rewards.len()]);

            for (user_reward, reward_token) in user_rewards
                .into_iter()
                .zip(self.get_state()?.reward_tokens.iter())
            {
                if user_reward > 0 {
                    let mut psp22_ref: ink::contract_ref!(PSP22) = (*reward_token).into();
                    psp22_ref.transfer(caller, user_reward, vec![])?;
                }
            }
            Ok(())
        }

        /// Returns how much reward tokens the caller account has accumulated.
        // We're using the `account` as an argument, instead of `&self.env().caller()`,
        // for easier frontend integration.
        #[ink(message)]
        pub fn claimmable(&self, account: AccountId) -> Result<Vec<u128>, FarmError> {
            let rewards_per_token = self.rewards_per_token()?;
            let user_rewards = self.rewards_earned(account, &rewards_per_token)?;

            Ok(user_rewards)
        }

        /// Returns the amount of rewards per token that have been accumulated for the given account.
        fn update_reward_index(&mut self) -> Result<Vec<u128>, FarmError> {
            let account = self.env().caller();

            let rewards_per_token = self.rewards_per_token()?;
            let user_rewards = self.rewards_earned(account, &rewards_per_token)?;

            self.user_reward_per_token_paid
                .insert(account, &rewards_per_token);
            self.user_uncollected_rewards.insert(account, &user_rewards);

            let mut running = self.get_state()?;
            running.reward_per_token_stored = rewards_per_token;
            self.state.set(&Some(running));

            Ok(user_rewards)
        }

        /// Returns the timestamp at which the last update is applicable.
        /// When the farm is running, this is the current block timestamp.
        /// When the farm is stopped, this is the farm's end timestamp.
        fn last_timestamp_applicable(&self) -> Result<Timestamp, FarmError> {
            let running = self.get_state()?;
            Ok(core::cmp::min(Self::env().block_timestamp(), running.end))
        }

        /// Calculates rewards per token due for providing liquidity to the farm
        /// since the last update until `last_timestamp_applicable`.
        ///
        /// Returned value is a vector of numbers, one for each reward token in the farm.
        fn rewards_per_token(&self) -> Result<Vec<u128>, FarmError> {
            let running = self.get_state()?;

            let mut rewards_per_token: Vec<u128> = Vec::with_capacity(running.reward_tokens.len());

            for i in 0..running.reward_tokens.len() {
                let reward_rate = running.reward_rates[i];
                let rpr = reward_per_token(
                    running.reward_per_token_stored[i],
                    reward_rate,
                    self.total_shares,
                    running.timestamp_at_last_update as u128,
                    self.last_timestamp_applicable()? as u128,
                )
                .ok_or(FarmError::ArithmeticError)?;
                rewards_per_token.push(rpr);
            }

            Ok(rewards_per_token)
        }

        /// Returns the amount of rewards earned by the given account.
        fn rewards_earned(
            &self,
            account: AccountId,
            rewards_per_token: &[u128],
        ) -> Result<Vec<u128>, FarmError> {
            let shares = self.shares.get(account).ok_or(FarmError::CallerNotFarmer)?;

            let rewards_per_token_paid_so_far = self
                .user_reward_per_token_paid
                .get(account)
                .unwrap_or(vec![0; rewards_per_token.len()]);

            let uncollected_user_rewards = self
                .user_uncollected_rewards
                .get(account)
                .unwrap_or(vec![0; rewards_per_token.len()]);

            let mut unclaimed_user_rewards = vec![];

            for i in 0..rewards_per_token.len() {
                let rewards_earned = rewards_earned(
                    shares,
                    rewards_per_token[i],
                    rewards_per_token_paid_so_far[i],
                )
                .ok_or(FarmError::ArithmeticError)?;
                unclaimed_user_rewards.push(rewards_earned + uncollected_user_rewards[i]);
            }

            Ok(unclaimed_user_rewards)
        }

        fn get_state(&self) -> Result<RunningState, FarmError> {
            self.state.get().flatten().ok_or(FarmError::StateMissing)
        }
    }

    /// Returns rewards due for providing liquidity from `last_update_time` to `last_time_reward_applicable`.
    ///
    /// r_j = r_j0 + R/T(t_j - t_j0)
    ///
    /// where:
    /// - r_j0 - reward per token stored at the last time the user interacted with the farm
    /// - R - total amount of rewards available for distribution
    /// - T - total shares in the farm
    /// - t_j - last time the user interacted with the farm, usually _now_.
    /// - t_j0 - last time the user "claimed" rewards.
    /// - r_j - rewards due to the user for providing liquidity from t_j0 to t_j
    ///
    /// See https://github.com/stakewithus/notes/blob/main/excalidraw/staking-rewards.png for more.
    fn reward_per_token(
        reward_per_token_stored: u128,
        reward_rate: u128,
        total_supply: u128,
        last_update_time: u128,
        last_time_reward_applicable: u128,
    ) -> Option<u128> {
        if total_supply == 0 {
            return Some(reward_per_token_stored)
        }
        reward_rate
            .checked_mul(last_time_reward_applicable - last_update_time)
            .and_then(|r| r.checked_mul(1_u128.pow(18)))
            .and_then(|r| r.checked_div(total_supply))
            .and_then(|r| r.checked_add(reward_per_token_stored))
    }

    /// Returns rewards earned by the user given `rewards_per_token` for some period of time.
    fn rewards_earned(
        shares: u128,
        rewards_per_token: u128,
        paid_reward_per_token: u128,
    ) -> Option<u128> {
        rewards_per_token
            .checked_sub(paid_reward_per_token)
            .and_then(|r| r.checked_mul(shares))
            .and_then(|r| r.checked_div(1_u128.pow(18)))
    }

    use openbrush::modifier_definition;

    #[modifier_definition]
    pub fn ensure_running<F, T>(
        instance: &mut Farm,
        body: F,
        expected: bool,
    ) -> Result<T, FarmError>
    where
        F: FnOnce(&mut Farm) -> Result<T, FarmError>,
    {
        if instance.is_stopped != expected {
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

    impl ReentrancyGuardT for Farm {
        fn lock(&mut self) -> Result<(), ReentrancyGuardError> {
            if self.reentrancy_guard == REENTRANCY_GUARD_LOCKED {
                return Err(ReentrancyGuardError::ReentrancyError)
            }
            self.reentrancy_guard = REENTRANCY_GUARD_LOCKED;
            Ok(())
        }

        fn unlock(&mut self) {
            // It's safe to "unlock" already unlocked guard.
            self.reentrancy_guard = REENTRANCY_GUARD_FREE;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn new_creates_stopped_farm() {
            let pool_id = AccountId::from([0x01; 32]);
            let farm = Farm::new(pool_id);
            assert_eq!(farm.is_stopped, false);
        }
    }
}
