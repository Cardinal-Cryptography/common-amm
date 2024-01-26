use crate::factory_contract;
use crate::pair_contract;
use crate::router_contract;
use crate::utils::*;

use drink::{self, session::Session};
use ink_wrapper_types::{Connection, ToAccountId};

use factory_contract::Factory as _;
use router_contract::Router as _;

#[drink::test]
fn add_liquidity_collects_too_much_fee(mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    // initial amount of ICE is 1_000_000_000 * 10 ** 18
    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), BOB);
    let wazero = wazero::setup(&mut session);
    let router = router::setup(&mut session, factory.into(), wazero.into());
    // feed charlie some native tokens
    session
        .sandbox()
        .mint_into(CHARLIE, 10u128.pow(12))
        .unwrap();

    // set fee collector to CHARLIE [3u8;32]
    session
        .execute(factory.set_fee_to(charlie()))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    let token_amount = 1_000 * 10u128.pow(18);
    psp22::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB).unwrap();
    psp22::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB).unwrap();

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let deadline = now + 10;

    // bob mints the liquidity for the first time
    session
        .execute(router.add_liquidity(
            ice.into(),
            wood.into(),
            token_amount,
            token_amount,
            token_amount,
            token_amount,
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    // bob mints the liquidity for the second time
    let (_amount_ice, _amount_wood, _liquidity_minted) = session
        .execute(router.add_liquidity(
            ice.into(),
            wood.into(),
            token_amount,
            token_amount,
            0,
            0,
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    let ice_wood_pair: pair_contract::Instance = session
        .query(factory.get_pair(ice.into(), wood.into()))
        .unwrap()
        .result
        .unwrap()
        .unwrap()
        .into();

    // since no swaps occured charlie (`fee_to`) should not have any liquidity
    // however we can see that he has 1/6th of the second liquidity
    let charlie_lp = psp22::balance_of(&mut session, ice_wood_pair.into(), charlie());

    assert_eq!(0, charlie_lp);
}

#[drink::test]
fn test_fees(mut session: Session) {
    upload_all(&mut session);

    // Fix timestamp. Otherwise underlying UNIX clock is used.
    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);

    let fee_to_setter = bob();

    // initial amount of ICE is 1_000_000_000 * 10 ** 18
    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22::setup(&mut session, ICE.to_string(), BOB);
    let wood = psp22::setup(&mut session, WOOD.to_string(), BOB);
    let wazero = wazero::setup(&mut session);
    let router = router::setup(&mut session, factory.into(), wazero.into());

    // feed Charlie some native tokens
    session
        .sandbox()
        .mint_into(CHARLIE, 10u128.pow(12))
        .unwrap();

    // set fee collector to CHARLIE [3u8;32]
    session
        .execute(factory.set_fee_to(charlie()))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    let token_amount = 1_000 * 10u128.pow(18);
    psp22::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB).unwrap();
    psp22::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB).unwrap();

    let deadline = now + 10;

    let (_a, _b, liquidity_minted) = session
        .execute(router.add_liquidity(
            ice.into(),
            wood.into(),
            token_amount,
            token_amount,
            token_amount,
            token_amount,
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    let ice_wood_pair: pair_contract::Instance = session
        .query(factory.get_pair(ice.into(), wood.into()))
        .unwrap()
        .result
        .unwrap()
        .unwrap()
        .to_account_id()
        .into();

    let bob_lp_balance: u128 = psp22::balance_of(&mut session, ice_wood_pair.into(), bob());
    assert_eq!(liquidity_minted, bob_lp_balance);

    let swap_amount = 10_000;
    // 0.3% is would be 30 tokens exactly but due to rounding we can lose up to 1 dust.
    let min_amount_out = 9969;
    let swap_res = session
        .execute(router.swap_exact_tokens_for_tokens(
            swap_amount,
            min_amount_out,
            vec![ice.into(), wood.into()],
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();
    assert_eq!(swap_res[0], swap_amount);
    assert!(swap_res[1] >= min_amount_out);

    // No fees distributed until the burn/mint transaction.
    assert!(psp22::balance_of(&mut session, ice_wood_pair.into(), charlie()) == 0);
    // Burn some liquidity to trigger fee collection.
    psp22::increase_allowance(
        &mut session,
        ice_wood_pair.into(),
        router.into(),
        liquidity_minted,
        BOB,
    )
    .unwrap();
    let (_ice, _wood) = session
        .execute(router.remove_liquidity(
            ice.into(),
            wood.into(),
            liquidity_minted,
            0,
            0,
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    // Fees now sent to `fee_to` address (CHARLIE).
    let charlie_lp_balance = psp22::balance_of(&mut session, ice_wood_pair.into(), charlie());
    // Charlie withdraws his fees
    psp22::increase_allowance(
        &mut session,
        ice_wood_pair.into(),
        router.into(),
        charlie_lp_balance,
        CHARLIE,
    )
    .unwrap();
    let (protocol_fees_ice, protocol_fees_wood) = session
        .execute(router.remove_liquidity(
            ice.into(),
            wood.into(),
            charlie_lp_balance,
            0,
            0,
            charlie(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();
    // Trader paid 30 tokens in fees, 1/6 of that is 5. Due to rounding down we loose up to 1 dust.
    // Since we get paid in LP tokens, for 1:1 pool (which this almost is) we get 2 tokens of each.
    assert_eq!(protocol_fees_ice + protocol_fees_wood, 4);
}
