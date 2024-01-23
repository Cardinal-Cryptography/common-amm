use drink::{runtime::MinimalRuntime, session::Session, AccountId32};

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

pub fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
    session.sandbox().get_timestamp()
}

pub fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
    session.sandbox().set_timestamp(timestamp);
}
