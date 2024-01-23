mod farm_contract;

pub use farm_contract::{
    event, upload, Farm as FarmT, FarmDetails, FarmError, Instance as Farm, PSP22Error,
};

use crate::utils::handle_ink_error;

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection};

/// Uploads and creates a Farm instance with given pool_id and rewards.
/// Returns its AccountId casted to Farm interface.
pub fn setup(
    session: &mut Session<MinimalRuntime>,
    pool_id: AccountId,
    rewards: Vec<AccountId>,
    caller: drink::AccountId32,
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
    caller: drink::AccountId32,
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
    caller: drink::AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.deposit(amount)).unwrap())
}

pub fn owner_withdraw(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    token: AccountId,
    caller: drink::AccountId32,
) -> Result<(), FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.owner_withdraw_token(token)).unwrap())
}

pub fn query_unclaimed_rewards(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    reward_ids: Vec<u8>,
    caller: drink::AccountId32,
) -> Result<Vec<u128>, FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.query(farm.claim_rewards(reward_ids)).unwrap())
}

pub fn claim_rewards(
    session: &mut Session<MinimalRuntime>,
    farm: &Farm,
    reward_ids: Vec<u8>,
    caller: drink::AccountId32,
) -> Result<Vec<u128>, FarmError> {
    let _ = session.set_actor(caller);

    handle_ink_error(session.execute(farm.claim_rewards(reward_ids)).unwrap())
}
