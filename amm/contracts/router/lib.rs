#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod router {
    use amm_helpers::{
        ensure,
        math::casted_mul,
    };
    use ink::{
        codegen::TraitCallBuilder,
        contract_ref,
        env::CallFlags,
        prelude::{
            vec,
            vec::Vec,
        },
    };
    use psp22::{
        PSP22Error,
        PSP22,
    };
    use traits::{
        Factory,
        Pair,
        Router,
        RouterError,
        Wnative,
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

        #[inline]
        fn factory_ref(&self) -> contract_ref!(Factory) {
            self.factory.into()
        }

        #[inline]
        fn wnative_ref(&self) -> contract_ref!(Wnative) {
            self.wnative.into()
        }

        /// Returns address of a `Pair` contract instance (if exists) for
        /// `(token_a, token_b)` pair registered in `factory` Factory instance.
        #[inline]
        fn get_pair(
            &self,
            token_a: AccountId,
            token_b: AccountId,
        ) -> Result<AccountId, RouterError> {
            self.factory_ref()
                .get_pair(token_a, token_b)
                .ok_or(RouterError::PairNotFound)
        }

        #[inline]
        fn get_reserves(
            &self,
            token_a: AccountId,
            token_b: AccountId,
        ) -> Result<(u128, u128), RouterError> {
            ensure!(token_a != token_b, RouterError::IdenticalAddresses);
            let pair: contract_ref!(Pair) = self.get_pair(token_a, token_b)?.into();
            let (reserve_0, reserve_1, _) = pair.get_reserves();
            if token_a < token_b {
                Ok((reserve_0, reserve_1))
            } else {
                Ok((reserve_1, reserve_0))
            }
        }

        #[inline]
        fn wrap(&self, value: Balance) -> Result<(), RouterError> {
            match self
                .wnative_ref()
                .call_mut()
                .deposit()
                .transferred_value(value)
                .try_invoke()
            {
                Ok(res) => Ok(res??),
                Err(_) => Err(RouterError::TransferError),
            }
        }

        fn calculate_liquidity(
            &self,
            token_a: AccountId,
            token_b: AccountId,
            amount_a_desired: u128,
            amount_b_desired: u128,
            amount_a_min: u128,
            amount_b_min: u128,
        ) -> Result<(u128, u128), RouterError> {
            if self.get_pair(token_a, token_b).is_err() {
                self.factory_ref().create_pair(token_a, token_b)?;
            };

            let (reserve_a, reserve_b) = self.get_reserves(token_a, token_b)?;

            if reserve_a == 0 && reserve_b == 0 {
                return Ok((amount_a_desired, amount_b_desired))
            }

            let amount_b_optimal = self.quote(amount_a_desired, reserve_a, reserve_b)?;
            if amount_b_optimal <= amount_b_desired {
                ensure!(
                    amount_b_optimal >= amount_b_min,
                    RouterError::InsufficientBAmount
                );
                Ok((amount_a_desired, amount_b_optimal))
            } else {
                let amount_a_optimal = self.quote(amount_b_desired, reserve_b, reserve_a)?;
                // amount_a_optimal <= amount_a_desired holds as amount_b_optimal > amount_b_desired
                ensure!(
                    amount_a_optimal >= amount_a_min,
                    RouterError::InsufficientAAmount
                );
                Ok((amount_a_optimal, amount_b_desired))
            }
        }

        fn swap(
            &self,
            amounts: &[u128],
            path: &Vec<AccountId>,
            _to: AccountId,
        ) -> Result<(), RouterError> {
            for i in 0..path.len() - 1 {
                let (input, output) = (path[i], path[i + 1]);
                ensure!(input != output, RouterError::IdenticalAddresses);
                let amount_out = amounts[i + 1];
                let (amount_0_out, amount_1_out) = if input < output {
                    (0, amount_out)
                } else {
                    (amount_out, 0)
                };
                // If last pair in the path, transfer tokens to the `_to` recipient.
                // Otherwise, transfer to the next Pair contract instance.
                let to = if i < path.len() - 2 {
                    self.get_pair(output, path[i + 2])?
                } else {
                    _to
                };

                let mut pair: contract_ref!(Pair) = self.get_pair(input, output)?.into();
                match pair
                    .call_mut()
                    .swap(amount_0_out, amount_1_out, to)
                    .call_flags(CallFlags::default().set_allow_reentry(true))
                    .try_invoke()
                {
                    // TODO simplify
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

        /// Computes the amounts of tokens that have to be supplied
        /// at each step of the exchange `path`, to get exactly `amount_out`
        /// tokens at the end of the swaps.
        fn calculate_amounts_in(
            &self,
            amount_out: u128,
            path: &Vec<AccountId>,
        ) -> Result<Vec<u128>, RouterError> {
            ensure!(path.len() >= 2, RouterError::InvalidPath);

            let mut amounts = vec![0; path.len()];
            amounts[path.len() - 1] = amount_out;
            for i in (0..path.len() - 1).rev() {
                let (reserve_a, reserve_b) = self.get_reserves(path[i], path[i + 1])?;
                amounts[i] = self.get_amount_in(amounts[i + 1], reserve_a, reserve_b)?;
            }

            Ok(amounts)
        }

        /// Computes swap token amounts over the given path of token pairs.
        ///
        /// At each step, a swap for pair `(path[i], path[i+1])` is calculated,
        /// using tokens from the previous trade.
        ///
        /// Returns list of swap outcomes along the path.
        fn calculate_amounts_out(
            &self,
            amount_in: u128,
            path: &Vec<AccountId>,
        ) -> Result<Vec<u128>, RouterError> {
            ensure!(path.len() >= 2, RouterError::InvalidPath);

            let mut amounts = Vec::with_capacity(path.len());
            amounts.push(amount_in);
            for i in 0..path.len() - 1 {
                let (reserve_a, reserve_b) = self.get_reserves(path[i], path[i + 1])?;
                amounts.push(self.get_amount_out(amounts[i], reserve_a, reserve_b)?);
            }

            Ok(amounts)
        }

        /// Checks if the current block timestamp is not after the deadline.
        #[inline]
        fn check_timestamp(&self, deadline: u64) -> Result<(), RouterError> {
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
            amount_a_desired: u128,
            amount_b_desired: u128,
            amount_a_min: u128,
            amount_b_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128, u128), RouterError> {
            self.check_timestamp(deadline)?;
            let (amount_a, amount_b) = self.calculate_liquidity(
                token_a,
                token_b,
                amount_a_desired,
                amount_b_desired,
                amount_a_min,
                amount_b_min,
            )?;

            let pair_contract = self.get_pair(token_a, token_b)?;

            let caller = self.env().caller();
            psp22_transfer_from(token_a, caller, pair_contract, amount_a)?;
            psp22_transfer_from(token_b, caller, pair_contract, amount_b)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();
            let liquidity = pair.mint(to)?;

            Ok((amount_a, amount_b, liquidity))
        }

        #[ink(message)]
        fn add_liquidity_native(
            &mut self,
            token: AccountId,
            amount_token_desired: u128,
            amount_token_min: u128,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, Balance, u128), RouterError> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_value = self.env().transferred_value();

            let (amount_a, amount_native) = self.calculate_liquidity(
                token,
                wnative,
                amount_token_desired,
                received_value,
                amount_token_min,
                amount_native_min.into(),
            )?;

            let pair_contract = self.get_pair(token, wnative)?;

            let caller = self.env().caller();
            psp22_transfer_from(token, caller, pair_contract, amount_a)?;
            self.wrap(amount_native.into())?;
            psp22_transfer(wnative, pair_contract, amount_native)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();
            let liquidity = pair.mint(to)?;

            if received_value > amount_native {
                self.env()
                    .transfer(caller, received_value - amount_native)
                    .map_err(|_| RouterError::TransferError)?;
            }

            Ok((amount_a, amount_native.into(), liquidity))
        }

        #[ink(message)]
        fn remove_liquidity(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
            liquidity: u128,
            amount_a_min: u128,
            amount_b_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128), RouterError> {
            self.check_timestamp(deadline)?;
            ensure!(token_a != token_b, RouterError::IdenticalAddresses);
            let pair_contract = self.get_pair(token_a, token_b)?;

            psp22_transfer_from(pair_contract, self.env().caller(), pair_contract, liquidity)?;

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
            let (amount_a, amount_b) = if token_a < token_b {
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
            liquidity: u128,
            amount_token_min: u128,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, Balance), RouterError> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let (amount_token, amount_native) = self.remove_liquidity(
                token,
                wnative,
                liquidity,
                amount_token_min,
                amount_native_min.into(),
                self.env().account_id(),
                deadline,
            )?;
            psp22_transfer(token, to, amount_token)?;
            self.wnative_ref().withdraw(amount_native)?;
            self.env()
                .transfer(to, amount_native)
                .map_err(|_| RouterError::TransferError)?;
            Ok((amount_token, amount_native.into()))
        }

        #[ink(message)]
        fn swap_exact_tokens_for_tokens(
            &mut self,
            amount_in: u128,
            amount_out_min: u128,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterError> {
            self.check_timestamp(deadline)?;
            let amounts = self.calculate_amounts_out(amount_in, &path)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterError::InsufficientOutputAmount
            );
            psp22_transfer_from(
                path[0],
                self.env().caller(),
                self.get_pair(path[0], path[1])?,
                amounts[0],
            )?;
            self.swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_tokens(
            &mut self,
            amount_out: u128,
            amount_in_max: u128,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterError> {
            self.check_timestamp(deadline)?;
            let amounts = self.calculate_amounts_in(amount_out, &path)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterError::ExcessiveInputAmount
            );
            psp22_transfer_from(
                path[0],
                self.env().caller(),
                self.get_pair(path[0], path[1])?,
                amounts[0],
            )?;
            self.swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_exact_native_for_tokens(
            &mut self,
            amount_out_min: u128,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterError> {
            self.check_timestamp(deadline)?;
            let received_value = self.env().transferred_value();
            let wnative = self.wnative;
            ensure!(path[0] == wnative, RouterError::InvalidPath);
            let amounts = self.calculate_amounts_out(received_value, &path)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterError::InsufficientOutputAmount
            );
            self.wrap(received_value)?;
            psp22_transfer(wnative, self.get_pair(path[0], path[1])?, amounts[0])?;
            self.swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_native(
            &mut self,
            amount_out: Balance,
            amount_in_max: u128,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterError> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            ensure!(path[path.len() - 1] == wnative, RouterError::InvalidPath);
            let amounts = self.calculate_amounts_in(amount_out.into(), &path)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterError::ExcessiveInputAmount
            );
            psp22_transfer_from(
                path[0],
                self.env().caller(),
                self.get_pair(path[0], path[1])?,
                amounts[0],
            )?;
            self.swap(&amounts, &path, self.env().account_id())?;
            let native_out = amounts[amounts.len() - 1];
            self.wnative_ref().withdraw(native_out)?;
            self.env()
                .transfer(to, native_out.into())
                .map_err(|_| RouterError::TransferError)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_exact_tokens_for_native(
            &mut self,
            amount_in: u128,
            amount_out_min: Balance,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterError> {
            self.check_timestamp(deadline)?;
            ensure!(
                path[path.len() - 1] == self.wnative,
                RouterError::InvalidPath
            );
            let amounts = self.calculate_amounts_out(amount_in, &path)?;
            let native_out = amounts[amounts.len() - 1];
            ensure!(
                native_out >= amount_out_min.into(),
                RouterError::InsufficientOutputAmount
            );
            psp22_transfer_from(
                path[0],
                self.env().caller(),
                self.get_pair(path[0], path[1])?,
                amounts[0],
            )?;
            self.swap(&amounts, &path, self.env().account_id())?;
            self.wnative_ref().withdraw(native_out)?;
            self.env()
                .transfer(to, native_out.into())
                .map_err(|_| RouterError::TransferError)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_native_for_exact_tokens(
            &mut self,
            amount_out: u128,
            path: Vec<AccountId>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterError> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_native = self.env().transferred_value();
            ensure!(path[0] == wnative, RouterError::InvalidPath);
            let amounts = self.calculate_amounts_in(amount_out, &path)?;
            let native_in: Balance = amounts[0];
            ensure!(
                native_in <= received_native,
                RouterError::ExcessiveInputAmount
            );
            self.wrap(native_in)?;
            psp22_transfer(wnative, self.get_pair(path[0], path[1])?, native_in.into())?;
            self.swap(&amounts, &path, to)?;
            if received_native > native_in {
                self.env()
                    .transfer(self.env().caller(), received_native - native_in)
                    .map_err(|_| RouterError::TransferError)?;
            }
            Ok(amounts)
        }

        /// Returns how much of `token_B` tokens should be added
        /// to the pool to maintain the constant product `k = reserve_a * reserve_b`,
        /// given `amount_a` of `token_A`.
        #[ink(message)]
        fn quote(
            &self,
            amount_a: u128,
            reserve_a: u128,
            reserve_b: u128,
        ) -> Result<u128, RouterError> {
            ensure!(amount_a > 0, RouterError::InsufficientAmount);
            ensure!(
                reserve_a > 0 && reserve_b > 0,
                RouterError::InsufficientLiquidity
            );

            let amount_b: u128 = casted_mul(amount_a, reserve_b)
                .checked_div(reserve_a.into())
                .ok_or(RouterError::DivByZero)?
                .try_into()
                .map_err(|_| RouterError::CastOverFlow)?;

            Ok(amount_b)
        }

        /// Returns amount of `B` tokens received
        /// for `amount_in` of `A` tokens that maintains
        /// the constant product of `k = reserve_a * reserve_b`.
        #[ink(message)]
        fn get_amount_out(
            &self,
            amount_in: u128,
            reserve_a: u128,
            reserve_b: u128,
        ) -> Result<u128, RouterError> {
            ensure!(amount_in > 0, RouterError::InsufficientAmount);
            ensure!(
                reserve_a > 0 && reserve_b > 0,
                RouterError::InsufficientLiquidity
            );

            // Adjusts for fees paid in the `token_in`.
            let amount_in_with_fee = casted_mul(amount_in, 997);

            let numerator = amount_in_with_fee
                .checked_mul(reserve_b.into())
                .ok_or(RouterError::MulOverFlow)?;

            let denominator = casted_mul(reserve_a, 1000)
                .checked_add(amount_in_with_fee)
                .ok_or(RouterError::AddOverFlow)?;

            let amount_out: u128 = numerator
                .checked_div(denominator)
                .ok_or(RouterError::DivByZero)?
                .try_into()
                .map_err(|_| RouterError::CastOverFlow)?;

            Ok(amount_out)
        }

        /// Returns amount of `A` tokens user has to supply
        /// to get exactly `amount_out` of `B` token while maintaining
        /// the constant product of `k = reserve_a * reserve_b`.
        #[ink(message)]
        fn get_amount_in(
            &self,
            amount_out: u128,
            reserve_a: u128,
            reserve_b: u128,
        ) -> Result<u128, RouterError> {
            ensure!(amount_out > 0, RouterError::InsufficientAmount);
            ensure!(
                reserve_a > 0 && reserve_b > 0,
                RouterError::InsufficientLiquidity
            );

            let numerator = casted_mul(reserve_a, amount_out)
                .checked_mul(1000.into())
                .ok_or(RouterError::MulOverFlow)?;

            let denominator = casted_mul(
                reserve_b
                    .checked_sub(amount_out)
                    .ok_or(RouterError::SubUnderFlow)?,
                997,
            );

            let amount_in: u128 = numerator
                .checked_div(denominator)
                .ok_or(RouterError::DivByZero)?
                .checked_add(1.into())
                .ok_or(RouterError::AddOverFlow)?
                .try_into()
                .map_err(|_| RouterError::CastOverFlow)?;

            Ok(amount_in)
        }

        #[ink(message)]
        fn get_amounts_out(
            &self,
            amount_in: u128,
            path: Vec<AccountId>,
        ) -> Result<Vec<u128>, RouterError> {
            self.calculate_amounts_out(amount_in, &path)
        }

        #[ink(message)]
        fn get_amounts_in(
            &self,
            amount_out: u128,
            path: Vec<AccountId>,
        ) -> Result<Vec<u128>, RouterError> {
            self.calculate_amounts_in(amount_out, &path)
        }
    }

    #[inline]
    fn psp22_transfer(token: AccountId, to: AccountId, value: u128) -> Result<(), PSP22Error> {
        let mut token: contract_ref!(PSP22) = token.into();
        token.transfer(to, value, Vec::new())
    }

    #[inline]
    fn psp22_transfer_from(
        token: AccountId,
        from: AccountId,
        to: AccountId,
        value: u128,
    ) -> Result<(), PSP22Error> {
        let mut token: contract_ref!(PSP22) = token.into();
        token.transfer_from(from, to, value, Vec::new())
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
