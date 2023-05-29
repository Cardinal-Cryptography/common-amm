use anyhow::{
    ensure,
    Result,
};
use assert2::assert;

use ink_wrapper_types::util::ToAccountId;

use crate::{
    factory_contract::Factory,
    test::setup::{
        setup_test,
        Contracts,
        TestFixture,
        ZERO_ADDRESS,
    },
};

#[tokio::test]
pub async fn factory_contract_set_up_correctly() -> Result<()> {
    let TestFixture {
        wealthy_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let factory_contract = contracts.factory_contract;
    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);
    let regular_account_id = regular.account_id().to_account_id();

    let recipient = factory_contract.fee_to(&wealthy_connection).await??;
    let setter = factory_contract
        .fee_to_setter(&wealthy_connection)
        .await??;
    let all_pairs_length = factory_contract
        .all_pairs_length(&wealthy_connection)
        .await??;

    assert!(recipient == zero_account_id);
    assert!(setter == regular_account_id);
    assert!(all_pairs_length == 0);

    Ok(())
}

#[tokio::test]
pub async fn set_fee() -> Result<()> {
    let TestFixture {
        wealthy_connection,
        regular_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        token_a,
        ..
    } = contracts;

    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);
    let wealthy_recipient = factory_contract.fee_to(&wealthy_connection).await??;
    let regular_account_id = regular.account_id().to_account_id();

    assert!(wealthy_recipient == zero_account_id);

    ensure!(
        factory_contract
            .set_fee_to(&wealthy_connection, token_a.into())
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to(&regular_connection, regular_account_id)
        .await?;

    let regular_recipient = factory_contract.fee_to(&regular_connection).await??;

    assert!(regular_recipient == regular_account_id);

    Ok(())
}

#[tokio::test]
pub async fn set_fee_setter() -> Result<()> {
    let TestFixture {
        wealthy_connection,
        wealthy,
        regular_connection,
        regular,
        contracts,
        ..
    } = setup_test().await?;

    let factory_contract = contracts.factory_contract;

    let wealthy_account_id = wealthy.account_id().to_account_id();
    let regular_account_id = regular.account_id().to_account_id();
    let setter_before = factory_contract
        .fee_to_setter(&wealthy_connection)
        .await??;

    assert!(setter_before == regular_account_id);

    ensure!(
        factory_contract
            .set_fee_to_setter(&wealthy_connection, wealthy_account_id)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to_setter(&regular_connection, wealthy_account_id)
        .await?;

    let setter_after = factory_contract
        .fee_to_setter(&regular_connection)
        .await??;

    assert!(setter_after == wealthy_account_id);

    Ok(())
}
