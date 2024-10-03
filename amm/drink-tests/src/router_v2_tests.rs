use std::u128;

use crate::stable_swap_tests::*;
use crate::utils::*;
use crate::{factory_contract, pair_contract, router_v2_contract, wrapped_azero};

use drink::{runtime::MinimalRuntime, Weight};
use ink_primitives::AccountId;
use ink_wrapper_types::ToAccountId;
use pair_contract::Pair as _;
use router_v2_contract::{Pair, Pool, StablePool, Step};

use drink::{self, session::Session};
use ink_wrapper_types::Connection;

const A: u128 = 10_000;
const TRADE_FEE: u32 = 2_500_000;
const PROTOCOL_FEE: u32 = 200_000_000;

const U100K: u128 = 100_000;
const U1M: u128 = 1_000_000;

fn setup_router(
    session: &mut Session<MinimalRuntime>,
) -> (
    router_v2_contract::Instance,
    factory_contract::Instance,
    wrapped_azero::Instance,
    ink_primitives::AccountId,
) {
    let fee_to_setter = bob();
    let factory = factory::setup(session, fee_to_setter);
    let wazero = wazero::setup(session);
    let router = router_v2::setup(session, factory.into(), wazero.into());
    (router, factory, wazero, fee_to_setter)
}

/// Tests that a StablePool is cached in the Router
/// with the first swap.
#[drink::test]
fn test_cache_stable_pool(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, _, _, _) = setup_router(&mut session);

    let initial_reserves = vec![U100K * ONE_USDT, U100K * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * U1M)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply,
        A,
        TRADE_FEE,
        PROTOCOL_FEE,
        BOB,
        vec![],
    );

    let (usdt, usdc) = (tokens[0], tokens[1]);

    // we need to add some liquidity to the pool before the swap
    stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves,
        bob(),
    )
    .expect("Should successfully add liquidity");

    // ensure that the pool is not cached before the swap
    let res = router_v2::get_cached_pool(&mut session, router.into(), usdt_usdc_pool);
    assert_eq!(res, None, "StablePool should not be in the cache");

    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    router_v2::swap_exact_tokens_for_tokens(
        &mut session,
        router.into(),
        ONE_USDT,
        0,
        vec![Step {
            token_in: usdt,
            pool_id: usdt_usdc_pool.into(),
        }],
        usdc,
        bob(),
        BOB,
    )
    .expect("Should swap");

    let res = router_v2::get_cached_pool(&mut session, router.into(), usdt_usdc_pool)
        .expect("Should return cached StablePool");
    assert_eq!(
        res,
        Pool::StablePool(StablePool {
            id: usdt_usdc_pool,
            tokens
        }),
        "StablePool cache mismatch"
    );
}

/// Tests that a StablePool is cached in the Router
/// with the first liquidity deposit.
#[drink::test]
fn test_cache_stable_pool_with_add_liquidity(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, _, _, _) = setup_router(&mut session);

    let initial_reserves = vec![U100K * ONE_USDT, U100K * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * U1M)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply,
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    let (usdt, usdc) = (tokens[0], tokens[1]);

    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdc, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    // ensure that the pool is not cached before the swap
    let res = router_v2::get_cached_pool(&mut session, router.into(), usdt_usdc_pool);
    assert_eq!(res, None, "StablePool should not be in the cache");

    router_v2::add_stable_swap_liquidity(
        &mut session,
        router.into(),
        usdt_usdc_pool,
        1,
        initial_reserves.clone(),
        bob(),
        false,
        0,
        BOB,
    )
    .expect("Should add liquidity");

    let res = router_v2::get_cached_pool(&mut session, router.into(), usdt_usdc_pool)
        .expect("Should return cached StablePool");
    assert_eq!(
        res,
        Pool::StablePool(StablePool {
            id: usdt_usdc_pool,
            tokens
        }),
        "StablePool cache mismatch"
    );
}

