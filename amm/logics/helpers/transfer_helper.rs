use crate::{
    traits::wnative::Wnative,
    Balance,
    Env,
};
use ink::{
    codegen::TraitCallBuilder,
    contract_ref,
    prelude::{
        string::String,
        vec::Vec,
    },
    primitives::AccountId,
};
use psp22::{
    PSP22Error,
    PSP22,
};

/// Transfers `value` amount of `token` to an account controlled by `to` address.
#[inline]
pub fn safe_transfer(token: AccountId, to: AccountId, value: u128) -> Result<(), PSP22Error> {
    let mut token: contract_ref!(PSP22, Env) = token.into();
    token.transfer(to, value, Vec::new())
}

/// Transfers `value` amount of native tokens to an `to` address.
pub fn safe_transfer_native(to: AccountId, value: u128) -> Result<(), TransferHelperError> {
    ink::env::transfer::<Env>(to, value).map_err(|_| TransferHelperError::TransferFailed)
}

/// Transfers `value` amount of `token` tokens `from` account `to` address.
#[inline]
pub fn safe_transfer_from(
    token: AccountId,
    from: AccountId,
    to: AccountId,
    value: u128,
) -> Result<(), PSP22Error> {
    let mut token: contract_ref!(PSP22, Env) = token.into();
    token.transfer_from(from, to, value, Vec::new())
}

/// Wraps `value` amount of native tokens with a contract under `wnative` address.
#[inline]
pub fn wrap(wnative: &AccountId, value: Balance) -> Result<(), PSP22Error> {
    let mut wnative: contract_ref!(Wnative, Env) = (*wnative).into();

    match wnative
        .call_mut()
        .deposit()
        .transferred_value(value)
        .try_invoke()
    {
        Ok(res) => {
            match res {
                Ok(_) => Ok(()),
                Err(_) => Err(PSP22Error::Custom(String::from("deposit failed"))),
            }
        }
        Err(_) => Err(PSP22Error::Custom(String::from("deposit failed"))),
    }
}

/// Unwraps `value` amount of wrapped native tokens.
#[inline]
pub fn unwrap(wnative: &AccountId, value: u128) -> Result<(), PSP22Error> {
    let mut wnative: contract_ref!(Wnative, Env) = (*wnative).into();
    wnative.withdraw(value)
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TransferHelperError {
    TransferFailed,
}
