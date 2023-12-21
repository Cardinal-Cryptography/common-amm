#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod router {
    // Trading fee is 0.3%. This is the same as in Uniswap.
    // Adjusted to not deal with floating point numbers.
    const TRADING_FEE_ADJ_DENOM: u128 = 1000;
    const TRADING_FEE_ADJ_NUM: u128 = 997;

    use amm_helpers::{ensure, math::casted_mul};
    use ink::{
        codegen::TraitCallBuilder,
        contract_ref,
        prelude::{string::String, vec, vec::Vec},
    };
    use psp22::{PSP22Error, PSP22};
    use traits::{Factory, MathError, Pair, Router, RouterError};
    use wrapped_azero::WrappedAZERO;

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
        fn wnative_ref(&self) -> contract_ref!(WrappedAZERO) {
            self.wnative.into()
        }

        /// Returns address of a `Pair` contract instance (if exists) for
        /// `(token_0, token_1)` pair registered in `factory` Factory instance.
        #[inline]
        fn get_pair(
            &self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<AccountId, RouterError> {
            self.factory_ref()
                .get_pair(token_0, token_1)
                .ok_or(RouterError::PairNotFound)
        }

        #[inline]
        fn get_reserves(
            &self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<(u128, u128), RouterError> {
            ensure!(token_0 != token_1, RouterError::IdenticalAddresses);
            let pair: contract_ref!(Pair) = self.get_pair(token_0, token_1)?.into();
            let (reserve_0, reserve_1, _) = pair.get_reserves();
            if token_0 < token_1 {
                Ok((reserve_0, reserve_1))
            } else {
                Ok((reserve_1, reserve_0))
            }
        }

        #[inline]
        fn wrap(&self, value: Balance) -> Result<(), RouterError> {
            self.wnative_ref()
                .call_mut()
                .deposit()
                .transferred_value(value)
                .try_invoke()
                .map_err(|_| {
                    RouterError::CrossContractCallFailed(String::from("Wrapped AZERO: deposit"))
                })???;
            Ok(())
        }

        fn calculate_liquidity(
            &self,
            token_0: AccountId,
            token_1: AccountId,
            amount_0_desired: u128,
            amount_1_desired: u128,
            amount_0_min: u128,
            amount_1_min: u128,
        ) -> Result<(u128, u128), RouterError> {
            if self.get_pair(token_0, token_1).is_err() {
                self.factory_ref().create_pair(token_0, token_1)?;
            };

            let (reserve_0, reserve_1) = self.get_reserves(token_0, token_1)?;

            if reserve_0 == 0 && reserve_1 == 0 {
                return Ok((amount_0_desired, amount_1_desired));
            }

            let amount_1_optimal = self.quote(amount_0_desired, reserve_0, reserve_1)?;
            if amount_1_optimal <= amount_1_desired {
                ensure!(
                    amount_1_optimal >= amount_1_min,
                    RouterError::InsufficientAmountB
                );
                Ok((amount_0_desired, amount_1_optimal))
            } else {
                let amount_0_optimal = self.quote(amount_1_desired, reserve_1, reserve_0)?;
                // amount_0_optimal <= amount_0_desired holds as amount_1_optimal > amount_1_desired
                ensure!(
                    amount_0_optimal >= amount_0_min,
                    RouterError::InsufficientAmountA
                );
                Ok((amount_0_optimal, amount_1_desired))
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
                pair.call_mut()
                    .swap(amount_0_out, amount_1_out, to, None)
                    .try_invoke()
                    .map_err(|_| {
                        RouterError::CrossContractCallFailed(String::from("Pair:swap"))
                    })???;
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
                let (reserve_0, reserve_1) = self.get_reserves(path[i], path[i + 1])?;
                amounts[i] = self.get_amount_in(amounts[i + 1], reserve_0, reserve_1)?;
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
                let (reserve_0, reserve_1) = self.get_reserves(path[i], path[i + 1])?;
                amounts.push(self.get_amount_out(amounts[i], reserve_0, reserve_1)?);
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
            token_0: AccountId,
            token_1: AccountId,
            amount_0_desired: u128,
            amount_1_desired: u128,
            amount_0_min: u128,
            amount_1_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128, u128), RouterError> {
            self.check_timestamp(deadline)?;
            let (amount_0, amount_1) = self.calculate_liquidity(
                token_0,
                token_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min,
            )?;

            let pair_contract = self.get_pair(token_0, token_1)?;

            let caller = self.env().caller();
            psp22_transfer_from(token_0, caller, pair_contract, amount_0)?;
            psp22_transfer_from(token_1, caller, pair_contract, amount_1)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();
            let liquidity = pair.mint(to)?;

            Ok((amount_0, amount_1, liquidity))
        }

        #[ink(message, payable)]
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

            let (amount_0, amount_native) = self.calculate_liquidity(
                token,
                wnative,
                amount_token_desired,
                received_value,
                amount_token_min,
                amount_native_min,
            )?;

            let pair_contract = self.get_pair(token, wnative)?;

            let caller = self.env().caller();
            psp22_transfer_from(token, caller, pair_contract, amount_0)?;
            self.wrap(amount_native)?;
            psp22_transfer(wnative, pair_contract, amount_native)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();
            let liquidity = pair.mint(to)?;

            if received_value > amount_native {
                self.env()
                    .transfer(caller, received_value - amount_native)
                    .map_err(|_| RouterError::TransferError)?;
            }

            Ok((amount_0, amount_native, liquidity))
        }

        #[ink(message)]
        fn remove_liquidity(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            liquidity: u128,
            amount_0_min: u128,
            amount_1_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128), RouterError> {
            self.check_timestamp(deadline)?;
            ensure!(token_0 != token_1, RouterError::IdenticalAddresses);
            let pair_contract = self.get_pair(token_0, token_1)?;

            psp22_transfer_from(pair_contract, self.env().caller(), pair_contract, liquidity)?;

            let mut pair: contract_ref!(Pair) = pair_contract.into();

            let (amount_0, amount_1) =
                pair.call_mut().burn(to).try_invoke().map_err(|_| {
                    RouterError::CrossContractCallFailed(String::from("Pair:burn"))
                })???;
            let (amount_0, amount_1) = if token_0 < token_1 {
                (amount_0, amount_1)
            } else {
                (amount_1, amount_0)
            };

            ensure!(amount_0 >= amount_0_min, RouterError::InsufficientAmountA);
            ensure!(amount_1 >= amount_1_min, RouterError::InsufficientAmountB);

            Ok((amount_0, amount_1))
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
                amount_native_min,
                self.env().account_id(),
                deadline,
            )?;
            psp22_transfer(token, to, amount_token)?;
            self.wnative_ref().withdraw(amount_native)?;
            self.env()
                .transfer(to, amount_native)
                .map_err(|_| RouterError::TransferError)?;
            Ok((amount_token, amount_native))
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

        #[ink(message, payable)]
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
            self.swap(&amounts, &path, self.env().account_id())?;
            let native_out = amounts[amounts.len() - 1];
            self.wnative_ref().withdraw(native_out)?;
            self.env()
                .transfer(to, native_out)
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
                native_out >= amount_out_min,
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
                .transfer(to, native_out)
                .map_err(|_| RouterError::TransferError)?;
            Ok(amounts)
        }

        #[ink(message, payable)]
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
            psp22_transfer(wnative, self.get_pair(path[0], path[1])?, native_in)?;
            self.swap(&amounts, &path, to)?;
            if received_native > native_in {
                self.env()
                    .transfer(self.env().caller(), received_native - native_in)
                    .map_err(|_| RouterError::TransferError)?;
            }
            Ok(amounts)
        }

        /// Returns how much of `token_B` tokens should be added
        /// to the pool to maintain the constant ratio `k = reserve_0 / reserve_1`,
        /// given `amount_0` of `token_A`.
        #[ink(message)]
        fn quote(
            &self,
            amount_0: u128,
            reserve_0: u128,
            reserve_1: u128,
        ) -> Result<u128, RouterError> {
            ensure!(amount_0 > 0, RouterError::InsufficientAmount);
            ensure!(
                reserve_0 > 0 && reserve_1 > 0,
                RouterError::InsufficientLiquidity
            );

            let amount_1: u128 = casted_mul(amount_0, reserve_1)
                .checked_div(reserve_0.into())
                .ok_or(MathError::DivByZero(6))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(3))?;

            Ok(amount_1)
        }

        /// Returns amount of `B` tokens received
        /// for `amount_in` of `A` tokens that maintains
        /// the constant ratio of `k = reserve_0 / reserve_1`.
        #[ink(message)]
        fn get_amount_out(
            &self,
            amount_in: u128,
            reserve_0: u128,
            reserve_1: u128,
        ) -> Result<u128, RouterError> {
            ensure!(amount_in > 0, RouterError::InsufficientAmount);
            ensure!(
                reserve_0 > 0 && reserve_1 > 0,
                RouterError::InsufficientLiquidity
            );

            // Adjusts for fees paid in the `token_in`.
            let amount_in_with_fee = casted_mul(amount_in, TRADING_FEE_ADJ_NUM);

            let numerator = amount_in_with_fee
                .checked_mul(reserve_1.into())
                .ok_or(MathError::MulOverflow(13))?;

            let denominator = casted_mul(reserve_0, 1000)
                .checked_add(amount_in_with_fee)
                .ok_or(MathError::AddOverflow(2))?;

            let amount_out: u128 = numerator
                .checked_div(denominator)
                .ok_or(MathError::DivByZero(7))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(4))?;

            Ok(amount_out)
        }

        /// Returns amount of `A` tokens user has to supply
        /// to get exactly `amount_out` of `B` token while maintaining
        /// the constant ratio of `k = reserve_0 / reserve_1`.
        #[ink(message)]
        fn get_amount_in(
            &self,
            amount_out: u128,
            reserve_0: u128,
            reserve_1: u128,
        ) -> Result<u128, RouterError> {
            ensure!(amount_out > 0, RouterError::InsufficientAmount);
            ensure!(
                reserve_0 > 0 && reserve_1 > 0,
                RouterError::InsufficientLiquidity
            );

            let numerator = casted_mul(reserve_0, amount_out)
                .checked_mul(TRADING_FEE_ADJ_DENOM.into())
                .ok_or(MathError::MulOverflow(14))?;

            let denominator = casted_mul(
                reserve_1
                    .checked_sub(amount_out)
                    .ok_or(MathError::SubUnderflow(15))?,
                TRADING_FEE_ADJ_NUM,
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
