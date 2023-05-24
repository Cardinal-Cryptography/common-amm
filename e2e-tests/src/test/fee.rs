use anyhow::{
    ensure,
    Result,
};
use assert2::assert;

use ink_wrapper_types::util::ToAccountId;

use crate::{
    factory_contract::Factory,
    test::setup::{
        Contracts,
        TestFixture,
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
        ZERO_ADDRESS,
    },
};

pub async fn fee(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let factory_contract = contracts.factory_contract;
    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);
    let non_sudo_ink_account_id = non_sudo.account_id().to_account_id();

    let recipient = factory_contract.fee_to(sudo_connection).await??;
    let setter = factory_contract.fee_to_setter(sudo_connection).await??;
    let all_pairs_length = factory_contract.all_pairs_length(sudo_connection).await??;

    assert!(recipient == zero_account_id);
    assert!(setter == non_sudo_ink_account_id);
    assert!(all_pairs_length == EXPECTED_INITIAL_ALL_PAIRS_LENGTH);

    Ok(())
}

pub async fn set_fee(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        ..
    } = contracts;

    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);
    let token_a_account: ink_primitives::AccountId = (*token_a).into();

    let sudo_recipient = factory_contract.fee_to(sudo_connection).await??;

    assert!(sudo_recipient == zero_account_id);

    ensure!(
        factory_contract
            .set_fee_to(sudo_connection, token_a_account)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to(non_sudo_connection, token_a_account)
        .await?;

    let non_sudo_recipient = factory_contract.fee_to(non_sudo_connection).await??;

    assert!(non_sudo_recipient == token_a_account);

    Ok(())
}

pub async fn set_fee_setter(test_fixture: &TestFixture) -> Result<()> {
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a,
        ..
    } = contracts;

    let non_sudo_ink_account_id = non_sudo.account_id().to_account_id();
    let token_a_account: ink_primitives::AccountId = (*token_a).into();

    let setter_before = factory_contract.fee_to_setter(sudo_connection).await??;

    assert!(setter_before == non_sudo_ink_account_id);

    ensure!(
        factory_contract
            .set_fee_to_setter(sudo_connection, token_a_account)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to_setter(non_sudo_connection, token_a_account)
        .await?;

    let setter_after = factory_contract
        .fee_to_setter(non_sudo_connection)
        .await??;

    assert!(setter_after == token_a_account);

    Ok(())
}
