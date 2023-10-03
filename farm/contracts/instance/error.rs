use amm_helpers::math::MathError;
use farm_manager_trait::FarmManagerError;
use psp22_traits::PSP22Error;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmError {
    StillRunning,
    NotRunning,
    CallerNotOwner,
    PSP22Error(PSP22Error),
    FarmManagerError(FarmManagerError),
    CallerNotFarmer,
    ArithmeticError(MathError),
    InvalidAmountArgument,
    InsufficientDepositAmount,
    InvalidWithdrawAmount,
    NothingToClaim,
    StateMissing,
    SubUnderFlow,
}

impl From<FarmManagerError> for FarmError {
    fn from(e: FarmManagerError) -> Self {
        FarmError::FarmManagerError(e)
    }
}

impl From<PSP22Error> for FarmError {
    fn from(e: PSP22Error) -> Self {
        FarmError::PSP22Error(e)
    }
}

impl From<MathError> for FarmError {
    fn from(e: MathError) -> Self {
        FarmError::ArithmeticError(e)
    }
}
