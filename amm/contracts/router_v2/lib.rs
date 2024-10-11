#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod pair;
mod pool;
mod stable_pool;
mod utils;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct CallerIsNotOwner;

#[ink::contract]
pub mod router_v2 {
    use crate::{
        pool::{Pair, Pool, StablePool},
        utils::*,
    };
    use amm_helpers::ensure;
    use ink::{
        contract_ref,
        prelude::{vec, vec::Vec},
        storage::Mapping,
    };
    use traits::{Factory, RouterV2, RouterV2Error, Step};

    #[ink(storage)]
    pub struct RouterV2Contract {
        pair_factory: AccountId,
        wnative: AccountId,
        cached_pools: Mapping<AccountId, Pool>,
    }

    impl RouterV2Contract {
        #[ink(constructor)]
        pub fn new(pair_factory: AccountId, wnative: AccountId) -> Self {
            Self {
                pair_factory,
                wnative,
                cached_pools: Default::default(),
            }
        }

        #[ink(message)]
        pub fn read_cached_pool(&self, pool_id: AccountId) -> Option<Pool> {
            self.cached_pools.get(pool_id)
        }

        // ----------- HELPER METHODS ----------- //

        /// Returns Pool for `pool_id` if it exists.
        /// Adds the Pool to the cache.
        #[inline]
        fn get_and_cache_pool(&mut self, pool_id: AccountId) -> Result<Pool, RouterV2Error> {
            match self.cached_pools.get(pool_id) {
                Some(pool) => Ok(pool),
                None => {
                    let pool = Pool::try_new(pool_id).ok_or(RouterV2Error::InvalidPoolAddress)?;
                    self.cached_pools.insert(pool_id, &pool);
                    Ok(pool)
                }
            }
        }

        /// Returns StablePool for `pool_id`.
        /// Adds the StablePool to the cache.
        #[inline]
        fn get_and_cache_stable_pool(&mut self, pool_id: AccountId) -> Result<StablePool, RouterV2Error> {
            match self.get_and_cache_pool(pool_id)? {
                Pool::StablePool(pool) => Ok(pool),
                Pool::Pair(_) => Err(RouterV2Error::InvalidPoolAddress),
            }
        }

