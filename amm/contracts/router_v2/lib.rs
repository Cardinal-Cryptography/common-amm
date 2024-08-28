#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod pool;
mod pair;
mod stable_pool;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CallerIsNotOwner;

#[ink::contract]
pub mod router_v2 {
    use crate::{
        pool::{Pair, Pool, StablePool},
        CallerIsNotOwner,
    };
    use amm_helpers::ensure;
    use ink::{
        codegen::TraitCallBuilder,
        contract_ref,
        prelude::{string::String, vec, vec::Vec},
        storage::Mapping,
    };
    use psp22::{PSP22Error, PSP22};
    use traits::{Factory, Pair as _, PoolId, RouterV2, RouterV2Error, Step};
    use wrapped_azero::WrappedAZERO;

    struct ValidStep {
        token_in: AccountId,
        pool: Pool,
    }

    #[ink(storage)]
    pub struct RouterV2Contract {
        pair_factory: AccountId,
        wnative: AccountId,
        owner: AccountId,
        /// Mapping of cached Pairs. Maps `(token_0, token_1) => Pair`
        cached_pairs: Mapping<(AccountId, AccountId), Pair>,
        /// Mapping of cached StablePools, Maps `pool_id => StablePool`
        cached_stable_pools: Mapping<AccountId, StablePool>,
    }

    impl RouterV2Contract {
        #[ink(constructor)]
        pub fn new(pair_factory: AccountId, wnative: AccountId) -> Self {
            Self {
                pair_factory,
                wnative,
                owner: Self::env().caller(),
                cached_pairs: Default::default(),
                cached_stable_pools: Default::default(),
            }
        }

        #[ink(message)]
        pub fn owner(&self) -> AccountId {
            self.owner
        }

        #[ink(message)]
        pub fn set_owner(&mut self, new_owner: AccountId) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            self.owner = new_owner;
            Ok(())
        }

        #[ink(message)]
        pub fn read_cached_pair(&self, token_0: AccountId, token_1: AccountId) -> Option<Pair> {
            self.cached_pairs.get((token_0, token_1))
        }

        #[ink(message)]
        pub fn read_cached_stable_pool(&self, pool_id: AccountId) -> Option<StablePool> {
            self.cached_stable_pools.get(pool_id)
        }

