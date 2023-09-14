use crate::{
    ensure,
    helpers::helper::{
        get_reserves,
        pair_for_on_chain,
        quote,
        sort_tokens,
    },
    traits::{
        factory::{
            Factory,
            FactoryRef,
        },
        pair::PairRef,
    },
};
use ink::{
    env::CallFlags,
    prelude::vec::Vec,
};
use openbrush::{
    modifier_definition,
    traits::{
        AccountId,
        Balance,
        Storage,
    },
};

pub use crate::{
    impls::router::*,
    traits::router::*,
};

pub trait Internal {
    fn _add_liquidity(
        &self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: Balance,
        amount_b_desired: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
    ) -> Result<(Balance, Balance), RouterError>;

    fn _swap(
        &self,
        amounts: &Vec<Balance>,
        path: &Vec<AccountId>,
        to: AccountId,
    ) -> Result<(), RouterError>;
}

impl<T: Storage<data::Data>> Internal for T {
    fn _add_liquidity(
        &self,
        token_a: AccountId,
        token_b: AccountId,
        amount_a_desired: Balance,
        amount_b_desired: Balance,
        amount_a_min: Balance,
        amount_b_min: Balance,
    ) -> Result<(Balance, Balance), RouterError> {
        let factory = self.data().factory;
        if pair_for_on_chain(&factory, token_a, token_b).is_none() {
            let mut factory_ref: FactoryRef = factory.into();
            factory_ref.create_pair(token_a, token_b)?;
        };

        let (reserve_a, reserve_b) = get_reserves(&factory, token_a, token_b)?;
        if reserve_a == 0 && reserve_b == 0 {
            return Ok((amount_a_desired, amount_b_desired))
        }

        let amount_b_optimal = quote(amount_a_desired, reserve_a, reserve_b)?;
        if amount_b_optimal <= amount_b_desired {
            ensure!(
                amount_b_optimal >= amount_b_min,
                RouterError::InsufficientBAmount
            );
            Ok((amount_a_desired, amount_b_optimal))
        } else {
            let amount_a_optimal = quote(amount_b_desired, reserve_b, reserve_a)?;
            // amount_a_optimal <= amount_a_desired holds as amount_b_optimal > amount_b_desired
            ensure!(
                amount_a_optimal >= amount_a_min,
                RouterError::InsufficientAAmount
            );
            Ok((amount_a_optimal, amount_b_desired))
        }
    }

    fn _swap(
        &self,
        amounts: &Vec<Balance>,
        path: &Vec<AccountId>,
        _to: AccountId,
    ) -> Result<(), RouterError> {
        let factory = self.data().factory;
        for i in 0..path.len() - 1 {
            let (input, output) = (path[i], path[i + 1]);
            let (token_0, _) = sort_tokens(input, output)?;
            let amount_out = amounts[i + 1];
            let (amount_0_out, amount_1_out) = if input == token_0 {
                (0, amount_out)
            } else {
                (amount_out, 0)
            };
            // If last pair in the path, transfer tokens to the `_to` recipient.
            // Otherwise, transfer to the next Pair contract instance.
            let to = if i < path.len() - 2 {
                pair_for_on_chain(&factory, output, path[i + 2]).ok_or(RouterError::PairNotFound)?
            } else {
                _to
            };
            match PairRef::swap_builder(
                &pair_for_on_chain(&factory, input, output).ok_or(RouterError::PairNotFound)?,
                amount_0_out,
                amount_1_out,
                to,
            )
            .call_flags(CallFlags::default().set_allow_reentry(true))
            .try_invoke()
            {
                Ok(res) => {
                    match res {
                        Ok(v) => {
                            match v {
                                Ok(v) => Ok(v),
                                Err(err) => Err(RouterError::PairError(err)),
                            }
                        }
                        Err(err) => Err(RouterError::LangError(err)),
                    }
                }
                Err(_) => Err(RouterError::TransferError),
            }?;
        }
        Ok(())
    }
}

#[modifier_definition]
pub fn ensure_deadline<T, F, R, E>(instance: &mut T, body: F, deadline: u64) -> Result<R, E>
where
    T: Storage<data::Data>,
    F: FnOnce(&mut T) -> Result<R, E>,
    E: From<RouterError>,
{
    ensure!(deadline >= T::env().block_timestamp(), RouterError::Expired);
    body(instance)
}