/// Tests that a Pair is cached in the Router
/// with the first swap.
#[drink::test]
fn test_cache_pair(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);

    // create a custom Pair w/o the Factory contract
    let custom_fee: u8 = 1;
    let ice_wood_pair: pair_contract::Instance = session
        .instantiate(pair_contract::Instance::new(
            ice.into(),
            wood.into(),
            factory.into(),
            custom_fee,
        ))
        .unwrap()
        .result
        .to_account_id()
        .into();

    // add some liquidity w/o the router
    psp22_utils::transfer(&mut session, ice.into(), ice_wood_pair.into(), TOKEN, BOB)
        .expect("Should transfer PSP22");
    psp22_utils::transfer(&mut session, wood.into(), ice_wood_pair.into(), TOKEN, BOB)
        .expect("Should transfer PSP22");

    session
        .execute(ice_wood_pair.mint(bob()))
        .unwrap()
        .result
        .unwrap()
        .expect("Should mint");

    // ensure that the pair is not cached before the swap
    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into());
    assert_eq!(res, None, "Pair should not be in the cache");

    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    router_v2::swap_exact_tokens_for_tokens(
        &mut session,
        router.into(),
        ONE_USDT,
        0,
        vec![Step {
            token_in: ice.into(),
            pool_id: ice_wood_pair.into(),
        }],
        wood.into(),
        bob(),
        BOB,
    )
    .expect("Should swap");

    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into())
        .expect("Should return cached Pair");
    assert_eq!(
        res,
        Pool::Pair(Pair {
            id: ice_wood_pair.into(),
            token_0: ice.into(),
            token_1: wood.into(),
            fee: custom_fee,
        }),
        "Pair cache mismatch"
    );
}

/// Tests that a Pair is cached in the Router
/// with the first liquidity deposit.
#[drink::test]
fn test_cache_custom_pair_with_add_liqudity(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);

    // create a custom Pair w/o the Factory contract
    let custom_fee: u8 = 1;
    let ice_wood_pair: pair_contract::Instance = session
        .instantiate(pair_contract::Instance::new(
            ice.into(),
            wood.into(),
            factory.into(),
            custom_fee,
        ))
        .unwrap()
        .result
        .to_account_id()
        .into();

    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    // ensure that the pair is not cached before the swap
    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into());
    assert_eq!(res, None, "Pair should not be in the cache");

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        Some(ice_wood_pair.into()),
        ice.into(),
        wood.into(),
        U100K,
        U100K,
        U100K,
        U100K,
        bob(),
        BOB,
    )
    .expect("Should add liquidity");

    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into())
        .expect("Should return cached Pair");
    assert_eq!(
        res,
        Pool::Pair(Pair {
            id: ice_wood_pair.into(),
            token_0: ice.into(),
            token_1: wood.into(),
            fee: custom_fee,
        }),
        "Pair mismatch"
    );
}

/// Tests that a Pair is created and cached
/// with the first liquidity deposit.
#[drink::test]
fn test_create_and_cache_pair_with_add_liqudity(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);

    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        ice.into(),
        wood.into(),
        U100K,
        U100K,
        U100K,
        U100K,
        bob(),
        BOB,
    )
    .expect("Should add liquidity");

    let default_pair_fee = 3;
    let ice_wood_pair = factory::get_pair(&mut session, factory.into(), ice.into(), wood.into());
    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into())
        .expect("Should return cached Pair");
    assert_eq!(
        res,
        Pool::Pair(Pair {
            id: ice_wood_pair.into(),
            token_0: ice.into(),
            token_1: wood.into(),
            fee: default_pair_fee,
        }),
        "Pair mismatch"
    );
}

