use ink::{contract_ref, env::DefaultEnvironment, primitives::AccountId};
use traits::StablePool as StablePoolTrait;

#[derive(scale::Decode, scale::Encode)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct StablePool(AccountId, Vec<AccountId>);

impl StablePool {
    pub fn new(pool_id: AccountId) -> Self {
        let mut pool = Self(pool_id, vec![]);
        pool.1 = pool.contract_ref().tokens();
        pool
    }

    pub fn contract_ref(&self) -> contract_ref!(StablePoolTrait, DefaultEnvironment) {
        self.0.into()
    }

    pub fn pool_id(&self) -> AccountId {
        self.0
    }
}
