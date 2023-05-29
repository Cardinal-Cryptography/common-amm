use anyhow::Result;

use aleph_client::Balance;
use ink_wrapper_types::util::ToAccountId;

use crate::{
    factory_contract::Factory,
    psp22_token::PSP22 as TokenPSP22,
    router_contract::Router,
    test::setup::{
        setup_test,
        Contracts,
        TestFixture,
    },
    wnative_contract::{
        Wnative,
        PSP22 as WnativePSP22,
    },
};

const DEADLINE: u64 = 111_111_111_111_111_111;
const AMOUNT_AVAILABLE_FOR_WITHDRAWAL: Balance = 10_000;
const AMOUNT_OUT_MIN: Balance = 1_000;

// TODO: transfer native tokens to make the tests work
#[tokio::test]
pub async fn add_liquidity_via_router() -> Result<()> {
    println!("Running `add_liquidity_via_router` test.");
    let TestFixture {
        wealthy_connection,
        wealthy,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        first_token,
        router_contract,
        ..
    } = contracts;

    first_token
        .approve(
            &wealthy_connection,
            router_contract.into(),
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
        )
        .await?;

    let all_pairs_length_before = factory_contract
        .all_pairs_length(&wealthy_connection)
        .await??;

    let sudo_ink_account_id = wealthy.account_id().to_account_id();

    router_contract
        .add_liquidity_native(
            &wealthy_connection,
            first_token.into(),
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
            sudo_ink_account_id,
            DEADLINE,
        )
        .await?;

    let expected_all_pairs_length = all_pairs_length_before + 1;
    let all_pairs_length_after = factory_contract
        .all_pairs_length(&wealthy_connection)
        .await??;

    assert!(all_pairs_length_after == expected_all_pairs_length);

    Ok(())
}

#[tokio::test]
pub async fn swap_exact_native_for_tokens_via_router() -> Result<()> {
    println!("Running `swap_exact_native_for_tokens_via_router` test.");
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        first_token,
        router_contract,
        wnative_contract,
        ..
    } = contracts;

    let non_sudo_ink_account_id = regular.account_id().to_account_id();

    router_contract
        .swap_exact_native_for_tokens(
            &wealthy_connection,
            AMOUNT_OUT_MIN,
            vec![wnative_contract.into(), first_token.into()],
            non_sudo_ink_account_id,
            DEADLINE,
        )
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn swap_native_for_exact_tokens_via_router() -> Result<()> {
    println!("Running `swap_native_for_exact_tokens_via_router` test.");
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        first_token,
        router_contract,
        wnative_contract,
        ..
    } = contracts;

    let non_sudo_ink_account_id = regular.account_id().to_account_id();

    router_contract
        .swap_native_for_exact_tokens(
            &wealthy_connection,
            AMOUNT_OUT_MIN,
            vec![wnative_contract.into(), first_token.into()],
            non_sudo_ink_account_id,
            DEADLINE,
        )
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn swap_exact_tokens_for_tokens_via_router() -> Result<()> {
    println!("Running `swap_native_for_exact_tokens_via_router` test.");
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;
    let Contracts {
        first_token,
        router_contract,
        wnative_contract,
        ..
    } = contracts;

    wnative_contract.deposit(&wealthy_connection).await?;

    wnative_contract
        .approve(
            &wealthy_connection,
            router_contract.into(),
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
        )
        .await?;

    let non_sudo_ink_account_id = regular.account_id().to_account_id();

    router_contract
        .swap_exact_tokens_for_tokens(
            &wealthy_connection,
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
            AMOUNT_OUT_MIN,
            vec![wnative_contract.into(), first_token.into()],
            non_sudo_ink_account_id,
            DEADLINE,
        )
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn swap_tokens_for_exact_tokens_via_router() -> Result<()> {
    println!("Running `swap_tokens_for_exact_tokens_via_router` test.");
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        first_token,
        router_contract,
        wnative_contract,
        ..
    } = contracts;

    // TODO: Do we need this additional const?
    const AMOUNT_FOR_SWAP: Balance = 100_000;

    wnative_contract.deposit(&wealthy_connection).await?;

    wnative_contract
        .approve(&wealthy_connection, router_contract.into(), AMOUNT_FOR_SWAP)
        .await?;

    let non_sudo_ink_account_id = regular.account_id().to_account_id();

    router_contract
        .swap_tokens_for_exact_tokens(
            &wealthy_connection,
            AMOUNT_OUT_MIN,
            AMOUNT_FOR_SWAP,
            vec![wnative_contract.into(), first_token.into()],
            non_sudo_ink_account_id,
            DEADLINE,
        )
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn add_more_liquidity_via_router() -> Result<()> {
    println!("Running `add_more_liquidity_via_router` test.");
    let TestFixture {
        wealthy_connection,
        wealthy,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        first_token,
        router_contract,
        wnative_contract,
        ..
    } = contracts;

    first_token
        .approve(
            &wealthy_connection,
            router_contract.into(),
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
        )
        .await?;

    Ok(())
}