#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod router {
    use amm::{
        ensure,
        helpers::{
            helper::*,
            transfer_helper::*,
        },
        traits::{
            factory::Factory,
            pair::Pair,
            router::{
                Router,
                RouterError,
            },
        },
    };
    use ink::{
        codegen::TraitCallBuilder,
        contract_ref,
        env::CallFlags,
        prelude::vec::Vec,
    };

    #[ink(storage)]
    pub struct RouterContract {
        factory: AccountId,
        wnative: AccountId,
    }

    impl RouterContract {
        #[ink(constructor)]
        pub fn new(factory: AccountId, wnative: AccountId) -> Self {
            Self { factory, wnative }
        }

        fn _add_liquidity(
            &self,
            token_a: AccountId,
            token_b: AccountId,
            amount_a_desired: Balance,
            amount_b_desired: Balance,
            amount_a_min: Balance,
            amount_b_min: Balance,
        ) -> Result<(Balance, Balance), RouterError> {
            if pair_for_on_chain(&self.factory, token_a, token_b).is_none() {
                let mut factory: contract_ref!(Factory) = self.factory.into();
                factory.create_pair(token_a, token_b)?;
            };

            let (reserve_a, reserve_b) = get_reserves(&self.factory, token_a, token_b)?;
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
            amounts: &[Balance],
            path: &Vec<AccountId>,
            _to: AccountId,
        ) -> Result<(), RouterError> {
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
                    pair_for_on_chain(&self.factory, output, path[i + 2])
                        .ok_or(RouterError::PairNotFound)?
                } else {
                    _to
                };
                let pair = pair_for_on_chain(&self.factory, input, output)
                    .ok_or(RouterError::PairNotFound)?;
                let mut pair: contract_ref!(Pair) = pair.into();

                match pair
                    .call_mut()
                    .swap(amount_0_out, amount_1_out, to)
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

        fn check_timestamp(&self, deadline: Timestamp) -> Result<(), RouterError> {
            ensure!(
                deadline >= self.env().block_timestamp(),
                RouterError::Expired
            );
            Ok(())
        }
    }

    impl Router for RouterContract {
        #[ink(message)]
        fn factory(&self) -> AccountId {
            self.factory
        }

        #[ink(message)]
        fn wnative(&self) -> AccountId {
            self.wnative
        }

        #[ink(message)]
        fn add_liquidity(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
            amount_a_desired: Balance,
            amount_b_desired: Balance,
            amount_a_min: Balance,
            amount_b_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(Balance, Balance, Balance), RouterError> {
            self.check_timestamp(deadline)?;
            let (amount_a, amount_b) = self._add_liquidity(
                token_a,
                token_b,
                amount_a_desired,
                amount_b_desired,
                amount_a_min,
                amount_b_min,
            )?;

            let pair_contract = pair_for_on_chain(&self.factory, token_a, token_b)
                .ok_or(RouterError::PairNotFound)?;

            let caller = self.env().caller();
            safe_transfer_from(token_a, caller, pair_contract, amount_a)?;
            safe_transfer_from(token_b, caller, pair_contract, amount_b)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();
            let liquidity = pair.mint(to)?;

            Ok((amount_a, amount_b, liquidity))
        }

        #[ink(message)]
        fn add_liquidity_native(
            &mut self,
            token: AccountId,
            amount_token_desired: Balance,
            amount_token_min: Balance,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(Balance, Balance, Balance), RouterError> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_value = self.env().transferred_value();

            let (amount_a, amount_native) = self._add_liquidity(
                token,
                wnative,
                amount_token_desired,
                received_value,
                amount_token_min,
                amount_native_min,
            )?;

            let pair_contract = pair_for_on_chain(&self.factory, token, wnative)
                .ok_or(RouterError::PairNotFound)?;

            let caller = self.env().caller();
            safe_transfer_from(token, caller, pair_contract, amount_a)?;
            wrap(&wnative, amount_native)?;
            safe_transfer(wnative, pair_contract, amount_native)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();
            let liquidity = pair.mint(to)?;

            if received_value > amount_native {
                safe_transfer_native(caller, received_value - amount_native)?
            }

            Ok((amount_a, amount_native, liquidity))
        }

        #[ink(message)]
        fn remove_liquidity(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
            liquidity: Balance,
            amount_a_min: Balance,
            amount_b_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(Balance, Balance), RouterError> {
            self.check_timestamp(deadline)?;
            let pair_contract = pair_for_on_chain(&self.factory, token_a, token_b)
                .ok_or(RouterError::PairNotFound)?;

            safe_transfer_from(pair_contract, self.env().caller(), pair_contract, liquidity)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();

            let (amount_0, amount_1) = match pair
                .call_mut()
                .burn(to)
                .call_flags(CallFlags::default().set_allow_reentry(true))
                .try_invoke()
            {
                Ok(res) => {
                    match res {
                        Ok(v) => {
                            match v {
                                Ok(tuple) => Ok(tuple),
                                Err(err) => Err(RouterError::PairError(err)),
                            }
                        }
                        Err(_) => Err(RouterError::TransferError),
                    }
                }
                Err(_) => Err(RouterError::TransferError),
            }?;
            let (token_0, _) = sort_tokens(token_a, token_b)?;
            let (amount_a, amount_b) = if token_a == token_0 {
                (amount_0, amount_1)
            } else {
                (amount_1, amount_0)
            };

            ensure!(amount_a >= amount_a_min, RouterError::InsufficientAAmount);
            ensure!(amount_b >= amount_b_min, RouterError::InsufficientBAmount);

            Ok((amount_a, amount_b))
        }

        #[ink(message)]
        fn remove_liquidity_native(
            &mut self,
            token: AccountId,
            liquidity: Balance,
            amount_token_min: Balance,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(Balance, Balance), RouterError> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let (amount_token, amount_native) = self.remove_liquidity(
                token,
                wnative,
                liquidity,
                amount_token_min,
                amount_native_min,
                self.env().account_id(),
                deadline,
            )?;
            safe_transfer(token, to, amount_token)?;
            unwrap(&wnative, amount_native)?;
            safe_transfer_native(to, amount_native)?;
            Ok((amount_token, amount_native))
        }

        #[ink(message)]
        fn swap_exact_tokens_for_tokens(
            &mut self,
            amount_in: Balance,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            self.check_timestamp(deadline)?;
            let factory = self.factory;

            let amounts = get_amounts_out(&factory, amount_in, &path)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterError::InsufficientOutputAmount
            );
            safe_transfer_from(
                path[0],
                self.env().caller(),
                pair_for_on_chain(&factory, path[0], path[1]).ok_or(RouterError::PairNotFound)?,
                amounts[0],
            )?;
            self._swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_tokens(
            &mut self,
            amount_out: Balance,
            amount_in_max: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            self.check_timestamp(deadline)?;
            let factory = self.factory;
            let amounts = get_amounts_in(&factory, amount_out, &path)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterError::ExcessiveInputAmount
            );
            safe_transfer_from(
                path[0],
                self.env().caller(),
                pair_for_on_chain(&factory, path[0], path[1]).ok_or(RouterError::PairNotFound)?,
                amounts[0],
            )?;
            self._swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_exact_native_for_tokens(
            &mut self,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            self.check_timestamp(deadline)?;
            let factory = self.factory;

            let received_value = self.env().transferred_value();
            let wnative = self.wnative;
            ensure!(path[0] == wnative, RouterError::InvalidPath);
            let amounts = get_amounts_out(&factory, received_value, &path)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterError::InsufficientOutputAmount
            );
            wrap(&wnative, received_value)?;
            safe_transfer(
                wnative,
                pair_for_on_chain(&factory, path[0], path[1]).ok_or(RouterError::PairNotFound)?,
                amounts[0],
            )?;
            self._swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_native(
            &mut self,
            amount_out: Balance,
            amount_in_max: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            self.check_timestamp(deadline)?;
            let factory = self.factory;

            let wnative = self.wnative;
            ensure!(path[path.len() - 1] == wnative, RouterError::InvalidPath);
            let amounts = get_amounts_in(&factory, amount_out, &path)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterError::ExcessiveInputAmount
            );
            safe_transfer_from(
                path[0],
                self.env().caller(),
                pair_for_on_chain(&factory, path[0], path[1]).ok_or(RouterError::PairNotFound)?,
                amounts[0],
            )?;
            self._swap(&amounts, &path, self.env().account_id())?;
            let native_out = amounts[amounts.len() - 1];
            unwrap(&wnative, native_out)?;
            safe_transfer_native(to, native_out)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_exact_tokens_for_native(
            &mut self,
            amount_in: Balance,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            self.check_timestamp(deadline)?;
            let factory = self.factory;

            let wnative = self.wnative;
            ensure!(path[path.len() - 1] == wnative, RouterError::InvalidPath);
            let amounts = get_amounts_out(&factory, amount_in, &path)?;
            let native_out = amounts[amounts.len() - 1];
            ensure!(
                native_out >= amount_out_min,
                RouterError::InsufficientOutputAmount
            );
            safe_transfer_from(
                path[0],
                self.env().caller(),
                pair_for_on_chain(&factory, path[0], path[1]).ok_or(RouterError::PairNotFound)?,
                amounts[0],
            )?;
            self._swap(&amounts, &path, self.env().account_id())?;
            unwrap(&wnative, native_out)?;
            safe_transfer_native(to, native_out)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_native_for_exact_tokens(
            &mut self,
            amount_out: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            self.check_timestamp(deadline)?;
            let factory = self.factory;
            let wnative = self.wnative;
            let received_native = self.env().transferred_value();

            ensure!(path[0] == wnative, RouterError::InvalidPath);
            let amounts = get_amounts_in(&factory, amount_out, &path)?;
            let native_in = amounts[0];
            ensure!(
                native_in <= received_native,
                RouterError::ExcessiveInputAmount
            );
            wrap(&wnative, native_in)?;
            safe_transfer(
                wnative,
                pair_for_on_chain(&factory, path[0], path[1]).ok_or(RouterError::PairNotFound)?,
                native_in,
            )?;
            self._swap(&amounts, &path, to)?;
            if received_native > native_in {
                safe_transfer_native(self.env().caller(), received_native - native_in)?
            }
            Ok(amounts)
        }

        #[ink(message)]
        fn quote(
            &self,
            amount_a: Balance,
            reserve_a: Balance,
            reserve_b: Balance,
        ) -> Result<Balance, RouterError> {
            Ok(quote(amount_a, reserve_a, reserve_b)?)
        }

        #[ink(message)]
        fn get_amount_out(
            &self,
            amount_in: Balance,
            reserve_a: Balance,
            reserve_b: Balance,
        ) -> Result<Balance, RouterError> {
            Ok(get_amount_out(amount_in, reserve_a, reserve_b)?)
        }

        #[ink(message)]
        fn get_amount_in(
            &self,
            amount_out: Balance,
            reserve_a: Balance,
            reserve_b: Balance,
        ) -> Result<Balance, RouterError> {
            Ok(get_amount_in(amount_out, reserve_a, reserve_b)?)
        }

        #[ink(message)]
        fn get_amounts_out(
            &self,
            amount_in: Balance,
            path: Vec<AccountId>,
        ) -> Result<Vec<Balance>, RouterError> {
            Ok(get_amounts_out(&self.factory, amount_in, &path)?)
        }

        #[ink(message)]
        fn get_amounts_in(
            &self,
            amount_out: Balance,
            path: Vec<AccountId>,
        ) -> Result<Vec<Balance>, RouterError> {
            Ok(get_amounts_in(&self.factory, amount_out, &path)?)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn initialize_works() {
            let factory = AccountId::from([0x03; 32]);
            let wnative = AccountId::from([0x04; 32]);
            let router = RouterContract::new(factory, wnative);
            assert_eq!(router.factory(), factory);
            assert_eq!(router.wnative(), wnative);
        }
    }
}
