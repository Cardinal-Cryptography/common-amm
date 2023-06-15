use anyhow::{
    ensure,
    Result,
};
use assert2::assert;

use aleph_client::{
    pallets::contract::ContractsUserApi,
    KeyPair,
    SignedConnection,
    TxStatus,
};
use ink_wrapper_types::util::ToAccountId;

use crate::{
    factory_contract,
    factory_contract::Factory,
    pair_contract,
    psp22_token,
    test::setup::{
        get_env,
        replenish_account,
        set_up_connections,
        set_up_factory_contract,
        set_up_key_pairs,
        set_up_logger,
        set_up_psp22_token,
        upload_code_pair_contract,
        DEFAULT_NODE_ADDRESS,
        INITIAL_TRANSFER,
        TOKEN_A_NAME,
        TOKEN_A_SYMBOL,
        ZERO_ADDRESS,
    },
};

pub struct FactoryTestAccounts {
    wealthy_account: ink_primitives::AccountId,
    regular_account: ink_primitives::AccountId,
    zero_account: ink_primitives::AccountId,
}

struct FactoryTestSetup {
    factory_contract: factory_contract::Instance,
    token_a: psp22_token::Instance,
    wealthy_connection: SignedConnection,
    regular_connection: SignedConnection,
    wealthy_account: ink_primitives::AccountId,
    regular_account: ink_primitives::AccountId,
    zero_account: ink_primitives::AccountId,
}

struct FactoryTestTeardown {
    factory_contract: factory_contract::Instance,
    token_a: psp22_token::Instance,
    connection: SignedConnection,
}

pub fn set_up_accounts(wealthy: &KeyPair, regular: &KeyPair) -> FactoryTestAccounts {
    let wealthy_account = wealthy.account_id().to_account_id();
    let regular_account = regular.account_id().to_account_id();
    let zero_account = ink_primitives::AccountId::from(ZERO_ADDRESS);

    FactoryTestAccounts {
        wealthy_account,
        regular_account,
        zero_account,
    }
}

async fn set_up_factory_test() -> Result<FactoryTestSetup> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());

    let (wealthy, regular) = set_up_key_pairs();

    let FactoryTestAccounts {
        wealthy_account,
        regular_account,
        zero_account,
    } = set_up_accounts(&wealthy, &regular);

    let (wealthy_connection, regular_connection) =
        set_up_connections(&node_address, wealthy, regular.clone()).await;

    upload_code_pair_contract(&wealthy_connection).await?;

    let factory_contract = set_up_factory_contract(
        &wealthy_connection,
        regular_account,
        pair_contract::CODE_HASH.into(),
    )
    .await?;
    let token_a = set_up_psp22_token(&wealthy_connection, TOKEN_A_NAME, TOKEN_A_SYMBOL).await?;

    let factory_test_setup = FactoryTestSetup {
        factory_contract,
        token_a,
        wealthy_connection,
        regular_connection,
        wealthy_account,
        regular_account,
        zero_account,
    };

    Ok(factory_test_setup)
}

async fn tear_down_factory_test(factory_test_teardown: FactoryTestTeardown) -> Result<()> {
    let FactoryTestTeardown {
        factory_contract,
        token_a,
        connection,
    } = factory_test_teardown;

    token_a.terminate(&connection).await?;
    connection
        .remove_code(psp22_token::CODE_HASH.into(), TxStatus::InBlock)
        .await?;
    factory_contract.terminate(&connection).await?;
    connection
        .remove_code(factory_contract::CODE_HASH.into(), TxStatus::InBlock)
        .await?;
    connection
        .remove_code(pair_contract::CODE_HASH.into(), TxStatus::InBlock)
        .await?;

    Ok(())
}

#[tokio::test]
pub async fn factory_contract_set_up_correctly() -> Result<()> {
    set_up_logger();

    let FactoryTestSetup {
        factory_contract,
        token_a,
        wealthy_connection,
        regular_account,
        zero_account,
        ..
    } = set_up_factory_test().await?;

    let recipient = factory_contract.fee_to(&wealthy_connection).await??;
    let setter = factory_contract
        .fee_to_setter(&wealthy_connection)
        .await??;
    let all_pairs_length = factory_contract
        .all_pairs_length(&wealthy_connection)
        .await??;

    assert!(recipient == zero_account);
    assert!(setter == regular_account);
    assert!(all_pairs_length == 0);

    let factory_test_teardown = FactoryTestTeardown {
        factory_contract,
        token_a,
        connection: wealthy_connection,
    };
    tear_down_factory_test(factory_test_teardown).await?;

    Ok(())
}

#[tokio::test]
pub async fn set_fee() -> Result<()> {
    set_up_logger();

    let FactoryTestSetup {
        factory_contract,
        token_a,
        wealthy_connection,
        regular_connection,
        regular_account,
        zero_account,
        ..
    } = set_up_factory_test().await?;

    let wealthy_recipient = factory_contract.fee_to(&wealthy_connection).await??;

    assert!(wealthy_recipient == zero_account);

    ensure!(
        factory_contract
            .set_fee_to(&wealthy_connection, token_a.into())
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    let dest = aleph_client::AccountId::new(*regular_account.as_ref());
    replenish_account(&wealthy_connection, dest, INITIAL_TRANSFER).await?;

    factory_contract
        .set_fee_to(&regular_connection, regular_account)
        .await?;

    let regular_recipient = factory_contract.fee_to(&regular_connection).await??;

    assert!(regular_recipient == regular_account);

    let factory_test_teardown = FactoryTestTeardown {
        factory_contract,
        token_a,
        connection: wealthy_connection,
    };
    tear_down_factory_test(factory_test_teardown).await?;

    Ok(())
}

#[tokio::test]
pub async fn set_fee_setter() -> Result<()> {
    set_up_logger();

    let FactoryTestSetup {
        factory_contract,
        token_a,
        wealthy_connection,
        regular_connection,
        wealthy_account,
        regular_account,
        ..
    } = set_up_factory_test().await?;

    let setter_before = factory_contract
        .fee_to_setter(&wealthy_connection)
        .await??;

    assert!(setter_before == regular_account);

    ensure!(
        factory_contract
            .set_fee_to_setter(&wealthy_connection, wealthy_account)
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    let dest = aleph_client::AccountId::new(*regular_account.as_ref());
    replenish_account(&wealthy_connection, dest, INITIAL_TRANSFER).await?;

    factory_contract
        .set_fee_to_setter(&regular_connection, wealthy_account)
        .await?;

    let setter_after = factory_contract
        .fee_to_setter(&regular_connection)
        .await??;

    assert!(setter_after == wealthy_account);

    let factory_test_teardown = FactoryTestTeardown {
        factory_contract,
        token_a,
        connection: wealthy_connection,
    };
    tear_down_factory_test(factory_test_teardown).await?;

    Ok(())
}
