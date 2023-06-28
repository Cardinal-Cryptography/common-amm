use anyhow::{
    anyhow,
    Result,
};
use assert2::assert;
use tokio::sync::OnceCell;

use aleph_client::{
    Balance,
    SignedConnection,
};
use ink_wrapper_types::{
    util::ToAccountId,
    Connection,
    SignedConnection as _,
    UploadConnection,
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
        get_env,
        random_salt,
        replenish_account,
        set_up_logger,
        try_upload_contract_code,
        DEFAULT_NODE_ADDRESS,
        INITIAL_TRANSFER,
        PSP22_DECIMALS,
        PSP22_TOTAL_SUPPLY,
        REGULAR_SEED,
        TOKEN_A_NAME,
        TOKEN_A_SYMBOL,
        TOKEN_B_NAME,
        TOKEN_B_SYMBOL,
        WEALTHY_SEED,
        ZERO_ADDRESS,
    },
};

const BALANCE: Balance = 10_000;
const MIN_BALANCE: Balance = 1_000;
const EXPECTED_INITIAL_REGULAR_BALANCE: Balance = 0;

const FIRST_AMOUNT_IN: Balance = 1_020;
const FIRST_AMOUNT_OUT: Balance = 0;
const SECOND_AMOUNT_OUT: Balance = 900;

static PAIR_TESTS_CODE_UPLOAD: OnceCell<Result<()>> = OnceCell::const_new();

struct PairTestSetup {
    wealthy_connection: SignedConnection,
    regular_connection: SignedConnection,
    regular_account: ink_primitives::AccountId,
}

async fn pair_tests_code_upload() -> Result<()> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());
    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let wealthy_connection = SignedConnection::new(&node_address, wealthy).await;

    // Instances of the `Pair` contract are to be created indirectly via the `Factory` contract.
    wealthy_connection.upload(pair_contract::upload()).await?;
    wealthy_connection
        .upload(factory_contract::upload())
        .await?;
    wealthy_connection.upload(psp22_token::upload()).await?;

    Ok(())
}

async fn set_up_pair_test() -> Result<PairTestSetup> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());

    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let regular = aleph_client::keypair_from_string(REGULAR_SEED);
    let wealthy_connection = SignedConnection::new(&node_address, wealthy).await;
    let regular_connection = SignedConnection::new(&node_address, regular.clone()).await;

    let regular_account = regular.account_id().to_account_id();

    let pair_test_setup = PairTestSetup {
        wealthy_connection,
        regular_connection,
        regular_account,
    };

    Ok(pair_test_setup)
}

#[tokio::test]
pub async fn create_pair() -> Result<()> {
    set_up_logger();
    try_upload_contract_code(&PAIR_TESTS_CODE_UPLOAD, pair_tests_code_upload).await?;

    let PairTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_pair_test().await?;

    let salt = random_salt();

    let factory_contract: factory_contract::Instance = wealthy_connection
        .instantiate(
            factory_contract::Instance::new(regular_account, pair_contract::CODE_HASH.into())
                .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_a: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_A_NAME.to_string()),
                Some(TOKEN_A_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_b: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_B_NAME.to_string()),
                Some(TOKEN_B_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt),
        )
        .await?
        .into();

    let all_pairs_length_before = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    assert!(all_pairs_length_before == 0);

    let tx_info = wealthy_connection
        .exec(factory_contract.create_pair(token_a.into(), token_b.into()))
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

    let all_pairs_length_after = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    assert!(all_pairs_length_after == all_pairs_length_before + 1);

    Ok(())
}

#[tokio::test]
pub async fn mint_pair() -> Result<()> {
    set_up_logger();
    try_upload_contract_code(&PAIR_TESTS_CODE_UPLOAD, pair_tests_code_upload).await?;

    let PairTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_pair_test().await?;

    let salt = random_salt();

    let factory_contract: factory_contract::Instance = wealthy_connection
        .instantiate(
            factory_contract::Instance::new(regular_account, pair_contract::CODE_HASH.into())
                .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_a: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_A_NAME.to_string()),
                Some(TOKEN_A_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_b: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_B_NAME.to_string()),
                Some(TOKEN_B_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt),
        )
        .await?
        .into();

    wealthy_connection
        .exec(factory_contract.create_pair(token_a.into(), token_b.into()))
        .await?;
    let pair = wealthy_connection
        .read(factory_contract.get_pair(token_a.into(), token_b.into()))
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    wealthy_connection
        .exec(token_a.transfer(pair, BALANCE, vec![]))
        .await?;
    wealthy_connection
        .exec(token_b.transfer(pair, BALANCE, vec![]))
        .await?;

    let pair_contract: pair_contract::Instance = pair.into();
    let regular_balance_before = wealthy_connection
        .read(pair_contract.balance_of(regular_account))
        .await??;

    assert!(regular_balance_before == EXPECTED_INITIAL_REGULAR_BALANCE);

    let mint_tx_info = wealthy_connection
        .exec(pair_contract.mint(regular_account))
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
    let regular_balance_after = wealthy_connection
        .read(pair_contract.balance_of(regular_account))
        .await??;
    let zero_address_balance_after = wealthy_connection
        .read(pair_contract.balance_of(ZERO_ADDRESS.into()))
        .await??;

    assert!(regular_balance_after == expected_balance);
    assert!(zero_address_balance_after == MIN_BALANCE);

    Ok(())
}

