use std::{
    env,
    str::FromStr,
};

use anyhow::Result;

use aleph_client::{
    pallets::balances::BalanceUserApi,
    Balance,
    KeyPair,
    SignedConnection,
    TxStatus,
};
use ink_wrapper_types::util::ToAccountId;

use crate::{
    factory_contract,
    pair_contract,
    psp22_token,
    router_contract,
    wnative_contract,
};
pub use uniswap_v2::helpers::ZERO_ADDRESS;

pub const DEFAULT_NODE_ADDRESS: &str = "ws://127.0.0.1:9944";
const WEALTHY_SEED: &str = "//Alice";
const REGULAR_SEED: &str = "//0";

const INITIAL_TRANSFER: Balance = 1_000_000_000_000;
const PSP22_TOTAL_SUPPLY: Balance = 10_000_000;
const PSP22_DECIMALS: u8 = 18;

const TOKEN_A_NAME: &str = "TOKEN_A";
const TOKEN_B_NAME: &str = "TOKEN_B";
const TOKEN_A_SYMBOL: &str = "TKNA";
const TOKEN_B_SYMBOL: &str = "TKNB";

fn get_env<T>(name: &str) -> Option<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    env::var(name).ok().map(|v| {
        v.parse()
            .unwrap_or_else(|_| panic!("Failed to parse env var {name}"))
    })
}

fn setup_keypairs() -> (KeyPair, KeyPair) {
    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let regular = aleph_client::keypair_from_string(REGULAR_SEED);

    (wealthy, regular)
}

pub async fn setup_factory_contract(
    connection: &SignedConnection,
    regular_account_id: ink_primitives::AccountId,
    pair_code_hash: ink_primitives::Hash,
) -> Result<factory_contract::Instance> {
    factory_contract::upload(connection).await?;
    factory_contract::Instance::new(connection, vec![], regular_account_id, pair_code_hash).await
}

/// Instances of the `Pair` contract are to be created indirectly via the `Factory` contract.
async fn upload_code_pair_contract(connection: &SignedConnection) -> Result<()> {
    pair_contract::upload(connection).await?;
    Ok(())
}

async fn setup_psp22_token(
    connection: &SignedConnection,
    name: &str,
    symbol: &str,
) -> Result<psp22_token::Instance> {
    psp22_token::upload(connection).await?;
    psp22_token::Instance::new(
        connection,
        vec![],
        PSP22_TOTAL_SUPPLY,
        Some(name.to_string()),
        Some(symbol.to_string()),
        PSP22_DECIMALS,
    )
    .await
}

async fn setup_wnative_contract(
    connection: &SignedConnection,
) -> Result<wnative_contract::Instance> {
    wnative_contract::upload(connection).await?;
    wnative_contract::Instance::new(connection, vec![]).await
}

async fn setup_router_contract(
    connection: &SignedConnection,
    factory: ink_primitives::AccountId,
    wnative: ink_primitives::AccountId,
) -> Result<router_contract::Instance> {
    router_contract::upload(connection).await?;
    router_contract::Instance::new(connection, vec![], factory, wnative).await
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
    regular_account_id: ink_primitives::AccountId,
) -> Result<Contracts> {
    upload_code_pair_contract(connection).await?;
    let factory_contract = setup_factory_contract(
        connection,
        regular_account_id,
        pair_contract::CODE_HASH.into(),
    )
    .await?;
    let token_a = setup_psp22_token(connection, TOKEN_A_NAME, TOKEN_A_SYMBOL).await?;
    let token_b = setup_psp22_token(connection, TOKEN_B_NAME, TOKEN_B_SYMBOL).await?;
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
    pub wealthy_connection: SignedConnection,
    pub regular_connection: SignedConnection,
    pub wealthy: KeyPair,
    pub regular: KeyPair,
    pub contracts: Contracts,
}

pub async fn setup_test() -> Result<TestFixture> {
    let _ = env_logger::builder().is_test(true).try_init();

    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());
    let (wealthy, regular) = setup_keypairs();
    let wealthy_connection = SignedConnection::new(&node_address, wealthy.clone()).await;
    let regular_connection = SignedConnection::new(&node_address, regular.clone()).await;

    let regular_account_id = regular.account_id();

    wealthy_connection
        .transfer(
            regular_account_id.clone(),
            INITIAL_TRANSFER,
            TxStatus::InBlock,
        )
        .await?;

    let contracts =
        setup_contracts(&wealthy_connection, regular_account_id.to_account_id()).await?;
    Ok(TestFixture {
        wealthy_connection,
        regular_connection,
        wealthy,
        regular,
        contracts,
    })
}
