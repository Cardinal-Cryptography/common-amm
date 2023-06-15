use std::{
    env,
    str::FromStr,
};

use anyhow::Result;

use aleph_client::{
    pallets::{
        balances::BalanceUserApi,
        system::SystemApi,
    },
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
pub use uniswap_v2::helpers::ZERO_ADDRESS;

pub const DEFAULT_NODE_ADDRESS: &str = "ws://127.0.0.1:9944";
pub const WEALTHY_SEED: &str = "//Alice";
const REGULAR_SEED: &str = "//0";

pub const INITIAL_TRANSFER: Balance = 1_000_000_000_000;
const PSP22_TOTAL_SUPPLY: Balance = 10_000_000;
const PSP22_DECIMALS: u8 = 18;

pub const TOKEN_A_NAME: &str = "TOKEN_A";
pub const TOKEN_B_NAME: &str = "TOKEN_B";
pub const TOKEN_A_SYMBOL: &str = "TKNA";
pub const TOKEN_B_SYMBOL: &str = "TKNB";

pub fn get_env<T>(name: &str) -> Option<T>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    env::var(name).ok().map(|v| {
        v.parse()
            .unwrap_or_else(|_| panic!("Failed to parse env var {name}"))
    })
}

pub fn set_up_key_pairs() -> (KeyPair, KeyPair) {
    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let regular = aleph_client::keypair_from_string(REGULAR_SEED);

    (wealthy, regular)
}

pub async fn set_up_connections(
    node_address: &str,
    wealthy: KeyPair,
    regular: KeyPair,
) -> (SignedConnection, SignedConnection) {
    let wealthy_connection = SignedConnection::new(node_address, wealthy).await;
    let regular_connection = SignedConnection::new(node_address, regular).await;

    (wealthy_connection, regular_connection)
}

pub async fn set_up_factory_contract(
    connection: &SignedConnection,
    regular_account: ink_primitives::AccountId,
    pair_code_hash: ink_primitives::Hash,
) -> Result<factory_contract::Instance> {
    factory_contract::upload(connection).await?;
    factory_contract::Instance::new(connection, vec![], regular_account, pair_code_hash).await
}

/// Instances of the `Pair` contract are to be created indirectly via the `Factory` contract.
pub async fn upload_code_pair_contract(connection: &SignedConnection) -> Result<()> {
    pair_contract::upload(connection).await?;
    Ok(())
}

pub async fn set_up_psp22_token(
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

async fn set_up_wnative_contract(
    connection: &SignedConnection,
) -> Result<wnative_contract::Instance> {
    wnative_contract::upload(connection).await?;
    wnative_contract::Instance::new(connection, vec![]).await
}

async fn set_up_router_contract(
    connection: &SignedConnection,
    factory: ink_primitives::AccountId,
    wnative: ink_primitives::AccountId,
) -> Result<router_contract::Instance> {
    router_contract::upload(connection).await?;
    router_contract::Instance::new(connection, vec![], factory, wnative).await
}

pub fn set_up_logger() {
    let _ = env_logger::builder().is_test(true).try_init();
}

pub async fn replenish_account(
    wealthy_connection: &SignedConnection,
    destination: aleph_client::AccountId,
    desired_balance: Balance,
) -> Result<()> {
    let regular_balance = wealthy_connection
        .get_free_balance(destination.clone(), None)
        .await;
    if regular_balance < desired_balance {
        wealthy_connection
            .transfer(
                destination,
                INITIAL_TRANSFER - regular_balance,
                TxStatus::InBlock,
            )
            .await?;
    }

    Ok(())
}
