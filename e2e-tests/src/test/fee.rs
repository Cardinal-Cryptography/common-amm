use anyhow::{
    ensure,
    Result,
};
use log::info;

use aleph_client::SignedConnection;

use crate::{
    factory_contract,
    factory_contract::Factory,
    test::{
        setup::{
            inkify_account_id,
            Contracts,
            TestFixture,
            EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
            ZERO_ADDRESS,
        },
        tokens::all_pairs_length,
    },
};

pub async fn fee(test_fixture: &TestFixture) -> Result<()> {
    println!("Running `fee` test.");
    info!("Running `fee` test.");
    let TestFixture {
        sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract, ..
    } = contracts;

    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);

    fee_to(sudo_connection, factory_contract, &zero_account_id).await?;

    let non_sudo_ink_account_id = inkify_account_id(non_sudo.account_id());

    fee_to_setter(sudo_connection, factory_contract, &non_sudo_ink_account_id).await?;

    all_pairs_length(
        sudo_connection,
        factory_contract,
        EXPECTED_INITIAL_ALL_PAIRS_LENGTH,
    )
    .await?;

    Ok(())
}

pub async fn set_fee(test_fixture: &TestFixture) -> Result<()> {
    println!("Running `set_fee` test.");
    info!("Running `set_fee` test.");
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a: token,
        ..
    } = contracts;

    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);

    let token_a: ink_primitives::AccountId = (*token).into();

    fee_to(sudo_connection, factory_contract, &zero_account_id).await?;

    ensure!(
        factory_contract
            .set_fee_to(sudo_connection, token_a)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to(non_sudo_connection, token_a)
        .await?;

    fee_to(non_sudo_connection, factory_contract, &token_a).await?;

    Ok(())
}

pub async fn set_fee_setter(test_fixture: &TestFixture) -> Result<()> {
    println!("Running `set_fee_setter` test.");
    info!("Running `set_fee_setter` test.");
    let TestFixture {
        sudo_connection,
        non_sudo_connection,
        non_sudo,
        contracts,
        ..
    } = test_fixture;

    let Contracts {
        factory_contract,
        token_a: token,
        ..
    } = contracts;

    let token_a: ink_primitives::AccountId = (*token).into();

    let non_sudo_ink_account_id = inkify_account_id(non_sudo.account_id());

    fee_to_setter(sudo_connection, factory_contract, &non_sudo_ink_account_id).await?;

    ensure!(
        factory_contract
            .set_fee_to_setter(sudo_connection, token_a)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    factory_contract
        .set_fee_to_setter(non_sudo_connection, token_a)
        .await?;

    fee_to_setter(non_sudo_connection, factory_contract, &token_a).await?;

    Ok(())
}

async fn fee_to(
    connection: &SignedConnection,
    factory_contract: &factory_contract::Instance,
    expected_recipient: &ink_primitives::AccountId,
) -> Result<()> {
    let recipient = &factory_contract.fee_to(connection).await??;
    assert_eq!(recipient, expected_recipient);
    Ok(())
}

async fn fee_to_setter(
    connection: &SignedConnection,
    factory_contract: &factory_contract::Instance,
    expected_setter: &ink_primitives::AccountId,
) -> Result<()> {
    let setter = &factory_contract.fee_to_setter(connection).await??;
    assert_eq!(setter, expected_setter);
    Ok(())
}
