use crate::utils::*;
use amm_helpers::{ensure, math::casted_mul};
use ink::{
    codegen::TraitCallBuilder,
    contract_ref,
    env::{account_id, caller, transfer, transferred_value, DefaultEnvironment as Env},
    prelude::string::String,
    primitives::AccountId,
};
use traits::{Balance, MathError, Pair as PairTrait, RouterV2Error};
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
    /// Makes a cross-contract call to fetch the Pair reserves.
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

    pub fn contract_ref(&self) -> contract_ref!(PairTrait, Env) {
        self.0.into()
    }

    pub fn pool_id(&self) -> AccountId {
        self.0
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
        let (amount_0, amount_1) = self.calculate_liquidity(
            token_0,
            token_1,
            amount_0_desired,
            amount_1_desired,
            amount_0_min,
            amount_1_min,
        )?;

        let caller = caller::<Env>();
        psp22_transfer_from(token_0, caller, self.pool_id(), amount_0)?;
        psp22_transfer_from(token_1, caller, self.pool_id(), amount_1)?;

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
        psp22_transfer_from(token, caller, self.pool_id(), amount_0)?;
        wrap(wnative, amount_native)?;
        psp22_transfer(wnative, self.pool_id(), amount_native)?;

        let liquidity = self.contract_ref().mint(to)?;

        if received_value > amount_native {
            transfer::<Env>(caller, received_value - amount_native)
                .map_err(|_| RouterV2Error::TransferError)?;
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
        ensure!(token_0 != token_1, RouterV2Error::IdenticalAddresses);
        psp22_transfer_from(self.pool_id(), caller::<Env>(), self.pool_id(), liquidity)?;

        let (amount_0, amount_1) = self
            .contract_ref()
            .call_mut()
            .burn(to)
            .try_invoke()
            .map_err(|_| RouterV2Error::CrossContractCallFailed(String::from("Pair:burn")))???;
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
        check_timestamp(deadline)?;
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
        transfer::<Env>(to, amount_native).map_err(|_| RouterV2Error::TransferError)?;
        Ok((amount_token, amount_native))
    }
}
