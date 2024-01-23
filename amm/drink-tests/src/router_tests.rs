use crate::factory_contract;
use crate::pair_contract;
use crate::pair_contract::Pair;
use crate::router_contract;
use crate::utils::*;

use drink::frame_support::sp_runtime::traits::IntegerSquareRoot;
use drink::frame_support::sp_runtime::traits::Scale;
use drink::session::Session;
use ink_wrapper_types::Connection;

use factory_contract::Factory as _;
use router_contract::Router as _;

#[drink::test]
fn add_liquidity(mut session: Session) {
    upload_all(&mut session);

    let fee_to_setter = bob();

    let factory = setup_factory(&mut session, fee_to_setter);
    let ice = setup_psp22(&mut session, ICE.to_string(), BOB);
    let wazero = setup_wAzero(&mut session);
    let router = setup_router(&mut session, factory.into(), wazero.into());

    let token_amount = 10_000;
    increase_allowance(&mut session, ice.into(), router.into(), token_amount, BOB).unwrap();

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

    assert!(all_pairs_length_before + 1 == all_pairs_length_after);
    assert!(amount_ice == token_amount);
    assert!(amount_native == token_amount);
    // Matches the formula from the whitepaper for minting liquidity tokens for a newly created pair.
    assert!(liquidity_minted == token_amount.mul(token_amount).integer_sqrt() - minimum_liquidity);
}
