#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod manager {

    type TokenId = AccountId;
    type UserId = AccountId;
    type FarmId = u32;

    use amm_farm::FarmRef;
    use farm_instance_trait::Farm as FarmT;
    use farm_manager_trait::{
        FarmManager as FarmManagerTrait,
        FarmManagerError,
    };
    use ink::{
        codegen::EmitEvent,
        contract_ref,
        env::hash::Blake2x256,
        reflect::ContractEventBase,
        storage::Mapping,
        ToAccountId,
    };

    use ink::prelude::vec::Vec;

    use psp22_traits::{
        PSP22Error,
        PSP22,
    };

    #[ink(event)]
    pub struct FarmInstantiated {
        /// Address of token pair for which this farm was created.
        #[ink(topic)]
        pool_id: AccountId,
        /// Owner of the pair - address seeding the rewards.
        owner: AccountId,
        /// Address of the farm.
        address: AccountId,
    }

    pub type Event = <FarmManager as ContractEventBase>::Type;

    #[ink(storage)]
    pub struct FarmManager {
        /// Address of the token pool for which this farm is created.
        pool_id: AccountId,
        /// Address of the farm creator.
        owner: AccountId,
        /// How many shares each user has in the farm.
        shares: Mapping<UserId, u128>,
        /// Total shares in the farm after the last action.
        total_shares: u128,
        /// Reference to a farm contract code.
        farm_code_hash: Hash,
        /// Reward tokens.
        reward_tokens: Vec<TokenId>,
        /// Latest farm instance.
        latest_farm: Option<FarmId>,
        /// List of farms created by this manager.
        /// Notably, latest_farm could be currently active farm (depending on its is_running status)
        /// and all farms with lower indexes are past, finished farm instances.
        farm_by_id: Mapping<FarmId, AccountId>,
        /// All farms created via this manager.
        farms: Mapping<AccountId, ()>,
    }

    impl FarmManager {
        #[ink(constructor)]
        pub fn new(pool_id: AccountId, farm_code_hash: Hash, reward_tokens: Vec<TokenId>) -> Self {
            FarmManager {
                pool_id,
                owner: Self::env().caller(),
                shares: Mapping::default(),
                total_shares: 0,
                farm_code_hash,
                reward_tokens,
                latest_farm: None,
                farm_by_id: Mapping::new(),
                farms: Mapping::new(),
            }
        }

        #[ink(message)]
        pub fn set_farm_code_hash(&mut self, farm_code_hash: Hash) -> Result<(), FarmManagerError> {
            if self.env().caller() != self.owner {
                return Err(FarmManagerError::CallerNotOwner)
            }
            self.farm_code_hash = farm_code_hash;
            Ok(())
        }

        fn _instantiate_farm(&self, salt: &[u8]) -> Result<FarmRef, FarmManagerError> {
            let farm = match FarmRef::new(self.pool_id, self.env().account_id(), self.owner)
                .endowment(0)
                .salt_bytes(&salt)
                .code_hash(self.farm_code_hash)
                .try_instantiate()
            {
                Ok(Ok(address)) => Ok(address),
                _ => Err(FarmManagerError::FarmInstantiationFailed),
            }?;
            Ok(farm)
        }

        fn check_no_active_farm(&self) -> Result<(), FarmManagerError> {
            if let Some(latest_farm_id) = self.latest_farm {
                let farm_address = self
                    .farm_by_id
                    .get(latest_farm_id)
                    .ok_or(FarmManagerError::FarmNotFound(latest_farm_id))?;
                let farm: contract_ref!(FarmT) = farm_address.into();
                if farm.is_running() {
                    return Err(FarmManagerError::FarmAlreadyRunning(farm_address))
                }
            }
            Ok(())
        }

        fn emit_event<EE: EmitEvent<Self>>(emitter: EE, event: Event) {
            emitter.emit_event(event);
        }
    }

    impl FarmManagerTrait for FarmManager {
        #[ink(message)]
        fn pool_id(&self) -> AccountId {
            self.pool_id
        }

        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.total_shares
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.shares.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        fn withdraw_shares(
            &mut self,
            account: AccountId,
            amount: u128,
        ) -> Result<(), FarmManagerError> {
            let caller = self.env().caller();
            if !self.farms.contains(&caller) {
                return Err(FarmManagerError::FarmUnknown(caller))
            }
            let shares = self.shares.get(account).unwrap_or(0);

            match shares.checked_sub(amount) {
                Some(new_shares) => {
                    self.shares.insert(account, &new_shares);
                    self.total_shares -= amount;
                    Ok(())
                }
                None => Err(PSP22Error::InsufficientBalance.into()),
            }
        }

        #[ink(message)]
        fn deposit_shares(
            &mut self,
            account: AccountId,
            amount: u128,
        ) -> Result<(), FarmManagerError> {
            let caller = self.env().caller();
            if !self.farms.contains(&caller) {
                return Err(FarmManagerError::FarmUnknown(caller))
            }
            let shares = self.shares.get(account).unwrap_or(0);
            self.shares.insert(account, &(shares + amount));
            self.total_shares += amount;
            Ok(())
        }

        #[ink(message)]
        fn latest_farm_id(&self) -> Option<AccountId> {
            self.latest_farm.and_then(|id| self.farm_by_id.get(id))
        }

        #[ink(message)]
        fn get_farm_address(&self, farm_id: u32) -> Option<AccountId> {
            self.farm_by_id.get(farm_id)
        }

        #[ink(message)]
        fn reward_tokens(&self) -> Vec<AccountId> {
            self.reward_tokens.clone()
        }

        #[ink(message)]
        fn instantiate_farm(
            &mut self,
            end: Timestamp,
            rewards: Vec<(TokenId, u128)>,
        ) -> Result<AccountId, FarmManagerError> {
            if self.env().caller() != self.owner {
                return Err(FarmManagerError::CallerNotOwner)
            }
            // There can be only one instance of this farm running at a time.
            self.check_no_active_farm()?;

            let farm_id = self.latest_farm.unwrap_or_default() + 1;
            let salt = self
                .env()
                .hash_encoded::<Blake2x256, _>(&(self.pool_id, farm_id));

            let mut farm = self._instantiate_farm(&salt)?;

            let farm_address = farm.to_account_id();

            let reward_tokens: Vec<AccountId> =
                rewards.iter().map(|(token, _amount)| *token).collect();

            for (token, amount) in rewards {
                let mut psp22: contract_ref!(PSP22) = token.into();
                psp22.transfer_from(self.env().caller(), farm_address, amount, Vec::new())?;
            }

            farm.start(end, reward_tokens.clone())?;

            self.latest_farm = Some(farm_id);
            self.farm_by_id.insert(farm_id, &farm_address);
            self.farms.insert(farm_address, &());

            FarmManager::emit_event(
                self.env(),
                Event::FarmInstantiated(FarmInstantiated {
                    pool_id: self.pool_id,
                    owner: self.owner,
                    address: farm_address,
                }),
            );
            Ok(farm_address)
        }
    }
}
