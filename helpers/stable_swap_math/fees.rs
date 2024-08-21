use crate::{
    constants::stable_pool::{FEE_DENOM, MAX_PROTOCOL_FEE, MAX_TRADE_FEE},
    math::{casted_mul, MathError},
};

#[ink::storage_item]
#[derive(Debug, Default)]
pub struct Fees {
    pub trade_fee: u32,
    pub protocol_fee: u32,
}

impl Fees {
    /// Crate new fee instance.
    /// - `trade_fee` given as an integer with 1e9 precision restricted to [`MAX_TRADE_FEE`](const@MAX_TRADE_FEE)
    /// - `protocol_fee` given as an integer with 1e9 precision restricted to [`MAX_PROTOCOL_FEE`](const@MAX_PROTOCOL_FEE).
    ///    Taken as part of the `trade_fee`
    pub fn new(trade_fee: u32, protocol_fee: u32) -> Option<Self> {
        if trade_fee > MAX_TRADE_FEE || protocol_fee > MAX_PROTOCOL_FEE {
            None
        } else {
            Some(Self {
                trade_fee,
                protocol_fee,
            })
        }
    }

    pub fn zero() -> Self {
        Self {
            trade_fee: 0,
            protocol_fee: 0,
        }
    }

    pub fn trade_fee_from_gross(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(amount, self.trade_fee, FEE_DENOM)
    }

    pub fn trade_fee_from_net(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(
            amount,
            self.trade_fee,
            FEE_DENOM
                .checked_sub(self.trade_fee)
                .ok_or(MathError::SubUnderflow(61))?,
        )
    }

    pub fn protocol_trade_fee(&self, amount: u128) -> Result<u128, MathError> {
        u128_ratio(amount, self.protocol_fee, FEE_DENOM)
    }

    /// Used to normalize fee applied on difference amount with ideal u128, This logic is from
    /// https://github.com/ref-finance/ref-contracts/blob/main/ref-exchange/src/stable_swap/math.rs#L48
    /// https://github.com/saber-hq/stable-swap/blob/5db776fb0a41a0d1a23d46b99ef412ca7ccc5bf6/stable-swap-program/program/src/fees.rs#L73
    /// https://github.com/curvefi/curve-contract/blob/e5fb8c0e0bcd2fe2e03634135806c0f36b245511/tests/simulation.py#L124
    pub fn normalized_trade_fee(&self, num_coins: u32, amount: u128) -> Result<u128, MathError> {
        let adjusted_trade_fee = (self
            .trade_fee
            .checked_mul(num_coins)
            .ok_or(MathError::MulOverflow(61)))?
        .checked_div(
            (num_coins.checked_sub(1).ok_or(MathError::SubUnderflow(62)))?
                .checked_mul(4)
                .ok_or(MathError::MulOverflow(62))?,
        )
        .ok_or(MathError::DivByZero(61))?;
        u128_ratio(amount, adjusted_trade_fee, FEE_DENOM)
    }
}

fn u128_ratio(amount: u128, num: u32, denom: u32) -> Result<u128, MathError> {
    casted_mul(amount, num.into())
        .checked_div(denom.into())
        .ok_or(MathError::DivByZero(61))?
        .try_into()
        .map_err(|_| MathError::CastOverflow(61))
}
#[cfg(test)]
mod tests {
    use crate::constants::stable_pool::{MAX_PROTOCOL_FEE, MAX_TRADE_FEE};

    use super::Fees;

    #[test]
    fn test_max_fees() {
        _ = Fees::new(MAX_TRADE_FEE, MAX_PROTOCOL_FEE).expect("Should instantiate fee");
        assert!(
            Fees::new(MAX_TRADE_FEE + 1, MAX_PROTOCOL_FEE).is_none(),
            "Should fail to instantiate fee"
        );
        assert!(
            Fees::new(MAX_TRADE_FEE, MAX_PROTOCOL_FEE + 1).is_none(),
            "Should fail to instantiate fee"
        );
    }

    #[test]
    fn test_fees() {
        let fees = Fees::new(MAX_TRADE_FEE, MAX_PROTOCOL_FEE).expect("Should instantiate fee");
        let amount_gross: u128 = 1_000_000;
        let expected_trade_fee: u128 = amount_gross / 100; // 1%
        let expected_protocol_fee: u128 = expected_trade_fee / 2; // 50%
        let trade_fee_from_gross = fees
            .trade_fee_from_gross(amount_gross)
            .expect("Should compute fee");
        let trade_fee_from_net = fees
            .trade_fee_from_net(amount_gross - expected_trade_fee)
            .expect("Should compute fee");
        assert_eq!(
            expected_trade_fee, trade_fee_from_gross,
            "Trade fee should be 1%"
        );
        assert_eq!(
            expected_trade_fee, trade_fee_from_net,
            "Trade fee should be 1%"
        );
        let protocol_fee = fees
            .protocol_trade_fee(expected_trade_fee)
            .expect("Should compute protocol fee");
        assert_eq!(
            expected_protocol_fee, protocol_fee,
            "Protocol fee should be 50%"
        );
    }
}
