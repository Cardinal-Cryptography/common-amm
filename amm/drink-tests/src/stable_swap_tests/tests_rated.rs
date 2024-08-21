use crate::mock_rate_provider_contract;
use crate::stable_pool_contract;
use crate::utils::*;

use super::*;

use drink::{self, runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

const WAZERO_DEC: u8 = 12;
const SAZERO_DEC: u8 = 12;

const ONE_LPT: u128 = 10u128.pow(18);
const ONE_WAZERO: u128 = 10u128.pow(WAZERO_DEC as u32);
const ONE_SAZERO: u128 = 10u128.pow(SAZERO_DEC as u32);

fn deploy_rate_provider(session: &mut Session<MinimalRuntime>, salt: Vec<u8>) -> AccountId {
    let instance = mock_rate_provider_contract::Instance::new().with_salt(salt);
    session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into()
}

fn setup_rated_swap_with_tokens(
    session: &mut Session<MinimalRuntime>,
    caller: drink::AccountId32,
    rate_providers: Vec<Option<AccountId>>,
    initial_token_supply: u128,
    init_amp_coef: u128,
    trade_fee: u32,
    protocol_fee: u32,
) -> (AccountId, Vec<AccountId>) {
    let _ = session.set_actor(caller.clone());

    let tokens: Vec<AccountId> = rate_providers
        .iter()
        .enumerate()
        .map(|(i, _)| {
            psp22_utils::setup_with_amounts(
                session,
                format!("Token{i}"),
                WAZERO_DEC,
                initial_token_supply * ONE_WAZERO,
                caller.clone(),
            )
            .into()
        })
        .collect();

    let instance = stable_pool_contract::Instance::new_rated(
        tokens.clone(),
        vec![WAZERO_DEC; rate_providers.len()],
        rate_providers,
        init_amp_coef,
        caller.to_account_id(),
        trade_fee,
        protocol_fee,
        Some(fee_receiver()),
    );

    let rated_swap = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();

    for token in tokens.clone() {
        psp22_utils::increase_allowance(
            session,
            token.into(),
            rated_swap,
            u128::MAX,
            caller.clone(),
        )
        .unwrap();
    }

    (rated_swap, tokens)
}

fn set_mock_rate(session: &mut Session<MinimalRuntime>, mock_rate_contract: AccountId, rate: u128) {
    _ = handle_ink_error(
        session
            .execute(mock_rate_provider_contract::Instance::from(mock_rate_contract).set_rate(rate))
            .unwrap(),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_rated_pool.rs#L27
#[drink::test]
fn test_01(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    upload_all(&mut session);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let initial_token_supply: u128 = 1_000_000_000;
    let mock_rate_provider = deploy_rate_provider(&mut session, vec![0]);
    let (rated_swap, tokens) = setup_rated_swap_with_tokens(
        &mut session,
        BOB,
        vec![Some(mock_rate_provider), None],
        initial_token_supply,
        10000,
        2_500_000,
        200_000_000,
    );
    let [sazero, wazero]: [AccountId; 2] = tokens.try_into().unwrap();

    set_timestamp(&mut session, now + 1);
    set_mock_rate(&mut session, mock_rate_provider, 2 * RATE_PRECISION);

    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        BOB,
        1,
        vec![50000 * ONE_SAZERO, 100000 * ONE_WAZERO],
        bob(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, bob()),
        200000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        200000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    transfer_and_increase_allowance(
        &mut session,
        rated_swap,
        vec![sazero, wazero],
        CHARLIE,
        vec![100000 * ONE_SAZERO, 100000 * ONE_WAZERO],
        BOB,
    );
    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        1,
        vec![50000 * ONE_SAZERO, 100000 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        200000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        400000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        200000 * ONE_LPT,
        vec![1 * ONE_SAZERO, 1 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully remove liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        0,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        200000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    // --- DIFF ----
    // Allow withdrawing all liquidity from the pool

    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        rated_swap.into(),
        BOB,
        200000 * ONE_LPT,
        vec![1 * ONE_SAZERO, 1 * ONE_WAZERO],
        bob(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, bob()),
        0,
        "Incorrect user share"
    );

    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);

    // no shares left
    assert_eq!(last_total_shares, 0, "Incorrect total shares");
    assert_eq!(last_share_price, 0, "Incorrect share price");
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_rated_pool.rs#L116
#[drink::test]
fn test_02(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    upload_all(&mut session);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let mock_token_2_rate = deploy_rate_provider(&mut session, vec![0]);

    let initial_token_supply: u128 = 1_000_000_000;
    let (rated_swap, tokens) = setup_rated_swap_with_tokens(
        &mut session,
        BOB,
        vec![None, Some(mock_token_2_rate), None],
        initial_token_supply,
        10000,
        2_500_000,
        200_000_000,
    );

    set_timestamp(&mut session, now);
    set_mock_rate(&mut session, mock_token_2_rate, 2 * RATE_PRECISION);

    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        BOB,
        1,
        vec![100000 * ONE_WAZERO, 50000 * ONE_WAZERO, 100000 * ONE_WAZERO],
        bob(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, bob()),
        300000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        300000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    transfer_and_increase_allowance(
        &mut session,
        rated_swap,
        tokens,
        CHARLIE,
        vec![
            100000 * ONE_WAZERO,
            100000 * ONE_WAZERO,
            100000 * ONE_WAZERO,
        ],
        BOB,
    );
    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        1,
        vec![100000 * ONE_WAZERO, 50000 * ONE_WAZERO, 100000 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        300000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        600000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        300000 * ONE_LPT,
        vec![1 * ONE_WAZERO, 1 * ONE_WAZERO, 1 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully remove liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        0,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        300000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_rated_pool.rs#L197
#[drink::test]
fn test_03(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    upload_all(&mut session);

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let mock_token_2_rate = deploy_rate_provider(&mut session, vec![0]);
    let mock_token_3_rate = deploy_rate_provider(&mut session, vec![1]);

    let initial_token_supply: u128 = 1_000_000_000;
    let (rated_swap, tokens) = setup_rated_swap_with_tokens(
        &mut session,
        BOB,
        vec![None, Some(mock_token_2_rate), Some(mock_token_3_rate)],
        initial_token_supply,
        10000,
        2_500_000,
        200_000_000,
    );

    set_timestamp(&mut session, now);
    set_mock_rate(&mut session, mock_token_2_rate, 2 * RATE_PRECISION);
    set_mock_rate(&mut session, mock_token_3_rate, 4 * RATE_PRECISION);

    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        BOB,
        1,
        vec![100000 * ONE_WAZERO, 50000 * ONE_WAZERO, 25000 * ONE_WAZERO],
        bob(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, bob()),
        300000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        300000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    transfer_and_increase_allowance(
        &mut session,
        rated_swap,
        tokens,
        CHARLIE,
        vec![
            100000 * ONE_WAZERO,
            100000 * ONE_WAZERO,
            100000 * ONE_WAZERO,
        ],
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        1,
        vec![100000 * ONE_WAZERO, 50000 * ONE_WAZERO, 25000 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        300000 * ONE_LPT,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        600000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");

    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        300000 * ONE_LPT,
        vec![1 * ONE_WAZERO, 1 * ONE_WAZERO, 1 * ONE_WAZERO],
        charlie(),
    )
    .expect("Should successfully remove liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, charlie()),
        0,
        "Incorrect user share"
    );
    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        300000 * ONE_LPT,
        "Incorrect total shares"
    );
    assert_eq!(last_share_price, 100000000, "Incorrect share price");
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_rated_pool.rs#L303
#[drink::test]
fn test_04(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    upload_all(&mut session);

    let initial_token_supply: u128 = 1_000_000_000;
    let (rated_swap, tokens) = setup_rated_swap_with_tokens(
        &mut session,
        BOB,
        vec![None, None],
        initial_token_supply,
        10000,
        2_500_000,
        200_000_000,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        rated_swap.into(),
        BOB,
        1,
        vec![100000 * ONE_WAZERO, 100000 * ONE_WAZERO],
        bob(),
    )
    .expect("Should successfully add liquidity");
    assert_eq!(
        psp22_utils::balance_of(&mut session, rated_swap, bob()),
        200000 * ONE_LPT,
        "Incorrect user share"
    );
    let (_, last_total_shares) = share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        200000 * ONE_LPT,
        "Incorrect total shares"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, tokens[0], charlie()),
        0,
        "Incorrect user token balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, tokens[1], charlie()),
        0,
        "Incorrect user token balance"
    );
    transfer_and_increase_allowance(
        &mut session,
        rated_swap,
        tokens.clone(),
        CHARLIE,
        vec![ONE_WAZERO, 0],
        BOB,
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, tokens[0], charlie()),
        ONE_WAZERO,
        "Incorrect user token balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, tokens[1], charlie()),
        0,
        "Incorrect user token balance"
    );

    _ = stable_swap::swap_exact_in(
        &mut session,
        rated_swap.into(),
        CHARLIE,
        tokens[0],
        tokens[1],
        ONE_WAZERO,
        1,
        charlie(),
    )
    .expect("Should swap");
    assert_eq!(
        psp22_utils::balance_of(&mut session, tokens[0], charlie()),
        0,
        "Incorrect user token balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, tokens[1], charlie()),
        997499999501, // -- DIFF -- 997499999501 [274936452669]
        "Incorrect user token balance"
    );

    let (_, last_total_shares) = share_price_and_total_shares(&mut session, rated_swap);
    assert_eq!(
        last_total_shares,
        200000 * ONE_LPT + 499999994249708, // -- DIFF -- 499999994999720 [058346]
        "Incorrect total shares"
    );
    assert_eq!(
        stable_swap::reserves(&mut session, rated_swap),
        vec![100001 * ONE_WAZERO, 99999 * ONE_WAZERO + 2500000499], // DIFF -- 2500000498 [725063547331]
        "Incorrect reserves"
    );
}