/// Tests a simple swap along [Pair -> StableSwap -> Pair] path
/// using `swap_exact_tokens_for_tokens` and
/// `swap_tokens_for_exact_tokens` methods
#[drink::test]
fn test_psp22_swap(mut session: Session) {
    upload_all(&mut session);

    // seed test accounts with some native token
    seed_account(&mut session, BOB);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    // setup stable pool
    let initial_reserves = vec![U100K * ONE_USDT, U100K * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * U1M)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        A,
        TRADE_FEE,
        PROTOCOL_FEE,
        BOB,
        vec![],
    );

    let (usdt, usdc) = (tokens[0], tokens[1]);

    stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // setup pairs
    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdc, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    let token_amount = U100K * TOKEN;
    let stable_amount = U100K * ONE_USDC;

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        ice.into(),
        usdc,
        token_amount,
        stable_amount,
        token_amount,
        stable_amount,
        bob(),
        BOB,
    )
    .expect("Should add liquidity");

    let ice_usdc_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), ice.into(), usdc);

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        wood.into(),
        usdt,
        token_amount,
        stable_amount,
        token_amount,
        stable_amount,
        bob(),
        BOB,
    )
    .expect("Should add liquidity");

    let wood_usdt_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), wood.into(), usdt);

    // increase gas limit (swaps with more than 3 tokens require more gas)
    let gas_limit = session.get_gas_limit();
    session.set_gas_limit(Weight::from_parts(
        10 * gas_limit.ref_time(),
        10 * gas_limit.proof_size(),
    ));

    let swap_amount = 100 * TOKEN;

    let first_step_output = v2_amounts::get_amount_out(
        &mut session,
        ice_usdc_pair.into(),
        ice.into(),
        usdc.into(),
        swap_amount,
    )
    .unwrap();

    let second_step_output = stable_swap::get_swap_amount_out(
        &mut session,
        usdt_usdc_pool,
        usdc.into(),
        usdt.into(),
        first_step_output,
    )
    .unwrap();

    let third_step_output = v2_amounts::get_amount_out(
        &mut session,
        wood_usdt_pair.into(),
        usdt.into(),
        wood.into(),
        second_step_output.0,
    )
    .unwrap();

    let init_bob_ice_balance = psp22_utils::balance_of(&mut session, ice.into(), bob());
    let init_bob_wood_balance = psp22_utils::balance_of(&mut session, wood.into(), bob());

    let amounts = router_v2::swap_exact_tokens_for_tokens(
        &mut session,
        router.into(),
        swap_amount,
        0,
        vec![
            Step {
                token_in: ice.into(),
                pool_id: ice_usdc_pair.into(),
            },
            Step {
                token_in: usdc,
                pool_id: usdt_usdc_pool.into(),
            },
            Step {
                token_in: usdt,
                pool_id: wood_usdt_pair.into(),
            },
        ],
        wood.into(),
        bob(),
        BOB,
    )
    .expect("Should swap");

    assert_eq!(amounts.len(), 4);
    assert_eq!(amounts[0], swap_amount);
    assert_eq!(amounts[1], first_step_output);
    assert_eq!(amounts[2], second_step_output.0);
    assert_eq!(amounts[3], third_step_output);

    let bob_ice_balance = psp22_utils::balance_of(&mut session, ice.into(), bob());
    let bob_wood_balance = psp22_utils::balance_of(&mut session, wood.into(), bob());

    assert_eq!(init_bob_ice_balance - bob_ice_balance, swap_amount);
    assert_eq!(bob_wood_balance - init_bob_wood_balance, third_step_output);

    let init_bob_ice_balance = bob_ice_balance;
    let init_bob_wood_balance = bob_wood_balance;

    let third_step_input = v2_amounts::get_amount_in(
        &mut session,
        wood_usdt_pair.into(),
        usdt.into(),
        wood.into(),
        swap_amount,
    )
    .unwrap();

    let second_step_input = stable_swap::get_swap_amount_in(
        &mut session,
        usdt_usdc_pool,
        usdc.into(),
        usdt.into(),
        third_step_input,
    )
    .unwrap();

    let first_step_input = v2_amounts::get_amount_in(
        &mut session,
        ice_usdc_pair.into(),
        ice.into(),
        usdc.into(),
        second_step_input.0,
    )
    .unwrap();

    let amounts = router_v2::swap_tokens_for_exact_tokens(
        &mut session,
        router.into(),
        swap_amount,
        u128::MAX,
        vec![
            Step {
                token_in: ice.into(),
                pool_id: ice_usdc_pair.into(),
            },
            Step {
                token_in: usdc,
                pool_id: usdt_usdc_pool.into(),
            },
            Step {
                token_in: usdt,
                pool_id: wood_usdt_pair.into(),
            },
        ],
        wood.into(),
        bob(),
        BOB,
    )
    .expect("Should swap");

    assert_eq!(amounts.len(), 4);
    assert_eq!(amounts[0], first_step_input);
    assert_eq!(amounts[1], second_step_input.0);
    assert_eq!(amounts[2], third_step_input);
    assert_eq!(amounts[3], swap_amount);

    let bob_ice_balance = psp22_utils::balance_of(&mut session, ice.into(), bob());
    let bob_wood_balance = psp22_utils::balance_of(&mut session, wood.into(), bob());

    assert_eq!(init_bob_ice_balance - bob_ice_balance, first_step_input);
    assert_eq!(bob_wood_balance - init_bob_wood_balance, swap_amount);

    for token in [usdt, usdc, ice.into(), wood.into()] {
        assert_eq!(
            psp22_utils::balance_of(&mut session, token, router.into()),
            0,
            "Router should not hold any tokens"
        );
    }
}

