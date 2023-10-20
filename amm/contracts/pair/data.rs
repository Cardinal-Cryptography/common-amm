use amm::{
    helpers::ZERO_ADDRESS,
    Balance,
    Timestamp,
};
use amm_helpers::types::WrappedU256;
use ink::primitives::AccountId;
use primitive_types::U256;
use sp_arithmetic::{
    FixedPointNumber,
    FixedU128,
};

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

#[inline]
pub fn update_cumulative(
    price_0_cumulative_last: WrappedU256,
    price_1_cumulative_last: WrappedU256,
    time_elapsed: U256,
    reserve_0: Balance,
    reserve_1: Balance,
) -> (WrappedU256, WrappedU256) {
    let price_cumulative_last_0: WrappedU256 = U256::from(
        FixedU128::checked_from_rational(reserve_1, reserve_0)
            .unwrap_or_default()
            .into_inner(),
    )
    .saturating_mul(time_elapsed)
    .saturating_add(price_0_cumulative_last.into())
    .into();
    let price_cumulative_last_1: WrappedU256 = U256::from(
        FixedU128::checked_from_rational(reserve_0, reserve_1)
            .unwrap_or_default()
            .into_inner(),
    )
    .saturating_mul(time_elapsed)
    .saturating_add(price_1_cumulative_last.into())
    .into();
    (price_cumulative_last_0, price_cumulative_last_1)
}
