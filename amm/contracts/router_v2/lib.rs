#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod pool;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CallerIsNotOwner;

#[ink::contract]
pub mod router_v2 {
    use crate::{
        pool::{calculate_pair_liquidity, pair_ref, Pool},
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
    use traits::{Factory, Pair, PoolId, RouterV2, RouterV2Error, StablePool, Step};
    use wrapped_azero::WrappedAZERO;

    #[ink(storage)]
    pub struct RouterV2Contract {
        pair_factory: AccountId,
        wnative: AccountId,
        owner: AccountId,
        /// Mapping of cached Pairs. Maps `(token_0, token_1) => (pair_id, fee)`
        cached_pairs: Mapping<(AccountId, AccountId), (AccountId, u8)>,
        /// Mapping of cached StablePools, Maps `pool_id => tokens[]`
        cached_stable_pools: Mapping<AccountId, Vec<AccountId>>,
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
        pub fn read_cached_pair(
            &self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Option<(AccountId, u8)> {
            self.cached_pairs.get((token_0, token_1))
        }

        #[ink(message)]
        pub fn read_cached_stable_pool(&self, pool_id: AccountId) -> Option<Pool> {
            self.cached_stable_pools
                .get(pool_id)
                .map(|_| Pool::StablePool(pool_id))
        }

        #[ink(message)]
        pub fn add_pair_to_cache(&mut self, pair: AccountId) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            self.cache_pair(pair);
            Ok(())
        }

        #[ink(message)]
        pub fn add_stable_pool_to_cache(
            &mut self,
            pool_id: AccountId,
        ) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            let pool_ref: contract_ref!(StablePool) = pool_id.into();
            let tokens = pool_ref.tokens();
            self.cached_stable_pools.insert(pool_id, &tokens);
            Ok(())
        }

        #[ink(message)]
        pub fn remove_pair_from_cache(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<(), CallerIsNotOwner> {
            ensure!(self.env().caller() == self.owner, CallerIsNotOwner);
            if let Some((pair, _)) = self.cached_pairs.take((token_0, token_1)) {
                self.cached_stable_pools.remove(pair);
            }
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
        fn cache_pair(&mut self, pair: AccountId) {
            let pair_ref = pair_ref(pair);
            let token_0 = pair_ref.get_token_0();
            let token_1 = pair_ref.get_token_1();
            let fee = pair_ref.get_fee();
            self.cached_pairs.insert((token_0, token_1), &(pair, fee));
            self.cached_pairs.insert((token_1, token_0), &(pair, fee));
        }

        // ----------- HELPER METHODS ----------- //

        #[inline]
        fn get_pair(
            &self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<AccountId, RouterV2Error> {
            if let Some((pair, _)) = self.cached_pairs.get((token_0, token_1)) {
                Ok(pair)
            } else {
                self.pair_factory_ref()
                    .get_pair(token_0, token_1)
                    .ok_or(RouterV2Error::PairNotFound)
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
                if self.cached_stable_pools.get(pool_id).is_some() {
                    return Ok(Pool::StablePool(pool_id));
                }
            }
            match self.get_pair(pair.0, pair.1) {
                Ok(pair_contract) => {
                    let fee = pair_ref(pair_contract).get_fee();
                    Ok(Pool::Pair(pair_contract, fee))
                }
                Err(_) => Err(RouterV2Error::PoolNotFound),
            }
        }

        #[inline]
        fn get_and_cache_pair(
            &mut self,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<AccountId, RouterV2Error> {
            if let Some((pair, _)) = self.cached_pairs.get((token_0, token_1)) {
                Ok(pair)
            } else {
                let pair = self
                    .pair_factory_ref()
                    .get_pair(token_0, token_1)
                    .ok_or(RouterV2Error::PairNotFound)?;
                self.cache_pair(pair);
                Ok(pair)
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
        ) -> Result<AccountId, RouterV2Error> {
            match self.get_and_cache_pair(token_0, token_1) {
                Ok(pair) => Ok(pair),
                Err(_) => {
                    let new_pair = self.pair_factory_ref().create_pair(token_0, token_1)?;
                    self.cache_pair(new_pair);
                    Ok(new_pair)
                }
            }
        }

        fn swap(
            &mut self,
            amounts: &[u128],
            path: &[Step],
            _to: AccountId,
        ) -> Result<(), RouterV2Error> {
            for i in 0..path.len() - 1 {
                // If last pool in the path, transfer tokens to the `_to` recipient.
                // Otherwise, transfer to the next Pair or StablePool.
                let to = if i < path.len() - 2 {
                    path[i + 1]
                        .0
                        .unwrap_or(self.get_pair(path[i + 1].1, path[i + 2].1)?)
                } else {
                    _to
                };
                let pool = self.get_pool(path[i].0, (path[i].1, path[i + 1].1))?;
                pool.swap(path[i].1, path[i + 1].1, amounts[i + 1], to)?;
            }
            Ok(())
        }

        /// Computes the amounts of tokens that have to be supplied
        /// at each step of the exchange `path`, to get exactly `amount_out`
        /// tokens at the end of the swaps.
        fn calculate_amounts_in(
            &self,
            amount_out: u128,
            path: &[Step],
        ) -> Result<Vec<u128>, RouterV2Error> {
            ensure!(path.len() >= 2, RouterV2Error::InvalidPath);

            let mut amounts = vec![0; path.len()];
            amounts[path.len() - 1] = amount_out;
            for i in (0..path.len() - 1).rev() {
                let pool = self.get_pool(path[i].0, (path[i].1, path[i + 1].1))?;
                amounts[i] = pool.get_amount_in(path[i].1, path[i + 1].1, amounts[i + 1])?;
            }

            Ok(amounts)
        }

        /// Computes swap token amounts over the given path of pools and tokens.
        ///
        /// Returns list of swap outcomes along the path.
        fn calculate_amounts_out(
            &self,
            amount_in: u128,
            path: &[Step],
        ) -> Result<Vec<u128>, RouterV2Error> {
            ensure!(path.len() >= 2, RouterV2Error::InvalidPath);

            let mut amounts = Vec::with_capacity(path.len());
            amounts.push(amount_in);
            for i in 0..path.len() - 1 {
                let pool = self.get_pool(path[i].0, (path[i].1, path[i + 1].1))?;
                amounts.push(pool.get_amount_out(path[i].1, path[i + 1].1, amounts[i])?);
            }

            Ok(amounts)
        }

        fn validate_path(&self, path: Vec<Step>) -> Result<(Vec<(Pool, AccountId)>, AccountId), RouterV2Error> {
            let mut pool_path = Vec::with_capacity(path.len());
            pool_path.push((self.get_pool(path[0].0, (path[0].1, path[1].1))?, path[0].1));
            for i in 1..(path.len() - 1) {
                pool_path.push((self.get_pool(path[i].0, (path[i].1, path[i + 1].1))? , path[i].1));
            }
            Ok((pool_path, path[path.len() - 1].1))
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
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let amounts = self.calculate_amounts_out(amount_in, &path)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            let first_pool_id = self
                .get_pool(path[0].0, (path[0].1, path[1].1))?
                .account_id();
            psp22_transfer_from(path[0].1, self.env().caller(), first_pool_id, amounts[0])?;
            self.swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message)]
        fn swap_tokens_for_exact_tokens(
            &mut self,
            amount_out: u128,
            amount_in_max: u128,
            path: Vec<Step>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let amounts = self.calculate_amounts_in(amount_out, &path)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterV2Error::ExcessiveInputAmount
            );
            let first_pool_id = self
                .get_pool(path[0].0, (path[0].1, path[1].1))?
                .account_id();
            psp22_transfer_from(path[0].1, self.env().caller(), first_pool_id, amounts[0])?;
            self.swap(&amounts, &path, to)?;
            Ok(amounts)
        }

        #[ink(message, payable)]
        fn swap_exact_native_for_tokens(
            &mut self,
            amount_out_min: u128,
            path: Vec<Step>,
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let received_value = self.env().transferred_value();
            let wnative = self.wnative;
            ensure!(path[0].1 == wnative, RouterV2Error::InvalidPath);
            let amounts = self.calculate_amounts_out(received_value, &path)?;
            ensure!(
                amounts[amounts.len() - 1] >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            self.wrap(received_value)?;
            let first_pool_id = self
                .get_pool(path[0].0, (path[0].1, path[1].1))?
                .account_id();
            psp22_transfer(wnative, first_pool_id, amounts[0])?;
            self.swap(&amounts, &path, to)?;
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
            ensure!(
                path[path.len() - 1].1 == wnative,
                RouterV2Error::InvalidPath
            );
            let amounts = self.calculate_amounts_in(amount_out, &path)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterV2Error::ExcessiveInputAmount
            );
            let first_pool_id = self
                .get_pool(path[0].0, (path[0].1, path[1].1))?
                .account_id();
            psp22_transfer_from(path[0].1, self.env().caller(), first_pool_id, amounts[0])?;
            self.swap(&amounts, &path, self.env().account_id())?;
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
            ensure!(
                path[path.len() - 1].1 == self.wnative,
                RouterV2Error::InvalidPath
            );
            let amounts = self.calculate_amounts_out(amount_in, &path)?;
            let native_out = amounts[amounts.len() - 1];
            ensure!(
                native_out >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            let first_pool_id = self
                .get_pool(path[0].0, (path[0].1, path[1].1))?
                .account_id();
            psp22_transfer_from(path[0].1, self.env().caller(), first_pool_id, amounts[0])?;
            self.swap(&amounts, &path, self.env().account_id())?;
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
            to: AccountId,
            deadline: u64,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_native = self.env().transferred_value();
            ensure!(path[0].1 == wnative, RouterV2Error::InvalidPath);
            let amounts = self.calculate_amounts_in(amount_out, &path)?;
            let native_in: Balance = amounts[0];
            ensure!(
                native_in <= received_native,
                RouterV2Error::ExcessiveInputAmount
            );
            self.wrap(native_in)?;
            let first_pool_id = self
                .get_pool(path[0].0, (path[0].1, path[1].1))?
                .account_id();
            psp22_transfer(wnative, first_pool_id, native_in)?;
            self.swap(&amounts, &path, to)?;
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
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.calculate_amounts_out(amount_in, &path)
        }

        #[ink(message)]
        fn get_amounts_in(
            &self,
            amount_out: u128,
            path: Vec<Step>,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.calculate_amounts_in(amount_out, &path)
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
            let pair_contract = self.get_or_create_pair(token_0, token_1)?;
            let (amount_0, amount_1) = calculate_pair_liquidity(
                pair_contract,
                token_0,
                token_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min,
            )?;

            let caller = self.env().caller();
            psp22_transfer_from(token_0, caller, pair_contract, amount_0)?;
            psp22_transfer_from(token_1, caller, pair_contract, amount_1)?;

            let liquidity = pair_ref(pair_contract).mint(to)?;

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

            let pair_contract = self.get_or_create_pair(token, wnative)?;
            let (amount_0, amount_native) = calculate_pair_liquidity(
                pair_contract,
                token,
                wnative,
                amount_token_desired,
                received_value,
                amount_token_min,
                amount_native_min,
            )?;

            let caller = self.env().caller();
            psp22_transfer_from(token, caller, pair_contract, amount_0)?;
            self.wrap(amount_native)?;
            psp22_transfer(wnative, pair_contract, amount_native)?;

            let liquidity = pair_ref(pair_contract).mint(to)?;

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

            psp22_transfer_from(pair, self.env().caller(), pair, liquidity)?;

            let mut pair = pair_ref(pair);

            let (amount_0, amount_1) =
                pair.call_mut().burn(to).try_invoke().map_err(|_| {
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