#[tokio::test]
pub async fn swap_tokens() -> Result<()> {
    set_up_logger();
    try_upload_contract_code(&PAIR_TESTS_CODE_UPLOAD, pair_tests_code_upload).await?;

    let PairTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_pair_test().await?;

    let salt = random_salt();

    let factory_contract: factory_contract::Instance = wealthy_connection
        .instantiate(
            factory_contract::Instance::new(regular_account, pair_contract::CODE_HASH.into())
                .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_a: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_A_NAME.to_string()),
                Some(TOKEN_A_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_b: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_B_NAME.to_string()),
                Some(TOKEN_B_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt),
        )
        .await?
        .into();

    wealthy_connection
        .exec(factory_contract.create_pair(token_a.into(), token_b.into()))
        .await?;
    let pair = wealthy_connection
        .read(factory_contract.get_pair(token_a.into(), token_b.into()))
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    wealthy_connection
        .exec(token_a.transfer(pair, BALANCE, vec![]))
        .await?;
    wealthy_connection
        .exec(token_b.transfer(pair, BALANCE, vec![]))
        .await?;
    let pair_contract: pair_contract::Instance = pair.into();
    wealthy_connection
        .exec(pair_contract.mint(regular_account))
        .await?;

    let (first_token, second_token) = sort_tokens(token_a, token_b);
    wealthy_connection
        .exec(first_token.transfer(pair, FIRST_AMOUNT_IN, vec![]))
        .await?;
    let regular_balance_before = wealthy_connection
        .read(second_token.balance_of(regular_account))
        .await??;

    assert!(regular_balance_before == EXPECTED_INITIAL_REGULAR_BALANCE);

    let swap_tx_info = wealthy_connection
        .exec(pair_contract.swap(FIRST_AMOUNT_OUT, SECOND_AMOUNT_OUT, regular_account))
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

    let regular_balance_after = wealthy_connection
        .read(second_token.balance_of(regular_account))
        .await??;

    assert!(regular_balance_after == SECOND_AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn burn_liquidity_provider_token() -> Result<()> {
    set_up_logger();
    try_upload_contract_code(&PAIR_TESTS_CODE_UPLOAD, pair_tests_code_upload).await?;

    const FIRST_BALANCE_LOCKED: Balance = 2_204;
    const SECOND_BALANCE_LOCKED: Balance = 1_820;
    const PAIR_TRANSFER: Balance = 2_000;

    let PairTestSetup {
        wealthy_connection,
        regular_connection,
        regular_account,
    } = set_up_pair_test().await?;

    let salt = random_salt();

    let factory_contract: factory_contract::Instance = wealthy_connection
        .instantiate(
            factory_contract::Instance::new(regular_account, pair_contract::CODE_HASH.into())
                .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_a: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_A_NAME.to_string()),
                Some(TOKEN_A_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt.clone()),
        )
        .await?
        .into();
    let token_b: psp22_token::Instance = wealthy_connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_B_NAME.to_string()),
                Some(TOKEN_B_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt),
        )
        .await?
        .into();

    wealthy_connection
        .exec(factory_contract.create_pair(token_a.into(), token_b.into()))
        .await?;
    let pair = wealthy_connection
        .read(factory_contract.get_pair(token_a.into(), token_b.into()))
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?;
    wealthy_connection
        .exec(token_a.transfer(pair, BALANCE, vec![]))
        .await?;
    wealthy_connection
        .exec(token_b.transfer(pair, BALANCE, vec![]))
        .await?;
    let pair_contract: pair_contract::Instance = pair.into();
    wealthy_connection
        .exec(pair_contract.mint(regular_account))
        .await?;
    let (first_token, second_token) = sort_tokens(token_a, token_b);
    wealthy_connection
        .exec(first_token.transfer(pair, FIRST_AMOUNT_IN, vec![]))
        .await?;
    wealthy_connection
        .exec(pair_contract.swap(FIRST_AMOUNT_OUT, SECOND_AMOUNT_OUT, regular_account))
        .await?;

    let first_token_balance_before = wealthy_connection
        .read(first_token.balance_of(regular_account))
        .await??;
    let second_token_balance_before = wealthy_connection
        .read(second_token.balance_of(regular_account))
        .await??;

    let dest = aleph_client::AccountId::new(*regular_account.as_ref());
    replenish_account(&wealthy_connection, dest, INITIAL_TRANSFER).await?;
    regular_connection
        .exec(pair_contract.transfer(pair, PAIR_TRANSFER, vec![]))
        .await?;
    let burn_tx_info = regular_connection
        .exec(pair_contract.burn(regular_account))
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

    let first_token_balance_after = wealthy_connection
        .read(first_token.balance_of(regular_account))
        .await??;
    let second_token_balance_after = wealthy_connection
        .read(second_token.balance_of(regular_account))
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
