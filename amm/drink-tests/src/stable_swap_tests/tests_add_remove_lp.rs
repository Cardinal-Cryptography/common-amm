use drink::{self, session::Session};
use stable_pool_contract::MathError;

use super::*;

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_stable_pool.rs#L123
#[drink::test]
fn test_01(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);
    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let (last_share_price, last_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        CHARLIE,
        vec![500 * ONE_DAI, 500 * ONE_USDT, 500 * ONE_USDC],
        BOB,
    );

    // add more liquidity with balanced tokens (charlie)
    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        CHARLIE,
        1,
        vec![500 * ONE_DAI, 500 * ONE_USDT, 500 * ONE_USDC],
        charlie(),
    )
    .expect("Should successfully add liquidity");

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares + 1500 * ONE_LPT)
    );

    let last_total_shares = last_total_shares + 1500 * ONE_LPT;

    // remove by shares (charlie)
    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        CHARLIE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        charlie(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances,
        vec![100 * ONE_DAI, 100 * ONE_USDT, 100 * ONE_USDC],
        "Incorrect Users tokens balances"
    );
    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares - 300 * ONE_LPT)
    );
    let last_total_shares = last_total_shares - 300 * ONE_LPT;

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        DAVE,
        vec![100 * ONE_DAI, 200 * ONE_USDT, 400 * ONE_USDC],
        BOB,
    );

    // add more liquidity with imbalanced tokens (dave)
    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        DAVE,
        1,
        vec![100 * ONE_DAI, 200 * ONE_USDT, 400 * ONE_USDC],
        dave(),
    )
    .expect("Should successfully add liquidity");
    // Ref
    // "Mint 699699997426210330025 shares for user2, fee is 299999998348895348 shares",
    // "Exchange swap got 59999999669779069 shares",
    // -- DIFF --
    // Common
    // "Mint 699687497426279107411 shares for dave, fee is 312499998280117962 shares",
    // "Exchange swap got 62499999656023592 shares, No referral fee (not implemented)",
    //
    // The difference is due to the implemented fee precision (1e9 in Common vs 1e4 in Ref)

    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        vec![100500 * ONE_DAI, 100600 * ONE_USDT, 100800 * ONE_USDC],
        "Incorrect reserves"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        301200 * ONE_LPT + 699687497426279107411 + 62499999656023592,
        "Incorrect total shares"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699687497426279107411,
        "Incorrect Users share"
    );

    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    let last_share_price = current_share_price;

    assert_eq!(
        current_total_shares,
        last_total_shares + 699687497426279107411 + 62499999656023592
    );
    let last_total_shares = current_total_shares;

    // remove by tokens (charlie)
    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        CHARLIE,
        550 * ONE_LPT,
        vec![1 * ONE_DAI, 500 * ONE_USDT, 1 * ONE_USDC],
        charlie(),
    )
    .expect("Should successfully remove liquidity. Err: {err:?}");
    // "LP charlie removed 502623448746385017122 shares by given tokens, and fee is 623853418327862983 shares",
    // "Exchange swap got 124770683665572596 shares, No referral fee (not implemented)",

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT - 502623448746385017122,
        "Incorrect users share"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances,
        vec![101 * ONE_DAI, 600 * ONE_USDT, 101 * ONE_USDC],
        "Incorrect Users tokens balances"
    );

    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        vec![100499 * ONE_DAI, 100100 * ONE_USDT, 100799 * ONE_USDC],
        "Incorrect reserves"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        last_total_shares - 502623448746385017122 + 124770683665572596,
        "Incorrect total shares"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1200 * ONE_LPT - 502623448746385017122,
        "Incorrect users share"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699687497426279107411,
        "Incorrect users share"
    );
    let (current_share_price, _) = share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    let last_share_price = current_share_price;
    let last_total_shares = last_total_shares - 502623448746385017122 + 124770683665572596;

    // transfer some LPT to from charlie to dave
    _ = psp22_utils::transfer(&mut session, stable_swap, dave(), 100 * ONE_LPT, CHARLIE);

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502623448746385017122,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699687497426279107411 + 100 * ONE_LPT,
        "Incorrect user balance"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    // dave remove by shares trigger slippage
    let res = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 298 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        res,
        StablePoolError::InsufficientOutputAmount(),
        "Should return correct error"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    // dave remove by tokens trigger slippage
    let res = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 298 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        res,
        StablePoolError::InsufficientLiquidityBurned(),
        "Should return correct error"
    );

    assert_eq!(
        share_price_and_total_shares(&mut session, stable_swap),
        (last_share_price, last_total_shares),
        "Incorrect share price and/or total shares"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502623448746385017122,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699687497426279107411 + 100 * ONE_LPT,
        "Incorrect user balance"
    );

    // dave remove by share
    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        300 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        dave(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502623448746385017122,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699687497426279107411 - 200 * ONE_LPT,
        "Incorrect user balance"
    );
    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert_eq!(
        current_share_price, last_share_price,
        "Incorrect share price"
    );
    assert_eq!(
        current_total_shares,
        last_total_shares - 300 * ONE_LPT,
        "Incorrect total shares"
    );
    let last_total_shares = last_total_shares - 300 * ONE_LPT;

    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        DAVE,
        499 * ONE_LPT,
        vec![498 * ONE_DAI, 0 * ONE_USDT, 0 * ONE_USDC],
        dave(),
    )
    .expect("Should successfully remove liquidity");
    // "LP dave removed 498621166533015126275 shares by given tokens, and fee is 622396225347309589 shares",
    // "Exchange swap got 124479245069461917 shares, No referral fee (not implemented)",

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, charlie()),
        1100 * ONE_LPT - 502623448746385017122,
        "Incorrect user balance"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        699687497426279107411 - 200 * ONE_LPT - 498621166533015126275,
        "Incorrect user balance"
    );
    let last_total_shares = last_total_shares - 498621166533015126275 + 124479245069461917;
    let (current_share_price, current_total_shares) =
        share_price_and_total_shares(&mut session, stable_swap);
    assert!(
        current_share_price > last_share_price,
        "Incorrect share price"
    );
    assert_eq!(
        current_total_shares, last_total_shares,
        "Incorrect total shares"
    );

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        EVA,
        vec![
            100_000_000_000 * ONE_DAI,
            100_000_000_000 * ONE_USDT,
            100_000_000_000 * ONE_USDC,
        ],
        BOB,
    );
    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        EVA,
        1,
        vec![
            100_000_000_000 * ONE_DAI,
            100_000_000_000 * ONE_USDT,
            100_000_000_000 * ONE_USDC,
        ],
        eva(),
    )
    .expect("Should successfully add liquidity");
    // "Mint 299997824748271184577117019598 shares for eva, fee is 933133378066387864612868 shares",
    // "Exchange swap got 186626675613277572922573 shares, No referral fee (not implemented)",

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, eva()),
        299997824748271184577117019598,
        "Incorrect user balance"
    );
    let last_total_shares =
        last_total_shares + 299997824748271184577117019598 + 186626675613277572922573;
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        last_total_shares,
        "Incorrect total shares"
    );
}