/// Tests a simple swap along [Pair_native -> StableSwap -> Pair] path
/// using `swap_exact_native_for_tokens` and
/// `swap_native_for_exact_tokens` methods
#[drink::test]
fn test_native_in_swap(mut session: Session) {
    upload_all(&mut session);

    // seed test accounts with some native token
    seed_account(&mut session, BOB);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, wnative, _) = setup_router(&mut session);

    // setup stable pool
    let initial_reserves = vec![U100K * ONE_USDT, U100K * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * U1M)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        A,
        TRADE_FEE,
        PROTOCOL_FEE,
        BOB,
        vec![],
    );

    let (usdt, usdc) = (tokens[0], tokens[1]);

    stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // setup pairs
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdc, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    let stable_amount = U100K * ONE_USDC;
    let native_amount = U100K * ONE_AZERO;

    router_v2::add_pair_liquidity_native(
        &mut session,
        router.into(),
        None,
        usdc,
        stable_amount,
        stable_amount,
        native_amount,
        bob(),
        native_amount,
        BOB,
    )
    .expect("Should add liquidity");

    let wnative_usdc_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), wnative.into(), usdc);

    let token_amount = U100K * TOKEN;

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        wood.into(),
        usdt,
        token_amount,
        stable_amount,
        token_amount,
        stable_amount,
        bob(),
        BOB,
    )
    .expect("Should add liquidity");

    let wood_usdt_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), wood.into(), usdt);

    // increase gas limit (swaps with more than 3 tokens require more gas)
    let gas_limit = session.get_gas_limit();
    session.set_gas_limit(Weight::from_parts(
        10 * gas_limit.ref_time(),
        10 * gas_limit.proof_size(),
    ));

    let native_amount = 100 * ONE_AZERO;

    let router_native_balance = native_balance_of(&mut session, router.into());

    router_v2::swap_exact_native_for_tokens(
        &mut session,
        router.into(),
        native_amount,
        0,
        vec![
            Step {
                token_in: wnative.into(),
                pool_id: wnative_usdc_pair.into(),
            },
            Step {
                token_in: usdc,
                pool_id: usdt_usdc_pool.into(),
            },
            Step {
                token_in: usdt,
                pool_id: wood_usdt_pair.into(),
            },
        ],
        wood.into(),
        bob(),
        BOB,
    )
    .expect("Should swap");

    let swap_amount = 100 * TOKEN;
    let native_amount = 120 * ONE_AZERO;

    router_v2::swap_native_for_exact_tokens(
        &mut session,
        router.into(),
        native_amount,
        swap_amount,
        vec![
            Step {
                token_in: wnative.into(),
                pool_id: wnative_usdc_pair.into(),
            },
            Step {
                token_in: usdc,
                pool_id: usdt_usdc_pool.into(),
            },
            Step {
                token_in: usdt,
                pool_id: wood_usdt_pair.into(),
            },
        ],
        wood.into(),
        bob(),
        BOB,
    )
    .expect("Should swap");

    for token in [usdt, usdc, wnative.into(), wood.into()] {
        assert_eq!(
            psp22_utils::balance_of(&mut session, token, router.into()),
            0,
            "Router should not hold any tokens"
        );
    }

    assert_eq!(
        router_native_balance,
        native_balance_of(&mut session, router.into()),
        "Router native balance should not change"
    );
}