        /// Returns Pair for `pool_id`.
        /// If `pool_id` is `None`, it creates a new Pair for
        /// `(token_0, token_1)` tokens if the Pair does not
        /// exist in the pair Factory.
        /// Adds the Pair to the cache.
        #[inline]
        fn get_and_cache_pair(
            &mut self,
            pool_id: Option<AccountId>,
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<Pair, RouterV2Error> {
            let pool_id = match pool_id {
                Some(pool_id) => pool_id,
                None => self.pair_factory_ref().create_pair(token_0, token_1)?,
            };
            match self.get_and_cache_pool(pool_id)? {
                Pool::Pair(pair) => Ok(pair),
                Pool::StablePool(_) => Err(RouterV2Error::InvalidPoolAddress),
            }
        }

        fn swap(
            &mut self,
            amounts: &[u128],
            path: &[Step],
            token_out: AccountId,
            to: AccountId,
        ) -> Result<(), RouterV2Error> {
            let n_pools = path.len();
            for i in 0..n_pools - 1 {
                self.get_and_cache_pool(path[i].pool_id)?.swap(
                    path[i].token_in,
                    path[i + 1].token_in,
                    amounts[i + 1],
                    path[i + 1].pool_id,
                )?;
            }
            // If last pool in the path, transfer tokens to the `to` recipient.
            self.get_and_cache_pool(path[n_pools - 1].pool_id)?.swap(
                path[n_pools - 1].token_in,
                token_out,
                amounts[n_pools],
                to,
            )?;
            Ok(())
        }

        /// Computes the amounts of tokens that have to be supplied
        /// at each step of the exchange `path`, to get exactly `amount_out` of `token_out`
        /// tokens at the end of the swaps.
        fn calculate_amounts_in(
            &mut self,
            amount_out: u128,
            path: &[Step],
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let n_pools = path.len();
            let mut amounts = vec![0; n_pools + 1];
            amounts[n_pools] = amount_out;
            amounts[n_pools - 1] = self.get_and_cache_pool(path[n_pools - 1].pool_id)?.get_amount_in(
                path[n_pools - 1].token_in,
                token_out,
                amount_out,
            )?;
            for i in (0..n_pools - 1).rev() {
                amounts[i] = self.get_and_cache_pool(path[i].pool_id)?.get_amount_in(
                    path[i].token_in,
                    path[i + 1].token_in,
                    amounts[i + 1],
                )?;
            }

            Ok(amounts)
        }

        /// Computes swap token amounts over the given path.
        ///
        /// Returns list of swap outcomes along the path.
        fn calculate_amounts_out(
            &mut self,
            amount_in: u128,
            path: &[Step],
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let n_pools = path.len();
            let mut amounts = vec![0; n_pools + 1];
            amounts[0] = amount_in;
            for i in 0..n_pools - 1 {
                amounts[i + 1] = self.get_and_cache_pool(path[i].pool_id)?.get_amount_out(
                    path[i].token_in,
                    path[i + 1].token_in,
                    amounts[i],
                )?;
            }
            amounts[n_pools] = self.get_and_cache_pool(path[n_pools - 1].pool_id)?.get_amount_out(
                path[n_pools - 1].token_in,
                token_out,
                amounts[n_pools - 1],
            )?;

            Ok(amounts)
        }

        #[inline]
        fn pair_factory_ref(&self) -> contract_ref!(Factory) {
            self.pair_factory.into()
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
            check_timestamp(deadline)?;
            let amounts = self.calculate_amounts_out(amount_in, &path, token_out)?;
            ensure!(
                *amounts.last().ok_or(RouterV2Error::EmptyAmounts)? >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            psp22_transfer_from(
                path[0].token_in,
                self.env().caller(),
                path[0].pool_id,
                amounts[0],
            )?;
            self.swap(&amounts, &path, token_out, to)?;
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
            check_timestamp(deadline)?;
            let amounts = self.calculate_amounts_in(amount_out, &path, token_out)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterV2Error::ExcessiveInputAmount
            );
            psp22_transfer_from(
                path[0].token_in,
                self.env().caller(),
                path[0].pool_id,
                amounts[0],
            )?;
            self.swap(&amounts, &path, token_out, to)?;
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
            check_timestamp(deadline)?;
            let received_value = self.env().transferred_value();
            let wnative = self.wnative;
            ensure!(path[0].token_in == wnative, RouterV2Error::InvalidToken);
            let amounts = self.calculate_amounts_out(received_value, &path, token_out)?;
            ensure!(
                *amounts.last().ok_or(RouterV2Error::EmptyAmounts)? >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            wrap(wnative, received_value)?;
            psp22_transfer(wnative, path[0].pool_id, amounts[0])?;
            self.swap(&amounts, &path, token_out, to)?;
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
            check_timestamp(deadline)?;
            let wnative = self.wnative;
            let amounts = self.calculate_amounts_in(amount_out, &path, wnative)?;
            ensure!(
                amounts[0] <= amount_in_max,
                RouterV2Error::ExcessiveInputAmount
            );
            psp22_transfer_from(
                path[0].token_in,
                self.env().caller(),
                path[0].pool_id,
                amounts[0],
            )?;
            self.swap(&amounts, &path, wnative, self.env().account_id())?;
            let native_out = amounts.last().ok_or(RouterV2Error::EmptyAmounts)?;
            withdraw(wnative, native_out)?;
            transfer_native(to, native_out)?;
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
            check_timestamp(deadline)?;
            let wnative = self.wnative;
            let amounts = self.calculate_amounts_out(amount_in, &path, wnative)?;
            let native_out = amounts.last().ok_or(RouterV2Error::EmptyAmounts)?;
            ensure!(
                native_out >= amount_out_min,
                RouterV2Error::InsufficientOutputAmount
            );
            psp22_transfer_from(
                path[0].token_in,
                self.env().caller(),
                path[0].pool_id,
                amounts[0],
            )?;
            self.swap(&amounts, &path, wnative, self.env().account_id())?;
            withdraw(wnative, native_out)?;
            transfer_native(to, native_out)?;
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
            check_timestamp(deadline)?;
            let wnative = self.wnative;
            let received_native = self.env().transferred_value();
            ensure!(path[0].token_in == wnative, RouterV2Error::InvalidToken);
            let amounts = self.calculate_amounts_in(amount_out, &path, token_out)?;
            let native_in: Balance = amounts[0];
            ensure!(
                native_in <= received_native,
                RouterV2Error::ExcessiveInputAmount
            );
            wrap(wnative, native_in)?;
            psp22_transfer(wnative, path[0].pool_id, native_in)?;
            self.swap(&amounts, &path, token_out, to)?;
            if received_native > native_in {
                self.env()
                    .transfer(self.env().caller(), received_native - native_in)
                    .map_err(|_| RouterV2Error::TransferError)?;
            }
            Ok(amounts)
        }

        #[ink(message)]
        fn get_amounts_out(
            &mut self,
            amount_in: u128,
            path: Vec<Step>,
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.calculate_amounts_out(amount_in, &path, token_out)
        }

        #[ink(message)]
        fn get_amounts_in(
            &mut self,
            amount_out: u128,
            path: Vec<Step>,
            token_out: AccountId,
        ) -> Result<Vec<u128>, RouterV2Error> {
            self.calculate_amounts_in(amount_out, &path, token_out)
        }

        // ----------- PAIR LIQUIDITY METHODS ----------- //

        #[ink(message)]
        fn add_pair_liquidity(
            &mut self,
            pair: Option<AccountId>,
            token_0: AccountId,
            token_1: AccountId,
            amount_0_desired: u128,
            amount_1_desired: u128,
            amount_0_min: u128,
            amount_1_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128, u128), RouterV2Error> {
            let pair = self.get_and_cache_pair(pair, token_0, token_1)?;
            pair.add_liquidity(
                token_0,
                token_1,
                amount_0_desired,
                amount_1_desired,
                amount_0_min,
                amount_1_min,
                to,
                deadline,
            )
        }

        #[ink(message, payable)]
        fn add_pair_liquidity_native(
            &mut self,
            pair: Option<AccountId>,
            token: AccountId,
            amount_token_desired: u128,
            amount_token_min: u128,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, Balance, u128), RouterV2Error> {
            let wnative = self.wnative;
            let pair = self.get_and_cache_pair(pair, token, wnative)?;
            pair.add_liquidity_native(
                token,
                wnative,
                amount_token_desired,
                amount_token_min,
                amount_native_min,
                to,
                deadline,
            )
        }

        #[ink(message)]
        fn remove_pair_liquidity(
            &mut self,
            pair: AccountId,
            token_0: AccountId,
            token_1: AccountId,
            liquidity: u128,
            amount_0_min: u128,
            amount_1_min: u128,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, u128), RouterV2Error> {
            let pair = self.get_and_cache_pair(Some(pair), token_0, token_1)?;
            pair.remove_liquidity(
                token_0,
                token_1,
                liquidity,
                amount_0_min,
                amount_1_min,
                to,
                deadline,
            )
        }

        #[ink(message)]
        fn remove_pair_liquidity_native(
            &mut self,
            pair: AccountId,
            token: AccountId,
            liquidity: u128,
            amount_token_min: u128,
            amount_native_min: Balance,
            to: AccountId,
            deadline: u64,
        ) -> Result<(u128, Balance), RouterV2Error> {
            let wnative = self.wnative;
            let pair = self.get_and_cache_pair(Some(pair), token, wnative)?;
            pair.remove_liquidity_native(
                token,
                wnative,
                liquidity,
                amount_token_min,
                amount_native_min,
                to,
                deadline,
            )
        }

        // ----------- STABLE POOL LIQUIDITY METHODS ----------- //

        #[ink(message, payable)]
        fn add_stable_pool_liquidity(
            &mut self,
            pool: AccountId,
            min_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
            deadline: u64,
            native: bool,
        ) -> Result<(u128, u128), RouterV2Error> {
            let wnative = if native { Some(self.wnative) } else { None };
            self.get_and_cache_stable_pool(pool)?.add_liquidity(
                min_share_amount,
                amounts,
                to,
                deadline,
                wnative,
            )
        }

        #[ink(message)]
        fn remove_stable_pool_liquidity(
            &mut self,
            pool: AccountId,
            max_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
            deadline: u64,
            native: bool,
        ) -> Result<(u128, u128), RouterV2Error> {
            let wnative = if native { Some(self.wnative) } else { None };
            self.get_and_cache_stable_pool(pool)?.remove_liquidity(
                max_share_amount,
                amounts,
                to,
                deadline,
                wnative,
            )
        }

        #[ink(message)]
        fn remove_stable_pool_liquidity_by_share(
            &mut self,
            pool: AccountId,
            share_amount: u128,
            min_amounts: Vec<u128>,
            to: AccountId,
            deadline: u64,
            native: bool,
        ) -> Result<Vec<u128>, RouterV2Error> {
            let wnative = if native { Some(self.wnative) } else { None };
            self.get_and_cache_stable_pool(pool)?.remove_liquidity_by_share(
                share_amount,
                min_amounts,
                to,
                deadline,
                wnative,
            )
        }
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
