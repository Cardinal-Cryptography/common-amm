use amm::{
    helpers::ZERO_ADDRESS,
    Balance,
    Timestamp,
};
use amm_helpers::types::WrappedU256;
use ink::primitives::AccountId;

#[ink::storage_item]
#[derive(Debug)]
pub struct Data {
    pub factory: AccountId,
    pub token_0: AccountId,
    pub token_1: AccountId,
    pub reserve_0: Balance,
    pub reserve_1: Balance,
    pub block_timestamp_last: Timestamp,
    pub price_0_cumulative_last: WrappedU256,
    pub price_1_cumulative_last: WrappedU256,
    pub k_last: WrappedU256,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            factory: ZERO_ADDRESS.into(),
            token_0: ZERO_ADDRESS.into(),
            token_1: ZERO_ADDRESS.into(),
            reserve_0: 0,
            reserve_1: 0,
            block_timestamp_last: 0,
            price_0_cumulative_last: Default::default(),
            price_1_cumulative_last: Default::default(),
            k_last: Default::default(),
        }
    }
}
