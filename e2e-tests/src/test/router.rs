use anyhow::Result;
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
    factory_contract,
    factory_contract::Factory,
    pair_contract,
    psp22_token,
    psp22_token::PSP22 as TokenPSP22,
    router_contract,
    router_contract::Router,
    test::setup::{
        get_env,
        random_salt,
        set_up_logger,
        try_upload_contract_code,
        DEFAULT_NODE_ADDRESS,
        PSP22_DECIMALS,
        PSP22_TOTAL_SUPPLY,
        REGULAR_SEED,
        TOKEN_A_NAME,
        TOKEN_A_SYMBOL,
        WEALTHY_SEED,
    },
    wnative_contract,
    wnative_contract::{
        Wnative,
        PSP22 as WnativePSP22,
    },
};

const DEADLINE: u64 = 111_111_111_111_111_111;
const AMOUNT_AVAILABLE_FOR_WITHDRAWAL: Balance = 10_000;
const AMOUNT_OUT_MIN: Balance = 1_000;

static ROUTER_TESTS_CODE_UPLOAD: OnceCell<Result<()>> = OnceCell::const_new();

struct RouterTestSetup {
    wealthy_connection: SignedConnection,
    regular_connection: SignedConnection,
    wealthy_account: ink_primitives::AccountId,
    regular_account: ink_primitives::AccountId,
}

async fn router_tests_code_upload() -> Result<()> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());
    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let wealthy_connection = SignedConnection::new(&node_address, wealthy).await;

    // Instances of the `Pair` contract are to be created indirectly via the `Factory` contract.
    wealthy_connection.upload(pair_contract::upload()).await?;
    wealthy_connection
        .upload(factory_contract::upload())
        .await?;
    wealthy_connection.upload(psp22_token::upload()).await?;
    wealthy_connection.upload(router_contract::upload()).await?;

    Ok(())
}

async fn set_up_router_test() -> Result<RouterTestSetup> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());

    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let regular = aleph_client::keypair_from_string(REGULAR_SEED);
    let wealthy_connection = SignedConnection::new(&node_address, wealthy.clone()).await;
    let regular_connection = SignedConnection::new(&node_address, regular.clone()).await;

    let wealthy_account = wealthy.account_id().to_account_id();
    let regular_account = regular.account_id().to_account_id();

    let router_test_setup = RouterTestSetup {
        wealthy_connection,
        regular_connection,
        wealthy_account,
        regular_account,
    };

    Ok(router_test_setup)
}

// TODO: transfer native tokens to make the tests work
#[tokio::test]
pub async fn add_liquidity_via_router() -> Result<()> {
    println!("Running `add_liquidity_via_router` test.");
    set_up_logger();
    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        ..
    } = set_up_router_test().await?;

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
    let wnative_contract: wnative_contract::Instance = wealthy_connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?
        .into();
    let router_contract: router_contract::Instance = wealthy_connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_AVAILABLE_FOR_WITHDRAWAL))
        .await?;

    let all_pairs_length_before = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    // TODO: Check whether `with_value` here receives what is necessary.
    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
                    AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
                    AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
                    wealthy_account,
                    DEADLINE,
                )
                .with_value(AMOUNT_AVAILABLE_FOR_WITHDRAWAL),
        )
        .await?;

    let expected_all_pairs_length = all_pairs_length_before + 1;
    let all_pairs_length_after = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    assert!(all_pairs_length_after == expected_all_pairs_length);

    Ok(())
}

#[tokio::test]
pub async fn swap_exact_native_for_tokens_via_router() -> Result<()> {
    println!("Running `swap_exact_native_for_tokens_via_router` test.");
    set_up_logger();
    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_router_test().await?;

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
    let wnative_contract: wnative_contract::Instance = wealthy_connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?
        .into();
    let router_contract: router_contract::Instance = wealthy_connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    // TODO: Check whether `with_value` here receives what is necessary.
    wealthy_connection
        .exec(
            router_contract
                .swap_exact_native_for_tokens(
                    AMOUNT_OUT_MIN,
                    vec![wnative_contract.into(), token_a.into()],
                    regular_account,
                    DEADLINE,
                )
                .with_value(AMOUNT_AVAILABLE_FOR_WITHDRAWAL),
        )
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn swap_native_for_exact_tokens_via_router() -> Result<()> {
    println!("Running `swap_native_for_exact_tokens_via_router` test.");
    set_up_logger();
    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_router_test().await?;

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
    let wnative_contract: wnative_contract::Instance = wealthy_connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?
        .into();
    let router_contract: router_contract::Instance = wealthy_connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    // TODO: Check whether `with_value` here receives what is necessary.
    wealthy_connection
        .exec(
            router_contract
                .swap_native_for_exact_tokens(
                    AMOUNT_OUT_MIN,
                    vec![wnative_contract.into(), token_a.into()],
                    regular_account,
                    DEADLINE,
                )
                .with_value(AMOUNT_AVAILABLE_FOR_WITHDRAWAL),
        )
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn swap_exact_tokens_for_tokens_via_router() -> Result<()> {
    println!("Running `swap_native_for_exact_tokens_via_router` test.");
    set_up_logger();
    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_router_test().await?;

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
    let wnative_contract: wnative_contract::Instance = wealthy_connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?
        .into();
    let router_contract: router_contract::Instance = wealthy_connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    // TODO: Check whether `with_value` here receives what is necessary.
    wealthy_connection
        .exec(
            wnative_contract
                .deposit()
                .with_value(AMOUNT_AVAILABLE_FOR_WITHDRAWAL),
        )
        .await?;

    wealthy_connection
        .exec(wnative_contract.approve(router_contract.into(), AMOUNT_AVAILABLE_FOR_WITHDRAWAL))
        .await?;

    wealthy_connection
        .exec(router_contract.swap_exact_tokens_for_tokens(
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
            AMOUNT_OUT_MIN,
            vec![wnative_contract.into(), token_a.into()],
            regular_account,
            DEADLINE,
        ))
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn swap_tokens_for_exact_tokens_via_router() -> Result<()> {
    println!("Running `swap_tokens_for_exact_tokens_via_router` test.");
    set_up_logger();
    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_router_test().await?;

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
    let wnative_contract: wnative_contract::Instance = wealthy_connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?
        .into();
    let router_contract: router_contract::Instance = wealthy_connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    // TODO: Do we need this additional const?
    const AMOUNT_FOR_SWAP: Balance = 100_000;

    // TODO: Check whether `with_value` here receives what is necessary.
    wealthy_connection
        .exec(
            wnative_contract
                .deposit()
                .with_value(AMOUNT_AVAILABLE_FOR_WITHDRAWAL),
        )
        .await?;

    wealthy_connection
        .exec(wnative_contract.approve(router_contract.into(), AMOUNT_FOR_SWAP))
        .await?;

    wealthy_connection
        .exec(router_contract.swap_tokens_for_exact_tokens(
            AMOUNT_OUT_MIN,
            AMOUNT_FOR_SWAP,
            vec![wnative_contract.into(), token_a.into()],
            regular_account,
            DEADLINE,
        ))
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn add_more_liquidity_via_router() -> Result<()> {
    println!("Running `add_more_liquidity_via_router` test.");
    set_up_logger();
    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterTestSetup {
        wealthy_connection,
        regular_account,
        ..
    } = set_up_router_test().await?;

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
    let wnative_contract: wnative_contract::Instance = wealthy_connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?
        .into();
    let router_contract: router_contract::Instance = wealthy_connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_AVAILABLE_FOR_WITHDRAWAL))
        .await?;

    Ok(())
}
