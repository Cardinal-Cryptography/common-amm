use anyhow::{
    ensure,
    Result,
};
use assert2::assert;
use tokio::sync::OnceCell;

use aleph_client::{
    KeyPair,
    SignedConnection,
};
use ink_wrapper_types::{
    util::ToAccountId,
    Connection,
    SignedConnection as _,
    UploadConnection,
};

use crate::{
    factory_contract,
    factory_contract::Factory,
    pair_contract,
    test::setup::{
        get_env,
        random_salt,
        replenish_account,
        set_up_logger,
        try_upload_contract_code,
        DEFAULT_NODE_ADDRESS,
        INITIAL_TRANSFER,
        REGULAR_SEED,
        WEALTHY_SEED,
        ZERO_ADDRESS,
    },
};

static FACTORY_TESTS_CODE_UPLOAD: OnceCell<Result<()>> = OnceCell::const_new();

pub struct FactoryTestAccounts {
    wealthy_account: ink_primitives::AccountId,
    regular_account: ink_primitives::AccountId,
    zero_account: ink_primitives::AccountId,
}

struct FactoryTestSetup {
    wealthy_connection: SignedConnection,
    regular_connection: SignedConnection,
    wealthy_account: ink_primitives::AccountId,
    regular_account: ink_primitives::AccountId,
    zero_account: ink_primitives::AccountId,
    factory_contract: factory_contract::Instance,
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

async fn factory_tests_code_upload() -> Result<()> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());
    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let wealthy_connection = SignedConnection::new(&node_address, wealthy).await;

    wealthy_connection.upload(pair_contract::upload()).await?;
    wealthy_connection
        .upload(factory_contract::upload())
        .await?;

    Ok(())
}

async fn set_up_factory_test() -> Result<FactoryTestSetup> {
    let node_address = get_env("NODE_ADDRESS").unwrap_or(DEFAULT_NODE_ADDRESS.to_string());

    let wealthy = aleph_client::keypair_from_string(WEALTHY_SEED);
    let regular = aleph_client::keypair_from_string(REGULAR_SEED);

    let FactoryTestAccounts {
        wealthy_account,
        regular_account,
        zero_account,
    } = set_up_accounts(&wealthy, &regular);

    let wealthy_connection = SignedConnection::new(&node_address, wealthy).await;
    let regular_connection = SignedConnection::new(&node_address, regular).await;

    try_upload_contract_code(&FACTORY_TESTS_CODE_UPLOAD, factory_tests_code_upload).await?;

    let salt = random_salt();

    let factory_contract = wealthy_connection
        .instantiate(
            factory_contract::Instance::new(regular_account, pair_contract::CODE_HASH.into())
                .with_salt(salt),
        )
        .await?;

    let factory_test_setup = FactoryTestSetup {
        wealthy_connection,
        regular_connection,
        wealthy_account,
        regular_account,
        zero_account,
        factory_contract,
    };

    Ok(factory_test_setup)
}

#[tokio::test]
pub async fn factory_contract_set_up_correctly() -> Result<()> {
    set_up_logger();

    let FactoryTestSetup {
        wealthy_connection,
        regular_account,
        zero_account,
        factory_contract,
        ..
    } = set_up_factory_test().await?;

    let recipient = wealthy_connection.read(factory_contract.fee_to()).await??;
    let setter = wealthy_connection
        .read(factory_contract.fee_to_setter())
        .await??;
    let all_pairs_length = wealthy_connection
        .read(factory_contract.all_pairs_length())
        .await??;

    assert!(recipient == zero_account);
    assert!(setter == regular_account);
    assert!(all_pairs_length == 0);

    Ok(())
}

#[tokio::test]
pub async fn set_fee() -> Result<()> {
    set_up_logger();

    let FactoryTestSetup {
        wealthy_connection,
        regular_connection,
        regular_account,
        zero_account,
        factory_contract,
        ..
    } = set_up_factory_test().await?;

    let fee_recipient = wealthy_connection.read(factory_contract.fee_to()).await??;

    assert!(fee_recipient == zero_account);

    ensure!(
        wealthy_connection
            .exec(factory_contract.set_fee_to(zero_account))
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    let dest = aleph_client::AccountId::new(*regular_account.as_ref());
    replenish_account(&wealthy_connection, dest, INITIAL_TRANSFER).await?;

    regular_connection
        .exec(factory_contract.set_fee_to(regular_account))
        .await?;

    let regular_recipient = regular_connection.read(factory_contract.fee_to()).await??;

    assert!(regular_recipient == regular_account);

    Ok(())
}

#[tokio::test]
pub async fn set_fee_setter() -> Result<()> {
    set_up_logger();

    let FactoryTestSetup {
        wealthy_connection,
        regular_connection,
        wealthy_account,
        regular_account,
        factory_contract,
        ..
    } = set_up_factory_test().await?;

    let setter_before = wealthy_connection
        .read(factory_contract.fee_to_setter())
        .await??;

    assert!(setter_before == regular_account);

    ensure!(
        wealthy_connection
            .exec(factory_contract.set_fee_to_setter(wealthy_account))
            .await
            .is_err(),
        "Call should have errored out - caller is not fee setter!"
    );

    let dest = aleph_client::AccountId::new(*regular_account.as_ref());
    replenish_account(&wealthy_connection, dest, INITIAL_TRANSFER).await?;

    regular_connection
        .exec(factory_contract.set_fee_to_setter(wealthy_account))
        .await?;

    let setter_after = regular_connection
        .read(factory_contract.fee_to_setter())
        .await??;

    assert!(setter_after == wealthy_account);

    Ok(())
}
