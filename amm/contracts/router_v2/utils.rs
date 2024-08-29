use amm_helpers::ensure;
use ink::{
    codegen::TraitCallBuilder,
    contract_ref,
    env::{block_timestamp, DefaultEnvironment as Env},
    prelude::{string::String, vec::Vec},
    primitives::AccountId,
};
use psp22::{PSP22Error, PSP22};
use traits::{Balance, RouterV2Error};
use wrapped_azero::WrappedAZERO;

/// Checks if the current block timestamp is not after the deadline.
#[inline]
pub fn check_timestamp(deadline: u64) -> Result<(), RouterV2Error> {
    ensure!(deadline >= block_timestamp::<Env>(), RouterV2Error::Expired);
    Ok(())
}

#[inline]
pub fn psp22_transfer(token: AccountId, to: AccountId, value: u128) -> Result<(), PSP22Error> {
    let mut token: contract_ref!(PSP22, Env) = token.into();
    token.transfer(to, value, Vec::new())
}

#[inline]
pub fn psp22_transfer_from(
    token: AccountId,
    from: AccountId,
    to: AccountId,
    value: u128,
) -> Result<(), PSP22Error> {
    let mut token: contract_ref!(PSP22, Env) = token.into();
    token.transfer_from(from, to, value, Vec::new())
}

#[inline]
pub fn wrap(wnative: AccountId, value: Balance) -> Result<(), RouterV2Error> {
    let mut wnative_ref: contract_ref!(WrappedAZERO, Env) = wnative.into();
    wnative_ref
        .call_mut()
        .deposit()
        .transferred_value(value)
        .try_invoke()
        .map_err(|_| {
            RouterV2Error::CrossContractCallFailed(String::from("Wrapped AZERO: deposit"))
        })???;
    Ok(())
}

#[inline]
pub fn withdraw(wnative: AccountId, value: Balance) -> Result<(), RouterV2Error> {
    let mut wnative_ref: contract_ref!(WrappedAZERO, Env) = wnative.into();
    Ok(wnative_ref.withdraw(value)?)
}