/// Test withdrawing all liquidity with all shares
#[drink::test]
fn test_02(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // remove by shares
    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT,
        vec![1 * ONE_DAI, 1 * ONE_USDT, 1 * ONE_USDC],
        bob(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(psp22_utils::balance_of(&mut session, stable_swap, bob()), 0);
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), 0);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(balances, initial_supply, "Incorrect Users tokens balances");
}

/// Test withdrawing all liquidity by amounts
#[drink::test]
fn test_03(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT,
        initial_reserves,
        bob(),
    )
    .expect("Should successfully remove liquidity");

    assert_eq!(psp22_utils::balance_of(&mut session, stable_swap, bob()), 0);
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), 0);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(balances, initial_supply, "Incorrect Users tokens balances");
}

/// Test withdrawing all liquidity with shares - 1
#[drink::test]
fn test_04(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT - 1,
        initial_reserves.clone(),
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientOutputAmount(),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT - 1,
        initial_reserves,
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientLiquidityBurned(),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        300000 * ONE_LPT
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300000 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Test withdrawing single token whole reserve
#[drink::test]
fn test_05(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        300000 * ONE_LPT,
        vec![initial_reserves[0], 0, 0],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");

    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        300000 * ONE_LPT
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300000 * ONE_LPT
    );
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Test withdrawing all liquidity with shares - 1 (with different initial reserves)
#[drink::test]
fn test_06(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![543257 * ONE_DAI, 123123 * ONE_USDT, 32178139 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    let (shares, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        shares - 1,
        initial_reserves.clone(),
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientOutputAmount(),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares - 1,
        initial_reserves,
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::InsufficientLiquidityBurned(),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        shares
    );
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), shares);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Test withdrawing single token whole reserve (with different initial reserves)
#[drink::test]
fn test_07(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![543257 * ONE_DAI, 123123 * ONE_USDT, 32178139 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let initial_supply_sub_reserves = initial_supply
        .iter()
        .zip(initial_reserves.iter())
        .map(|(supply, reserve)| supply - reserve)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    let (shares, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares,
        vec![initial_reserves[0], 0, 0],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares,
        vec![0, initial_reserves[1], 0],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        shares,
        vec![0, 0, initial_reserves[2]],
        bob(),
    )
    .expect_err("Liquidity withdraw should fail");
    assert_eq!(
        err,
        StablePoolError::MathError(MathError::DivByZero(1)),
        "Should return appropriate error"
    );

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        shares
    );
    assert_eq!(psp22_utils::total_supply(&mut session, stable_swap), shares);
    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect::<Vec<u128>>();
    assert_eq!(
        balances, initial_supply_sub_reserves,
        "Incorrect Users tokens balances"
    );
}

/// Tests that after depositing X tokens amounts for L shares, user cannot withdraw X tokens amounts for L - 1 shares
#[drink::test]
fn test_08(mut session: Session) {
    let initial_reserves = vec![100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply: Vec<u128> = initial_reserves.iter().map(|amount| amount * 10).collect();
    let amp_coef = 10_000u128;
    let trade_fee = 2_500_000u32;
    let protocol_fee = 200_000_000u32;
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![12, 12],
        initial_supply.clone(),
        amp_coef,
        trade_fee,
        protocol_fee,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    _ = stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0],               // in USDT
        tokens[1],               // out USDC
        123 * ONE_USDT + 132312, // amount_in
        1,                       // min_token_out
        bob(),
    )
    .expect("Should successfully swap");

    let total_shares = psp22_utils::total_supply(&mut session, stable_swap);
    let share = total_shares / 20; // 4.999...% -  5%

    let deposit_amounts =
        stable_swap::get_amounts_for_liquidity_mint(&mut session, stable_swap, share)
            .expect("Should compute");

    let (share_mint, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        deposit_amounts.clone(),
        bob(),
    )
    .expect("Should mint LPT");

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        share_mint - 1,
        deposit_amounts,
        bob(),
    )
    .expect_err("Should fail to remove lpt");

    assert_eq!(
        StablePoolError::InsufficientLiquidityBurned(),
        err,
        "Should be insufficient"
    );
}

