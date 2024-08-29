use crate::pair_contract;
use crate::router_v2_contract;
use crate::stable_swap_tests::*;
use crate::utils::*;

use drink::Weight;
use router_v2_contract::RouterV2 as _;
use router_v2_contract::Step;

use drink::{self, session::Session};
use ink_wrapper_types::Connection;

#[drink::test]
fn test_cache_stable_pool(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let fee_to_setter = bob();

    // initial amount of ICE is 1_000_000_000 * 10 ** 18
    let factory = factory::setup(&mut session, fee_to_setter);
    let wazero = wazero::setup(&mut session);
    let router = router_v2::setup(&mut session, factory.into(), wazero.into());

    let initial_reserves = vec![100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();

    let (usdt_usdc_pool, _) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = session
        .execute(router.add_stable_pool_to_cache(usdt_usdc_pool))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    let res = router_v2::get_cached_stable_pool(&mut session, router.into(), usdt_usdc_pool);
    assert_eq!(res, usdt_usdc_pool, "Pool Id mismatch");
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

    // setup router
    let fee_to_setter = bob();
    let factory = factory::setup(&mut session, fee_to_setter);
    let wazero = wazero::setup(&mut session);
    let router = router_v2::setup(&mut session, factory.into(), wazero.into());

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

    _ = session
        .execute(router.add_stable_pool_to_cache(usdt_usdc_pool))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    // setup pairs
    // initial amount of ICE is 1_000_000_000 * 10 ** 18
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
        ice.into(),
        wood.into(),
        token_amount,
        token_amount,
        token_amount,
        token_amount,
        bob(),
        BOB,
    );
    let _ice_wood_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), ice.into(), wood.into());

    _ = router_v2::add_pair_liquidity(
        &mut session,
        router.into(),
        ice.into(),
        usdt,
        token_amount,
        100_000 * ONE_USDT,
        token_amount,
        100_000 * ONE_USDT,
        bob(),
        BOB,
    );
    let _ice_usdt_pair: pair_contract::Instance =
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
                    pool_id: None,
                },
                Step {
                    token_in: ice.into(),
                    pool_id: None,
                },
                Step {
                    token_in: usdt.into(),
                    pool_id: Some(usdt_usdc_pool),
                },
            ],
            usdc.into(),
            bob(),
            deadline,
        ))
        .unwrap();
    _ = swap_res.result.unwrap().unwrap();
}
