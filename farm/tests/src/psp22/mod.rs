mod psp22_contract;

pub use psp22_contract::{upload, Instance as PSP22, PSP22Error, PSP22 as PSP22T};

use crate::utils::handle_ink_error;

use drink::{runtime::MinimalRuntime, session::Session};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection};

/// Uploads and creates a PSP22 instance with 1B*10^18 issuance and given names.
/// Returns its AccountId casted to PSP22 interface.
pub fn setup(
    session: &mut Session<MinimalRuntime>,
    name: String,
    symbol: String,
    caller: drink::AccountId32,
) -> PSP22 {
    let _code_hash = session.upload_code(upload()).unwrap();

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

pub fn transfer(
    session: &mut Session<MinimalRuntime>,
    token: AccountId,
    to: AccountId,
    amount: u128,
    caller: drink::AccountId32,
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
    caller: drink::AccountId32,
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