/// Tests a simple swap along [Pair -> StableSwap -> Pair_native] path
/// using `swap_exact_tokens_for_native` and
/// `swap_tokens_for_exact_native` methods
#[drink::test]
fn test_native_out_swap(mut session: Session) {
    upload_all(&mut session);

    // seed test accounts with some native token
    seed_account(&mut session, BOB);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, wnative, _) = setup_router(&mut session);

    // setup stable pool
    let initial_reserves = vec![U100K * ONE_USDT, U100K * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * U1M)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        A,
        TRADE_FEE,
        PROTOCOL_FEE,
        BOB,
        vec![],
    );

    let (usdt, usdc) = (tokens[0], tokens[1]);

    stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // setup pairs
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdc, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    let stable_amount = U100K * ONE_USDC;
    let native_amount = U100K * ONE_AZERO;

    router_v2::add_pair_liquidity_native(
        &mut session,
        router.into(),
        None,
        usdc,
        stable_amount,
        stable_amount,
        native_amount,
        bob(),
        native_amount,
        BOB,
    )
    .expect("Should add liquidity");

    let wnative_usdc_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), wnative.into(), usdc);

    let token_amount = U100K * TOKEN;

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        wood.into(),
        usdt,
        token_amount,
        stable_amount,
        token_amount,
        stable_amount,
        bob(),
        BOB,
    )
    .expect("Should add liquidity");

    let wood_usdt_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), wood.into(), usdt);

    // increase gas limit (swaps with more than 3 tokens require more gas)
    let gas_limit = session.get_gas_limit();
    session.set_gas_limit(Weight::from_parts(
        10 * gas_limit.ref_time(),
        10 * gas_limit.proof_size(),
    ));

    let swap_amount = 100 * TOKEN;

    let router_native_balance = native_balance_of(&mut session, router.into());

    router_v2::swap_exact_tokens_for_native(
        &mut session,
        router.into(),
        swap_amount,
        0,
        vec![
            Step {
                token_in: wood.into(),
                pool_id: wood_usdt_pair.into(),
            },
            Step {
                token_in: usdt,
                pool_id: usdt_usdc_pool.into(),
            },
            Step {
                token_in: usdc,
                pool_id: wnative_usdc_pair.into(),
            },
        ],
        bob(),
        BOB,
    )
    .expect("Should swap");

    let swap_amount = 100 * ONE_AZERO;

    router_v2::swap_tokens_for_exact_native(
        &mut session,
        router.into(),
        swap_amount,
        u128::MAX,
        vec![
            Step {
                token_in: wood.into(),
                pool_id: wood_usdt_pair.into(),
            },
            Step {
                token_in: usdt,
                pool_id: usdt_usdc_pool.into(),
            },
            Step {
                token_in: usdc,
                pool_id: wnative_usdc_pair.into(),
            },
        ],
        bob(),
        BOB,
    )
    .expect("Should swap");

    for token in [usdt, usdc, wnative.into(), wood.into()] {
        assert_eq!(
            psp22_utils::balance_of(&mut session, token, router.into()),
            0,
            "Router should not hold any tokens"
        );
    }

    assert_eq!(
        router_native_balance,
        native_balance_of(&mut session, router.into()),
        "Router native balance should not change"
    );
}

