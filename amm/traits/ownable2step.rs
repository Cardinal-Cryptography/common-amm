use ink::primitives::AccountId;
use scale::{Decode, Encode};

/// Implement this trait to enable two-step ownership trasfer process in your contract.
///
/// The process looks like this:
/// * current owner (Alice) calls `self.transfer_ownership(bob)`,
/// * the contract still has the owner: Alice and a pending owner: bob,
/// * when Bob claims the ownership by calling `self.accept_ownership()` he becomes the new owner and pending owner is removed.
///
/// The ownership can be also renounced:
/// * current owner calls `self.transfer_ownership(this_contract_address)`
/// * current owner calls `self.renounce_ownership()` - transfers the ownership to
///   this contract's address
#[ink::trait_definition]
pub trait Ownable2Step {
    /// Returns the address of the current owner.
    #[ink(message)]
    fn get_owner(&self) -> Ownable2StepResult<AccountId>;

    /// Returns the address of the pending owner.
    #[ink(message)]
    fn get_pending_owner(&self) -> Ownable2StepResult<AccountId>;

    /// Starts the ownership transfer of the contract to a new account. Replaces the pending transfer if there is one.
    /// Can only be called by the current owner.
    #[ink(message)]
    fn transfer_ownership(&mut self, new_owner: AccountId) -> Ownable2StepResult<()>;

    /// The new owner accepts the ownership transfer.
    #[ink(message)]
    fn accept_ownership(&mut self) -> Ownable2StepResult<()>;

    /// The owner of the contract renounces the ownership.
    /// To start the process, the owner has to initiate ownership transfer to this contract's address.
    /// Can only be called by the current owner.
    #[ink(message)]
    fn renounce_ownership(&mut self) -> Ownable2StepResult<()>;

    /// Return error if called by any account other than the owner.
    #[ink(message)]
    fn ensure_owner(&self) -> Ownable2StepResult<()>;
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Ownable2StepError {
    /// The caller didn't have the permissions to call a given method
    CallerNotOwner(AccountId),
    /// The caller tried to accept ownership but caller in not the pending owner
    CallerNotPendingOwner(AccountId),
    /// The owner tried to renounce ownership but the contract's address has not been set as the pending owner.
    ContractNotPendingOwner(AccountId),
    /// The caller tried to accept ownership but the process hasn't been started
    NoPendingOwner,
}

pub type Ownable2StepResult<T> = Result<T, Ownable2StepError>;

#[derive(Debug)]
#[ink::storage_item]
pub struct Ownable2StepData {
    owner: AccountId,
    pending_owner: Option<AccountId>,
}

impl Ownable2StepData {
    pub fn new(owner: AccountId) -> Self {
        Self {
            owner,
            pending_owner: None,
        }
    }

    pub fn transfer_ownership(
        &mut self,
        caller: AccountId,
        new_owner: AccountId,
    ) -> Ownable2StepResult<()> {
        self.ensure_owner(caller)?;
        self.pending_owner = Some(new_owner);
        Ok(())
    }

    pub fn accept_ownership(&mut self, caller: AccountId) -> Ownable2StepResult<()> {
        let pending_owner = self.get_pending_owner()?;

        if caller != pending_owner {
            return Err(Ownable2StepError::CallerNotPendingOwner(caller));
        }

        self.owner = pending_owner;
        self.pending_owner = None;

        Ok(())
    }

    pub fn renounce_ownership(
        &mut self,
        caller: AccountId,
        contract_address: AccountId,
    ) -> Ownable2StepResult<()> {
        self.ensure_owner(caller)?;
        let pending_owner = self.get_pending_owner()?;
        if pending_owner != contract_address {
            return Err(Ownable2StepError::ContractNotPendingOwner(pending_owner));
        }
        self.owner = contract_address;
        self.pending_owner = None;

        Ok(())
    }

    pub fn get_owner(&self) -> Ownable2StepResult<AccountId> {
        Ok(self.owner)
    }

    pub fn get_pending_owner(&self) -> Ownable2StepResult<AccountId> {
        self.pending_owner.ok_or(Ownable2StepError::NoPendingOwner)
    }

    pub fn ensure_owner(&self, caller: AccountId) -> Ownable2StepResult<()> {
        if caller != self.owner {
            Err(Ownable2StepError::CallerNotOwner(caller))
        } else {
            Ok(())
        }
    }
}
