use amm_helpers::ensure;
use ink::{
    codegen::TraitCallBuilder,
    contract_ref,
    env::{account_id, caller, transferred_value, DefaultEnvironment as Env},
    prelude::vec::Vec,
    primitives::AccountId,
};
use traits::{RouterV2Error, StablePool as StablePoolTrait, StablePoolError};

use crate::utils::{
    check_timestamp, psp22_approve, psp22_transfer, psp22_transfer_from, transfer_native, withdraw,
    wrap,
};

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct StablePool {
    id: AccountId,
    tokens: Vec<AccountId>,
}

impl StablePool {
    /// Returns `StablePool` struct for given `pool_id`.
    ///
    /// Returns `None` if `pool_id` is not a StablePool contract.
    pub fn try_new(pool_id: AccountId) -> Option<Self> {
        let contract_ref: contract_ref!(StablePoolTrait, Env) = pool_id.into();
        // Assume that the `pool_id` is a StablePool contract and try to get the tokens.
        // If the call is not successful return None indicating that the `pool_id`
        // is not a StablePool contract.
        let tokens = match contract_ref.call().tokens().try_invoke() {
            Ok(tokens_result) => match tokens_result {
                Ok(tokens_value) => tokens_value,
                Err(_) => return None,
            },
            Err(_) => return None,
        };
        // set spending allowance of each token for the pool to `u128::MAX`
        // required for adding liquidity
        for &token in &tokens {
            if psp22_approve(token, pool_id, u128::MAX).is_err() {
                return None;
            };
        }
        Some(Self {
            id: pool_id,
            tokens,
        })
    }

    pub fn contract_ref(&self) -> contract_ref!(StablePoolTrait, Env) {
        self.id.into()
    }

    /// Adds liquidity to the pool.
    ///
    /// If `wnative` is specified, it attemps to wrap the transferred native token
    /// and use it instead of transferring the wrapped version.
    pub fn add_liquidity(
        &self,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
        deadline: u64,
        wnative: Option<AccountId>,
    ) -> Result<(u128, u128), RouterV2Error> {
        check_timestamp(deadline)?;
        ensure!(
            self.tokens.len() == amounts.len(),
            RouterV2Error::StablePoolError(StablePoolError::IncorrectAmountsCount)
        );
        let native_received = transferred_value::<Env>();
        let (wnative_idx, native_surplus) = match wnative {
            Some(wnative) => {
                let wnative_idx = self.wnative_idx(wnative)?;
                let wnative_amount = amounts[wnative_idx];
                ensure!(
                    native_received >= wnative_amount,
                    RouterV2Error::InsufficientTransferredAmount
                );
                wrap(wnative, wnative_amount)?;
                (wnative_idx, native_received.saturating_sub(wnative_amount))
            }
            None => (self.tokens.len(), native_received),
        };
        if native_surplus > 0 {
            transfer_native(caller::<Env>(), native_surplus)?;
        }
        for i in (0..self.tokens.len()).filter(|&idx| idx != wnative_idx) {
            psp22_transfer_from(
                self.tokens[i],
                caller::<Env>(),
                account_id::<Env>(),
                amounts[i],
            )?;
        }
        Ok(self
            .contract_ref()
            .add_liquidity(min_share_amount, amounts, to)?)
    }

