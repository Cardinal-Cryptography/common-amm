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
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
        ZERO_ADDRESS,
    },
};

#[tokio::test]
pub async fn fee() -> Result<()> {
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
    assert!(all_pairs_length == EXPECTED_INITIAL_ALL_PAIRS_LENGTH);

    Ok(())
}

#[tokio::test]
pub async fn set_fee() -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
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

    assert!(sudo_recipient == zero_account_id);

    ensure!(
        factory_contract
            .set_fee_to(&sudo_connection, first_token.into())
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to(&non_sudo_connection, first_token.into())
        .await?;

    let non_sudo_recipient = factory_contract.fee_to(&non_sudo_connection).await??;

    assert!(non_sudo_recipient == first_token.into());

    Ok(())
}

#[tokio::test]
pub async fn set_fee_setter() -> Result<()> {
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

    let non_sudo_ink_account_id = non_sudo.account_id().to_account_id();

    let setter_before = factory_contract.fee_to_setter(&sudo_connection).await??;

    assert!(setter_before == non_sudo_ink_account_id);

    ensure!(
        factory_contract
            .set_fee_to_setter(&sudo_connection, first_token.into())
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to_setter(&non_sudo_connection, first_token.into())
        .await?;

    let setter_after = factory_contract
        .fee_to_setter(&non_sudo_connection)
        .await??;

    assert!(setter_after == first_token.into());

    Ok(())
}
