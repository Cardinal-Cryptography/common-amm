use crate::*;

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{util::ToAccountId, Connection};

pub const INITIAL_TRANSFER: u128 = 1_000_000_000_000;

pub const PSP22_TOTAL_SUPPLY: u128 = 10_000_000;
pub const PSP22_DECIMALS: u8 = 18;

pub const ICE: &str = "ICE";
pub const WOOD: &str = "WOOD";
pub const SAND: &str = "SAND";

pub const ALICE: drink::AccountId32 = AccountId32::new([2u8; 32]);
pub const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);
pub const CHARLIE: drink::AccountId32 = AccountId32::new([3u8; 32]);

pub fn alice() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&ALICE).clone().into()
}

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

pub fn charlie() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&CHARLIE).clone().into()
}

pub fn upload_all(session: &mut Session<MinimalRuntime>) {
    session
        .upload_code(psp22_contract::upload())
        .expect("Upload psp22 code");
    session
        .upload_code(factory_contract::upload())
        .expect("Upload factory_contract code");
    session
        .upload_code(pair_contract::upload())
        .expect("Upload pair_contract code");
    session
        .upload_code(router_contract::upload())
        .expect("Upload router_contract code");
    session
        .upload_code(wrapped_azero::upload())
        .expect("Upload wrapped_azero code");
}

pub mod wazero {
    use super::*;

    pub fn setup(session: &mut Session<MinimalRuntime>) -> wrapped_azero::Instance {
        let instance = wrapped_azero::Instance::new();

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }
}

pub mod factory {
    use super::*;

    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        fee_to_setter: AccountId,
    ) -> factory_contract::Instance {
        let instance =
            factory_contract::Instance::new(fee_to_setter, pair_contract::CODE_HASH.into());

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }
}

pub mod router {
    use super::*;

    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        factory: AccountId,
        wazero: AccountId,
    ) -> router_contract::Instance {
        let instance = router_contract::Instance::new(factory, wazero);

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }
}

pub mod psp22 {
    use super::*;
    use psp22_contract::{Instance as PSP22, PSP22 as _};

    /// Uploads and creates a PSP22 instance with 1B*10^18 issuance and given names.
    /// Returns its AccountId casted to PSP22 interface.
    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        name: String,
        caller: drink::AccountId32,
    ) -> psp22_contract::Instance {
        let _code_hash = session.upload_code(psp22_contract::upload()).unwrap();

        let _ = session.set_actor(caller);

        let instance = PSP22::new(
            1_000_000_000u128 * 10u128.pow(18),
            Some(name.clone()),
            Some(name),
            18,
        );

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

    /// Returns balance of given token for given account.
    /// Fails if anything other than success.
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

    pub fn total_supply(session: &mut Session<MinimalRuntime>, token: AccountId) -> u128 {
        session
            .query(PSP22::total_supply(&token.into()))
            .unwrap()
            .result
            .unwrap()
    }
}

pub fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
    session.sandbox().get_timestamp()
}

pub fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
    session.sandbox().set_timestamp(timestamp);
}
