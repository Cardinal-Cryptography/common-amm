use ink::{
    codegen::TraitCallBuilder, contract_ref, env::DefaultEnvironment as Env, prelude::vec::Vec,
    primitives::AccountId,
};
use traits::{RouterV2Error, StablePool as StablePoolTrait};

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
        Some(Self {
            id: pool_id,
            tokens,
        })
    }

    pub fn contract_ref(&self) -> contract_ref!(StablePoolTrait, Env) {
        self.id.into()
    }

    pub fn swap(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_out: u128,
        to: AccountId,
    ) -> Result<(), RouterV2Error> {
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
        Ok(self
            .contract_ref()
            .get_swap_amount_in(token_in, token_out, amount_out)
            .map(|(amount_in, _)| amount_in)?)
    }

    pub fn get_amount_out(
        &self,
        token_in: AccountId,
        token_out: AccountId,
        amount_in: u128,
    ) -> Result<u128, RouterV2Error> {
        Ok(self
            .contract_ref()
            .get_swap_amount_out(token_in, token_out, amount_in)
            .map(|(amount_out, _)| amount_out)?)
    }
}
