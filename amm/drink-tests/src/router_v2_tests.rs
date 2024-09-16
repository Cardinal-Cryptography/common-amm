use crate::stable_swap_tests::*;
use crate::utils::*;
use crate::{factory_contract, pair_contract, router_v2_contract, wrapped_azero};

use drink::{runtime::MinimalRuntime, Weight};
use ink_wrapper_types::ToAccountId;
use router_v2_contract::RouterV2 as _;
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

    _ = stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    psp22_utils::increase_allowance(
        &mut session,
        tokens[0].into(),
        router.into(),
        u128::MAX,
        BOB,
    )
    .unwrap();

    let deadline = get_timestamp(&mut session) + 10;
    session
        .execute(router.swap_exact_tokens_for_tokens(
            ONE_USDT,
            0,
            vec![Step {
                token_in: tokens[0].into(),
                pool_id: usdt_usdc_pool.into(),
            }],
            tokens[1].into(),
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .expect("Should swap");

    let res = router_v2::get_cached_pool(&mut session, router.into(), usdt_usdc_pool)
        .expect("Should return cached StablePool");
    assert_eq!(
        res,
        Pool::StablePool(StablePool {
            id: usdt_usdc_pool,
            tokens
        }),
        "StablePool mismatch"
    );
}

#[drink::test]
fn test_cache_pair(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);

    let ice_wood_pair: pair_contract::Instance = session
        .instantiate(pair_contract::Instance::new(
            ice.into(),
            wood.into(),
            factory.into(),
            1,
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
    );
    psp22_utils::transfer(
        &mut session,
        wood.into(),
        ice_wood_pair.into(),
        10000000,
        BOB,
    );

    use pair_contract::Pair as _;
    session
        .execute(ice_wood_pair.mint(bob()))
        .unwrap()
        .result
        .unwrap()
        .expect("Should mint");

    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .unwrap();

    let deadline = get_timestamp(&mut session) + 10;
    session
        .execute(router.swap_exact_tokens_for_tokens(
            ONE_USDT,
            0,
            vec![Step {
                token_in: ice.into(),
                pool_id: ice_wood_pair.into(),
            }],
            wood.into(),
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .expect("Should swap");

    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into())
        .expect("Should return cached Pair");
    assert_eq!(
        res,
        Pool::Pair(Pair {
            id: ice_wood_pair.into(),
            token_0: ice.into(),
            token_1: wood.into(),
            fee: 1,
        }),
        "Pair mismatch"
    );
}

#[drink::test]
fn test_cache_custom_pair_with_add_liqudity(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let (router, factory, _, _) = setup_router(&mut session);

    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);

    let ice_wood_pair: pair_contract::Instance = session
        .instantiate(pair_contract::Instance::new(
            ice.into(),
            wood.into(),
            factory.into(),
            1,
        ))
        .unwrap()
        .result
        .to_account_id()
        .into();

    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .unwrap();
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .unwrap();

    router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        Some(ice_wood_pair.into()),
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

    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into())
        .expect("Should return cached Pair");
    assert_eq!(
        res,
        Pool::Pair(Pair {
            id: ice_wood_pair.into(),
            token_0: ice.into(),
            token_1: wood.into(),
            fee: 1,
        }),
        "Pair mismatch"
    );
}

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

    let ice_wood_pair = factory::get_pair(&mut session, factory.into(), ice.into(), wood.into());

    let res = router_v2::get_cached_pool(&mut session, router.into(), ice_wood_pair.into())
        .expect("Should return cached Pair");
    assert_eq!(
        res,
        Pool::Pair(Pair {
            id: ice_wood_pair.into(),
            token_0: ice.into(),
            token_1: wood.into(),
            fee: 3,
        }),
        "Pair mismatch"
    );
}

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

    _ = stable_swap::add_liquidity(
        &mut session,
        usdt_usdc_pool,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // setup pairs
    // initial amount of ICE/WOOD is 1_000_000_000 * 10 ** 18
    let ice = psp22_utils::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22_utils::setup(&mut session, WOOD.to_string(), BOB);
    psp22_utils::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB)
        .unwrap();
    psp22_utils::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB)
        .unwrap();
    psp22_utils::increase_allowance(&mut session, usdt, router.into(), u128::MAX, BOB).unwrap();
    psp22_utils::increase_allowance(&mut session, usdc, router.into(), u128::MAX, BOB).unwrap();

    let token_amount = 100_000 * TOKEN;

    _ = router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        ice.into(),
        wood.into(),
        token_amount,
        token_amount,
        token_amount,
        token_amount,
        bob(),
        BOB,
    );
    let ice_wood_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), ice.into(), wood.into());

    _ = router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        None,
        ice.into(),
        usdt,
        token_amount,
        100_000 * ONE_USDT,
        token_amount,
        100_000 * ONE_USDT,
        bob(),
        BOB,
    );
    let ice_usdt_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), ice.into(), usdt);

    // increase gas limit (swaps with more than 3 tokens require more gas)
    let gas_limit = session.get_gas_limit();
    session.set_gas_limit(Weight::from_parts(
        10 * gas_limit.ref_time(),
        10 * gas_limit.proof_size(),
    ));

    // swap wood -> ice -> usdt -> usdc
    let deadline = now + 10;
    let swap_amount = 100 * TOKEN;

    let swap_res = session
        .execute(router.swap_exact_tokens_for_tokens(
            swap_amount,
            0,
            vec![
                Step {
                    token_in: wood.into(),
                    pool_id: ice_wood_pair.into(),
                },
                Step {
                    token_in: ice.into(),
                    pool_id: ice_usdt_pair.into(),
                },
                Step {
                    token_in: usdt.into(),
                    pool_id: usdt_usdc_pool.into(),
                },
            ],
            usdc.into(),
            bob(),
            deadline,
        ))
        .unwrap();
    _ = swap_res.result.unwrap().unwrap();
}