    /// Withdraws liquidity from the pool by the specified amounts.
    ///
    /// If `wnative` is specified, it attemps to unwrap the wrapped native token
    /// and withdraw it to the `to` account.
    pub fn remove_liquidity(
        &self,
        max_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
        deadline: u64,
        wnative: Option<AccountId>,
    ) -> Result<(u128, u128), RouterV2Error> {
        check_timestamp(deadline)?;
        ensure!(
            self.tokens.len() == amounts.len(),
            RouterV2Error::StablePoolError(StablePoolError::IncorrectAmountsCount)
        );
        psp22_transfer_from(
            self.id,
            caller::<Env>(),
            account_id::<Env>(),
            max_share_amount,
        )?;
        let (lp_burned, fee_part) = match wnative {
            Some(wnative) => {
                let wnative_idx = self.wnative_idx(wnative)?;
                let res = self.contract_ref().remove_liquidity_by_amounts(
                    max_share_amount,
                    amounts.clone(),
                    account_id::<Env>(),
                )?;
                withdraw(wnative, amounts[wnative_idx])?;
                transfer_native(to, amounts[wnative_idx])?;
                for i in (0..self.tokens.len()).filter(|&idx| idx != wnative_idx) {
                    psp22_transfer(self.tokens[i], to, amounts[i])?;
                }
                res
            }
            None => {
                self.contract_ref()
                    .remove_liquidity_by_amounts(max_share_amount, amounts, to)?
            }
        };
        if max_share_amount > lp_burned {
            psp22_transfer(self.id, caller::<Env>(), max_share_amount - lp_burned)?;
        }
        Ok((lp_burned, fee_part))
    }

    /// Withdraws liquidity from the pool in balanced propotions.
    ///
    /// If `wnative` is specified, it attemps to unwrap the wrapped native token
    /// and withdraw it to the `to` account.
    pub fn remove_liquidity_by_share(
        &self,
        share_amount: u128,
        min_amounts: Vec<u128>,
        to: AccountId,
        deadline: u64,
        wnative: Option<AccountId>,
    ) -> Result<Vec<u128>, RouterV2Error> {
        check_timestamp(deadline)?;
        ensure!(
            self.tokens.len() == min_amounts.len(),
            RouterV2Error::StablePoolError(StablePoolError::IncorrectAmountsCount)
        );
        psp22_transfer_from(self.id, caller::<Env>(), account_id::<Env>(), share_amount)?;
        match wnative {
            Some(wnative) => {
                let wnative_idx = self.wnative_idx(wnative)?;
                let amounts = self.contract_ref().remove_liquidity_by_shares(
                    share_amount,
                    min_amounts,
                    account_id::<Env>(),
                )?;
                withdraw(wnative, amounts[wnative_idx])?;
                transfer_native(to, amounts[wnative_idx])?;
                for i in (0..self.tokens.len()).filter(|&idx| idx != wnative_idx) {
                    psp22_transfer(self.tokens[i], to, amounts[i])?;
                }
                Ok(amounts)
            }
            None => {
                Ok(self
                    .contract_ref()
                    .remove_liquidity_by_shares(share_amount, min_amounts, to)?)
            }
        }
    }

    pub fn swap(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
        to: AccountId,
    ) -> Result<(), RouterV2Error> {
        ensure!(
            self.tokens.contains(&token_in) && self.tokens.contains(&token_out),
            RouterV2Error::InvalidPath
        );
        self.contract_ref()
            .swap_received(token_in, token_out, amount_out, to)?;
        Ok(())
    }

    pub fn get_amount_in(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
    ) -> Result<u128, RouterV2Error> {
        ensure!(
            self.tokens.contains(&token_in) && self.tokens.contains(&token_out),
            RouterV2Error::InvalidPath
        );
        match self
            .contract_ref()
            .get_swap_amount_in(token_in, token_out, amount_out)
        {
            Ok((amount_in, _)) => Ok(amount_in),
            Err(err) => Err(err.into()),
        }
    }

    pub fn get_amount_out(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
    ) -> Result<u128, RouterV2Error> {
        ensure!(
            self.tokens.contains(&token_in) && self.tokens.contains(&token_out),
            RouterV2Error::InvalidPath
        );
        match self
            .contract_ref()
            .get_swap_amount_out(token_in, token_out, amount_in)
        {
            Ok((amount_out, _)) => Ok(amount_out),
            Err(err) => Err(err.into()),
        }
    }

    fn wnative_idx(&self, wnative: AccountId) -> Result<usize, RouterV2Error> {
        self.tokens
            .iter()
            .position(|&token| wnative == token)
            .ok_or(RouterV2Error::InvalidToken)
    }
}
