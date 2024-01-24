use crate::factory_contract;
use crate::pair_contract;
use crate::router_contract;
use crate::utils::*;

use drink::{self, session::Session};
use ink_wrapper_types::Connection;

use factory_contract::Factory as _;
use router_contract::Router as _;

#[drink::test]
fn add_liquidity_collects_too_much_fee(mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    // initial amount of ICE is 2_000_000_000 * 10 ** 18
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
