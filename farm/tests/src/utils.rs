use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_wrapper_types::{ContractResult, InkLangError};

pub const ICE: &str = "ICE";
pub const WOOD: &str = "WOOD";
pub const SAND: &str = "SAND";

pub const ALICE: AccountId32 = AccountId32::new([2u8; 32]);
pub const BOB: AccountId32 = AccountId32::new([1u8; 32]);

pub fn alice() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&ALICE).clone().into()
}

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

pub fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
    session.sandbox().get_timestamp()
}

pub fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
    session.sandbox().set_timestamp(timestamp);
}

pub fn inc_timestamp(session: &mut Session<MinimalRuntime>) {
    let timestamp = get_timestamp(session);
    set_timestamp(session, timestamp + 1);
}

pub fn handle_ink_error<R>(res: ContractResult<Result<R, InkLangError>>) -> R {
    match res.result {
        Err(ink_lang_err) => panic!("InkLangError: {:?}", ink_lang_err),
        Ok(r) => r,
    }
}

pub fn seed_account(session: &mut Session<MinimalRuntime>, account: AccountId32) {
    session
        .sandbox()
        .mint_into(account, 1_000_000_000u128)
        .unwrap();
}
