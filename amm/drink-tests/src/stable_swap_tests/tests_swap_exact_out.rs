use std::ops::{Add, Sub};

use drink::{self, runtime::MinimalRuntime, session::Session};

use super::*;

/// Assert if `a` and `b` are equal +/- `delta`
fn assert_approx<T>(a: T, b: T, delta: T, message: &str)
where
    T: Add + Sub + Copy + PartialEq + PartialOrd + Eq + Ord + std::fmt::Display,
    <T as Sub>::Output: PartialOrd<T>,
{
    if a > b {
        if a - b > delta {
            panic!("{}, {}, {}", a, b, message)
        }
    } else {
        if b - a > delta {
            panic!("{}, {}, {}", a, b, message)
        }
    }
}

/// Cross test swap methods.
/// Tests if `swap_exact_out` performs a fair swap
/// and produces correct result.
/// Tests swap of the token at index 0 to token at index 1
///
/// NOTE: When token_decimals[0] > token_decimals[1], the method tests if
/// `swap_amount_out` is approximatelly correct (`+/- 10^(token_decimals[0] - token_decimals[1])`) and
/// always in the protocols favour.
fn test_swap_exact_out(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    initial_reserves: Vec<u128>,
    amp_coef: u128,
    trade_fee: u32,
    protocol_fee: u32,
    swap_amount_out: u128,
) {
    let initial_supply = vec![initial_reserves[0] * 2, initial_reserves[1] * 2];
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

    let swap_exact_out_result = stable_swap::swap_exact_out(
        session,
        stable_swap_1,
        BOB,
        tokens_1[0],     // in
        tokens_1[1],     // out
        swap_amount_out, // amount_out
        u128::MAX,       // max_token_in
        bob(),
    )
    .expect("swap_exact_out failed");

    let swap_exact_in_result = stable_swap::swap_exact_in(
        session,
        stable_swap_2,
        BOB,
        tokens_2[0],             // in
        tokens_2[1],             // out
        swap_exact_out_result.0, // amount_in - cross test result
        0,                       // min_token_out
        bob(),
    )
    .expect("swap_exact_in failed");

    // If token_out has more decimals than token_in, allow rounding error
    // up to the difference in their precision.
    // Otherwise the results should be exact.
    let delta: u128 = if token_decimals[1] > token_decimals[0] {
        10u128.pow((token_decimals[1] - token_decimals[0]) as u32)
    } else {
        0
    };

    // check returned amount swapped and fee
    assert_approx(
        swap_amount_out,
        swap_exact_in_result.0,
        delta,
        "Amount out mismatch",
    );
    assert!(
        swap_amount_out <= swap_exact_in_result.0,
        "Protocol at loss (amount out)",
    );
    assert_approx(
        swap_exact_out_result.1,
        swap_exact_in_result.1,
        delta,
        "Fee mismatch",
    );
    assert!(
        swap_exact_out_result.1 <= swap_exact_in_result.1,
        "Protocol at loss (fee)",
    );

    // check if reserves were updated properly
    let reserves = stable_swap::reserves(session, stable_swap_1);
    assert_eq!(
        reserves,
        [
            initial_reserves[0] + swap_exact_out_result.0,
            initial_reserves[1] - swap_amount_out
        ],
        "Reserves not updated properly"
    );

    // check if reserves are equal the actual balances
    let balance_0 = psp22_utils::balance_of(session, tokens_1[0], stable_swap_1);
    let balance_1 = psp22_utils::balance_of(session, tokens_1[1], stable_swap_1);
    assert_eq!(
        reserves,
        vec![balance_0, balance_1],
        "Balances - reserves mismatch"
    );

    // check bobs balances
    let balance_0 = psp22_utils::balance_of(session, tokens_1[0], bob());
    let balance_1 = psp22_utils::balance_of(session, tokens_1[1], bob());
    assert_eq!(
        [
            initial_reserves[0] - swap_exact_out_result.0,
            initial_reserves[1] + swap_amount_out
        ],
        [balance_0, balance_1],
        "Incorrect Bob's balances"
    );

    // check protocol fee
    let expected_protocol_fee_part = swap_exact_out_result.1 * protocol_fee as u128 / FEE_DENOM;
    let protocol_fee_lp = psp22_utils::balance_of(session, stable_swap_1, fee_receiver());
    let (total_lp_required, lp_fee_part) = stable_swap::remove_liquidity_by_amounts(
        session,
        stable_swap_1,
        BOB,
        protocol_fee_lp * 2,
        [0, expected_protocol_fee_part].to_vec(),
        bob(),
    )
    .expect("Should remove lp");
    assert_eq!(
        total_lp_required - lp_fee_part,
        protocol_fee_lp,
        "Incorrect protocol fee"
    );
}

// test when tokens precisions are the same
#[drink::test]
fn test_01(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![6, 6],                       // decimals
        vec![100000000000, 100000000000], // initial reserves
        10000,                            // A
        600_000,                          // trade fee
        200_000_000,                      // protocol fee
        100000000,                        // expected amount out
    );
}

#[drink::test]
fn test_02(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![12, 12],
        vec![100000000000000000, 100000000000000000],
        10000,
        600_000,
        200_000_000,
        100000000000000,
    );
}

#[drink::test]
fn test_03(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![18, 18],
        vec![100000000000000000000000, 100000000000000000000000],
        10000,
        600_000,
        200_000_000,
        100000000000000000000,
    );
}

// test when token_in precision is smaller than token_out precision
#[drink::test]
fn test_04(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![6, 12],
        vec![100000000000, 100000000000000000],
        10000,
        600_000,
        200_000_000,
        100000000000000,
    );
}

#[drink::test]
fn test_05(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![6, 18],
        vec![100000000000, 100000000000000000000000],
        10000,
        600_000,
        200_000_000,
        100000000000000000000,
    );
}

#[drink::test]
fn test_06(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![12, 18],
        vec![100000000000000000, 100000000000000000000000],
        10000,
        600_000,
        200_000_000,
        100000000000000000000,
    );
}

// test when token_out precision is smaller than token_in precision
#[drink::test]
fn test_07(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![12, 6],
        vec![100000000000000000, 100000000000],
        10000,
        600_000,
        200_000_000,
        100000000,
    );
}

#[drink::test]
fn test_08(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![18, 6],
        vec![100000000000000000000000, 100000000000],
        10000,
        600_000,
        200_000_000,
        100000000,
    );
}

#[drink::test]
fn test_09(mut session: Session) {
    test_swap_exact_out(
        &mut session,
        vec![18, 12],
        vec![100000000000000000000000, 100000000000000000],
        10000,
        600_000,
        200_000_000,
        100000000000000,
    );
}
