use crate::{
    ensure,
    helpers::math::casted_mul,
    traits::{
        factory::{
            Factory,
            FactoryRef,
        },
        types::WrappedU256,
    },
};
pub use crate::{
    impls::pair::*,
    traits::pair::*,
};
use openbrush::{
    contracts::psp22::*,
    traits::{
        AccountId,
        AccountIdExt,
        Balance,
        Storage,
    },
};
use primitive_types::U256;
use sp_arithmetic::{
    FixedPointNumber,
    FixedU128,
};

/// Minimum liquidity threshold that is subtracted
/// from the minted liquidity and sent to the `ZERO_ADDRESS`.
/// Prevents price manipulation and saturation.
/// See UniswapV2 whitepaper for more details.
/// NOTE: This value is taken from UniswapV2 whitepaper and is correct
/// only for liquidity tokens with precision = 18.
pub const MINIMUM_LIQUIDITY: u128 = 1000;

pub trait Internal {
    /// If turned on, controlled via `fee_to` parameter, mints protocol fee
    /// and transfer to `fee_to` address. Mints liquidity equivalent to 1/6th of the growth in sqrt(k).
    /// SHOULD be called before any new tokens are minted or burnt so that no fees are lost.
    fn _mint_fee(&mut self, reserve_0: Balance, reserve_1: Balance) -> Result<bool, PairError>;

    fn _update(
        &mut self,
        balance_0: Balance,
        balance_1: Balance,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<(), PairError>;

    fn _emit_mint_event(&self, _sender: AccountId, _amount_0: Balance, _amount_1: Balance);
    fn _emit_burn_event(
        &self,
        _sender: AccountId,
        _amount_0: Balance,
        _amount_1: Balance,
        _to: AccountId,
    );
    fn _emit_swap_event(
        &self,
        _sender: AccountId,
        _amount_0_in: Balance,
        _amount_1_in: Balance,
        _amount_0_out: Balance,
        _amount_1_out: Balance,
        _to: AccountId,
    );
    fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance);
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

impl<T: Storage<data::Data> + Storage<psp22::Data>> Internal for T {
    default fn _mint_fee(
        &mut self,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<bool, PairError> {
        let factory_ref: FactoryRef = self.data::<data::Data>().factory.into();
        let fee_to = factory_ref.fee_to();
        let fee_on = !fee_to.is_zero();
        let k_last: U256 = self.data::<data::Data>().k_last.into();
        if fee_on {
            // Section 2.4 Protocol fee in the whitepaper.
            if !k_last.is_zero() {
                let root_k: Balance = casted_mul(reserve_0, reserve_1)
                    .integer_sqrt()
                    .try_into()
                    .map_err(|_| PairError::CastOverflow1)?;
                let root_k_last = k_last
                    .integer_sqrt()
                    .try_into()
                    .map_err(|_| PairError::CastOverflow2)?;
                if root_k > root_k_last {
                    let total_supply = self.data::<psp22::Data>().supply;
                    let numerator = total_supply
                        .checked_mul(
                            root_k
                                .checked_sub(root_k_last)
                                .ok_or(PairError::SubUnderFlow14)?,
                        )
                        .ok_or(PairError::MulOverFlow13)?;
                    let denominator = root_k
                        .checked_mul(5)
                        .ok_or(PairError::MulOverFlow13)?
                        .checked_add(root_k_last)
                        .ok_or(PairError::AddOverflow1)?;
                    let liquidity = numerator
                        .checked_div(denominator)
                        .ok_or(PairError::DivByZero5)?;
                    if liquidity > 0 {
                        self._mint_to(fee_to, liquidity)?;
                    }
                }
            }
        } else if !k_last.is_zero() {
            self.data::<data::Data>().k_last = 0.into();
        }
        Ok(fee_on)
    }

    default fn _update(
        &mut self,
        balance_0: Balance,
        balance_1: Balance,
        reserve_0: Balance,
        reserve_1: Balance,
    ) -> Result<(), PairError> {
        ensure!(
            balance_0 <= u128::MAX && balance_1 <= u128::MAX,
            PairError::Overflow
        );
        let now = Self::env().block_timestamp();
        let last_timestamp = self.data::<data::Data>().block_timestamp_last;
        if now != last_timestamp {
            let (price_0_cumulative_last, price_1_cumulative_last) = update_cumulative(
                self.data::<data::Data>().price_0_cumulative_last,
                self.data::<data::Data>().price_1_cumulative_last,
                now.saturating_sub(last_timestamp).into(),
                reserve_0,
                reserve_1,
            );
            self.data::<data::Data>().price_0_cumulative_last = price_0_cumulative_last;
            self.data::<data::Data>().price_1_cumulative_last = price_1_cumulative_last;
        }
        self.data::<data::Data>().reserve_0 = balance_0;
        self.data::<data::Data>().reserve_1 = balance_1;
        self.data::<data::Data>().block_timestamp_last = now;

        self._emit_sync_event(balance_0, balance_1);
        Ok(())
    }

    default fn _emit_mint_event(&self, _sender: AccountId, _amount_0: Balance, _amount_1: Balance) {
    }
    default fn _emit_burn_event(
        &self,
        _sender: AccountId,
        _amount_0: Balance,
        _amount_1: Balance,
        _to: AccountId,
    ) {
    }
    default fn _emit_swap_event(
        &self,
        _sender: AccountId,
        _amount_0_in: Balance,
        _amount_1_in: Balance,
        _amount_0_out: Balance,
        _amount_1_out: Balance,
        _to: AccountId,
    ) {
    }
    default fn _emit_sync_event(&self, _reserve_0: Balance, _reserve_1: Balance) {}
}

#[cfg(test)]
mod tests {
    use primitive_types::U256;
    use sp_arithmetic::FixedU128;

    use super::update_cumulative;

    #[ink::test]
    fn update_cumulative_from_zero_time_elapsed() {
        let (cumulative0, cumulative1) = update_cumulative(0.into(), 0.into(), 0.into(), 10, 10);
        assert_eq!(cumulative0, 0.into());
        assert_eq!(cumulative1, 0.into());
    }

    #[ink::test]
    fn update_cumulative_from_one_time_elapsed() {
        let (cumulative0, cumulative1) = update_cumulative(0.into(), 0.into(), 1.into(), 10, 10);
        assert_eq!(
            FixedU128::from_inner(U256::from(cumulative0).as_u128()),
            1.into()
        );
        assert_eq!(
            FixedU128::from_inner(U256::from(cumulative1).as_u128()),
            1.into()
        );
    }
}
