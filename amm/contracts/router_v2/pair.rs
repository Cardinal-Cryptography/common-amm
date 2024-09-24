use crate::utils::*;
use amm_helpers::{ensure, math::casted_mul};
use ink::{
    codegen::TraitCallBuilder,
    contract_ref,
    env::{account_id, caller, transferred_value, DefaultEnvironment as Env},
    primitives::AccountId,
};
use traits::{Balance, MathError, Pair as PairTrait, RouterV2Error};

const PAIR_TRADING_FEE_DENOM: u128 = 1000;

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct Pair {
    id: AccountId,
    token_0: AccountId,
    token_1: AccountId,
    fee: u8,
}

impl Pair {
    /// Returns `Pair` struct for given `pair_id`.
    ///
    /// Returns `None` if `pair_id` is not a Pair contract.
    pub fn try_new(pair_id: AccountId) -> Option<Self> {
        let contract_ref: contract_ref!(PairTrait, Env) = pair_id.into();
        // Assume that the `pair_id` is a Pair contract and try to get the fee value.
        // If the call is not successful return None indicating that the `pair_id`
        // is not a Pair contract
        let fee = match contract_ref.call().get_fee().try_invoke() {
            Ok(fee_result) => match fee_result {
                Ok(fee_value) => fee_value,
                Err(_) => return None,
            },
            Err(_) => return None,
        };
        let token_0 = contract_ref.get_token_0();
        let token_1 = contract_ref.get_token_1();
        Some(Pair {
            id: pair_id,
            token_0,
            token_1,
            fee,
        })
    }

    pub fn contract_ref(&self) -> contract_ref!(PairTrait, Env) {
        self.id.into()
    }

    fn check_tokens(&self, token_0: AccountId, token_1: AccountId) -> Result<(), RouterV2Error> {
        ensure!(
            (self.token_0 == token_0 && self.token_1 == token_1)
                || (self.token_0 == token_1 && self.token_1 == token_0),
            RouterV2Error::InvalidToken
        );
        Ok(())
    }

