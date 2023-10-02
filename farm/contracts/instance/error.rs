use amm_helpers::math::MathError;
use psp22_traits::PSP22Error;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmError {
    StillRunning,
    NotRunning,
    CallerNotOwner,
    PSP22Error(PSP22Error),
    CallerNotFarmer,
    ArithmeticError(MathError),
    InvalidAmountArgument,
    InsufficientDepositAmount,
    InvalidWithdrawAmount,
    NothingToClaim,
    StateMissing,
    SubUnderFlow,
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum FarmStartError {
    StillRunning,
    FarmAlreadyStarted,
    CallerNotOwner,
    InvalidInitParams,
    FarmEndBeforeStart,
    FarmTooLong,
    FarmAlreadyFinished,
    TooManyRewardTokens,
    ZeroRewardAmount,
    ZeroRewardRate,
    InsufficientRewardAmount,
    PSP22Error(PSP22Error),
    ArithmeticError,
}

impl From<PSP22Error> for FarmError {
    fn from(e: PSP22Error) -> Self {
        FarmError::PSP22Error(e)
    }
}

impl From<PSP22Error> for FarmStartError {
    fn from(e: PSP22Error) -> Self {
        FarmStartError::PSP22Error(e)
    }
}

impl From<MathError> for FarmError {
    fn from(e: MathError) -> Self {
        FarmError::ArithmeticError(e)
    }
}