/// Tests that after withdrawing X tokens amounts for L shares, user cannot deposit X tokens amounts for L + 1 shares
#[drink::test]
fn test_09(mut session: Session) {
    let initial_reserves = vec![100000 * ONE_USDT * ONE_USDT, 100000 * ONE_USDC * ONE_USDC];
    let initial_supply: Vec<u128> = initial_reserves.iter().map(|amount| amount * 10).collect();
    let amp_coef = 10_000u128;
    let trade_fee = 2_500_000u32;
    let protocol_fee = 200_000_000u32;
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![12, 12],
        initial_supply.clone(),
        amp_coef,
        trade_fee,
        protocol_fee,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    _ = stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0],                          // in USDT
        tokens[1],                          // out USDC
        123 * ONE_USDT * ONE_USDC + 132312, // amount_in
        1,                                  // min_token_out
        bob(),
    )
    .expect("Should successfully swap");

    let total_shares = psp22_utils::total_supply(&mut session, stable_swap);
    let share = total_shares / 20; // 4.999...% -  5%

    let withdraw_amounts =
        stable_swap::get_amounts_for_liquidity_burn(&mut session, stable_swap, share)
            .expect("Should compute");
    let (share_burn, _) = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        u128::MAX,
        withdraw_amounts.clone(),
        bob(),
    )
    .expect("Should burn LPT");

    let err = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        share_burn + 1,
        withdraw_amounts.clone(),
        bob(),
    )
    .expect_err("Should fail to mint lpt");

    assert_eq!(
        StablePoolError::InsufficientLiquidityMinted(),
        err,
        "Should be insufficient"
    );
}

