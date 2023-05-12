use anyhow::Result;
use log;

use crate::factory_contract::Factory;

use crate::test::setup::setup_test;

// const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000000000000000000000000000";
const ZERO_ADDRESS: [u8; 32] = [0; 32];

#[tokio::test]
async fn fee_to() -> Result<()> {
    let test_fixture = setup_test().await?;
    let zero_account_id = ink_primitives::AccountId::from(ZERO_ADDRESS);

    let connection = test_fixture.connection;
    let factory_contract = test_fixture.contracts.factory_contract;
    let recipient = factory_contract.fee_to(&connection).await??;

    assert_eq!(recipient, zero_account_id);

    return Ok(())
}