        #[ink(message)]
        pub fn add_pair_to_cache(&mut self, pair: AccountId) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            _ = self.cache_pair(pair);
            Ok(())
        }

        #[ink(message)]
        pub fn add_stable_pool_to_cache(
            &mut self,
            pool_id: AccountId,
        ) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            self.cached_stable_pools
                .insert(pool_id, &StablePool::new(pool_id));
            Ok(())
        }

        #[ink(message)]
        pub fn remove_pair_from_cache(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            self.cached_pairs.remove((token_0, token_1));
            self.cached_pairs.remove((token_1, token_0));
            Ok(())
        }

        #[ink(message)]
        pub fn remove_stable_pool_from_cache(
            &mut self,
            pool_id: AccountId,
        ) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            self.cached_stable_pools.remove(pool_id);
            Ok(())
        }

        #[inline]
        fn cache_pair(&mut self, pair: AccountId) -> Pair {
            let pair = Pair::new(pair);
            let token_0 = pair.contract_ref().get_token_0();
            let token_1 = pair.contract_ref().get_token_1();
            self.cached_pairs.insert((token_0, token_1), &pair);
            self.cached_pairs.insert((token_1, token_0), &pair);
            pair
        }

        // ----------- HELPER METHODS ----------- //

        #[inline]
        fn get_pair(&self, token_0: AccountId, token_1: AccountId) -> Result<Pair, RouterV2Error> {
            if let Some(pair) = self.cached_pairs.get((token_0, token_1)) {
                Ok(pair)
            } else {
                let pair_id = self
                    .pair_factory_ref()
                    .get_pair(token_0, token_1)
                    .ok_or(RouterV2Error::PairNotFound)?;
                Ok(Pair::new(pair_id))
            }
        }

        /// Returns
        /// 1. `Pool::StablePool` for given `pool_id` if one exists in the cache,
        /// 2. `Pool::Pair` for given `pair` tokens if pair exists in the cache
        /// 3. `Pool::Pair` for given `pair` tokens if pair was deployed with the Factory contract.
        /// 4. `RouterV2Error::PoolNotFound` error if the pool was not found.
        #[inline]
        fn get_pool(
            &self,
            pool_id: Option<PoolId>,
            pair: (AccountId, AccountId),
        ) -> Result<Pool, RouterV2Error> {
            if let Some(pool_id) = pool_id {
                if let Some(pool) = self.cached_stable_pools.get(pool_id) {
                    return Ok(Pool::StablePool(pool));
                }
            }
            match self.get_pair(pair.0, pair.1) {
                Ok(pair) => Ok(Pool::Pair(pair)),
                Err(_) => Err(RouterV2Error::PoolNotFound),
            }
        }

         /// Returns cached Pair or Pair created with the Factory contract for `(token_0, token_1)` tokens.
        ///
        /// If Pair exists but was not cached, it adds the Pair to the cache.
        #[inline]
        fn get_and_cache_pair(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<Pair, RouterV2Error> {
            if let Some(pair) = self.cached_pairs.get((token_0, token_1)) {
                Ok(pair)
            } else {
                let pair = self
                    .pair_factory_ref()
                    .get_pair(token_0, token_1)
                    .ok_or(RouterV2Error::PairNotFound)?;
                Ok(self.cache_pair(pair))
            }
        }

        /// Returns cached Pair or Pair created with the Factory contract for `(token_0, token_1)` tokens.
        ///
        /// If Pair does not exist, it creates one and adds it to the cache.
        #[inline]
        fn get_or_create_pair(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<Pair, RouterV2Error> {
            match self.get_and_cache_pair(token_0, token_1) {
                Ok(pair) => Ok(pair),
                Err(_) => {
                    let new_pair = self.pair_factory_ref().create_pair(token_0, token_1)?;
                    Ok(self.cache_pair(new_pair))
                }
            }
        }

        fn swap(
            &mut self,
            amounts: &[u128],
            valid_path: &[ValidStep],
            token_out: AccountId,
            to: AccountId,
        ) -> Result<(), RouterV2Error> {
            for i in 0..valid_path.len() - 1 {
                valid_path[i].pool.swap(
                    valid_path[i].token_in,
                    valid_path[i + 1].token_in,
                    amounts[i + 1],
                    valid_path[i + 1].pool.pool_id(),
                )?;
            }
            // If last pool in the path, transfer tokens to the `to` recipient.
            valid_path[valid_path.len() - 1].pool.swap(
                valid_path[valid_path.len() - 1].token_in,
                token_out,
                amounts[valid_path.len()],
                to,
            )?;
            Ok(())
        }

        /// Computes the amounts of tokens that have to be supplied
        /// at each step of the exchange `path`, to get exactly `amount_out` of `token_out`
        /// tokens at the end of the swaps.
        fn calculate_amounts_in(
            &self,
            amount_out: u128,
            valid_path: &[ValidStep],
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let mut amounts = vec![0; valid_path.len() + 1];
            amounts[valid_path.len() - 1] = amount_out;
            amounts[valid_path.len() - 2] = valid_path[valid_path.len() - 2].pool.get_amount_in(
                valid_path[valid_path.len() - 2].token_in,
                token_out,
                amount_out,
            )?;
            for i in (0..valid_path.len() - 1).rev() {
                amounts[i] = valid_path[i].pool.get_amount_in(
                    valid_path[i].token_in,
                    valid_path[i + 1].token_in,
                    amounts[i + 1],
                )?;
            }

            Ok(amounts)
        }

        /// Computes swap token amounts over the given path.
        ///
        /// Returns list of swap outcomes along the path.
        fn calculate_amounts_out(
            &self,
            amount_in: u128,
            valid_path: &[ValidStep],
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let mut amounts = Vec::with_capacity(valid_path.len() + 1);
            amounts.push(amount_in);
            for i in 0..valid_path.len() - 1 {
                amounts.push(valid_path[i].pool.get_amount_out(
                    valid_path[i].token_in,
                    valid_path[i + 1].token_in,
                    amounts[i],
                )?);
            }
            amounts.push(valid_path[valid_path.len() - 1].pool.get_amount_out(
                valid_path[valid_path.len() - 1].token_in,
                token_out,
                amounts[valid_path.len() - 1],
            )?);

            Ok(amounts)
        }

        /// Checks if the path is valid.
        ///
        /// Returns valid path
        fn validate_path(
            &self,
            path: &[Step],
            token_out: AccountId,
        ) -> Result<Vec<ValidStep>, RouterV2Error> {
            ensure!(path.len() >= 1, RouterV2Error::InvalidPath);
            let mut valid_path = Vec::with_capacity(path.len());
            for i in 0..(path.len() - 1) {
                ensure!(
                    path[i].token_in != path[i + 1].token_in,
                    RouterV2Error::IdenticalAddresses
                );
                valid_path.push(ValidStep {
                    token_in: path[i].token_in,
                    pool: self
                        .get_pool(path[i].pool_id, (path[i].token_in, path[i + 1].token_in))?,
                });
            }
            ensure!(
                path[path.len() - 1].token_in != token_out,
                RouterV2Error::IdenticalAddresses
            );
            valid_path.push(ValidStep {
                token_in: path[path.len() - 1].token_in,
                pool: self.get_pool(
                    path[path.len() - 1].pool_id,
                    (path[path.len() - 1].token_in, token_out),
                )?,
            });
            Ok(valid_path)
        }

        #[inline]
        fn pair_factory_ref(&self) -> contract_ref!(Factory) {
            self.pair_factory.into()
        }

        #[inline]
        fn wnative_ref(&self) -> contract_ref!(WrappedAZERO) {
            self.wnative.into()
        }

        /// Checks if the current block timestamp is not after the deadline.
        #[inline]
        fn check_timestamp(&self, deadline: u64) -> Result<(), RouterV2Error> {
            ensure!(
                deadline >= self.env().block_timestamp(),
                RouterV2Error::Expired
            );
            Ok(())
        }

        #[inline]
        fn wrap(&self, value: Balance) -> Result<(), RouterV2Error> {
            self.wnative_ref()
                .call_mut()
                .deposit()
                .transferred_value(value)
                .try_invoke()
                .map_err(|_| {
                    RouterV2Error::CrossContractCallFailed(String::from("Wrapped AZERO: deposit"))
                })???;
            Ok(())
        }
    }

    impl RouterV2 for RouterV2Contract {
        #[ink(message)]
        fn pair_factory(&self) -> AccountId {
            self.pair_factory
        }

        #[ink(message)]
        fn wnative(&self) -> AccountId {
            self.wnative
        }

        // ----------- SWAP METHODS ----------- //

        #[ink(message)]
        fn swap_exact_tokens_for_tokens(
            &mut self,
            amount_in: u128,
            amount_out_min: u128,
            path: Vec<Step>,
            token_out: AccountId,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let valid_path = self.validate_path(&path, token_out)?;
            let amounts = self.calculate_amounts_out(amount_in, &valid_path, token_out)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            psp22_transfer_from(
                valid_path[0].token_in,
                self.env().caller(),
                valid_path[0].pool.pool_id(),
                amounts[0],
            )?;
            self.swap(&amounts, &valid_path, token_out, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_tokens(
            &mut self,
            amount_out: u128,
            amount_in_max: u128,
            path: Vec<Step>,
            token_out: AccountId,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let valid_path = self.validate_path(&path, token_out)?;
            let amounts = self.calculate_amounts_in(amount_out, &valid_path, token_out)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterV2Error::ExcessiveInputAmount
            );
            psp22_transfer_from(
                valid_path[0].token_in,
                self.env().caller(),
                valid_path[0].pool.pool_id(),
                amounts[0],
            )?;
            self.swap(&amounts, &valid_path, token_out, to)?;
            Ok(amounts)
        }

        #[ink(message, payable)]
        fn swap_exact_native_for_tokens(
            &mut self,
            amount_out_min: u128,
            path: Vec<Step>,
            token_out: AccountId,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let received_value = self.env().transferred_value();
            let wnative = self.wnative;
            ensure!(path[0].token_in == wnative, RouterV2Error::InvalidPath);
            let valid_path = self.validate_path(&path, token_out)?;
            let amounts = self.calculate_amounts_out(received_value, &valid_path, token_out)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            self.wrap(received_value)?;
            psp22_transfer(wnative, valid_path[0].pool.pool_id(), amounts[0])?;
            self.swap(&amounts, &valid_path, token_out, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_native(
            &mut self,
            amount_out: Balance,
            amount_in_max: u128,
            path: Vec<Step>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let valid_path = self.validate_path(&path, wnative)?;
            let amounts = self.calculate_amounts_in(amount_out, &valid_path, wnative)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterV2Error::ExcessiveInputAmount
            );
            psp22_transfer_from(
                valid_path[0].token_in,
                self.env().caller(),
                valid_path[0].pool.pool_id(),
                amounts[0],
            )?;
            self.swap(&amounts, &valid_path, wnative, self.env().account_id())?;
            let native_out = amounts[amounts.len() - 1];
            self.wnative_ref().withdraw(native_out)?;
            self.env()
                .transfer(to, native_out)
                .map_err(|_| RouterV2Error::TransferError)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_exact_tokens_for_native(
            &mut self,
            amount_in: u128,
            amount_out_min: Balance,
            path: Vec<Step>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let valid_path = self.validate_path(&path, wnative)?;
            let amounts = self.calculate_amounts_out(amount_in, &valid_path, wnative)?;
            let native_out = amounts[amounts.len() - 1];
            ensure!(
                native_out >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            psp22_transfer_from(
                valid_path[0].token_in,
                self.env().caller(),
                valid_path[0].pool.pool_id(),
                amounts[0],
            )?;
            self.swap(&amounts, &valid_path, wnative, self.env().account_id())?;
            self.wnative_ref().withdraw(native_out)?;
            self.env()
                .transfer(to, native_out)
                .map_err(|_| RouterV2Error::TransferError)?;
            Ok(amounts)
        }

        #[ink(message, payable)]
        fn swap_native_for_exact_tokens(
            &mut self,
            amount_out: u128,
            path: Vec<Step>,
            token_out: AccountId,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_native = self.env().transferred_value();
            ensure!(path[0].token_in == wnative, RouterV2Error::InvalidPath);
            let valid_path = self.validate_path(&path, token_out)?;
            let amounts = self.calculate_amounts_in(amount_out, &valid_path, token_out)?;
            let native_in: Balance = amounts[0];
            ensure!(
                native_in <= received_native,
                RouterV2Error::ExcessiveInputAmount
            );
            self.wrap(native_in)?;
            psp22_transfer(wnative, path[0].token_in, native_in)?;
            self.swap(&amounts, &valid_path, token_out, to)?;
            if received_native > native_in {
                self.env()
                    .transfer(self.env().caller(), received_native - native_in)
                    .map_err(|_| RouterV2Error::TransferError)?;
            }
            Ok(amounts)
        }

        #[ink(message)]
        fn get_amounts_out(
            &self,
            amount_in: u128,
            path: Vec<Step>,
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let valid_path = self.validate_path(&path, token_out)?;
            self.calculate_amounts_out(amount_in, &valid_path, token_out)
        }

        #[ink(message)]
        fn get_amounts_in(
            &self,
            amount_out: u128,
            path: Vec<Step>,
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let valid_path = self.validate_path(&path, token_out)?;
            self.calculate_amounts_in(amount_out, &valid_path, token_out)
        }

        // ----------- PAIR LIQUIDITY METHODS ----------- //

        #[ink(message)]
        fn add_pair_liquidity(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            amount_0_desired: u128,
            amount_1_desired: u128,
            amount_0_min: u128,
            amount_1_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128, u128), RouterV2Error> {
            self.check_timestamp(deadline)?;
            let pair = self.get_or_create_pair(token_0, token_1)?;
            let (amount_0, amount_1) = pair.calculate_liquidity(
                token_0,
                token_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min,
            )?;

            let caller = self.env().caller();
            psp22_transfer_from(token_0, caller, pair.pool_id(), amount_0)?;
            psp22_transfer_from(token_1, caller, pair.pool_id(), amount_1)?;

            let liquidity = pair.contract_ref().mint(to)?;

            Ok((amount_0, amount_1, liquidity))
        }

        #[ink(message, payable)]
        fn add_pair_liquidity_native(
            &mut self,
            token: AccountId,
            amount_token_desired: u128,
            amount_token_min: u128,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, Balance, u128), RouterV2Error> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_value = self.env().transferred_value();

            let pair = self.get_or_create_pair(token, wnative)?;
            let (amount_0, amount_native) = pair.calculate_liquidity(
                token,
                wnative,
                amount_token_desired,
                received_value,
                amount_token_min,
                amount_native_min,
            )?;

            let caller = self.env().caller();
            psp22_transfer_from(token, caller, pair.pool_id(), amount_0)?;
            self.wrap(amount_native)?;
            psp22_transfer(wnative, pair.pool_id(), amount_native)?;

            let liquidity = pair.contract_ref().mint(to)?;

            if received_value > amount_native {
                self.env()
                    .transfer(caller, received_value - amount_native)
                    .map_err(|_| RouterV2Error::TransferError)?;
            }

            Ok((amount_0, amount_native, liquidity))
        }

        #[ink(message)]
        fn remove_pair_liquidity(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
            liquidity: u128,
            amount_0_min: u128,
            amount_1_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128), RouterV2Error> {
            self.check_timestamp(deadline)?;
            ensure!(token_0 != token_1, RouterV2Error::IdenticalAddresses);
            let pair = self.get_pair(token_0, token_1)?;

            psp22_transfer_from(pair.pool_id(), self.env().caller(), pair.pool_id(), liquidity)?;

            let (amount_0, amount_1) =
                pair.contract_ref().call_mut().burn(to).try_invoke().map_err(|_| {
                    RouterV2Error::CrossContractCallFailed(String::from("Pair:burn"))
                })???;
            let (amount_0, amount_1) = if token_0 < token_1 {
                (amount_0, amount_1)
            } else {
                (amount_1, amount_0)
            };

            ensure!(amount_0 >= amount_0_min, RouterV2Error::InsufficientAmountA);
            ensure!(amount_1 >= amount_1_min, RouterV2Error::InsufficientAmountB);

            Ok((amount_0, amount_1))
        }

        #[ink(message)]
        fn remove_pair_liquidity_native(
            &mut self,
            token: AccountId,
            liquidity: u128,
            amount_token_min: u128,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, Balance), RouterV2Error> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let (amount_token, amount_native) = self.remove_pair_liquidity(
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
                .map_err(|_| RouterV2Error::TransferError)?;
            Ok((amount_token, amount_native))
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
            let router = RouterV2Contract::new(factory, wnative);
            assert_eq!(router.pair_factory(), factory);
            assert_eq!(router.wnative(), wnative);
        }
    }
}
