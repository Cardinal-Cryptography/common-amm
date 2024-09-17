use std::u128;

use crate::stable_swap_tests::*;
use crate::utils::*;
use crate::{factory_contract, pair_contract, router_v2_contract, wrapped_azero};

use drink::{runtime::MinimalRuntime, Weight};
use ink_wrapper_types::ToAccountId;
use pair_contract::Pair as _;
use router_v2_contract::{Pair, Pool, StablePool, Step};

use drink::{self, session::Session};
use ink_wrapper_types::Connection;

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

    let initial_reserves = vec![100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
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

    // we need to add some liquidity to the pool before the swap
    stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // ensure that the pool is not cached before the swap
    let res = router_v2::get_cached_pool(&mut session, router.into(), usdt_usdc_pool);
    assert_eq!(res, None, "StablePool should not be in the cache");

    psp22_utils::increase_allowance(
        &mut session,
        tokens[0].into(),
        router.into(),
        u128::MAX,
        BOB,
    )
    .expect("Should increase allowance");

    router_v2::swap_exact_tokens_for_tokens(
        &mut session,
        router.into(),
        ONE_USDT,
        0,
        vec![Step {
            token_in: tokens[0].into(),
            pool_id: usdt_usdc_pool.into(),
        }],
        tokens[1].into(),
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

    psp22_utils::transfer(
        &mut session,
        ice.into(),
        ice_wood_pair.into(),
        10000000,
        BOB,
    )
    .expect("Should transfer PSP22");
    psp22_utils::transfer(
        &mut session,
        wood.into(),
        ice_wood_pair.into(),
        10000000,
        BOB,
    )
    .expect("Should transfer PSP22");

    session
        .execute(ice_wood_pair.mint(bob()))
        .unwrap()
        .result
        .unwrap()
        .expect("Should mint");

    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .expect("Should increase allowance");

    // ensure that the pair is not cached before the swap
    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into());
    assert_eq!(res, None, "Pair should not be in the cache");

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

    let init_liqudity_amount = 100000;
    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        Some(ice_wood_pair.into()),
        ice.into(),
        wood.into(),
        init_liqudity_amount,
        init_liqudity_amount,
        init_liqudity_amount,
        init_liqudity_amount,
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
        .unwrap();
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .unwrap();

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        ice.into(),
        wood.into(),
        100000,
        100000,
        100000,
        100000,
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
fn test_simple_swap(mut session: Session) {
    upload_all(&mut session);

    // seed test accounts with some native token
    seed_account(&mut session, BOB);
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    // setup stable pool
    let initial_reserves = vec![100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    let usdt = tokens[0];
    let usdc = tokens[1];

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

    let token_amount = 100_000 * TOKEN;
    let stable_amount = 100_000 * ONE_USDC;

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
    router_v2::swap_exact_tokens_for_tokens(
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
    
    router_v2::swap_tokens_for_exact_tokens(
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
}
