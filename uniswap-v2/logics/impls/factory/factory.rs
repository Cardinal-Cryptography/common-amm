pub use crate::{
    impls::factory::*,
    traits::factory::*,
};
use openbrush::{
    modifier_definition,
    traits::{
        AccountId,
        Storage,
    },
};

pub trait Internal {
    fn _emit_create_pair_event(
        &self,
        _token_0: AccountId,
        _token_1: AccountId,
        _pair: AccountId,
        _pair_len: u64,
    );

    /// Creates an instance of the `Pair` contract.
    fn _instantiate_pair(
        &mut self,
        salt_bytes: &[u8],
        token_0: AccountId,
        token_1: AccountId,
    ) -> Result<AccountId, FactoryError>;

    /// Adds a new pair to the contract's storage.
    fn _add_new_pair(&mut self, pair: AccountId);
}

#[modifier_definition]
pub fn only_fee_setter<T, F, R, E>(instance: &mut T, body: F) -> Result<R, E>
where
    T: Storage<data::Data>,
    F: FnOnce(&mut T) -> Result<R, E>,
    E: From<FactoryError>,
{
    if instance.data().fee_to_setter != T::env().caller() {
        return Err(From::from(FactoryError::CallerIsNotFeeSetter))
    }
    body(instance)
}
