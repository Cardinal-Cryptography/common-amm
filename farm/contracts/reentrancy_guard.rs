use openbrush::modifier_definition;

#[modifier_definition]
pub fn non_reentrant<I, F, T, E>(instance: &mut I, body: F) -> Result<T, E>
where
    F: FnOnce(&mut I) -> Result<T, E>,
    E: From<ReentrancyGuardError>,
    I: ReentrancyGuardT,
{
    instance.lock()?;
    let res = body(instance);
    instance.unlock();
    res
}

pub trait ReentrancyGuardT {
    /// Sets the reentrancy lock flag to `true` (conceptually, as it's implementation-dependent).
    /// Must fail if lock is already taken.
    fn lock(&mut self) -> Result<(), ReentrancyGuardError>;

    /// Sets the reentrancy lock flag to `false` (conceptually, as it's implementation-dependent).
    fn unlock(&mut self);
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum ReentrancyGuardError {
    ReentrancyError,
}

/// Simple enum representing reentrancy lock state.
#[derive(scale::Encode, scale::Decode, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
#[repr(u8)]
pub enum ReentrancyLock {
    Locked,
    Unlocked,
}
