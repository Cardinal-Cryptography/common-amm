use ink::LangError;
use psp22::PSP22Error;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DexError {
    LangError(LangError),
    PSP22Error(PSP22Error),

    CallerIsNotFeeSetter,
    CrossContractCallFailed(u8),
    Expired,
    IdenticalAddresses(u8),
    InvalidPath(u8),
    InvalidTo,
    PairExists,
    PairInstantiationFailed,
    PairNotFound,
    TransferError(u8),

    ExcessiveInputAmount,
    InsufficientAAmount,
    InsufficientAmount,
    InsufficientBAmount,
    InsufficientInputAmount,
    InsufficientOutputAmount,
    InsufficientLiquidity,
    InsufficientLiquidityBurned,
    InsufficientLiquidityMinted,
    KInvariantChanged,

    AddOverflow(u8),
    CastOverflow(u8),
    DivByZero(u8),
    MulOverflow(u8),
    SubUnderflow(u8),
}

impl From<LangError> for DexError {
    fn from(error: LangError) -> Self {
        DexError::LangError(error)
    }
}

impl From<PSP22Error> for DexError {
    fn from(error: PSP22Error) -> Self {
        DexError::PSP22Error(error)
    }
}
