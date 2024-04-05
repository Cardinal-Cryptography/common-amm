mod farm_contract;

pub use farm_contract::{
    event, upload, Farm as FarmT, FarmDetails, FarmError, Instance as Farm, PSP22Error,
};

use crate::utils::handle_ink_error;

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

/// Uploads and creates a Farm instance with given pool_id and rewards.
/// Returns its AccountId casted to Farm interface.
pub fn setup(
    session: &mut Session<MinimalRuntime>,
    pool_id: AccountId,
    rewards: Vec<AccountId>,
    caller: AccountId32,
) -> Farm {
    let _code_hash = session.upload_code(upload()).unwrap();

    let _ = session.set_actor(caller);

    let instance = Farm::new(pool_id, rewards);

    session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into()
}

/// Returns farm details.
/// Fails if anything other than success.
pub fn get_farm_details(session: &mut Session<MinimalRuntime>, farm: &Farm) -> FarmDetails {
    handle_ink_error(session.query(farm.view_farm_details()).unwrap())
}

/// Starts a farm with given start and end timestamps and rewards.
pub fn start(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    start: u64,
    end: u64,
    rewards: Vec<u128>,
    caller: AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(
        session
            .execute(farm.owner_start_new_farm(start, end, rewards))
            .unwrap(),
    )
}

pub fn deposit_to_farm(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    amount: u128,
    caller: AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.deposit(amount)).unwrap())
}

pub fn withdraw_from_farm(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    amount: u128,
    caller: AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.withdraw(amount)).unwrap())
}

pub fn owner_withdraw(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    token: AccountId,
    caller: AccountId32,
) -> Result<u128, FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.owner_withdraw_token(token)).unwrap())
}

pub fn query_unclaimed_rewards(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    reward_ids: Vec<u8>,
    caller: AccountId32,
) -> Result<Vec<u128>, FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.query(farm.claim_rewards(reward_ids)).unwrap())
}

pub fn claim_rewards(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    reward_ids: Vec<u8>,
    caller: AccountId32,
) -> Result<Vec<u128>, FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.claim_rewards(reward_ids)).unwrap())
}

pub fn owner_stop_farm(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    caller: AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.owner_stop_farm()).unwrap())
}

pub fn owner_add_reward_token(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    caller: AccountId32,
    token: AccountId,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.owner_add_reward_token(token)).unwrap())
}

pub fn join_farm(
    session: &mut Session<MinimalRuntime>,
    pool: AccountId,
    farm: &Farm,
    deposit_amount: u128,
    caller: AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller.clone());

    // deposit ICE (pool LP token) into farm
    crate::psp22::increase_allowance(
        session,
        pool,
        (*farm).into(),
        deposit_amount,
        caller.clone(),
    );
    deposit_to_farm(session, &farm, deposit_amount, caller)
}
