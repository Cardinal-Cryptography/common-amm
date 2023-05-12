use anyhow::Result;
use log;

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

const DEFAULT_NODE_ADDRESS: &str = "ws://127.0.0.1:9944";
const SUDO_SEED: &str = "//Alice";
const NON_SUDO_SEED: &str = "//0";
const INITIAL_TRANSFER: Balance = 1_000_000_000_000;
const PSP22_TOTAL_SUPPLY: Balance = 10_000_000;
const TOKEN_A_NAME: &str = "TOKEN_A";
const TOKEN_B_NAME: &str = "TOKEN_B";
const TOKEN_A_SYMBOL: &str = "TKNA";
const TOKEN_B_SYMBOL: &str = "TKNB";
const DECIMALS: u8 = 18;

pub fn inkify_account_id(account_id: &AccountId) -> ink_primitives::AccountId {
    let inner: [u8; 32] = *account_id.as_ref();
    inner.into()
}

fn setup_keypairs(sudo_seed: &str, non_sudo_seed: &str) -> (KeyPair, KeyPair) {
    let sudo = aleph_client::keypair_from_string(sudo_seed);
    let non_sudo = aleph_client::keypair_from_string(non_sudo_seed);

    (sudo, non_sudo)
}

async fn setup_pair_contract(connection: &SignedConnection) -> Result<pair_contract::Instance> {
    let salt = 1u8.to_le_bytes();
    pair_contract::Instance::new(connection, salt.into()).await
}

async fn setup_factory_contract(
    connection: &SignedConnection,
    sudo_ink_account_id: ink_primitives::AccountId,
) -> Result<factory_contract::Instance> {
    let salt = 1u8.to_le_bytes();
    // TODO: can we get this from `ink-wrapper`?
    let pair_code_hash = [
        20, 50, 227, 100, 237, 15, 226, 122, 184, 218, 36, 232, 170, 107, 225, 198, 177, 36, 234,
        235, 240, 142, 147, 208, 48, 183, 103, 78, 186, 81, 202, 141,
    ];
    factory_contract::Instance::new(
        connection,
        salt.into(),
        sudo_ink_account_id,
        pair_code_hash.into(),
    )
    .await
}

async fn setup_psp22_token(
    connection: &SignedConnection,
    total_supply: Balance,
    name: Option<String>,
    symbol: Option<String>,
    decimals: u8,
) -> Result<psp22_token::Instance> {
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
    let salt = 1u8.to_le_bytes();
    wnative_contract::Instance::new(connection, salt.into()).await
}

async fn setup_router_contract(
    connection: &SignedConnection,
    factory: ink_primitives::AccountId,
    wnative: ink_primitives::AccountId,
) -> Result<router_contract::Instance> {
    let salt = 1u8.to_le_bytes();
    router_contract::Instance::new(connection, salt.into(), factory, wnative).await
}

pub struct Contracts {
    pub factory_contract: factory_contract::Instance,
    pub pair_contract: pair_contract::Instance,
    pub token_a: psp22_token::Instance,
    pub token_b: psp22_token::Instance,
    pub router_contract: router_contract::Instance,
    pub wnative_contract: wnative_contract::Instance,
}

async fn setup_contracts(
    connection: &SignedConnection,
    sudo_ink_account_id: ink_primitives::AccountId,
    total_supply: Balance,
    token_a_name: Option<String>,
    token_a_symbol: Option<String>,
    token_b_name: Option<String>,
    token_b_symbol: Option<String>,
    decimals: u8,
) -> Result<Contracts> {
    let pair_contract = setup_pair_contract(connection).await?;
    let factory_contract = setup_factory_contract(connection, sudo_ink_account_id).await?;
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
        pair_contract,
        token_a,
        token_b,
        router_contract,
        wnative_contract,
    })
}

pub struct TestFixture {
    pub connection: SignedConnection,
    pub sudo: KeyPair,
    pub non_sudo: KeyPair,
    pub contracts: Contracts,
}

pub async fn setup_test() -> Result<TestFixture> {
    let (sudo, non_sudo) = setup_keypairs(SUDO_SEED, NON_SUDO_SEED);
    let connection = SignedConnection::new(DEFAULT_NODE_ADDRESS, sudo.clone()).await;

    let sudo_account_id = sudo.account_id();
    let non_sudo_account_id = non_sudo.account_id();

    connection
        .transfer(
            non_sudo_account_id.clone(),
            INITIAL_TRANSFER,
            TxStatus::InBlock,
        )
        .await?;

    let sudo_ink_account_id = inkify_account_id(sudo_account_id);

    let contracts = setup_contracts(
        &connection,
        sudo_ink_account_id,
        PSP22_TOTAL_SUPPLY,
        Some(TOKEN_A_NAME.to_string()),
        Some(TOKEN_A_SYMBOL.to_string()),
        Some(TOKEN_B_NAME.to_string()),
        Some(TOKEN_B_SYMBOL.to_string()),
        DECIMALS,
    )
    .await?;
    Ok(TestFixture {
        connection,
        sudo,
        non_sudo,
        contracts,
    })
}