    /// Makes a cross-contract call to fetch the Pair reserves.
    /// Returns reserves `(reserve_0, reserve_1)` in order of `token_0` and `token_1`
    fn get_reserves(&self, token_0: &AccountId, token_1: &AccountId) -> (u128, u128) {
        let (reserve_0, reserve_1, _) = self.contract_ref().get_reserves();
        if token_0 < token_1 {
            (reserve_0, reserve_1)
        } else {
            (reserve_1, reserve_0)
        }
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
    fn calculate_liquidity(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        amount_0_desired: u128,
        amount_1_desired: u128,
        amount_0_min: u128,
        amount_1_min: u128,
    ) -> Result<(u128, u128), RouterV2Error> {
        let (reserve_0, reserve_1) = self.get_reserves(&token_0, &token_1);

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

    pub fn add_liquidity(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        amount_0_desired: u128,
        amount_1_desired: u128,
        amount_0_min: u128,
        amount_1_min: u128,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, u128, u128), RouterV2Error> {
        check_timestamp(deadline)?;
        self.check_tokens(token_0, token_1)?;
        let (amount_0, amount_1) = self.calculate_liquidity(
            token_0,
            token_1,
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
        )?;

        let caller = caller::<Env>();
        psp22_transfer_from(token_0, caller, self.id, amount_0)?;
        psp22_transfer_from(token_1, caller, self.id, amount_1)?;

        let liquidity = self.contract_ref().mint(to)?;

        Ok((amount_0, amount_1, liquidity))
    }

    pub fn add_liquidity_native(
        &self,
        token: AccountId,
        wnative: AccountId,
        amount_token_desired: u128,
        amount_token_min: u128,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, Balance, u128), RouterV2Error> {
        check_timestamp(deadline)?;
        self.check_tokens(token, wnative)?;
        let received_value = transferred_value::<Env>();
        let (amount_0, amount_native) = self.calculate_liquidity(
            token,
            wnative,
            amount_token_desired,
            received_value,
            amount_token_min,
            amount_native_min,
        )?;

        let caller = caller::<Env>();
        psp22_transfer_from(token, caller, self.id, amount_0)?;
        wrap(wnative, amount_native)?;
        psp22_transfer(wnative, self.id, amount_native)?;

        let liquidity = self.contract_ref().mint(to)?;

        if received_value > amount_native {
            transfer_native(caller, received_value - amount_native)?;
        }

        Ok((amount_0, amount_native, liquidity))
    }

    pub fn remove_liquidity(
        &self,
        token_0: AccountId,
        token_1: AccountId,
        liquidity: u128,
        amount_0_min: u128,
        amount_1_min: u128,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, u128), RouterV2Error> {
        check_timestamp(deadline)?;
        self.check_tokens(token_0, token_1)?;
        psp22_transfer_from(self.id, caller::<Env>(), self.id, liquidity)?;

        let (amount_0, amount_1) = self.contract_ref().burn(to)?;
        let (amount_0, amount_1) = if token_0 < token_1 {
            (amount_0, amount_1)
        } else {
            (amount_1, amount_0)
        };

        ensure!(amount_0 >= amount_0_min, RouterV2Error::InsufficientAmountA);
        ensure!(amount_1 >= amount_1_min, RouterV2Error::InsufficientAmountB);

        Ok((amount_0, amount_1))
    }

    pub fn remove_liquidity_native(
        &self,
        token: AccountId,
        wnative: AccountId,
        liquidity: u128,
        amount_token_min: u128,
        amount_native_min: Balance,
        to: AccountId,
        deadline: u64,
    ) -> Result<(u128, Balance), RouterV2Error> {
        let (amount_token, amount_native) = self.remove_liquidity(
            token,
            wnative,
            liquidity,
            amount_token_min,
            amount_native_min,
            account_id::<Env>(),
            deadline,
        )?;
        psp22_transfer(token, to, amount_token)?;
        withdraw(wnative, amount_native)?;
        transfer_native(to, amount_native)?;
        Ok((amount_token, amount_native))
    }

    pub fn swap(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
        to: AccountId,
    ) -> Result<(), RouterV2Error> {
        self.check_tokens(token_in, token_out)
            .map_err(|_| RouterV2Error::InvalidPath)?;
        let (amount_0_out, amount_1_out) = if token_in < token_out {
            (0, amount_out)
        } else {
            (amount_out, 0)
        };
        self.contract_ref()
            .swap(amount_0_out, amount_1_out, to, None)?;
        Ok(())
    }

    pub fn get_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
    ) -> Result<u128, RouterV2Error> {
        self.check_tokens(token_in, token_out)
            .map_err(|_| RouterV2Error::InvalidPath)?;
        let (reserve_in, reserve_out) = self.get_reserves(&token_in, &token_out);
        ensure!(amount_out > 0, RouterV2Error::InsufficientAmount);
        ensure!(
            reserve_in > 0 && reserve_out > 0,
            RouterV2Error::InsufficientLiquidity
        );

        let numerator = casted_mul(reserve_in, amount_out)
            .checked_mul(PAIR_TRADING_FEE_DENOM.into())
            .ok_or(MathError::MulOverflow(14))?;

        let denominator = casted_mul(
            reserve_out
                .checked_sub(amount_out)
                .ok_or(MathError::SubUnderflow(15))?,
            PAIR_TRADING_FEE_DENOM - (self.fee as u128),
        );

        let amount_in: u128 = numerator
            .checked_div(denominator)
            .ok_or(MathError::DivByZero(8))?
            .checked_add(1.into())
            .ok_or(MathError::AddOverflow(3))?
            .try_into()
            .map_err(|_| MathError::CastOverflow(5))?;

        Ok(amount_in)
    }

    pub fn get_amount_out(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
    ) -> Result<u128, RouterV2Error> {
        self.check_tokens(token_in, token_out)
            .map_err(|_| RouterV2Error::InvalidPath)?;
        let (reserve_in, reserve_out) = self.get_reserves(&token_in, &token_out);
        ensure!(amount_in > 0, RouterV2Error::InsufficientAmount);
        ensure!(
            reserve_in > 0 && reserve_out > 0,
            RouterV2Error::InsufficientLiquidity
        );

        // Adjusts for fees paid in the `token_in`.
        let amount_in_with_fee = casted_mul(amount_in, PAIR_TRADING_FEE_DENOM - (self.fee as u128));

        let numerator = amount_in_with_fee
            .checked_mul(reserve_out.into())
            .ok_or(MathError::MulOverflow(13))?;

        let denominator = casted_mul(reserve_in, PAIR_TRADING_FEE_DENOM)
            .checked_add(amount_in_with_fee)
            .ok_or(MathError::AddOverflow(2))?;

        let amount_out: u128 = numerator
            .checked_div(denominator)
            .ok_or(MathError::DivByZero(7))?
            .try_into()
            .map_err(|_| MathError::CastOverflow(4))?;

        Ok(amount_out)
    }
}
