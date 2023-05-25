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
        get_mint_events,
        get_pair_created_events,
        get_swap_events,
    },
    factory_contract,
    factory_contract::Factory,
    pair_contract,
    pair_contract::{
        Pair,
        PSP22 as PairPSP22,
    },
    psp22_token::PSP22 as TokenPSP22,
    test::setup::{
        setup_test,
        Contracts,
        TestFixture,
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
        ZERO_ADDRESS,
    },
};

const EXPECTED_ALL_PAIR_LENGTH: u64 = 1;
pub const BALANCE: Balance = 10_000;
const MIN_BALANCE: Balance = 1_000;
const EXPECTED_INITIAL_NON_SUDO_BALANCE: Balance = 0;

const FIRST_AMOUNT_IN: Balance = 1_020;
const FIRST_AMOUNT_OUT: Balance = 0;
const SECOND_AMOUNT_OUT: Balance = 900;

const FIRST_BALANCE_LOCKED: Balance = 2_204;
const SECOND_BALANCE_LOCKED: Balance = 1_820;
const PAIR_TRANSFER: Balance = 2_000;

#[tokio::test]
pub async fn create_pair() -> Result<()> {
    let TestFixture {
        sudo_connection,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        first_token,
        second_token,
        ..
    } = contracts;

    let all_pairs_length_before = factory_contract
        .all_pairs_length(&sudo_connection)
        .await??;

    assert!(all_pairs_length_before == EXPECTED_INITIAL_ALL_PAIRS_LENGTH);

    let tx_info = factory_contract
        .create_pair(&sudo_connection, first_token.into(), second_token.into())
        .await?;

    let all_events = sudo_connection.get_contract_events(tx_info).await?;
    let contract_events = all_events.for_contract(factory_contract);
    let pair_created_events = get_pair_created_events(contract_events);
    let first_pair_created_event = pair_created_events
        .first()
        .ok_or(anyhow!("No `PairCreated` events have been emitted!"))?;
    let factory_contract::event::Event::PairCreated {
        token_0,
        token_1,
        pair,
        pair_len,
    } = first_pair_created_event;

    assert!(*pair != ZERO_ADDRESS.into());
    assert!(*token_0 == first_token.into());
    assert!(*token_1 == second_token.into());
    assert!(*pair_len == EXPECTED_ALL_PAIR_LENGTH);

    let all_pairs_length_after = factory_contract
        .all_pairs_length(&sudo_connection)
        .await??;

    assert!(all_pairs_length_after == EXPECTED_ALL_PAIR_LENGTH);

    Ok(())
}

#[tokio::test]
pub async fn mint_pair() -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        first_token,
        second_token,
        ..
    } = contracts;

    factory_contract
        .create_pair(&sudo_connection, first_token.into(), second_token.into())
        .await?;
    let pair = factory_contract
        .get_pair(&sudo_connection, first_token.into(), second_token.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    first_token
        .transfer(&sudo_connection, pair, BALANCE, vec![])
        .await?;
    second_token
        .transfer(&sudo_connection, pair, BALANCE, vec![])
        .await?;

    let pair_contract: pair_contract::Instance = pair.into();
    let non_sudo_ink_account = non_sudo.account_id().to_account_id();
    let non_sudo_balance_before = pair_contract
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_before == EXPECTED_INITIAL_NON_SUDO_BALANCE);

    let mint_tx_info = pair_contract
        .mint(&sudo_connection, non_sudo_ink_account)
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(mint_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let mint_events = get_mint_events(pair_contract_events);
    mint_events
        .first()
        .ok_or(anyhow!("No `Mint` events have been emitted!"))?;

    let expected_balance = BALANCE - MIN_BALANCE;
    let non_sudo_balance_after = pair_contract
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_after == expected_balance);

    Ok(())
}

#[tokio::test]
pub async fn swap_tokens() -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        first_token,
        second_token,
        ..
    } = contracts;

    factory_contract
        .create_pair(&sudo_connection, first_token.into(), second_token.into())
        .await?;
    let pair = factory_contract
        .get_pair(&sudo_connection, first_token.into(), second_token.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    first_token
        .transfer(&sudo_connection, pair, BALANCE, vec![])
        .await?;
    second_token
        .transfer(&sudo_connection, pair, BALANCE, vec![])
        .await?;
    let non_sudo_ink_account = non_sudo.account_id().to_account_id();
    let pair_contract: pair_contract::Instance = pair.into();
    pair_contract
        .mint(&sudo_connection, non_sudo_ink_account)
        .await?;

    first_token
        .transfer(&sudo_connection, pair, FIRST_AMOUNT_IN, vec![])
        .await?;
    let non_sudo_balance_before = second_token
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_before == EXPECTED_INITIAL_NON_SUDO_BALANCE);

    let swap_tx_info = pair_contract
        .swap(
            &sudo_connection,
            FIRST_AMOUNT_OUT,
            SECOND_AMOUNT_OUT,
            non_sudo_ink_account,
        )
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(swap_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let swap_events = get_swap_events(pair_contract_events);
    swap_events
        .first()
        .ok_or(anyhow!("No `Swap` events have been emitted!"))?;

    let non_sudo_balance_after = second_token
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;

    assert!(non_sudo_balance_after == SECOND_AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn burn_liquidity_provider_token() -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        non_sudo,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        first_token,
        second_token,
        ..
    } = contracts;

    factory_contract
        .create_pair(&sudo_connection, first_token.into(), second_token.into())
        .await?;
    let pair = factory_contract
        .get_pair(&sudo_connection, first_token.into(), second_token.into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    first_token
        .transfer(&sudo_connection, pair, BALANCE, vec![])
        .await?;
    second_token
        .transfer(&sudo_connection, pair, BALANCE, vec![])
        .await?;
    let non_sudo_ink_account = non_sudo.account_id().to_account_id();
    let pair_contract: pair_contract::Instance = pair.into();
    pair_contract
        .mint(&sudo_connection, non_sudo_ink_account)
        .await?;
    first_token
        .transfer(&sudo_connection, pair, FIRST_AMOUNT_IN, vec![])
        .await?;
    pair_contract
        .swap(
            &sudo_connection,
            FIRST_AMOUNT_OUT,
            SECOND_AMOUNT_OUT,
            non_sudo_ink_account,
        )
        .await?;

    let first_token_balance_before = first_token
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;
    let second_token_balance_before = second_token
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;

    pair_contract
        .transfer(&non_sudo_connection, pair, PAIR_TRANSFER, vec![])
        .await?;
    let burn_tx_info = pair_contract
        .burn(&non_sudo_connection, non_sudo_ink_account)
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(burn_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let burn_events = get_burn_events(pair_contract_events);
    burn_events
        .first()
        .ok_or(anyhow!("No `Burn` events have been emitted!"))?;

    let first_token_balance_after = first_token
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;
    let second_token_balance_after = second_token
        .balance_of(&sudo_connection, non_sudo_ink_account)
        .await??;
    let first_token_balance_diff = first_token_balance_after - first_token_balance_before;
    let second_token_balance_diff = second_token_balance_after - second_token_balance_before;

    assert!(first_token_balance_diff == FIRST_BALANCE_LOCKED);
    assert!(second_token_balance_diff == SECOND_BALANCE_LOCKED);

    Ok(())
}
