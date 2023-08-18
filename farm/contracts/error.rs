use psp22_traits::PSP22Error;

use crate::reentrancy_guard::ReentrancyGuardError;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmError {
    StillRunning,
    NotRunning,
    InvalidInitParams,
    CallerNotOwner,
    PSP22Error(PSP22Error),
    CallerNotFarmer,
    ArithmeticError,
    InvalidAmountArgument,
    InsufficientDepositAmount,
    InvalidWithdrawAmount,
    NothingToClaim,
    StateMissing,
    SubUnderFlow,
    ReentrancyError(ReentrancyGuardError),
}

impl From<PSP22Error> for FarmError {
    fn from(e: PSP22Error) -> Self {
        FarmError::PSP22Error(e)
    }
}

impl From<ReentrancyGuardError> for FarmError {
    fn from(e: ReentrancyGuardError) -> Self {
        FarmError::ReentrancyError(e)
    }
}
