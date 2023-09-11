use std::time::{
    Duration,
    SystemTime,
    UNIX_EPOCH,
};

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
use amm::impls::pair::pair::MINIMUM_LIQUIDITY;
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
    pair_contract::{
        Pair,
        PSP22,
    },
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

const AMOUNT_TOKEN_DESIRED: Balance = 10_000;
const AMOUNT_OUT: Balance = 1_000;

static ROUTER_TESTS_CODE_UPLOAD: OnceCell<Result<()>> = OnceCell::const_new();

struct RouterContractsSetup {
    factory_contract: factory_contract::Instance,
    token_a: psp22_token::Instance,
    wnative_contract: wnative_contract::Instance,
    router_contract: router_contract::Instance,
}

struct RouterTestSetup {
    wealthy_connection: SignedConnection,
    wealthy_account: ink_primitives::AccountId,
    regular_account: ink_primitives::AccountId,
    factory_contract: factory_contract::Instance,
    token_a: psp22_token::Instance,
    wnative_contract: wnative_contract::Instance,
    router_contract: router_contract::Instance,
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
    wealthy_connection
        .upload(wnative_contract::upload())
        .await?;
    wealthy_connection.upload(router_contract::upload()).await?;

    Ok(())
}

async fn set_up_contracts(
    connection: &SignedConnection,
    fee_to_setter: ink_primitives::AccountId,
) -> Result<RouterContractsSetup> {
    let salt = random_salt();

    let factory_contract = connection
        .instantiate(
            factory_contract::Instance::new(fee_to_setter, pair_contract::CODE_HASH.into())
                .with_salt(salt.clone()),
        )
        .await?;
    let token_a = connection
        .instantiate(
            psp22_token::Instance::new(
                PSP22_TOTAL_SUPPLY,
                Some(TOKEN_A_NAME.to_string()),
                Some(TOKEN_A_SYMBOL.to_string()),
                PSP22_DECIMALS,
            )
            .with_salt(salt.clone()),
        )
        .await?;
    let wnative_contract = connection
        .instantiate(wnative_contract::Instance::new().with_salt(salt.clone()))
        .await?;
    let router_contract = connection
        .instantiate(
            router_contract::Instance::new(factory_contract.into(), wnative_contract.into())
                .with_salt(salt),
        )
        .await?;

    let router_contracts_setup = RouterContractsSetup {
        factory_contract,
        token_a,
        wnative_contract,
        router_contract,
    };

    Ok(router_contracts_setup)
}

async fn set_up_router_test() -> Result<RouterTestSetup> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());

    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let regular = aleph_client::keypair_from_string(REGULAR_SEED);
    let wealthy_connection = SignedConnection::new(&node_address, wealthy.clone()).await;

    let wealthy_account = wealthy.account_id().to_account_id();
    let regular_account = regular.account_id().to_account_id();

    try_upload_contract_code(&ROUTER_TESTS_CODE_UPLOAD, router_tests_code_upload).await?;

    let RouterContractsSetup {
        factory_contract,
        token_a,
        wnative_contract,
        router_contract,
    } = set_up_contracts(&wealthy_connection, regular_account).await?;

    let router_test_setup = RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        factory_contract,
        token_a,
        wnative_contract,
        router_contract,
    };

    Ok(router_test_setup)
}

#[tokio::test]
pub async fn add_liquidity() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        factory_contract,
        token_a,
        router_contract,
        wnative_contract,
        ..
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;

    let all_pairs_length_before = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline.try_into().unwrap(),
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;

    let all_pairs_length_after = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    assert!(all_pairs_length_after == all_pairs_length_before + 1);

    let pair_contract: pair_contract::Instance = wealthy_connection
        .read(factory_contract.get_pair(wnative_contract.into(), token_a.into()))
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?
        .into();

    let wealthy_account_pair_balance = wealthy_connection
        .read(pair_contract.balance_of(wealthy_account))
        .await??;

    assert!(wealthy_account_pair_balance == AMOUNT_TOKEN_DESIRED - MINIMUM_LIQUIDITY);

    Ok(())
}

