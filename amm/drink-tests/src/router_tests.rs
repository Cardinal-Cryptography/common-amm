use crate::factory_contract;
use crate::pair_contract;
use crate::pair_contract::Pair;
use crate::router_contract;
use crate::utils::*;

use factory_contract::Factory as _;
use router_contract::Router as _;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::{self, session::Session};
use ink_wrapper_types::Connection;

#[drink::test]
fn add_liquidity(mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    let factory = factory::setup(&mut session, fee_to_setter);
    let ice = psp22::setup(&mut session, ICE.to_string(), BOB);
    let wazero = wazero::setup(&mut session);
    let router = router::setup(&mut session, factory.into(), wazero.into());

    let token_amount = 10_000;
    psp22::increase_allowance(&mut session, ice.into(), router.into(), token_amount, BOB).unwrap();

    let all_pairs_length_before = session
        .query(factory.all_pairs_length())
        .unwrap()
        .result
        .unwrap();

    let now = get_timestamp(&mut session);
    set_timestamp(&mut session, now);
    let deadline = now + 10;

    let (amount_ice, amount_native, liquidity_minted) = session
        .execute(
            router
                .add_liquidity_native(
                    ice.into(),
                    token_amount,
                    token_amount,
                    token_amount,
                    bob(),
                    deadline,
                )
                .with_value(token_amount),
        )
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    let ice_wazero_pair: pair_contract::Instance = session
        .query(factory.get_pair(ice.into(), wazero.into()))
        .unwrap()
        .result
        .unwrap()
        .unwrap()
        .into();

    let minimum_liquidity = session
        .query(ice_wazero_pair.get_minimum_liquidity())
        .unwrap()
        .result
        .unwrap();

    let all_pairs_length_after = session
        .query(factory.all_pairs_length())
        .unwrap()
        .result
        .unwrap();

    assert!(
        all_pairs_length_before + 1 == all_pairs_length_after,
        "There should be one more pair"
    );
    assert!(amount_ice == token_amount,);
    assert!(amount_native == token_amount,);
    // Matches the formula from the whitepaper for minting liquidity tokens for a newly created pair.
    assert!(
        liquidity_minted == token_amount.mul(token_amount).integer_sqrt() - minimum_liquidity,
        "Should mint expected amount of LP tokens"
    );
}

#[drink::test]
fn add_liquidity_collects_too_much_fee(mut session: Session) {
    // Hats submission 0xfa2f634e33a1c66390a57717bef271960460209ff9ecd5aab11e7bb43ebce999
    upload_all(&mut session);

    let fee_to_setter = bob();
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

    // bob mints the liquidity for the first time
    let _ = router::add_liquidity(
        &mut session,
        router.into(),
        ice.into(),
        wood.into(),
        token_amount,
        token_amount,
        bob(),
    );

    // bob mints the liquidity for the second time
    let (_amount_ice, _amount_wood, _liquidity_minted) = router::add_liquidity(
        &mut session,
        router.into(),
        ice.into(),
        wood.into(),
        token_amount,
        0,
        bob(),
    );

    let ice_wood_pair: pair_contract::Instance = session
        .query(factory.get_pair(ice.into(), wood.into()))
        .unwrap()
        .result
        .unwrap()
        .unwrap()
        .into();

    // Since no swaps occured Charlie (`fee_to`) should not have any liquidity
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

    let token_amount = 1_000_000 * 10u128.pow(18);
    psp22::increase_allowance(&mut session, ice.into(), router.into(), u128::MAX, BOB).unwrap();
    psp22::increase_allowance(&mut session, wood.into(), router.into(), u128::MAX, BOB).unwrap();

    let deadline = now + 10;

    let (_a, _b, liquidity_minted) = router::add_liquidity(
        &mut session,
        router.into(),
        ice.into(),
        wood.into(),
        token_amount,
        token_amount,
        bob(),
    );

    let ice_wood_pair: pair_contract::Instance =
        factory::get_pair(&mut session, factory.into(), ice.into(), wood.into());

    let bob_lp_balance: u128 = psp22::balance_of(&mut session, ice_wood_pair.into(), bob());
    assert_eq!(liquidity_minted, bob_lp_balance);

    let swap_amount = 10_000 * 10u128.pow(18);
    // 0.3% is would be 30 tokens exactly but due to rounding we can lose up to 1 dust.
    let swap_res = session
        .execute(router.swap_exact_tokens_for_tokens(
            swap_amount,
            0,
            vec![ice.into(), wood.into()],
            bob(),
            deadline,
        ))
        .unwrap()
        .result
        .unwrap()
        .unwrap();

    assert_eq!(swap_res[0], swap_amount);
    // We cannot assert that as it depends on the pool's liquidity and swap amount. The price will move from the spot price.
    // assert!(swap_res[1] >= min_amount_out);

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
    // Trading fee is 0.3%.
    let trading_fee = 0.3 / 100.0;
    // Protocol fees are 1/6 of the trading fees.
    let expected = (swap_amount as f64 * trading_fee) / 6.0;
    // Charlie withdraws his fees
    psp22::increase_allowance(
        &mut session,
        ice_wood_pair.into(),
        router.into(),
        charlie_lp_balance,
        CHARLIE,
    )
    .unwrap();
    let (protocol_fees_ice, protocol_fees_wood) = router::remove_liquidity(
        &mut session,
        router.into(),
        ice.into(),
        wood.into(),
        charlie_lp_balance,
        0,
        0,
        charlie(),
    );
    // We cannot assert exactly how much fees Charlie will get, due to roundings etc,
    // but it should be close to the expected value
    let protocol_fees = protocol_fees_ice + protocol_fees_wood;
    let percentile = expected * 9999.0 / 10000.0;
    assert!(protocol_fees >= expected as u128 - percentile as u128);
    assert!(protocol_fees <= expected as u128);
}
