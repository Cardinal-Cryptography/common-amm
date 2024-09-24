use crate::*;

use anyhow::Result;
use drink::{runtime::MinimalRuntime, session::Session, AccountId32};
use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ContractResult, InkLangError, ToAccountId};

pub const ICE: &str = "ICE";
pub const WOOD: &str = "WOOD";

pub const BOB: drink::AccountId32 = AccountId32::new([1u8; 32]);
pub const CHARLIE: drink::AccountId32 = AccountId32::new([3u8; 32]);
pub const DAVE: drink::AccountId32 = AccountId32::new([4u8; 32]);
pub const EVA: drink::AccountId32 = AccountId32::new([5u8; 32]);

pub const TOKEN: u128 = 10u128.pow(18);

pub const FEE_RECEIVER: AccountId32 = AccountId32::new([42u8; 32]);

pub fn fee_receiver() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&FEE_RECEIVER).clone().into()
}

pub fn bob() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&BOB).clone().into()
}

pub fn charlie() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&CHARLIE).clone().into()
}

pub fn dave() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&DAVE).clone().into()
}

pub fn eva() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&EVA).clone().into()
}

pub fn seed_account(session: &mut Session<MinimalRuntime>, account: AccountId32) {
    session
        .sandbox()
        .mint_into(account, 1_000_000 * TOKEN)
        .unwrap();
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
        .upload_code(stable_pool_contract::upload())
        .expect("Upload stable_pool_contract code");
    session
        .upload_code(router_contract::upload())
        .expect("Upload router_contract code");
    session
        .upload_code(router_v2_contract::upload())
        .expect("Upload router_v2_contract code");
    session
        .upload_code(wrapped_azero::upload())
        .expect("Upload wrapped_azero code");
    session
        .upload_code(mock_rate_provider_contract::upload())
        .expect("Upload mock_rate_provider_contract code");
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

    pub fn get_cached_pair(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        token0: AccountId,
        token1: AccountId,
    ) -> AccountId {
        session
            .query(router_contract::Instance::from(router).read_cache(token0, token1))
            .unwrap()
            .result
            .unwrap()
            .unwrap()
            .0
    }
}

pub mod router_v2 {
    use super::*;
    use drink::frame_support::dispatch::PaysFee;
    use router_v2_contract::RouterV2 as _;
    use router_v2_contract::{Pool, RouterV2Error, Step};

    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        factory: AccountId,
        wazero: AccountId,
    ) -> router_v2_contract::Instance {
        let instance = router_v2_contract::Instance::new(factory, wazero);

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    pub fn add_pair_liquidity(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        pair: Option<AccountId>,
        token_0: AccountId,
        token_1: AccountId,
        desired_amount_0: u128,
        desired_amount_1: u128,
        min_amount_0: u128,
        min_amount_1: u128,
        to: AccountId,
        caller: drink::AccountId32,
    ) -> Result<(u128, u128, u128), RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);

        session
            .execute(
                router_v2_contract::Instance::from(router).add_pair_liquidity(
                    pair,
                    token_0,
                    token_1,
                    desired_amount_0,
                    desired_amount_1,
                    min_amount_0,
                    min_amount_1,
                    to,
                    deadline,
                ),
            )
            .unwrap()
            .result
            .unwrap()
        // .unwrap()
    }

    pub fn add_pair_liquidity_native(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        pair: Option<AccountId>,
        token: AccountId,
        amount_token_desired: u128,
        amount_token_min: u128,
        amount_native_min: u128,
        to: AccountId,
        native_amount: u128,
        caller: drink::AccountId32,
    ) -> Result<(u128, u128, u128), RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);

        session
            .execute(
                router_v2_contract::Instance::from(router)
                    .add_pair_liquidity_native(
                        pair,
                        token,
                        amount_token_desired,
                        amount_token_min,
                        amount_native_min,
                        to,
                        deadline,
                    )
                    .with_value(native_amount),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn remove_pair_liquidity(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        pair: AccountId,
        first_token: AccountId,
        second_token: AccountId,
        liquidity: u128,
        min_token0: u128,
        min_token1: u128,
        to: AccountId,
        caller: drink::AccountId32,
    ) -> Result<(u128, u128), RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);

        session
            .execute(
                router_v2_contract::Instance::from(router).remove_pair_liquidity(
                    pair,
                    first_token,
                    second_token,
                    liquidity,
                    min_token0,
                    min_token1,
                    to,
                    deadline,
                ),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn add_stable_swap_liquidity(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        pool: AccountId,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
        native: bool,
        native_amount: u128,
        caller: drink::AccountId32,
    ) -> Result<(u128, u128), RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);

        session
            .execute(
                router_v2_contract::Instance::from(router)
                    .add_stable_pool_liquidity(
                        pool,
                        min_share_amount,
                        amounts,
                        to,
                        deadline,
                        native,
                    )
                    .with_value(native_amount),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn get_cached_pool(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        pool_id: AccountId,
    ) -> Option<Pool> {
        session
            .query(router_v2_contract::Instance::from(router).read_cached_pool(pool_id))
            .unwrap()
            .result
            .unwrap()
    }

    pub fn swap_exact_tokens_for_tokens(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<Step>,
        token_out: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        caller: drink::AccountId32,
    ) -> Result<Vec<u128>, RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);
        session
            .execute(
                router_v2_contract::Instance::from(router).swap_exact_tokens_for_tokens(
                    amount_in,
                    amount_out_min,
                    path,
                    token_out,
                    to,
                    deadline,
                ),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn swap_tokens_for_exact_tokens(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<Step>,
        token_out: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        caller: drink::AccountId32,
    ) -> Result<Vec<u128>, RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);
        session
            .execute(
                router_v2_contract::Instance::from(router).swap_tokens_for_exact_tokens(
                    amount_out,
                    amount_in_max,
                    path,
                    token_out,
                    to,
                    deadline,
                ),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn swap_exact_native_for_tokens(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        native_amount: u128,
        amount_out_min: u128,
        path: Vec<Step>,
        token_out: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        caller: drink::AccountId32,
    ) -> Result<Vec<u128>, RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);
        session
            .execute(
                router_v2_contract::Instance::from(router)
                    .swap_exact_native_for_tokens(amount_out_min, path, token_out, to, deadline)
                    .with_value(native_amount),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn swap_exact_tokens_for_native(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<Step>,
        to: ink_primitives::AccountId,
        caller: drink::AccountId32,
    ) -> Result<Vec<u128>, RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);
        session
            .execute(
                router_v2_contract::Instance::from(router).swap_exact_tokens_for_native(
                    amount_in,
                    amount_out_min,
                    path,
                    to,
                    deadline,
                ),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn swap_tokens_for_exact_native(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<Step>,
        to: ink_primitives::AccountId,
        caller: drink::AccountId32,
    ) -> Result<Vec<u128>, RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);
        session
            .execute(
                router_v2_contract::Instance::from(router).swap_tokens_for_exact_native(
                    amount_out,
                    amount_in_max,
                    path,
                    to,
                    deadline,
                ),
            )
            .unwrap()
            .result
            .unwrap()
    }

    pub fn swap_native_for_exact_tokens(
        session: &mut Session<MinimalRuntime>,
        router: AccountId,
        native_amount: u128,
        amount_out: u128,
        path: Vec<Step>,
        token_out: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        caller: drink::AccountId32,
    ) -> Result<Vec<u128>, RouterV2Error> {
        let now = get_timestamp(session);
        let deadline = now + 10;
        let _ = session.set_actor(caller);
        session
            .execute(
                router_v2_contract::Instance::from(router)
                    .swap_native_for_exact_tokens(amount_out, path, token_out, to, deadline)
                    .with_value(native_amount),
            )
            .unwrap()
            .result
            .unwrap()
    }
}

