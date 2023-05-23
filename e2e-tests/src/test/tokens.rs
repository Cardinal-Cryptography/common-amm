use anyhow::{
    anyhow,
    Result,
};
use log::info;

use aleph_client::{
    Balance,
    SignedConnection,
};
use ink_wrapper_types::Connection;

use crate::{
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
        inkify_account_id,
        Contracts,
        TestFixture,
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
        ZERO_ADDRESS,
    },
};

const EXPECTED_ALL_PAIR_LENGTH: u64 = 1;
const BALANCE: Balance = 10_000;
const MIN_BALANCE: Balance = 1_000;
const EXPECTED_INITIAL_NON_SUDO_BALANCE: Balance = 0;

const AMOUNT_A_IN: Balance = 1_020;
const AMOUNT_A_OUT: Balance = 0;
const AMOUNT_B_OUT: Balance = 900;

pub async fn create_pair(test_fixture: &TestFixture) -> Result<()> {
    println!("Running `create_pair` test.");
    info!("Running `create_pair` test.");
    let TestFixture {
        sudo_connection,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a: first_token,
        token_b: second_token,
        ..
    } = contracts;

    all_pairs_length(
        sudo_connection,
        factory_contract,
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
    )
    .await?;

    let token_a: ink_primitives::AccountId = (*first_token).into();
    let token_b: ink_primitives::AccountId = (*second_token).into();

    let tx_info = factory_contract
        .create_pair(sudo_connection, token_a, token_b)
        .await?;
    let all_events = sudo_connection.get_contract_events(tx_info).await?;
    let contract_events = all_events.for_contract(*factory_contract);
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

    let mut expected_token_pair = vec![token_a, token_b];
    expected_token_pair.sort();

    let actual_token_pair = vec![*token_0, *token_1];

    assert_ne!(*pair, ZERO_ADDRESS.into());

    assert_eq!(actual_token_pair, expected_token_pair);

    assert_eq!(*pair_len, EXPECTED_ALL_PAIR_LENGTH);
    all_pairs_length(sudo_connection, factory_contract, EXPECTED_ALL_PAIR_LENGTH).await?;

    Ok(())
}

pub async fn mint_pair(test_fixture: &TestFixture) -> Result<()> {
    println!("Running `mint_pair` test.");
    info!("Running `mint_pair` test.");
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    let pair = factory_contract
        .get_pair(sudo_connection, (*token_a).into(), (*token_b).into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;

    token_a
        .transfer(sudo_connection, pair, BALANCE, vec![])
        .await?;
    token_b
        .transfer(sudo_connection, pair, BALANCE, vec![])
        .await?;

    let pair_contract: pair_contract::Instance = pair.into();
    let non_sudo_ink_account = inkify_account_id(non_sudo.account_id());
    balance_of(
        sudo_connection,
        pair_contract,
        non_sudo_ink_account,
        EXPECTED_INITIAL_NON_SUDO_BALANCE,
    )
    .await?;

    let mint_tx_info = pair_contract
        .mint(sudo_connection, non_sudo_ink_account)
        .await?;
    let all_pair_contract_events = sudo_connection.get_contract_events(mint_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let mint_events = get_mint_events(pair_contract_events);
    mint_events
        .first()
        .ok_or(anyhow!("No `Mint` events have been emitted!"))?;

    let expected_balance = BALANCE - MIN_BALANCE;
    balance_of(
        sudo_connection,
        pair_contract,
        non_sudo_ink_account,
        expected_balance,
    )
    .await?;

    Ok(())
}

pub async fn swap_tokens(test_fixture: &TestFixture) -> Result<()> {
    println!("Running `swap_tokens` test.");
    info!("Running `swap_tokens` test.");
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        token_b,
        ..
    } = contracts;

    let pair = factory_contract
        .get_pair(sudo_connection, (*token_a).into(), (*token_b).into())
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;

    let mut tokens: Vec<ink_primitives::AccountId> = vec![(*token_a).into(), (*token_b).into()];
    tokens.sort();

    let first_token: psp22_token::Instance = tokens[0].into();
    let second_token: psp22_token::Instance = tokens[1].into();

    first_token
        .transfer(sudo_connection, pair, AMOUNT_A_IN, vec![])
        .await?;

    let non_sudo_ink_account = inkify_account_id(non_sudo.account_id());
    balance_of(
        sudo_connection,
        second_token,
        non_sudo_ink_account,
        EXPECTED_INITIAL_NON_SUDO_BALANCE,
    )
    .await?;

    let pair_contract: pair_contract::Instance = pair.into();

    let swap_tx_info = pair_contract
        .swap(
            sudo_connection,
            AMOUNT_A_OUT,
            AMOUNT_B_OUT,
            non_sudo_ink_account,
        )
        .await?;

    let all_pair_contract_events = sudo_connection.get_contract_events(swap_tx_info).await?;
    let pair_contract_events = all_pair_contract_events.for_contract(pair_contract);
    let swap_events = get_swap_events(pair_contract_events);

    swap_events
        .first()
        .ok_or(anyhow!("No `Swap` events have been emitted!"))?;
    balance_of(
        sudo_connection,
        second_token,
        non_sudo_ink_account,
        AMOUNT_B_OUT,
    )
    .await?;

    Ok(())
}

pub async fn all_pairs_length(
    connection: &SignedConnection,
    factory_contract: &factory_contract::Instance,
    expected_all_pairs_length: u64,
) -> Result<()> {
    let all_pair_length = factory_contract.all_pairs_length(connection).await??;
    assert_eq!(all_pair_length, expected_all_pairs_length);
    Ok(())
}

fn get_pair_created_events(
    contract_events: Vec<Result<factory_contract::event::Event, scale::Error>>,
) -> Vec<factory_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| res.ok())
        .collect()
}

fn get_mint_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| {
            let event = res.ok();
            match event {
                Some(pair_contract::event::Event::Mint { .. }) => event,
                _ => None,
            }
        })
        .collect()
}

fn get_swap_events(
    contract_events: Vec<Result<pair_contract::event::Event, scale::Error>>,
) -> Vec<pair_contract::event::Event> {
    contract_events
        .into_iter()
        .filter_map(|res| {
            let event = res.ok();
            match event {
                Some(pair_contract::event::Event::Swap { .. }) => event,
                _ => None,
            }
        })
        .collect()
}

// TODO: Can we get rid of this indirection?
#[async_trait::async_trait]
pub trait BalanceOf {
    async fn balance<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E>;
}

#[async_trait::async_trait]
impl BalanceOf for pair_contract::Instance {
    async fn balance<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E> {
        self.balance_of(conn, owner).await
    }
}

#[async_trait::async_trait]
impl BalanceOf for psp22_token::Instance {
    async fn balance<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E> {
        self.balance_of(conn, owner).await
    }
}

pub async fn balance_of<Contract: BalanceOf>(
    connection: &SignedConnection,
    contract: Contract,
    non_sudo_ink_account: ink_primitives::AccountId,
    expected_balance: Balance,
) -> Result<()> {
    let non_sudo_balance = contract.balance(connection, non_sudo_ink_account).await??;
    assert_eq!(non_sudo_balance, expected_balance);
    Ok(())
}