/// Tests StablePool add liquidity PSP22 tokens
#[drink::test]
fn test_stable_pool_liqudity(mut session: Session) {
    upload_all(&mut session);

    // seed test accounts with some native token
    seed_account(&mut session, BOB);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, _, _, _) = setup_router(&mut session);

    // setup stable pool
    let initial_reserves = vec![U100K * ONE_USDT, U100K * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * U1M)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        A,
        TRADE_FEE,
        PROTOCOL_FEE,
        BOB,
        vec![],
    );

    let (usdt, usdc) = (tokens[0], tokens[1]);

    // add liquidity via Router
    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    psp22_utils::increase_allowance(&mut session, usdc, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    router_v2::add_stable_swap_liquidity(
        &mut session,
        router.into(),
        usdt_usdc_pool,
        1,
        initial_reserves.clone(),
        bob(),
        false,
        0,
        BOB,
    )
    .expect("Should successfully add liquidity");

    psp22_utils::increase_allowance(&mut session, usdt_usdc_pool, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    let to_withdraw = vec![100 * ONE_USDT, 100 * ONE_USDC];
    let (max_share, _) = stable_swap::get_burn_liquidity_for_amounts(
        &mut session,
        usdt_usdc_pool,
        to_withdraw.clone(),
    )
    .expect("Should estimate burn liquidity");

    router_v2::remove_stable_pool_liquidity(
        &mut session,
        router.into(),
        usdt_usdc_pool,
        max_share,
        to_withdraw,
        bob(),
        false,
        BOB,
    )
    .expect("Should successfully add liquidity");

    let share_amount = psp22_utils::balance_of(&mut session, usdt_usdc_pool, bob()) / 100; // ~1%
    let min_amounts =
        stable_swap::get_amounts_for_liquidity_burn(&mut session, usdt_usdc_pool, share_amount)
            .expect("Should estimate amounts");

    router_v2::remove_stable_pool_liquidity_by_share(
        &mut session,
        router.into(),
        usdt_usdc_pool,
        share_amount,
        min_amounts,
        bob(),
        false,
        BOB,
    )
    .expect("Should successfully add liquidity");

    for token in [usdt, usdc] {
        assert_eq!(
            psp22_utils::balance_of(&mut session, token, router.into()),
            0,
            "Router should not hold any tokens"
        );
    }
}

/// Tests StablePool add liquidity with native token
#[drink::test]
fn test_stable_pool_liqudity_native(mut session: Session) {
    upload_all(&mut session);

    // seed test accounts with some native token
    seed_account(&mut session, BOB);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    // setup router and tokens
    let (router, _, wnative, _) = setup_router(&mut session);
    let sazero = session
        .instantiate(crate::psp22::Instance::new(
            1_000_000u128 * ONE_AZERO,
            Some("SAZERO".to_string()),
            Some("SAZERO".to_string()),
            12,
        ))
        .unwrap()
        .result
        .to_account_id()
        .into();

    // setup stable pool
    let initial_reserves = vec![U100K * ONE_AZERO, U100K * ONE_AZERO];

    let wazero_sazero_pool: AccountId = stable_swap::setup(
        &mut session,
        vec![wnative.into(), sazero],
        vec![12, 12],
        A,
        BOB,
        TRADE_FEE,
        PROTOCOL_FEE,
        Some(fee_receiver()),
    )
    .into();

    let router_native_balance = native_balance_of(&mut session, router.into());

    // add liquidity via Router
    psp22_utils::increase_allowance(&mut session, sazero, router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");
    router_v2::add_stable_swap_liquidity(
        &mut session,
        router.into(),
        wazero_sazero_pool,
        1,
        initial_reserves.clone(),
        bob(),
        true,
        initial_reserves[0],
        BOB,
    )
    .expect("Should successfully add liquidity");

    psp22_utils::increase_allowance(
        &mut session,
        wazero_sazero_pool,
        router.into(),
        u128::MAX,
        BOB,
    )
    .expect("Should increase allowance");

    let to_withdraw = vec![100 * ONE_AZERO, 100 * ONE_AZERO];
    let (max_share, _) = stable_swap::get_burn_liquidity_for_amounts(
        &mut session,
        wazero_sazero_pool,
        to_withdraw.clone(),
    )
    .expect("Should estimate burn liquidity");

    router_v2::remove_stable_pool_liquidity(
        &mut session,
        router.into(),
        wazero_sazero_pool,
        max_share,
        to_withdraw,
        bob(),
        true,
        BOB,
    )
    .expect("Should successfully add liquidity");

    let share_amount = psp22_utils::balance_of(&mut session, wazero_sazero_pool, bob()) / 100; // ~1%
    let min_amounts =
        stable_swap::get_amounts_for_liquidity_burn(&mut session, wazero_sazero_pool, share_amount)
            .expect("Should estimate amounts");

    router_v2::remove_stable_pool_liquidity_by_share(
        &mut session,
        router.into(),
        wazero_sazero_pool,
        share_amount,
        min_amounts,
        bob(),
        true,
        BOB,
    )
    .expect("Should successfully add liquidity");

    for token in [wnative.into(), sazero] {
        assert_eq!(
            psp22_utils::balance_of(&mut session, token, router.into()),
            0,
            "Router should not hold any tokens"
        );
    }

    assert_eq!(
        router_native_balance,
        native_balance_of(&mut session, router.into()),
        "Router native balance should not change"
    );
}
