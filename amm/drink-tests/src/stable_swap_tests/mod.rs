mod tests_add_remove_lp;
mod tests_getters;
mod tests_rated;
mod tests_swap_exact_in_received;
mod tests_swap_exact_out;

use crate::stable_pool_contract;
pub use crate::utils::*;
use primitive_types::U256;

// pub use stable_pool_contract::StablePool as _;
pub use stable_pool_contract::StablePoolError;

use drink::{self, runtime::MinimalRuntime, session::Session, AccountId32};

use ink_primitives::AccountId;
use ink_wrapper_types::{Connection, ToAccountId};

/// Fee denominator. Fees are expressed in 1e9 precision (1_000_000_000 is 100%)
pub const FEE_DENOM: u128 = 1_000_000_000;

pub const RATE_PRECISION: u128 = 10u128.pow(12);

pub const FEE_RECEIVER: AccountId32 = AccountId32::new([42u8; 32]);

pub fn fee_receiver() -> ink_primitives::AccountId {
    AsRef::<[u8; 32]>::as_ref(&FEE_RECEIVER).clone().into()
}

pub const ONE_LPT: u128 = 1000000000000000000;
pub const ONE_DAI: u128 = 1000000000000000000;
pub const ONE_USDT: u128 = 1000000;
pub const ONE_USDC: u128 = 1000000;

pub fn setup_stable_swap_with_tokens(
    session: &mut Session<MinimalRuntime>,
    token_decimals: Vec<u8>,
    token_supply: Vec<u128>,
    amp_coef: u128,
    trade_fee: u32,
    protocol_trade_fee: u32,
    caller: AccountId32,
    salt: Vec<u8>,
) -> (AccountId, Vec<AccountId>) {
    let _ = session.set_actor(caller);

    if token_decimals.len() != token_supply.len() {
        panic!("SETUP: Inconsistent number of tokens.")
    }

    upload_all(session);

    let salty_str = String::from_utf8(salt.clone()).unwrap_or("Test token".to_string());
    // instantiate tokens
    let tokens: Vec<AccountId> = token_decimals
        .iter()
        .zip(token_supply.iter())
        .enumerate()
        .map(|(id, (&decimals, &supply))| {
            psp22_utils::setup_with_amounts(
                session,
                format!("{salty_str} {id}").to_string(),
                decimals,
                supply,
                BOB,
            )
            .into()
        })
        .collect::<Vec<AccountId>>();

    // instantiate stable_swap
    let instance = stable_pool_contract::Instance::new_stable(
        tokens.clone(),
        token_decimals,
        amp_coef,
        bob(),
        trade_fee,
        protocol_trade_fee,
        Some(fee_receiver()),
    )
    .with_salt(salt);

    let stable_swap: stable_pool_contract::Instance = session
        .instantiate(instance)
        .unwrap()
        .result
        .to_account_id()
        .into();

    // setup max allowance for stable swap contract on both tokens
    for token in tokens.clone() {
        psp22_utils::increase_allowance(session, token.into(), stable_swap.into(), u128::MAX, BOB)
            .unwrap();
    }

    (stable_swap.into(), tokens)
}

pub fn share_price_and_total_shares(
    session: &mut Session<MinimalRuntime>,
    stable_swap: AccountId,
) -> (u128, u128) {
    let total_shares = psp22_utils::total_supply(session, stable_swap);
    let reserves = stable_swap::reserves(session, stable_swap);
    let token_rates = stable_swap::token_rates(session, stable_swap);

    let sum_token = stable_swap::tokens(session, stable_swap)
        .iter()
        .zip(reserves.iter())
        .zip(token_rates.iter())
        .fold(0, |acc, ((&token, reserve), rate)| {
            acc + reserve
                * 10u128.pow((18 - psp22_utils::token_decimals(session, token)).into())
                * rate
                / RATE_PRECISION
        });

    (
        U256::from(sum_token)
            .checked_mul(100000000.into())
            .unwrap()
            .checked_div(total_shares.into())
            .unwrap_or(0.into()) // return 0 if total shares 0
            .as_u128(),
        total_shares,
    )
}

pub fn transfer_and_increase_allowance(
    session: &mut Session<MinimalRuntime>,
    stable_swap: AccountId,
    tokens: Vec<AccountId>,
    receiver: AccountId32,
    amounts: Vec<u128>,
    caller: AccountId32,
) {
    for (&token, &amount) in tokens.iter().zip(amounts.iter()) {
        _ = psp22_utils::transfer(
            session,
            token,
            receiver.to_account_id(),
            amount,
            caller.clone(),
        );
        _ = psp22_utils::increase_allowance(
            session,
            token,
            stable_swap,
            u128::MAX,
            receiver.clone(),
        );
    }
}
