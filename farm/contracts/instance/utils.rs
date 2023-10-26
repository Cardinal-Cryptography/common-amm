use ink::{
    codegen::TraitCallBuilder,
    contract_ref,
    prelude::vec,
};
use psp22_traits::PSP22;

use crate::UserId;

// We're making a concious choice here that we don't want to fail the whole transaction
// if a PSP22::transfer fails with a panic.
// This is to ensure that funds are not locked in the farm if someone uses malicious
// PSP22 token impl for rewards.
pub fn safe_transfer<Env: ink::env::Environment>(
    psp22: &mut contract_ref!(PSP22, Env),
    recipient: UserId,
    amount: u128,
) -> Result<(), psp22_traits::PSP22Error> {
    match psp22
        .call_mut()
        .transfer(recipient, amount, vec![])
        .try_invoke()
    {
        Err(ink_env_err) => {
            ink::env::debug_println!("ink env error: {:?}", ink_env_err);
            Ok(())
        }
        Ok(Err(ink_lang_err)) => {
            ink::env::debug_println!("ink lang error: {:?}", ink_lang_err);
            Ok(())
        }
        Ok(Ok(Err(psp22_error))) => {
            ink::env::debug_println!("psp22 error: {:?}", psp22_error);
            Ok(())
        }
        Ok(Ok(Ok(res))) => Ok(res),
    }
}

// We don't want to fail the whole transaction if PSP22::balance_of fails with a panic either.
// We choose to use `0` to denote the "panic" scenarios b/c it's a noop for the farm.
pub fn safe_balance_of<Env: ink::env::Environment>(
    psp22: &contract_ref!(PSP22, Env),
    account: UserId,
) -> u128 {
    match psp22.call().balance_of(account).try_invoke() {
        Err(ink_env_err) => {
            ink::env::debug_println!("ink env error: {:?}", ink_env_err);
            0
        }
        Ok(Err(ink_lang_err)) => {
            ink::env::debug_println!("ink lang error: {:?}", ink_lang_err);
            0
        }
        Ok(Ok(res)) => res,
    }
}
