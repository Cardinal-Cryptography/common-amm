use ink::env::{
    DefaultEnvironment,
    Environment,
};

type AccountId = <DefaultEnvironment as Environment>::AccountId;

fn set_sender(sender: AccountId) {
    ink::env::test::set_caller::<DefaultEnvironment>(sender);
}

fn default_accounts() -> ink::env::test::DefaultAccounts<DefaultEnvironment> {
    ink::env::test::default_accounts::<DefaultEnvironment>()
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
        crate::reward_per_token(
            reward_per_token_stored,
            reward_rate,
            total_supply,
            last_update_time,
            last_time_reward_applicable,
        )
        .expect("to calculate reward per token")
    }

    fn rewards_earned(shares: u128, rewards_per_token: U256, paid_reward_per_token: U256) -> u128 {
        crate::rewards_earned(shares, rewards_per_token, paid_reward_per_token)
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

        let rewards_per_token = reward_per_token(U256::zero(), REWARD_RATE, total_supply, 3, 5);
        // = r_j0 + R/T(t_j - t_j0)
        // = 0 + 100/100 * 2
        // = 2
        assert_eq!(rewards_per_token, casted_mul(2, SCALING_FACTOR));
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
        let reward_earned = rewards_earned(shares, rewards_per_token_from0_till5, U256::zero());
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
        let expected_second = rewards_per_token_from0_till5 + 1 * SCALING_FACTOR;
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
        let reward_earned = rewards_earned(shares, rewards_per_token_from0_till5, U256::zero());
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
        let alice_reward_earned = rewards_earned(alice, reward_per_token_from0_till7, U256::zero());

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
        let bob_rewards_earned = rewards_earned(bob, reward_per_token_from0_till10, U256::zero());
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

        let reward_per_token_from0_till5 = reward_per_token(U256::zero(), REWARD_RATE, alice, 3, 5);

        let reward_per_token_from0_till6 =
            reward_per_token(reward_per_token_from0_till5, REWARD_RATE, alice + bob, 5, 6);

        let reward_per_token_from0_till9 = reward_per_token(
            reward_per_token_from0_till6,
            REWARD_RATE,
            alice + bob + carol,
            6,
            9,
        );

        let alice_reward_earned = rewards_earned(alice, reward_per_token_from0_till9, U256::zero());
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
    use crate::{
        error::FarmError,
        farm::{
            Farm,
            MAX_REWARD_TOKENS,
        },
    };
    use farm_instance_trait::{
        Farm as FarmT,
        FarmStartError,
    };

    use ink::env::test::set_block_timestamp;

    fn pool_id() -> AccountId {
        AccountId::from([0x01; 32])
    }

    fn manager() -> AccountId {
        AccountId::from([0x02; 32])
    }

    fn farm() -> Farm {
        Farm::new(pool_id(), manager(), alice())
    }

    fn single_reward_token() -> Vec<AccountId> {
        vec![AccountId::from([0x11; 32])]
    }

    #[ink::test]
    fn new_creates_uninitialised_farm() {
        let farm = farm();
        assert_eq!(farm.is_running(), Result::Err(FarmError::StateMissing));
    }

    #[ink::test]
    fn non_owner_cannot_start_farm() {
        let mut farm = farm();
        set_block_timestamp::<DefaultEnvironment>(1);
        let reward_tokens = single_reward_token();
        set_sender(bob());
        assert_eq!(
            farm.start(5, reward_tokens),
            Err(FarmStartError::CallerNotOwner)
        );
    }

    #[ink::test]
    fn farm_end_before_start() {
        let mut farm = farm();
        set_block_timestamp::<DefaultEnvironment>(5);
        let reward_tokens = single_reward_token();
        assert_eq!(
            farm.start(2, reward_tokens),
            Err(FarmStartError::FarmEndBeforeStart)
        );
    }

    #[ink::test]
    fn farm_too_many_tokens_fails() {
        let mut farm = farm();
        set_block_timestamp::<DefaultEnvironment>(1);
        let reward_tokens = (0..MAX_REWARD_TOKENS + 10)
            .into_iter()
            .map(|i| AccountId::from([i as u8; 32]))
            .collect::<Vec<_>>();
        assert_eq!(
            farm.start(1000, reward_tokens),
            Err(FarmStartError::TooManyRewardTokens)
        );
    }

    #[ink::test]
    fn fail_on_zero_reward_amount() {
        // unimplemented!("This now has to be done as e2e test")
    }

    #[ink::test]
    fn fail_on_insufficient_rewards() {
        // unimplemented!("This now has to be done as e2e test")
        // let mut farm = farm();
        // let reward_tokens = single_reward_token();
        // let reward_amounts = vec![10];
        // // reward_rate = reward / duration
        // // rr = 10 / 100 == 0;
        // assert_eq!(
        //     farm.start(100, reward_amounts, reward_tokens),
        //     Err(FarmStartError::ZeroRewardRate)
        // );
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
