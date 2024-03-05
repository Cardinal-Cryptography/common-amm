use crate::factory_contract;
use crate::pair_contract;
use crate::router_contract;
use crate::utils::*;

use factory_contract::Factory as _;
use router_contract::Router as _;

use drink::{self, session::Session};
use drink::contract_api::decode_debug_buffer;
use ink_wrapper_types::Connection;
use num_format::{Locale, ToFormattedString};

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

    let exp = 10u128.pow(18);
    let token_amount = 1_000_000 * exp;
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

    let swap_amount = 10_000 * exp;
    // 0.3% is would be 30 tokens exactly but due to rounding we can lose up to 1 dust.

    println!("===========================SWAPPING==============================");
    let result = session
        .execute(router.swap_exact_tokens_for_tokens(
            swap_amount,
            0,
            vec![ice.into(), wood.into()],
            bob(),
            deadline,
        ))
        .unwrap();

    let swap_res = result
        .result
        .unwrap()
        .unwrap();

    let gas_consumed = result.gas_consumed;

    println!("Total gas consumed: {}", gas_consumed.ref_time().to_formatted_string(&Locale::en));
    println!("Debug message: {:#?}", decode_debug_buffer(&result.debug_message));
    println!();

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
    let received_protocol_fees = protocol_fees_ice + protocol_fees_wood;

    // Trading fee is 0.3%.
    let trading_fee = 0.3 / 100.0;

    // Protocol fees are 1/6 of the trading fees.
    // Protocol fees are a sum of:
    // * % from the input trading amount
    // * the slippage value of the trade
    //
    // This makes it dynamic and hard to predict. So we're setting lower & upper bounds.

    // Lower bound for the received protocol fees is the exact 0.3%/6 of the output amount.
    let expected_protocol_fees = (swap_res[1] as f64 * trading_fee) / 6.0;
    // Upper bound for the received protocol fees is the % from the input amount.
    let expected_with_imp_loss = (swap_res[0] as f64 * trading_fee) / 6.0;

    assert!(received_protocol_fees >= expected_protocol_fees as u128);
    assert!(received_protocol_fees <= expected_with_imp_loss as u128);
}
