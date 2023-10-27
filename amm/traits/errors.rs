use ink::LangError;
use psp22::PSP22Error;

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum DexError {
    LangError(LangError),
    PSP22Error(PSP22Error),

    CallerIsNotFeeSetter,
    CrossContractCallFailed,
    Expired,
    IdenticalAddresses,
    InvalidPath,
    InvalidTo,
    PairExists,
    PairInstantiationFailed,
    PairNotFound,
    TransferError,

    ExcessiveInputAmount,
    InsufficientAAmount,
    InsufficientAmount,
    InsufficientBAmount,
    InsufficientInputAmount,
    InsufficientLiquidity,
    InsufficientLiquidityBurned,
    InsufficientLiquidityMinted,
    InsufficientOutputAmount,
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
