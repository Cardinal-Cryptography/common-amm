use anyhow::Result;
use log::info;

use aleph_client::{
    pallets::balances::BalanceUserApi,
    AccountId,
    Balance,
    KeyPair,
    SignedConnection,
    TxStatus,
};

use crate::{
    factory_contract,
    pair_contract,
    psp22_token,
    router_contract,
    wnative_contract,
};

pub const DEFAULT_NODE_ADDRESS: &str = "ws://127.0.0.1:9944";
const SUDO_SEED: &str = "//Alice";
const NON_SUDO_SEED: &str = "//0";
const INITIAL_TRANSFER: Balance = 1_000_000_000_000;
pub const ZERO_ADDRESS: [u8; 32] = [255; 32];
const PSP22_TOTAL_SUPPLY: Balance = 10_000_000;
const TOKEN_A_NAME: &str = "TOKEN_A";
const TOKEN_B_NAME: &str = "TOKEN_B";
const TOKEN_A_SYMBOL: &str = "TKNA";
const TOKEN_B_SYMBOL: &str = "TKNB";
const DECIMALS: u8 = 18;

pub const EXPECTED_INITIAL_ALL_PAIRS_LENGTH: u64 = 0;

pub fn inkify_account_id(account_id: &AccountId) -> ink_primitives::AccountId {
    let inner: [u8; 32] = *account_id.as_ref();
    inner.into()
}

fn setup_keypairs(sudo_seed: &str, non_sudo_seed: &str) -> (KeyPair, KeyPair) {
    let sudo = aleph_client::keypair_from_string(sudo_seed);
    let non_sudo = aleph_client::keypair_from_string(non_sudo_seed);

    (sudo, non_sudo)
}

async fn setup_factory_contract(
    connection: &SignedConnection,
    non_sudo_ink_account_id: ink_primitives::AccountId,
    pair_code_hash: ink_primitives::Hash,
) -> Result<factory_contract::Instance> {
    factory_contract::upload(connection).await?;
    let salt = 1u8.to_le_bytes();
    factory_contract::Instance::new(
        connection,
        salt.into(),
        non_sudo_ink_account_id,
        pair_code_hash,
    )
    .await
}

/// Instances of the `Pair` contract are to be created indirectly via the `Factory` contract.
async fn upload_code_pair_contract(connection: &SignedConnection) -> Result<()> {
    pair_contract::upload(connection).await?;
    Ok(())
}

async fn setup_psp22_token(
    connection: &SignedConnection,
    total_supply: Balance,
    name: Option<String>,
    symbol: Option<String>,
    decimals: u8,
) -> Result<psp22_token::Instance> {
    psp22_token::upload(connection).await?;
    let salt = 1u8.to_le_bytes();
    psp22_token::Instance::new(
        connection,
        salt.into(),
        total_supply,
        name,
        symbol,
        decimals,
    )
    .await
}

async fn setup_wnative_contract(
    connection: &SignedConnection,
) -> Result<wnative_contract::Instance> {
    wnative_contract::upload(connection).await?;
    let salt = 1u8.to_le_bytes();
    wnative_contract::Instance::new(connection, salt.into()).await
}

async fn setup_router_contract(
    connection: &SignedConnection,
    factory: ink_primitives::AccountId,
    wnative: ink_primitives::AccountId,
) -> Result<router_contract::Instance> {
    router_contract::upload(connection).await?;
    let salt = 1u8.to_le_bytes();
    router_contract::Instance::new(connection, salt.into(), factory, wnative).await
}

pub struct Contracts {
    pub factory_contract: factory_contract::Instance,
    pub token_a: psp22_token::Instance,
    pub token_b: psp22_token::Instance,
    pub router_contract: router_contract::Instance,
    pub wnative_contract: wnative_contract::Instance,
}

async fn setup_contracts(
    connection: &SignedConnection,
    non_sudo_ink_account_id: ink_primitives::AccountId,
    pair_code_hash: ink_primitives::Hash,
    total_supply: Balance,
    token_a_name: Option<String>,
    token_a_symbol: Option<String>,
    token_b_name: Option<String>,
    token_b_symbol: Option<String>,
    decimals: u8,
) -> Result<Contracts> {
    upload_code_pair_contract(connection).await?;
    let factory_contract =
        setup_factory_contract(connection, non_sudo_ink_account_id, pair_code_hash).await?;
    let token_a = setup_psp22_token(
        connection,
        total_supply,
        token_a_name,
        token_a_symbol,
        decimals,
    )
    .await?;
    let token_b = setup_psp22_token(
        connection,
        total_supply,
        token_b_name,
        token_b_symbol,
        decimals,
    )
    .await?;
    let wnative_contract = setup_wnative_contract(connection).await?;
    let router_contract =
        setup_router_contract(connection, factory_contract.into(), wnative_contract.into()).await?;

    Ok(Contracts {
        factory_contract,
        token_a,
        token_b,
        router_contract,
        wnative_contract,
    })
}

pub struct TestFixture {
    pub sudo_connection: SignedConnection,
    pub non_sudo_connection: SignedConnection,
    pub sudo: KeyPair,
    pub non_sudo: KeyPair,
    pub contracts: Contracts,
}

pub async fn setup_test() -> Result<TestFixture> {
    let (sudo, non_sudo) = setup_keypairs(SUDO_SEED, NON_SUDO_SEED);
    let sudo_connection = SignedConnection::new(DEFAULT_NODE_ADDRESS, sudo.clone()).await;
    let non_sudo_connection = SignedConnection::new(DEFAULT_NODE_ADDRESS, non_sudo.clone()).await;

    let non_sudo_account_id = non_sudo.account_id();

    sudo_connection
        .transfer(
            non_sudo_account_id.clone(),
            INITIAL_TRANSFER,
            TxStatus::InBlock,
        )
        .await?;

    let non_sudo_ink_account_id = inkify_account_id(non_sudo_account_id);

    let contracts = setup_contracts(
        &sudo_connection,
        non_sudo_ink_account_id,
        pair_contract::CODE_HASH.into(),
        PSP22_TOTAL_SUPPLY,
        Some(TOKEN_A_NAME.to_string()),
        Some(TOKEN_A_SYMBOL.to_string()),
        Some(TOKEN_B_NAME.to_string()),
        Some(TOKEN_B_SYMBOL.to_string()),
        DECIMALS,
    )
    .await?;
    Ok(TestFixture {
        sudo_connection,
        non_sudo_connection,
        sudo,
        non_sudo,
        contracts,
    })
}

use crate::test::{
    fee::{
        fee,
        set_fee,
        set_fee_setter,
    },
    tokens::{
        create_pair,
        mint_pair,
        swap_tokens,
    },
};

#[tokio::test]
async fn e2e_tests() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let test_fixture = setup_test().await?;

    fee(&test_fixture).await?;
    set_fee(&test_fixture).await?;
    set_fee_setter(&test_fixture).await?;

    create_pair(&test_fixture).await?;
    mint_pair(&test_fixture).await?;
    swap_tokens(&test_fixture).await?;

    Ok(())
}