pub mod psp22_utils {
    use super::*;
    use psp22::{Instance as PSP22, PSP22Metadata as _, PSP22 as _};

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
            1_000_000_000u128 * TOKEN,
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

    pub fn setup_with_amounts(
        session: &mut Session<MinimalRuntime>,
        name: String,
        decimals: u8,
        init_supply: u128,
        caller: drink::AccountId32,
    ) -> psp22::Instance {
        let _code_hash = session.upload_code(psp22::upload()).unwrap();

        let _ = session.set_actor(caller);

        let instance = PSP22::new(init_supply, Some(name.clone()), Some(name), decimals);

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

    /// Increases allowance of given token to given spender by given amount.
    pub fn transfer(
        session: &mut Session<MinimalRuntime>,
        token: AccountId,
        to: AccountId,
        amount: u128,
        caller: drink::AccountId32,
    ) -> Result<(), psp22::PSP22Error> {
        let _ = session.set_actor(caller);

        handle_ink_error(
            session
                .execute(PSP22::transfer(&token.into(), to, amount, [].to_vec()))
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

    pub fn token_decimals(session: &mut Session<MinimalRuntime>, token: AccountId) -> u8 {
        handle_ink_error(session.query(PSP22::token_decimals(&token.into())).unwrap())
    }
}

pub mod stable_swap {
    use super::*;
    use stable_pool_contract::{StablePool as _, StablePoolError};

    pub fn setup(
        session: &mut Session<MinimalRuntime>,
        tokens: Vec<AccountId>,
        tokens_decimals: Vec<u8>,
        init_amp_coef: u128,
        caller: drink::AccountId32,
        trade_fee: u32,
        protocol_fee: u32,
        fee_receiver: Option<AccountId>,
    ) -> stable_pool_contract::Instance {
        let _ = session.set_actor(caller.clone());
        let instance = stable_pool_contract::Instance::new_stable(
            tokens,
            tokens_decimals,
            init_amp_coef,
            caller.to_account_id(),
            trade_fee,
            protocol_fee,
            fee_receiver,
        );

        session
            .instantiate(instance)
            .unwrap()
            .result
            .to_account_id()
            .into()
    }

    pub fn add_liquidity(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError> {
        _ = session.set_actor(caller);
        handle_ink_error(
            session
                .execute(
                    stable_pool_contract::Instance::from(stable_pool).add_liquidity(
                        min_share_amount,
                        amounts,
                        to,
                    ),
                )
                .unwrap(),
        )
    }

    pub fn remove_liquidity_by_amounts(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        max_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError> {
        _ = session.set_actor(caller);
        handle_ink_error(
            session
                .execute(
                    stable_pool_contract::Instance::from(stable_pool).remove_liquidity_by_amounts(
                        max_share_amount,
                        amounts,
                        to,
                    ),
                )
                .unwrap(),
        )
    }

    pub fn remove_liquidity_by_shares(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        shares_amount: u128,
        min_amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<Vec<u128>, StablePoolError> {
        _ = session.set_actor(caller);
        handle_ink_error(
            session
                .execute(
                    stable_pool_contract::Instance::from(stable_pool).remove_liquidity_by_shares(
                        shares_amount,
                        min_amounts,
                        to,
                    ),
                )
                .unwrap(),
        )
    }

    pub fn swap_exact_in(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError> {
        _ = session.set_actor(caller);
        handle_ink_error(
            session
                .execute(
                    stable_pool_contract::Instance::from(stable_pool).swap_exact_in(
                        token_in,
                        token_out,
                        token_in_amount,
                        min_token_out_amount,
                        to,
                    ),
                )
                .unwrap(),
        )
    }

    pub fn swap_exact_out(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
        max_token_in_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError> {
        _ = session.set_actor(caller);
        handle_ink_error(
            session
                .execute(
                    stable_pool_contract::Instance::from(stable_pool).swap_exact_out(
                        token_in,
                        token_out,
                        token_out_amount,
                        max_token_in_amount,
                        to,
                    ),
                )
                .unwrap(),
        )
    }

    pub fn swap_received(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        caller: drink::AccountId32,
        token_in: AccountId,
        token_out: AccountId,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError> {
        _ = session.set_actor(caller);
        handle_ink_error(
            session
                .execute(
                    stable_pool_contract::Instance::from(stable_pool).swap_received(
                        token_in,
                        token_out,
                        min_token_out_amount,
                        to,
                    ),
                )
                .unwrap(),
        )
    }

    pub fn reserves(session: &mut Session<MinimalRuntime>, stable_pool: AccountId) -> Vec<u128> {
        handle_ink_error(
            session
                .query(stable_pool_contract::Instance::from(stable_pool).reserves())
                .unwrap(),
        )
    }

    pub fn amp_coef(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
    ) -> Result<u128, StablePoolError> {
        handle_ink_error(
            session
                .query(stable_pool_contract::Instance::from(stable_pool).amp_coef())
                .unwrap(),
        )
    }

    pub fn fees(session: &mut Session<MinimalRuntime>, stable_pool: AccountId) -> (u32, u32) {
        handle_ink_error(
            session
                .query(stable_pool_contract::Instance::from(stable_pool).fees())
                .unwrap(),
        )
    }

    pub fn token_rates(session: &mut Session<MinimalRuntime>, stable_pool: AccountId) -> Vec<u128> {
        handle_ink_error(
            session
                .query(stable_pool_contract::Instance::from(stable_pool).token_rates())
                .unwrap(),
        )
    }

    pub fn tokens(session: &mut Session<MinimalRuntime>, stable_pool: AccountId) -> Vec<AccountId> {
        handle_ink_error(
            session
                .query(stable_pool_contract::Instance::from(stable_pool).tokens())
                .unwrap(),
        )
    }

    pub fn get_amounts_for_liquidity_burn(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        liquidity: u128,
    ) -> Result<Vec<u128>, StablePoolError> {
        handle_ink_error(
            session
                .query(
                    stable_pool_contract::Instance::from(stable_pool)
                        .get_amounts_for_liquidity_burn(liquidity),
                )
                .unwrap(),
        )
    }

    pub fn get_amounts_for_liquidity_mint(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        liquidity: u128,
    ) -> Result<Vec<u128>, StablePoolError> {
        handle_ink_error(
            session
                .query(
                    stable_pool_contract::Instance::from(stable_pool)
                        .get_amounts_for_liquidity_mint(liquidity),
                )
                .unwrap(),
        )
    }

    pub fn get_burn_liquidity_for_amounts(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError> {
        handle_ink_error(
            session
                .query(
                    stable_pool_contract::Instance::from(stable_pool)
                        .get_burn_liquidity_for_amounts(amounts),
                )
                .unwrap(),
        )
    }

    pub fn get_mint_liquidity_for_amounts(
        session: &mut Session<MinimalRuntime>,
        stable_pool: AccountId,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError> {
        handle_ink_error(
            session
                .query(
                    stable_pool_contract::Instance::from(stable_pool)
                        .get_mint_liquidity_for_amounts(amounts),
                )
                .unwrap(),
        )
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

pub fn native_balance_of(session: &mut Session<MinimalRuntime>, account_id: AccountId) -> u128 {
    session.sandbox().free_balance(&AccountId32::from(
        AsRef::<[u8; 32]>::as_ref(&account_id).clone(),
    ))
}
