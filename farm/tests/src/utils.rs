use crate::*;

use farm::{Farm as _, Instance as Farm};
use psp22::{Instance as PSP22, PSP22 as _};

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection};

pub const ICE: &str = "ICE";
pub const WOOD: &str = "WOOD";
pub const SAND: &str = "SAND";

pub const ALICE: drink::AccountId32 = AccountId32::new([2u8; 32]);
pub const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);

pub fn alice() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&ALICE).clone().into()
}

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

/// Uploads and creates a PSP22 instance with 1B*10^18 issuance and given names.
/// Returns its AccountId casted to PSP22 interface.
pub fn setup_psp22(
    session: &mut Session<MinimalRuntime>,
    name: String,
    symbol: String,
    caller: drink::AccountId32,
) -> PSP22 {
    let _code_hash = session.upload_code(psp22::upload()).unwrap();

    let _ = session.set_actor(caller);

    let instance = PSP22::new(
        1_000_000_000u128 * 10u128.pow(18),
        Some(name),
        Some(symbol),
        18,
    );

    session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into()
}

/// Uploads and creates a Farm instance with given pool_id and rewards.
/// Returns its AccountId casted to Farm interface.
pub fn setup_farm(
    session: &mut Session<MinimalRuntime>,
    pool_id: AccountId,
    rewards: Vec<AccountId>,
    caller: drink::AccountId32,
) -> Farm {
    let _code_hash = session.upload_code(farm::upload()).unwrap();

    let _ = session.set_actor(caller);

    let instance = Farm::new(pool_id, rewards);

    session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into()
}

/// Increases allowance of given token to given spender by given amount.
pub fn increase_allowance(
    session: &mut Session<MinimalRuntime>,
    token: AccountId,
    spender: AccountId,
    amount: u128,
    caller: drink::AccountId32,
) -> Result<()> {
    let _ = session.set_actor(caller);

    session
        .execute(PSP22::increase_allowance(&token.into(), spender, amount))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    Ok(())
}

pub fn balance_of(
    session: &mut Session<MinimalRuntime>,
    token: AccountId,
    account: AccountId,
) -> u128 {
    session
        .query(PSP22::balance_of(&token.into(), account))
        .unwrap()
        .result
        .unwrap()
}

pub fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
    session.sandbox().get_timestamp()
}

pub fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
    session.sandbox().set_timestamp(timestamp);
}

/// Returns farm details.
pub fn get_farm_details(session: &mut Session<MinimalRuntime>, farm: &Farm) -> farm::FarmDetails {
    session
        .query(farm.view_farm_details())
        .unwrap()
        .result
        .unwrap()
}

/// Starts a farm with given start and end timestamps and rewards.
pub fn setup_farm_start(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    start: u64,
    end: u64,
    rewards: Vec<u128>,
    caller: drink::AccountId32,
) -> Result<()> {
    let _ = session.set_actor(caller);

    session
        .execute(farm.owner_start_new_farm(start, end, rewards))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    Ok(())
}
