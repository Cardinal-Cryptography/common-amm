use amm::{
    ensure,
    traits::router::RouterError,
};

use ink::primitives::AccountId;

#[inline]
pub fn tokens_sorted(token_a: AccountId, token_b: AccountId) -> Result<bool, RouterError> {
    ensure!(token_a != token_b, RouterError::IdenticalAddresses);
    Ok(token_a < token_b)
}