/// Tests that after depositing X tokens amounts for L shares, user cannot withdraw X tokens amounts for L - 1 shares (using remove_by_shares method)
#[drink::test]
fn test_10(mut session: Session) {
    let initial_reserves = vec![100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply: Vec<u128> = initial_reserves.iter().map(|amount| amount * 10).collect();
    let amp_coef = 10_000u128;
    let trade_fee = 2_500_000u32;
    let protocol_fee = 200_000_000u32;
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![12, 12],
        initial_supply.clone(),
        amp_coef,
        trade_fee,
        protocol_fee,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    _ = stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0],               // in USDT
        tokens[1],               // out USDC
        123 * ONE_USDT + 132312, // amount_in
        1,                       // min_token_out
        bob(),
    )
    .expect("Should successfully swap");

    let total_shares = psp22_utils::total_supply(&mut session, stable_swap);
    let share = total_shares / 20; // 4.999...% -  5%

    let deposit_amounts =
        stable_swap::get_amounts_for_liquidity_mint(&mut session, stable_swap, share)
            .expect("Should compute");

    let (share_mint, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        deposit_amounts.clone(),
        bob(),
    )
    .expect("Should mint LPT");

    let err = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        share_mint - 1,
        deposit_amounts,
        bob(),
    )
    .expect_err("Should fail to remove lpt");

    assert_eq!(
        StablePoolError::InsufficientOutputAmount(),
        err,
        "Should be insufficient"
    );
}

/// Tests that after withdrawing X tokens amounts for L shares, user cannot deposit X tokens amounts for L + 1 shares (using remove_by_shares method)
#[drink::test]
fn test_11(mut session: Session) {
    let initial_reserves = vec![100000 * ONE_USDT * ONE_USDT, 100000 * ONE_USDC * ONE_USDC];
    let initial_supply: Vec<u128> = initial_reserves.iter().map(|amount| amount * 10).collect();
    let amp_coef = 10_000u128;
    let trade_fee = 2_500_000u32;
    let protocol_fee = 200_000_000u32;
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![12, 12],
        initial_supply.clone(),
        amp_coef,
        trade_fee,
        protocol_fee,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    _ = stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0],                          // in USDT
        tokens[1],                          // out USDC
        123 * ONE_USDT * ONE_USDC + 132312, // amount_in
        1,                                  // min_token_out
        bob(),
    )
    .expect("Should successfully swap");

    let total_shares = psp22_utils::total_supply(&mut session, stable_swap);
    let share = total_shares / 20; // 4.999...% -  5%

    let withdraw_amounts = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        share,
        vec![1, 1],
        bob(),
    )
    .expect("Should burn LPT");

    let err = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        share + 1,
        withdraw_amounts.clone(),
        bob(),
    )
    .expect_err("Should fail to mint lpt");

    assert_eq!(
        StablePoolError::InsufficientLiquidityMinted(),
        err,
        "Should be insufficient"
    );
}

