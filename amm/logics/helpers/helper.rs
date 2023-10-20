use crate::Balance;
use amm_helpers::types::WrappedU256;

use primitive_types::U256;
use sp_arithmetic::{
    FixedPointNumber,
    FixedU128,
};

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into())
        }
    }};
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

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum HelperError {
    IdenticalAddresses,
    ZeroAddress,
    InsufficientAmount,
    InsufficientLiquidity,
    DivByZero,
    CastOverflow,
    MulOverFlow,
    AddOverFlow,
    DivByZero2,
    CastOverflow2,
    InvalidPath,
    SubUnderFlow,
    PairNotFound,
}