#[tokio::test]
pub async fn swap_exact_native_for_tokens() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        token_a,
        wnative_contract,
        router_contract,
        ..
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;
    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline,
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;

    let path = vec![wnative_contract.into(), token_a.into()];
    let amounts_in = wealthy_connection
        .read(router_contract.get_amounts_in(AMOUNT_OUT, path.clone()))
        .await??
        .map_err(|e| anyhow!("Cannot read amounts in from router contract: {:?}", e))?;
    let regular_account_balance_before = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    wealthy_connection
        .exec(
            router_contract
                .swap_exact_native_for_tokens(AMOUNT_OUT, path, regular_account, deadline)
                .with_value(amounts_in[0]),
        )
        .await?;
    let regular_account_balance_after = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    let balance_diff = regular_account_balance_after - regular_account_balance_before;

    assert!(balance_diff >= AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn swap_native_for_exact_tokens() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        token_a,
        wnative_contract,
        router_contract,
        ..
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;
    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline,
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;

    let path = vec![wnative_contract.into(), token_a.into()];
    let amounts_in = wealthy_connection
        .read(router_contract.get_amounts_in(AMOUNT_OUT, path.clone()))
        .await??
        .map_err(|e| anyhow!("Cannot read amounts in from router contract: {:?}", e))?;
    let regular_account_balance_before = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    wealthy_connection
        .exec(
            router_contract
                .swap_native_for_exact_tokens(AMOUNT_OUT, path, regular_account, deadline)
                .with_value(amounts_in[0]),
        )
        .await?;
    let regular_account_balance_after = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    let balance_diff = regular_account_balance_after - regular_account_balance_before;

    assert!(balance_diff == AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn swap_exact_tokens_for_tokens() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        token_a,
        wnative_contract,
        router_contract,
        ..
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;
    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline,
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;
    wealthy_connection
        .exec(wnative_contract.deposit().with_value(AMOUNT_TOKEN_DESIRED))
        .await?;

    wealthy_connection
        .exec(wnative_contract.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;

    let regular_account_balance_before = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    wealthy_connection
        .exec(router_contract.swap_exact_tokens_for_tokens(
            AMOUNT_TOKEN_DESIRED,
            AMOUNT_OUT,
            vec![wnative_contract.into(), token_a.into()],
            regular_account,
            deadline,
        ))
        .await?;
    let regular_account_balance_after = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    let balance_diff = regular_account_balance_after - regular_account_balance_before;

    assert!(balance_diff >= AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn swap_tokens_for_exact_tokens() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        token_a,
        wnative_contract,
        router_contract,
        ..
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;
    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline,
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;

    const AMOUNT_FOR_SWAP: Balance = 100_000;

    wealthy_connection
        .exec(wnative_contract.deposit().with_value(AMOUNT_FOR_SWAP))
        .await?;

    wealthy_connection
        .exec(wnative_contract.approve(router_contract.into(), AMOUNT_FOR_SWAP))
        .await?;

    let regular_account_balance_before = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;

    wealthy_connection
        .exec(router_contract.swap_tokens_for_exact_tokens(
            AMOUNT_OUT,
            AMOUNT_FOR_SWAP,
            vec![wnative_contract.into(), token_a.into()],
            regular_account,
            deadline,
        ))
        .await?;

    let regular_account_balance_after = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    let balance_diff = regular_account_balance_after - regular_account_balance_before;

    assert!(balance_diff == AMOUNT_OUT);

    Ok(())
}

#[tokio::test]
pub async fn add_more_liquidity() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        factory_contract,
        token_a,
        router_contract,
        ..
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    let all_pairs_length_before = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), 2 * AMOUNT_TOKEN_DESIRED))
        .await?;

    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline,
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;

    let wealthy_account_balance_before_second_liquidity_addition = wealthy_connection
        .read(token_a.balance_of(wealthy_account))
        .await??;

    const LARGE_AMOUNT: Balance = 1_000_000_000_000_000;

    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    0,
                    0,
                    wealthy_account,
                    deadline,
                )
                .with_value(LARGE_AMOUNT),
        )
        .await?;

    let all_pairs_length_after = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    let wealthy_account_balance_after_second_liquidity_addition = wealthy_connection
        .read(token_a.balance_of(wealthy_account))
        .await??;
    let balance_diff = wealthy_account_balance_before_second_liquidity_addition
        - wealthy_account_balance_after_second_liquidity_addition;

    assert!(balance_diff < LARGE_AMOUNT);
    assert!(all_pairs_length_after == all_pairs_length_before + 1);

    Ok(())
}

#[tokio::test]
pub async fn remove_liquidity() -> Result<()> {
    set_up_logger();

    let RouterTestSetup {
        wealthy_connection,
        wealthy_account,
        regular_account,
        factory_contract,
        token_a,
        wnative_contract,
        router_contract,
    } = set_up_router_test().await?;

    let deadline = timestamp_one_hour_forward_millis();

    wealthy_connection
        .exec(token_a.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;

    let all_pairs_length_before = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    wealthy_connection
        .exec(
            router_contract
                .add_liquidity_native(
                    token_a.into(),
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    AMOUNT_TOKEN_DESIRED,
                    wealthy_account,
                    deadline,
                )
                .with_value(AMOUNT_TOKEN_DESIRED),
        )
        .await?;

    let pair_contract: pair_contract::Instance = wealthy_connection
        .read(factory_contract.get_pair(wnative_contract.into(), token_a.into()))
        .await??
        .ok_or(anyhow!("Specified token pair does not exist!"))?
        .into();
    wealthy_connection
        .exec(pair_contract.approve(router_contract.into(), AMOUNT_TOKEN_DESIRED))
        .await?;

    let regular_account_balance_before = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;

    let wealthy_account_pair_balance_before = wealthy_connection
        .read(pair_contract.balance_of(wealthy_account))
        .await??;
    wealthy_connection
        .exec(router_contract.remove_liquidity_native(
            token_a.into(),
            wealthy_account_pair_balance_before,
            0,
            0,
            regular_account,
            deadline,
        ))
        .await?;

    let all_pairs_length_after = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    let regular_account_balance_after = wealthy_connection
        .read(token_a.balance_of(regular_account))
        .await??;
    let balance_diff = regular_account_balance_after - regular_account_balance_before;
    let pair_contract_reserves_after = wealthy_connection
        .read(pair_contract.get_reserves())
        .await??;

    assert!(pair_contract_reserves_after.0 == MINIMUM_LIQUIDITY);
    assert!(pair_contract_reserves_after.1 == MINIMUM_LIQUIDITY);
    assert!(balance_diff == wealthy_account_pair_balance_before);
    assert!(all_pairs_length_after == all_pairs_length_before + 1);

    Ok(())
}

fn timestamp_one_hour_forward_millis() -> u64 {
    let now = SystemTime::now();
    let one_hour = Duration::from_secs(3600);
    let one_hour_forward = now + one_hour;
    one_hour_forward
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}
