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
};

const DEADLINE: u64 = 111_111_111_111_111_111;
const AMOUNT_AVAILABLE_FOR_WITHDRAWAL: Balance = 10_000;

#[tokio::test]
pub async fn add_liquidity_via_router() -> Result<()> {
    println!("Running `add_liquidity_via_router` test.");
    let TestFixture {
        sudo_connection,
        sudo,
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
            &sudo_connection,
            router_contract.into(),
            AMOUNT_AVAILABLE_FOR_WITHDRAWAL,
        )
        .await?;

    let all_pairs_length_before = factory_contract
        .all_pairs_length(&sudo_connection)
        .await??;

    let sudo_ink_account_id = sudo.account_id().to_account_id();

    router_contract
        .add_liquidity_native(
            &sudo_connection,
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
        .all_pairs_length(&sudo_connection)
        .await??;

    assert!(all_pairs_length_after == expected_all_pairs_length);

    Ok(())
}
