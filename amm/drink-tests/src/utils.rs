use crate::*;

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ContractResult, InkLangError, ToAccountId};

pub const ICE: &str = "ICE";
pub const WOOD: &str = "WOOD";

pub const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);
pub const CHARLIE: drink::AccountId32 = AccountId32::new([3u8; 32]);

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

pub fn charlie() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&CHARLIE).clone().into()
}

pub fn upload_all(session: &mut Session<MinimalRuntime>) {
    session
        .upload_code(psp22::upload())
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
    use factory_contract::Factory as _;

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

    pub fn get_pair(
        session: &mut Session<MinimalRuntime>,
        factory: AccountId,
        token0: AccountId,
        token1: AccountId,
    ) -> pair_contract::Instance {
        session
            .query(factory_contract::Instance::from(factory).get_pair(token0, token1))
            .unwrap()
            .result
            .unwrap()
            .unwrap()
            .to_account_id()
            .into()
    }
}

pub mod router {
    use super::*;
    use router_contract::Router as _;

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

    pub fn add_liquidity(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        first_token: AccountId,
        second_token: AccountId,
        desired_token_amount: u128,
        min_token_amount: u128,
        caller: AccountId,
    ) -> (u128, u128, u128) {
        let now = get_timestamp(session);
        let deadline = now + 10;

        session
            .execute(router_contract::Instance::from(router).add_liquidity(
                first_token,
                second_token,
                desired_token_amount,
                desired_token_amount,
                min_token_amount,
                min_token_amount,
                caller,
                deadline,
            ))
            .unwrap()
            .result
            .unwrap()
            .unwrap()
    }

    pub fn remove_liquidity(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        first_token: AccountId,
        second_token: AccountId,
        liquidity: u128,
        min_token0: u128,
        min_token1: u128,
        caller: AccountId,
    ) -> (u128, u128) {
        let now = get_timestamp(session);
        let deadline = now + 10;

        session
            .execute(router_contract::Instance::from(router).remove_liquidity(
                first_token,
                second_token,
                liquidity,
                min_token0,
                min_token1,
                caller,
                deadline,
            ))
            .unwrap()
            .result
            .unwrap()
            .unwrap()
    }
}

pub mod psp22_utils {
    use super::*;
    use psp22::{Instance as PSP22, PSP22 as _};

    /// Uploads and creates a PSP22 instance with 1B*10^18 issuance and given names.
    /// Returns its AccountId casted to PSP22 interface.
    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        name: String,
        caller: drink::AccountId32,
    ) -> psp22::Instance {
        let _code_hash = session.upload_code(psp22::upload()).unwrap();

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
    ) -> Result<(), psp22::PSP22Error> {
        let _ = session.set_actor(caller);

        handle_ink_error(
            session
                .execute(PSP22::increase_allowance(&token.into(), spender, amount))
                .unwrap(),
        )
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

    pub fn total_supply(session: &mut Session<MinimalRuntime>, token: AccountId) -> u128 {
        handle_ink_error(session.query(PSP22::total_supply(&token.into())).unwrap())
    }
}

pub fn get_timestamp(session: &mut Session<MinimalRuntime>) -> u64 {
    session.sandbox().get_timestamp()
}

pub fn set_timestamp(session: &mut Session<MinimalRuntime>, timestamp: u64) {
    session.sandbox().set_timestamp(timestamp);
}

pub fn handle_ink_error<R>(res: ContractResult<Result<R, InkLangError>>) -> R {
    match res.result {
        Err(ink_lang_err) => panic!("InkLangError: {:?}", ink_lang_err),
        Ok(r) => r,
    }
}
