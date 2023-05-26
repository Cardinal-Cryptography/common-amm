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
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = setup_test().await?;

    let factory_contract = contracts.factory_contract;
    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);
    let non_sudo_ink_account_id = non_sudo.account_id().to_account_id();

    let recipient = factory_contract.fee_to(&sudo_connection).await??;
    let setter = factory_contract.fee_to_setter(&sudo_connection).await??;
    let all_pairs_length = factory_contract
        .all_pairs_length(&sudo_connection)
        .await??;

    assert!(recipient == zero_account_id);
    assert!(setter == non_sudo_ink_account_id);
    assert!(all_pairs_length == 0);

    Ok(())
}

#[tokio::test]
pub async fn set_fee() -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        non_sudo,
        contracts,
        ..
    } = setup_test().await?;

    let Contracts {
        factory_contract,
        first_token,
        ..
    } = contracts;

    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);
    let sudo_recipient = factory_contract.fee_to(&sudo_connection).await??;
    let non_sudo_ink_account_id = non_sudo.account_id().to_account_id();

    assert!(sudo_recipient == zero_account_id);

    ensure!(
        factory_contract
            .set_fee_to(&sudo_connection, first_token.into())
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to(&non_sudo_connection, non_sudo_ink_account_id)
        .await?;

    let non_sudo_recipient = factory_contract.fee_to(&non_sudo_connection).await??;

    assert!(non_sudo_recipient == non_sudo_ink_account_id);

    Ok(())
}

#[tokio::test]
pub async fn set_fee_setter() -> Result<()> {
    let TestFixture {
        sudo_connection,
        sudo,
        non_sudo_connection,
        non_sudo,
        contracts,
        ..
    } = setup_test().await?;

    let factory_contract = contracts.factory_contract;

    let sudo_ink_account_id = sudo.account_id().to_account_id();
    let non_sudo_ink_account_id = non_sudo.account_id().to_account_id();
    let setter_before = factory_contract.fee_to_setter(&sudo_connection).await??;

    assert!(setter_before == non_sudo_ink_account_id);

    ensure!(
        factory_contract
            .set_fee_to_setter(&sudo_connection, sudo_ink_account_id)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to_setter(&non_sudo_connection, sudo_ink_account_id)
        .await?;

    let setter_after = factory_contract
        .fee_to_setter(&non_sudo_connection)
        .await??;

    assert!(setter_after == sudo_ink_account_id);

    Ok(())
}
