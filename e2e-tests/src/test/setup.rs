use std::{
    env,
    future::Future,
    str::FromStr,
};

use anyhow::{
    anyhow,
    Result,
};
use rand::RngCore;
use tokio::sync::OnceCell;

use aleph_client::{
    pallets::{
        balances::BalanceUserApi,
        system::SystemApi,
    },
    Balance,
    SignedConnection,
    TxStatus,
};

pub use amm_helpers::constants::ZERO_ADDRESS;

pub const DEFAULT_NODE_ADDRESS: &str = "ws://127.0.0.1:9944";
pub const WEALTHY_SEED: &str = "//Alice";
pub const REGULAR_SEED: &str = "//0";

pub const INITIAL_TRANSFER: Balance = 1_000_000_000_000;

pub const PSP22_TOTAL_SUPPLY: Balance = 10_000_000;
pub const PSP22_DECIMALS: u8 = 18;

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

pub fn random_salt() -> Vec<u8> {
    let mut salt = vec![0; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub async fn try_upload_contract_code<F, Fut>(cell: &OnceCell<Result<()>>, upload: F) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<()>>,
{
    if let Err(e) = cell.get_or_init(upload).await {
        return Err(anyhow!("Failed to upload contract code: {:?}", e))
    }
    Ok(())
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
