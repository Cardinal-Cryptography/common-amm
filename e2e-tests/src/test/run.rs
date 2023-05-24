use anyhow::Result;

use crate::test::{
    fee::{
        fee,
        set_fee,
        set_fee_setter,
    },
    tokens::{
        burn_liquidity_provider_token,
        create_pair,
        mint_pair,
        swap_tokens,
    },
    setup::setup_test,
};

#[tokio::test]
async fn e2e_tests() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();

    let test_fixture = setup_test().await?;

    fee(&test_fixture).await?;
    set_fee(&test_fixture).await?;
    set_fee_setter(&test_fixture).await?;

    create_pair(&test_fixture).await?;
    mint_pair(&test_fixture).await?;
    swap_tokens(&test_fixture).await?;
    burn_liquidity_provider_token(&test_fixture).await?;

    Ok(())
}
