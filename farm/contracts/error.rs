use psp22_traits::PSP22Error;

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
    SubUnderFlow1,
}

impl From<PSP22Error> for FarmError {
    fn from(e: PSP22Error) -> Self {
        FarmError::PSP22Error(e)
    }
}
