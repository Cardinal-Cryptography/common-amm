use drink::{self, runtime::MinimalRuntime, session::Session};

use super::*;

/// Tests swap of token at index 0 to token at index 1.
/// Tests two ways of swapping: swap_exact_in and swap_received
fn test_swap_exact_in(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    amp_coef: u128,
    trade_fee: u32,
    protocol_fee: u32,
    swap_amount_in: u128,
    expected_swap_amount_out_total_result: Result<u128, StablePoolError>,
) {
    let initial_supply = vec![initial_reserves[0] + swap_amount_in, initial_reserves[1]];
    // setup two identical pools
    let (stable_swap_1, tokens_1) = setup_stable_swap_with_tokens(
        session,
        token_decimals.clone(),
        initial_supply.clone(),
        amp_coef,
        trade_fee,
        protocol_fee,
        BOB,
        "foo".as_bytes().to_vec(),
    );
    let (stable_swap_2, tokens_2) = setup_stable_swap_with_tokens(
        session,
        token_decimals.clone(),
        initial_supply,
        amp_coef,
        trade_fee,
        protocol_fee,
        BOB,
        "bar".as_bytes().to_vec(),
    );
    _ = stable_swap::add_liquidity(
        session,
        stable_swap_1,
        BOB,
        1,
        initial_reserves.to_vec(),
        bob(),
    );
    _ = stable_swap::add_liquidity(
        session,
        stable_swap_2,
        BOB,
        1,
        initial_reserves.to_vec(),
        bob(),
    );

    let swap_result_1 = stable_swap::swap_exact_in(
        session,
        stable_swap_1,
        BOB,
        tokens_1[0],    // in
        tokens_1[1],    // out
        swap_amount_in, // amount_in
        0,              // min_token_out
        bob(),
    );

    let _ = psp22_utils::transfer(session, tokens_2[0], stable_swap_2, swap_amount_in, BOB);

    let swap_result_2 = stable_swap::swap_received(
        session,
        stable_swap_2,
        BOB,
        tokens_2[0], // in
        tokens_2[1], // out
        0,           // min_token_out
        bob(),
    );

    if expected_swap_amount_out_total_result.is_err() {
        let swap_err_1 = swap_result_1.expect_err("swap_exact_in: Should return an error.");
        let swap_err_2 = swap_result_2.expect_err("swap_received: Should return an error.");
        let expected_err = expected_swap_amount_out_total_result.err().unwrap();
        assert_eq!(expected_err, swap_err_1);
        assert_eq!(expected_err, swap_err_2);
        return;
    }

    let expected_swap_amount_out_total = expected_swap_amount_out_total_result.unwrap();
    let expected_fee = expected_swap_amount_out_total * trade_fee as u128 / FEE_DENOM;
    let expected_swap_amount_out = expected_swap_amount_out_total - expected_fee;
    let expected_protocol_fee_part = expected_fee * protocol_fee as u128 / FEE_DENOM;

    let (amount_out_1, fee_1) = swap_result_1.unwrap();
    let (amount_out_2, fee_2) = swap_result_2.unwrap();

    // check returned amount swapped and fee
    assert_eq!(
        expected_swap_amount_out, amount_out_1,
        "swap_exact_in: Amount out mismatch"
    );
    assert_eq!(expected_fee, fee_1, "swap_exact_in: Fee mismatch");

    assert_eq!(
        expected_swap_amount_out, amount_out_2,
        "swap_received: Amount out mismatch"
    );
    assert_eq!(expected_fee, fee_2, "swap_received: Fee mismatch");

    // check if reserves were updated properly
    let expected_reserves = [
        initial_reserves[0] + swap_amount_in,
        initial_reserves[1] - expected_swap_amount_out,
    ];
    let reserves_1 = stable_swap::reserves(session, stable_swap_1);
    assert_eq!(
        reserves_1, expected_reserves,
        "swap_exact_in: Reserves not updated properly"
    );

    let reserves_2 = stable_swap::reserves(session, stable_swap_1);
    assert_eq!(
        reserves_2, expected_reserves,
        "swap_received: Reserves not updated properly"
    );

    // check if reserves are equal the actual balances
    let balance_0 = psp22_utils::balance_of(session, tokens_1[0], stable_swap_1);
    let balance_1 = psp22_utils::balance_of(session, tokens_1[1], stable_swap_1);
    assert_eq!(
        reserves_1,
        vec![balance_0, balance_1],
        "swap_exact_in: Balances - reserves mismatch"
    );
    let balance_0 = psp22_utils::balance_of(session, tokens_2[0], stable_swap_2);
    let balance_1 = psp22_utils::balance_of(session, tokens_2[1], stable_swap_2);
    assert_eq!(
        reserves_2,
        vec![balance_0, balance_1],
        "swap_received: Balances - reserves mismatch"
    );

    // check bobs balances
    let balance_0 = psp22_utils::balance_of(session, tokens_1[0], bob());
    let balance_1 = psp22_utils::balance_of(session, tokens_1[1], bob());
    assert_eq!(
        [0, expected_swap_amount_out],
        [balance_0, balance_1],
        "swap_exact_in: Incorrect Bob's balances"
    );
    let balance_0 = psp22_utils::balance_of(session, tokens_2[0], bob());
    let balance_1 = psp22_utils::balance_of(session, tokens_2[1], bob());
    assert_eq!(
        [0, expected_swap_amount_out],
        [balance_0, balance_1],
        "swap_received: Incorrect Bob's balances"
    );

    // check protocol fee
    let protocol_fee_lp = psp22_utils::balance_of(session, stable_swap_1, fee_receiver());
    let (total_lp_required, lp_fee_part) = stable_swap::remove_liquidity_by_amounts(
        session,
        stable_swap_1,
        BOB,
        protocol_fee_lp * 2,
        [0, expected_protocol_fee_part].to_vec(),
        bob(),
    )
    .unwrap_or((0, 0));
    assert_eq!(
        total_lp_required - lp_fee_part,
        protocol_fee_lp,
        "swap_exact_in: Incorrect protocol fee"
    );

    let protocol_fee_lp = psp22_utils::balance_of(session, stable_swap_2, fee_receiver());
    let (total_lp_required, lp_fee_part) = stable_swap::remove_liquidity_by_amounts(
        session,
        stable_swap_2,
        BOB,
        protocol_fee_lp * 2,
        [0, expected_protocol_fee_part].to_vec(),
        bob(),
    )
    .unwrap_or((0, 0));
    assert_eq!(
        total_lp_required - lp_fee_part,
        protocol_fee_lp,
        "swap_received: Incorrect protocol fee"
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L744
#[drink::test]
fn test_01(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![6, 6],                       // decimals
        vec![100000000000, 100000000000], // initial reserves
        1000,                             // A
        600_000,                          // trade fee in 1e9 precision
        2000,                             // protocol fee in 1e9 precision
        10000000000,                      // swap_amount_in
        Ok(9999495232),                   // expected out (with fee)
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L763
#[drink::test]
fn test_02(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        2000,
        10000000000000000,
        Ok(9999495232752197989995),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L782
#[drink::test]
fn test_03(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        2000,
        0,
        Err(StablePoolError::InsufficientInputAmount()),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L801
#[drink::test]
fn test_04(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        2000,
        0,
        Err(StablePoolError::InsufficientInputAmount()),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L820
#[drink::test]
fn test_05(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        2000,
        1,
        Ok(0),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L839
// Test that swapping 0.000000000001000000 gives 0.000000000000 (token precision cut)
#[drink::test]
fn test_06_a(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        1000,
        600_000,
        2000,
        1000000,
        Ok(0),
    );
}

// Test that swapping (with fees disabled) 0.000000000001000000 gives 0.000000000000
#[drink::test]
fn test_06_b(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        1000,
        0,
        0,
        1000000,
        Ok(0),
    );
}

// Test that swapping (with disabled fees) 0.000000000001000001 gives 0.000000000001
#[drink::test]
fn test_06_c(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        1000,
        0,
        0,
        1000001,
        Ok(1),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L858
#[drink::test]
fn test_07(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        2000,
        100000000000,
        Ok(98443663539),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L877
#[drink::test]
fn test_08(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        2000,
        100000000000000000,
        Ok(98443663539913153080656),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L896
#[drink::test]
fn test_09(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![6, 6],
        vec![100000000000, 100000000000],
        1000,
        600_000,
        2000,
        99999000000 + 1, // +1 because of accounting for fee rounding
        Ok(98443167413),
    );
}

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/src/stable_swap/mod.rs#L915
#[drink::test]
fn test_10(mut session: Session) {
    test_swap_exact_in(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        1000,
        600_000,
        2000,
        99999000000000000,
        Ok(98443167413204135506296),
    );
}
