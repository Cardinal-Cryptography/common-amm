use amm_helpers::{ensure, math::casted_mul};
use ink::{contract_ref, env::DefaultEnvironment, primitives::AccountId};
use traits::{MathError, RouterV2Error, Pair as PairTrait};

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Pair(AccountId, u8);

impl Pair {
    pub fn new(pair_id: AccountId) -> Self {
        let mut pair = Self(pair_id, 0);
        pair.1 = pair.contract_ref().get_fee();
        pair
    }
    /// Makes a cross-contract call to fetch `pair`'s reserves.
    /// Returns reserves `(reserve_0, reserve_1)` in order of `token_0` and `token_1`
    pub fn get_reserves(&self, token_0: AccountId, token_1: AccountId) -> (u128, u128) {
        let (reserve_0, reserve_1, _) = self.contract_ref().get_reserves();
        if token_0 < token_1 {
            (reserve_0, reserve_1)
        } else {
            (reserve_1, reserve_0)
        }
    }

    pub fn fee(&self) -> u8 {
        self.1
    }

    /// Returns how much of `token_1` tokens should be added
    /// to the pool to maintain the constant ratio `k = reserve_0 / reserve_1`,
    /// given `amount_0` of `token_0`.
    fn quote(amount_0: u128, reserve_0: u128, reserve_1: u128) -> Result<u128, RouterV2Error> {
        let amount_1: u128 = casted_mul(amount_0, reserve_1)
            .checked_div(reserve_0.into())
            .ok_or(MathError::DivByZero(6))?
            .try_into()
            .map_err(|_| MathError::CastOverflow(3))?;

        Ok(amount_1)
    }

    /// Calculates optimal amounts for `Pair` liquidity deposit.
    pub fn calculate_liquidity(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        amount_0_desired: u128,
        amount_1_desired: u128,
        amount_0_min: u128,
        amount_1_min: u128,
    ) -> Result<(u128, u128), RouterV2Error> {
        let (reserve_0, reserve_1) = self.get_reserves(token_0, token_1);

        if reserve_0 == 0 && reserve_1 == 0 {
            return Ok((amount_0_desired, amount_1_desired));
        }

        ensure!(
            reserve_0 > 0 && reserve_1 > 0,
            RouterV2Error::InsufficientLiquidity
        );
        ensure!(
            amount_0_desired > 0 && amount_1_desired > 0,
            RouterV2Error::InsufficientAmount
        );

        let amount_1_optimal = Pair::quote(amount_0_desired, reserve_0, reserve_1)?;
        if amount_1_optimal <= amount_1_desired {
            ensure!(
                amount_1_optimal >= amount_1_min,
                RouterV2Error::InsufficientAmountB
            );
            Ok((amount_0_desired, amount_1_optimal))
        } else {
            let amount_0_optimal = Pair::quote(amount_1_desired, reserve_1, reserve_0)?;
            // amount_0_optimal <= amount_0_desired holds as amount_1_optimal > amount_1_desired
            ensure!(
                amount_0_optimal >= amount_0_min,
                RouterV2Error::InsufficientAmountA
            );
            Ok((amount_0_optimal, amount_1_desired))
        }
    }

    pub fn contract_ref(&self) -> contract_ref!(PairTrait, DefaultEnvironment) {
        self.0.into()
    }

    pub fn pool_id(&self) -> AccountId {
        self.0
    }
}
