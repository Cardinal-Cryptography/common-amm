#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

use amm::helpers::ZERO_ADDRESS;
use ink::{
    primitives::{
        AccountId,
        Hash,
    },
    storage::Mapping,
};

#[derive(Debug)]
#[ink::storage_item]
struct Data {
    fee_to: AccountId,
    fee_to_setter: AccountId,
    get_pair: Mapping<(AccountId, AccountId), AccountId>,
    all_pairs: Mapping<u64, AccountId>,
    all_pairs_length: u64,
    pair_contract_code_hash: Hash,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            fee_to: ZERO_ADDRESS.into(),
            fee_to_setter: ZERO_ADDRESS.into(),
            get_pair: Default::default(),
            all_pairs: Mapping::default(),
            all_pairs_length: 0,
            pair_contract_code_hash: Default::default(),
        }
    }
}

#[ink::contract]
pub mod factory {
    use amm::{
        ensure,
        traits::factory::{
            Factory,
            FactoryError,
        },
    };
    use ink::{
        codegen::EmitEvent,
        env::hash::Blake2x256,
        ToAccountId,
    };
    use pair_contract::pair::PairContractRef;

    #[ink(event)]
    pub struct PairCreated {
        #[ink(topic)]
        pub token_0: AccountId,
        #[ink(topic)]
        pub token_1: AccountId,
        pub pair: AccountId,
        pub pair_len: u64,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct FactoryContract {
        factory: crate::Data,
    }

    impl Factory for FactoryContract {
        #[ink(message)]
        fn all_pairs(&self, pid: u64) -> Option<AccountId> {
            self.factory.all_pairs.get(pid)
        }

        #[ink(message)]
        fn all_pairs_length(&self) -> u64 {
            self.factory.all_pairs_length
        }

        #[ink(message)]
        fn pair_contract_code_hash(&self) -> Hash {
            self.factory.pair_contract_code_hash
        }

        #[ink(message)]
        fn create_pair(
            &mut self,
            token_a: AccountId,
            token_b: AccountId,
        ) -> Result<AccountId, FactoryError> {
            ensure!(token_a != token_b, FactoryError::IdenticalAddresses);
            let token_pair = if token_a < token_b {
                (token_a, token_b)
            } else {
                (token_b, token_a)
            };
            ensure!(
                self.factory.get_pair.get(token_pair).is_none(),
                FactoryError::PairExists
            );

            let salt = self.env().hash_encoded::<Blake2x256, _>(&token_pair);
            let pair_contract =
                self._instantiate_pair(salt.as_ref(), token_pair.0, token_pair.1)?;

            self.factory
                .get_pair
                .insert((token_pair.0, token_pair.1), &pair_contract);
            self.factory
                .get_pair
                .insert((token_pair.1, token_pair.0), &pair_contract);

            self._add_new_pair(pair_contract);

            self._emit_create_pair_event(
                token_pair.0,
                token_pair.1,
                pair_contract,
                self.all_pairs_length(),
            );

            Ok(pair_contract)
        }

        #[ink(message)]
        fn set_fee_to(&mut self, fee_to: AccountId) -> Result<(), FactoryError> {
            if self.env().caller() != self.factory.fee_to_setter {
                return Err(FactoryError::CallerIsNotFeeSetter)
            }
            self.factory.fee_to = fee_to;
            Ok(())
        }

        #[ink(message)]
        fn set_fee_to_setter(&mut self, fee_to_setter: AccountId) -> Result<(), FactoryError> {
            if self.env().caller() != self.factory.fee_to_setter {
                return Err(FactoryError::CallerIsNotFeeSetter)
            }
            self.factory.fee_to_setter = fee_to_setter;
            Ok(())
        }

        #[ink(message)]
        fn fee_to(&self) -> AccountId {
            self.factory.fee_to
        }

        #[ink(message)]
        fn fee_to_setter(&self) -> AccountId {
            self.factory.fee_to_setter
        }

        #[ink(message)]
        fn get_pair(&self, token_a: AccountId, token_b: AccountId) -> Option<AccountId> {
            self.factory.get_pair.get((token_a, token_b))
        }
    }

    impl FactoryContract {
        #[ink(constructor)]
        pub fn new(fee_to_setter: AccountId, pair_code_hash: Hash) -> Self {
            let mut instance = Self::default();
            instance.factory.pair_contract_code_hash = pair_code_hash;
            instance.factory.fee_to_setter = fee_to_setter;
            instance
        }

        fn _instantiate_pair(
            &mut self,
            salt_bytes: &[u8],
            token_0: AccountId,
            token_1: AccountId,
        ) -> Result<AccountId, FactoryError> {
            let pair_hash = self.factory.pair_contract_code_hash;
            let pair = match PairContractRef::new(token_0, token_1)
                .endowment(0)
                .code_hash(pair_hash)
                .salt_bytes(&salt_bytes[..4])
                .try_instantiate()
            {
                Ok(Ok(res)) => Ok(res),
                _ => Err(FactoryError::PairInstantiationFailed),
            }?;
            Ok(pair.to_account_id())
        }

        fn _emit_create_pair_event(
            &self,
            token_0: AccountId,
            token_1: AccountId,
            pair: AccountId,
            pair_len: u64,
        ) {
            EmitEvent::<FactoryContract>::emit_event(
                self.env(),
                PairCreated {
                    token_0,
                    token_1,
                    pair,
                    pair_len,
                },
            )
        }

        fn _add_new_pair(&mut self, pair: AccountId) {
            let pair_len = self.factory.all_pairs_length;
            self.factory.all_pairs.insert(pair_len, &pair);
            self.factory.all_pairs_length += 1;
        }
    }

    #[cfg(test)]
    mod tests {
        use ink::{
            env::test::default_accounts,
            primitives::Hash,
        };

        use super::*;

        #[ink::test]
        fn initialize_works() {
            let accounts = default_accounts::<ink::env::DefaultEnvironment>();
            let factory = FactoryContract::new(accounts.alice, Hash::default());
            assert_eq!(factory.factory.fee_to, amm::helpers::ZERO_ADDRESS.into());
        }
    }
}
