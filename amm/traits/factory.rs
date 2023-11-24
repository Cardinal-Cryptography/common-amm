use crate::PairError;
use ink::primitives::{AccountId, Hash};

/// Factory trait for tracking all pairs within the UniswapV2 DEX.
/// Creates new, unique instances of `Pair` smart contract per token pairs.
/// Contains the logic to turn on the protocol charge.
#[ink::trait_definition]
pub trait Factory {
    /// Returns address of the pair contract identified by `pid` id.
    #[ink(message)]
    fn all_pairs(&self, pid: u64) -> Option<AccountId>;

    /// Returns number of token pairs created by the factory contract.
    #[ink(message)]
    fn all_pairs_length(&self) -> u64;

    /// Returns code hash of the `Pair` contract this factory instance uses.
    #[ink(message)]
    fn pair_contract_code_hash(&self) -> Hash;

    /// Creates an instance of the `Pair` contract for the `(token_0, token_1)` pair.
    /// Returns the address of the contract instance if successful.
    /// Fails if the `Pair` instance of the token pair already exists
    /// or the token pair is illegal.
    #[ink(message)]
    fn create_pair(
        &mut self,
        token_0: AccountId,
        token_1: AccountId,
    ) -> Result<AccountId, FactoryError>;

    /// Sets the address for receiving protocol's share of trading fees.
    #[ink(message)]
    fn set_fee_to(&mut self, fee_to: AccountId) -> Result<(), FactoryError>;

    /// Sets the address eligible for calling `set_foo_to` method.
    #[ink(message)]
    fn set_fee_to_setter(&mut self, fee_to_setter: AccountId) -> Result<(), FactoryError>;

    /// Returns recipient address of the trading fees.
    #[ink(message)]
    fn fee_to(&self) -> Option<AccountId>;

    /// Returns account allowed to call `set_fee_to_setter`.
    #[ink(message)]
    fn fee_to_setter(&self) -> AccountId;

    /// Returns address of `Pair` contract instance (if any) for `(token_0, token_1)` pair.
    #[ink(message)]
    fn get_pair(&self, token_0: AccountId, token_1: AccountId) -> Option<AccountId>;
}

/// Errors that can be returned from calling `Factory`'s methods.
#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FactoryError {
    PairError(PairError),
    CallerIsNotFeeSetter,
    IdenticalAddresses,
    PairExists,
    PairInstantiationFailed,
}

impl From<PairError> for FactoryError {
    fn from(error: PairError) -> Self {
        FactoryError::PairError(error)
    }
}
