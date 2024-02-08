mod psp22_contract;

pub use psp22_contract::{event, upload, Instance as PSP22, PSP22Error, PSP22 as PSP22T};

use crate::utils::handle_ink_error;

use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

/// Uploads and creates a PSP22 instance with u128::MAX issuance and given names.
/// Returns its AccountId casted to PSP22 interface.
pub fn setup(
    session: &mut Session<MinimalRuntime>,
    name: String,
    symbol: String,
    caller: AccountId32,
) -> PSP22 {
    let _code_hash = session.upload_code(upload()).unwrap();

    let _ = session.set_actor(caller);

    let instance = PSP22::new(u128::MAX, Some(name), Some(symbol), 6);

    session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into()
}

pub fn transfer(
    session: &mut Session<MinimalRuntime>,
    token: AccountId,
    to: AccountId,
    amount: u128,
    caller: AccountId32,
) -> Result<(), PSP22Error> {
    let _ = session.set_actor(caller);

    handle_ink_error(
        session
            .execute(PSP22::transfer(&token.into(), to, amount, vec![]))
            .unwrap(),
    )
}

/// Increases allowance of given token to given spender by given amount.
pub fn increase_allowance(
    session: &mut Session<MinimalRuntime>,
    token: AccountId,
    spender: AccountId,
    amount: u128,
    caller: AccountId32,
) {
    let _ = session.set_actor(caller);

    handle_ink_error(
        session
            .execute(PSP22::increase_allowance(&token.into(), spender, amount))
            .unwrap(),
    )
    .expect("Increase allowance failed");
}

/// Returns balance of given token for given account.
/// Fails if anything other than success.
pub fn balance_of(
    session: &mut Session<MinimalRuntime>,
    token: AccountId,
    account: AccountId,
) -> u128 {
    handle_ink_error(
        session
            .query(PSP22::balance_of(&token.into(), account))
            .unwrap(),
    )
}
