#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod router {
    use ink::{
        codegen::Env,
        env::CallFlags,
        prelude::vec::Vec,
    };
    use openbrush::{
        modifiers,
        traits::Storage,
    };
    use uniswap_v2::{
        ensure,
        helpers::{
            helper::*,
            transfer_helper::*,
        },
        impls::router::{
            router::{
                ensure_deadline,
                Internal,
            },
            *,
        },
        traits::{
            pair::PairRef,
            router::*,
        },
    };

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct RouterContract {
        #[storage_field]
        router: data::Data,
    }

    impl RouterContract {
        #[ink(constructor)]
        pub fn new(factory: AccountId, wnative: AccountId) -> Self {
            let mut instance = Self::default();
            instance.router.factory = factory;
            instance.router.wnative = wnative;
            instance
        }
    }

    impl Router for RouterContract {
        #[ink(message)]
        fn factory(&self) -> AccountId {
            self.router.factory
        }

        #[ink(message)]
        fn wnative(&self) -> AccountId {
            self.router.wnative
        }

        #[modifiers(ensure_deadline(deadline))]
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
            let (amount_a, amount_b) = self._add_liquidity(
                token_a,
                token_b,
                amount_a_desired,
                amount_b_desired,
                amount_a_min,
                amount_b_min,
            )?;

            let pair_contract = pair_for_on_chain(&self.router.factory, token_a, token_b)
                .ok_or(RouterError::PairNotFound)?;

            let caller = self.env().caller();
            safe_transfer_from(token_a, caller, pair_contract, amount_a)?;
            safe_transfer_from(token_b, caller, pair_contract, amount_b)?;

            let liquidity = PairRef::mint(&pair_contract, to)?;

            Ok((amount_a, amount_b, liquidity))
        }

        #[modifiers(ensure_deadline(deadline))]
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
            let wnative = self.router.wnative;
            let received_value = self.env().transferred_value();

            let (amount_a, amount_native) = self._add_liquidity(
                token,
                wnative,
                amount_token_desired,
                received_value,
                amount_token_min,
                amount_native_min,
            )?;

            let pair_contract = pair_for_on_chain(&self.router.factory, token, wnative)
                .ok_or(RouterError::PairNotFound)?;

            let caller = self.env().caller();
            safe_transfer_from(token, caller, pair_contract, amount_a)?;
            wrap(&wnative, amount_native)?;
            safe_transfer(wnative, pair_contract, amount_native)?;

            let liquidity = PairRef::mint(&pair_contract, to)?;

            if received_value > amount_native {
                safe_transfer_native(caller, received_value - amount_native)?
            }

            Ok((amount_a, amount_native, liquidity))
        }

        #[modifiers(ensure_deadline(deadline))]
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
            let pair_contract = pair_for_on_chain(&self.router.factory, token_a, token_b)
                .ok_or(RouterError::PairNotFound)?;

            safe_transfer_from(pair_contract, self.env().caller(), pair_contract, liquidity)?;

            let (amount_0, amount_1) = match PairRef::burn_builder(&pair_contract, to)
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

        #[modifiers(ensure_deadline(deadline))]
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
            let wnative = self.router.wnative;
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

        #[modifiers(ensure_deadline(deadline))]
        #[ink(message)]
        fn swap_exact_tokens_for_tokens(
            &mut self,
            amount_in: Balance,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            let factory = self.router.factory;

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

        #[modifiers(ensure_deadline(deadline))]
        #[ink(message)]
        fn swap_tokens_for_exact_tokens(
            &mut self,
            amount_out: Balance,
            amount_in_max: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            let factory = self.router.factory;
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

        #[modifiers(ensure_deadline(deadline))]
        #[ink(message)]
        fn swap_exact_native_for_tokens(
            &mut self,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            let factory = self.router.factory;

            let received_value = self.env().transferred_value();
            let wnative = self.router.wnative;
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

        #[modifiers(ensure_deadline(deadline))]
        #[ink(message)]
        fn swap_tokens_for_exact_native(
            &mut self,
            amount_out: Balance,
            amount_in_max: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            let factory = self.router.factory;

            let wnative = self.router.wnative;
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

        #[modifiers(ensure_deadline(deadline))]
        #[ink(message)]
        fn swap_exact_tokens_for_native(
            &mut self,
            amount_in: Balance,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            let factory = self.router.factory;

            let wnative = self.router.wnative;
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

        #[modifiers(ensure_deadline(deadline))]
        #[ink(message)]
        fn swap_native_for_exact_tokens(
            &mut self,
            amount_out: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<Balance>, RouterError> {
            let factory = self.router.factory;
            let wnative = self.router.wnative;
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
            Ok(get_amounts_out(&self.router.factory, amount_in, &path)?)
        }

        #[ink(message)]
        fn get_amounts_in(
            &self,
            amount_out: Balance,
            path: Vec<AccountId>,
        ) -> Result<Vec<Balance>, RouterError> {
            Ok(get_amounts_in(&self.router.factory, amount_out, &path)?)
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
