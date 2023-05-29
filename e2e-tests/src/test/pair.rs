use anyhow::{
    anyhow,
    Result,
};
use assert2::assert;

use aleph_client::Balance;
use ink_wrapper_types::{
    util::ToAccountId,
    Connection,
};

use crate::{
    events::{
        get_burn_events,
        get_create_pair_events,
        get_mint_events,
        get_swap_events,
    },
    factory_contract,
    factory_contract::Factory,
    pair_contract,
    pair_contract::{
        Pair,
        PSP22 as PairPSP22,
    },
    psp22_token,
    psp22_token::PSP22 as TokenPSP22,
    test::setup::{
        setup_test,
        Contracts,
        TestFixture,
        ZERO_ADDRESS,
    },
};

const BALANCE: Balance = 10_000;
const MIN_BALANCE: Balance = 1_000;
const EXPECTED_INITIAL_REGULAR_BALANCE: Balance = 0;

const FIRST_AMOUNT_IN: Balance = 1_020;
const FIRST_AMOUNT_OUT: Balance = 0;
const SECOND_AMOUNT_OUT: Balance = 900;

#[tokio::test]
pub async fn create_pair() -> Result<()> {
    let TestFixture {
        wealthy_connection,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    let all_pairs_length_before = factory_contract
        .all_pairs_length(&wealthy_connection)
        .await??;

    assert!(all_pairs_length_before == 0);

    let tx_info = factory_contract
        .create_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await?;

    let all_events = wealthy_connection.get_contract_events(tx_info).await?;
    let contract_events = all_events.for_contract(factory_contract);
    let create_pair_events = get_create_pair_events(contract_events);
    let create_pair_events_len = create_pair_events.len();

    assert!(
        create_pair_events_len == 1,
        "The number of emitted `PairCreated` events is {}, should be 1.",
        create_pair_events_len
    );

    let factory_contract::event::Event::PairCreated {
        token_0,
        token_1,
        pair,
        pair_len,
    } = create_pair_events[0];

    let mut expected_token_pair: Vec<ink_primitives::AccountId> =
        vec![token_a.into(), token_b.into()];
    expected_token_pair.sort();
    let actual_token_pair = vec![token_0, token_1];

    assert!(pair != ZERO_ADDRESS.into());
    assert!(actual_token_pair == expected_token_pair);
    assert!(pair_len == 1);

    let all_pairs_length_after = factory_contract
        .all_pairs_length(&wealthy_connection)
        .await??;

    assert!(all_pairs_length_after == all_pairs_length_before + 1);

    Ok(())
}

#[tokio::test]
pub async fn mint_pair() -> Result<()> {
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    factory_contract
        .create_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await?;
    let pair = factory_contract
        .get_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    token_a
        .transfer(&wealthy_connection, pair, BALANCE, vec![])
        .await?;
    token_b
        .transfer(&wealthy_connection, pair, BALANCE, vec![])
        .await?;

    let pair_contract: pair_contract::Instance = pair.into();
    let regular_account = regular.account_id().to_account_id();
    let regular_balance_before = pair_contract
        .balance_of(&wealthy_connection, regular_account)
        .await??;

    assert!(regular_balance_before == EXPECTED_INITIAL_REGULAR_BALANCE);

    let mint_tx_info = pair_contract
        .mint(&wealthy_connection, regular_account)
        .await?;

    let all_pair_contract_events = wealthy_connection.get_contract_events(mint_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let mint_events = get_mint_events(pair_contract_events);
    let mint_events_len = mint_events.len();

    assert!(
        mint_events_len == 1,
        "The number of emitted `Mint` events is {}, should be 1.",
        mint_events_len
    );

    let expected_balance = BALANCE - MIN_BALANCE;
    let regular_balance_after = pair_contract
        .balance_of(&wealthy_connection, regular_account)
        .await??;
    let zero_address_balance_after = pair_contract
        .balance_of(&wealthy_connection, ZERO_ADDRESS.into())
        .await??;

    assert!(regular_balance_after == expected_balance);
    assert!(zero_address_balance_after == MIN_BALANCE);

    Ok(())
}

#[tokio::test]
pub async fn swap_tokens() -> Result<()> {
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    factory_contract
        .create_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await?;
    let pair = factory_contract
        .get_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    token_a
        .transfer(&wealthy_connection, pair, BALANCE, vec![])
        .await?;
    token_b
        .transfer(&wealthy_connection, pair, BALANCE, vec![])
        .await?;
    let regular_account = regular.account_id().to_account_id();
    let pair_contract: pair_contract::Instance = pair.into();
    pair_contract
        .mint(&wealthy_connection, regular_account)
        .await?;

    let (first_token, second_token) = sort_tokens(token_a, token_b);
    first_token
        .transfer(&wealthy_connection, pair, FIRST_AMOUNT_IN, vec![])
        .await?;
    let regular_balance_before = second_token
        .balance_of(&wealthy_connection, regular_account)
        .await??;

    assert!(regular_balance_before == EXPECTED_INITIAL_REGULAR_BALANCE);

    let swap_tx_info = pair_contract
        .swap(
            &wealthy_connection,
            FIRST_AMOUNT_OUT,
            SECOND_AMOUNT_OUT,
            regular_account,
        )
        .await?;

    let all_pair_contract_events = wealthy_connection.get_contract_events(swap_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let swap_events = get_swap_events(pair_contract_events);
    let swap_events_len = swap_events.len();

    assert!(
        swap_events_len == 1,
        "The number of emitted `Swap` events is {}, should be 1.",
        swap_events_len
    );

    let regular_balance_after = second_token
        .balance_of(&wealthy_connection, regular_account)
        .await??;

    assert!(regular_balance_after == SECOND_AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn burn_liquidity_provider_token() -> Result<()> {
    const FIRST_BALANCE_LOCKED: Balance = 2_204;
    const SECOND_BALANCE_LOCKED: Balance = 1_820;
    const PAIR_TRANSFER: Balance = 2_000;

    let TestFixture {
        wealthy_connection,
        regular_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    factory_contract
        .create_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await?;
    let pair = factory_contract
        .get_pair(&wealthy_connection, token_a.into(), token_b.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    token_a
        .transfer(&wealthy_connection, pair, BALANCE, vec![])
        .await?;
    token_b
        .transfer(&wealthy_connection, pair, BALANCE, vec![])
        .await?;
    let regular_account = regular.account_id().to_account_id();
    let pair_contract: pair_contract::Instance = pair.into();
    pair_contract
        .mint(&wealthy_connection, regular_account)
        .await?;
    let (first_token, second_token) = sort_tokens(token_a, token_b);
    first_token
        .transfer(&wealthy_connection, pair, FIRST_AMOUNT_IN, vec![])
        .await?;
    pair_contract
        .swap(
            &wealthy_connection,
            FIRST_AMOUNT_OUT,
            SECOND_AMOUNT_OUT,
            regular_account,
        )
        .await?;

    let first_token_balance_before = first_token
        .balance_of(&wealthy_connection, regular_account)
        .await??;
    let second_token_balance_before = second_token
        .balance_of(&wealthy_connection, regular_account)
        .await??;

    pair_contract
        .transfer(&regular_connection, pair, PAIR_TRANSFER, vec![])
        .await?;
    let burn_tx_info = pair_contract
        .burn(&regular_connection, regular_account)
        .await?;

    let all_pair_contract_events = wealthy_connection.get_contract_events(burn_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let burn_events = get_burn_events(pair_contract_events);
    let burn_events_len = burn_events.len();

    assert!(
        burn_events_len == 1,
        "The number of emitted `Burn` events is {}, should be 1.",
        burn_events_len
    );

    let first_token_balance_after = first_token
        .balance_of(&wealthy_connection, regular_account)
        .await??;
    let second_token_balance_after = second_token
        .balance_of(&wealthy_connection, regular_account)
        .await??;
    let first_token_balance_diff = first_token_balance_after - first_token_balance_before;
    let second_token_balance_diff = second_token_balance_after - second_token_balance_before;

    assert!(first_token_balance_diff == FIRST_BALANCE_LOCKED);
    assert!(second_token_balance_diff == SECOND_BALANCE_LOCKED);

    Ok(())
}

pub fn sort_tokens(
    token_a: psp22_token::Instance,
    token_b: psp22_token::Instance,
) -> (psp22_token::Instance, psp22_token::Instance) {
    let mut tokens: Vec<ink_primitives::AccountId> = vec![token_a.into(), token_b.into()];
    tokens.sort();

    (tokens[0].into(), tokens[1].into())
}