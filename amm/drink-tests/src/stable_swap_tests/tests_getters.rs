use super::*;

// ref https://github.com/ref-finance/ref-contracts/blob/d241d7aeaa6250937b160d56e5c4b5b48d9d97f7/ref-exchange/tests/test_stable_pool.rs#L23
#[drink::test]
fn test_01(mut session: Session) {
    let initial_reserves = vec![100000 * ONE_DAI, 100000 * ONE_USDT, 100000 * ONE_USDC];
    let initial_supply: Vec<u128> = initial_reserves.iter().map(|amount| amount * 10).collect();
    let amp_coef = 10_000u128;
    let trade_fee = 2_500_000u32;
    let protocol_fee = 200_000_000u32;
    let (stable_swap, tokens) = setup_stable_swap_with_tokens(
        &mut session,
        vec![18, 6, 6],
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

    assert_eq!(
        stable_swap::tokens(&mut session, stable_swap),
        tokens,
        "Incorrect token accounts"
    );
    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        initial_reserves,
        "Incorrect reserves"
    );
    assert_eq!(
        stable_swap::amp_coef(&mut session, stable_swap),
        Ok(amp_coef),
        "Incorrect A"
    );
    assert_eq!(
        stable_swap::fees(&mut session, stable_swap),
        (trade_fee, protocol_fee),
        "Incorrect fees"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300_000 * ONE_LPT,
        "Incorrect LP token supply"
    );
    assert_eq!(
        psp22_utils::balance_of(&mut session, stable_swap, bob()),
        300_000 * ONE_LPT,
        "Incorrect Users LP token balance"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, bob()))
        .collect();
    assert_eq!(
        balances,
        initial_supply
            .iter()
            .zip(initial_reserves)
            .map(|(init_token_supply, init_reserve)| init_token_supply - init_reserve)
            .collect::<Vec<u128>>(),
        "Incorrect Users tokens balances"
    );

    _ = stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0], // DAI
        tokens[2], // USDC
        ONE_DAI,   // amount_in
        1,         // min_token_out
        charlie(),
    )
    .expect("Should successfully swap");

    _ = stable_swap::swap_exact_in(
        &mut session,
        stable_swap,
        BOB,
        tokens[0], // DAI
        tokens[1], // USDT
        ONE_DAI,   // amount_in
        1,         // min_token_out
        charlie(),
    )
    .expect("Should successfully swap. ");

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, charlie()))
        .collect();
    assert_eq!(
        balances,
        vec![0, 997499, 997499],
        "Incorrect Users tokens balances"
    );

    let balances: Vec<u128> = tokens
        .iter()
        .map(|&token| psp22_utils::balance_of(&mut session, token, stable_swap))
        .collect();
    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        balances,
        "Pool reserves and token balances mismatch"
    );

    assert_eq!(
        stable_swap::reserves(&mut session, stable_swap),
        vec![
            100002 * ONE_DAI,
            99999 * ONE_USDT + 2501, // -- DIFF -- 99999 * ONE_USDT + 2500
            99999 * ONE_USDC + 2501  // -- DIFF -- 99999 * ONE_USDC + 2500
        ],
        "Incorrect reserves"
    );
    assert_eq!(
        psp22_utils::total_supply(&mut session, stable_swap),
        300000 * ONE_LPT + 498999996725367 + 498999993395420, // -- DIFF -- 300000 * ONE_LPT + 499999996666583 + 499999993277742
        "Incorrect LP token supply"
    );
}