#[drink::test]
fn test_lp_withdraw_all_but_no_more() {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let charlie_input = vec![1_434_543 * ONE_USDT, 1_112_323 * ONE_USDC];
    let dave_input = vec![1131 * 105 * ONE_USDT, 1157 * 105 * ONE_USDC]; // treat as 105 parts of (1131, 1157)
    let initial_supply: Vec<u128> = charlie_input.iter().map(|amount| amount * 10).collect();

    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![6, 6],
        initial_supply.clone(),
        10_000,
        0,
        0,
        BOB,
        vec![],
    );

    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        CHARLIE,
        vec![1_434_543 * ONE_USDT, 1_112_323 * ONE_USDC],
        BOB,
    );
    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens,
        DAVE,
        vec![1131 * 105 * ONE_USDT, 1157 * 105 * ONE_USDC],
        BOB,
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        CHARLIE,
        1,
        charlie_input.clone(),
        charlie(),
    )
    .expect("Charlie should successfully add liquidity");

    let (shares, _) = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        DAVE,
        1,
        dave_input.clone(),
        dave(),
    )
    .expect("Dave should successfully add liquidity");

    // 105 times withdraw (1131, 1157) which should withdraw the whole Dave's share
    for _ in 0..105 {
        let withdraw = vec![1131 * ONE_USDT, 1157 * ONE_USDC]; // 1/105 of Dave's
        _ = stable_swap::remove_liquidity_by_amounts(
            &mut session,
            stable_swap,
            DAVE,
            shares, // just an upper bound
            withdraw,
            dave(),
        )
        .expect("Should successfully remove liquidity");
    }

    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, dave()),
        0,
        "Dave shouldn't have any LPT left",
    );

    _ = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        DAVE,
        10,
        vec![1, 1], // at least withdraw something
        dave(),
    )
    .expect_err("Should not successfully remove liquidity");
}

#[drink::test]
fn test_for_zero_deposit(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    // setup max allowance for stable swap contract on both tokens
    transfer_and_increase_allowance(
        &mut session,
        stable_swap,
        tokens.clone(),
        CHARLIE,
        vec![500 * ONE_DAI, 500 * ONE_USDT, 500 * ONE_USDC],
        BOB,
    );

    let err = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        CHARLIE,
        0,
        vec![0, 0, 0],
        charlie(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        err,
        StablePoolError::ZeroAmounts(),
        "Should return appropriate error"
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        CHARLIE,
        0,
        vec![1, 0, 0],
        charlie(),
    )
    .expect("Should min liqudity");
}

#[drink::test]
fn test_for_zero_withdrawal(mut session: Session) {
    seed_account(&mut session, CHARLIE);
    seed_account(&mut session, DAVE);
    seed_account(&mut session, EVA);

    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply = initial_reserves
        .iter()
        .map(|amount| amount * 100_000_000_000)
        .collect::<Vec<u128>>();
    let (stable_swap, _) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
        initial_supply.clone(),
        10_000,
        2_500_000,
        200_000_000,
        BOB,
        vec![],
    );

    _ = stable_swap::add_liquidity(
        &mut session,
        stable_swap,
        BOB,
        1,
        initial_reserves.clone(),
        bob(),
    )
    .expect("Should successfully add liquidity");

    let err = stable_swap::remove_liquidity_by_shares(
        &mut session,
        stable_swap,
        BOB,
        1,
        vec![0, 0, 0],
        bob(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        err,
        StablePoolError::ZeroAmounts(),
        "Should return appropriate error"
    );

    let err = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        0,
        vec![0, 0, 0],
        bob(),
    )
    .expect_err("Should return an error");

    assert_eq!(
        err,
        StablePoolError::ZeroAmounts(),
        "Should return appropriate error"
    );

    _ = stable_swap::remove_liquidity_by_amounts(
        &mut session,
        stable_swap,
        BOB,
        1,
        vec![1, 0, 0],
        bob(),
    )
    .expect("Should burn liquidity");
}
