#![cfg_attr(not(feature = "std"), no_std, no_main)]
mod amp_coef;
mod token_rate;
/// Stabelswap implementation based on the CurveFi stableswap model.
///
/// This pool contract supports up to 8 PSP22 tokens.
///
/// Supports tokens which value increases at some on-chain discoverable rate
/// in terms of some other token, e.g. AZERO x sAZERO.
/// The rate oracle contract must implement [`RateProvider`](trait@traits::RateProvider).
///
/// IMPORTANT:
/// This stableswap implementation is NOT meant for yield-bearing assets which adjusts
/// its total supply to try and maintain a stable price a.k.a. rebasing tokens.
#[ink::contract]
pub mod stable_pool {
    use crate::{amp_coef::AmpCoef, token_rate::TokenRate};
    use amm_helpers::{
        constants::stable_pool::{MAX_COINS, RATE_PRECISION, TOKEN_TARGET_DECIMALS},
        ensure,
        stable_swap_math::{self as math, fees::Fees},
    };
    use ink::contract_ref;
    use ink::prelude::{
        string::{String, ToString},
        {vec, vec::Vec},
    };
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use traits::{
        MathError, Ownable2Step, Ownable2StepData, Ownable2StepResult, StablePool, StablePoolError,
    };

    #[ink(event)]
    pub struct AddLiquidity {
        #[ink(topic)]
        pub provider: AccountId,
        pub token_amounts: Vec<u128>,
        pub shares: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct RemoveLiquidity {
        #[ink(topic)]
        pub provider: AccountId,
        pub token_amounts: Vec<u128>,
        pub shares: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub token_in: AccountId,
        pub amount_in: u128,
        pub token_out: AccountId,
        pub amount_out: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Sync {
        pub reserves: Vec<u128>,
    }

    #[ink(event)]
    pub struct Approval {
        /// Account providing allowance.
        #[ink(topic)]
        pub owner: AccountId,
        /// Allowance beneficiary.
        #[ink(topic)]
        pub spender: AccountId,
        /// New allowance amount.
        pub amount: u128,
    }

    /// Event emitted when transfer of tokens occurs.
    #[ink(event)]
    pub struct Transfer {
        /// Transfer sender. `None` in case of minting new tokens.
        #[ink(topic)]
        pub from: Option<AccountId>,
        /// Transfer recipient. `None` in case of burning tokens.
        #[ink(topic)]
        pub to: Option<AccountId>,
        /// Amount of tokens transferred (or minted/burned).
        pub value: u128,
    }

    #[ink(event)]
    pub struct TransferOwnershipInitiated {
        #[ink(topic)]
        pub new_owner: AccountId,
    }

    #[ink(event)]
    pub struct TransferOwnershipAccepted {
        #[ink(topic)]
        pub new_owner: AccountId,
    }

    #[ink(event)]
    pub struct OwnershipRenounced {}

    #[ink(event)]
    pub struct FeeReceiverChanged {
        #[ink(topic)]
        pub new_fee_receiver: Option<AccountId>,
    }

    #[ink(event)]
    pub struct AmpCoefChange {
        pub init_amp_coef: u128,
        pub future_amp_coef: u128,
        pub init_time: u64,
        pub future_time: u64,
    }

    #[ink(event)]
    pub struct AmpCoefChangeStop {
        pub amp_coef: u128,
        pub time: u64,
    }

    #[ink(event)]
    pub struct FeeChanged {
        pub trade_fee: u32,
        pub protocol_fee: u32,
    }

    #[ink::storage_item]
    #[derive(Debug)]
    pub struct StablePoolData {
        /// List of tokens.
        tokens: Vec<AccountId>,
        /// Tokens precision factors used for normalization.
        precisions: Vec<u128>,
        /// Reserves of tokens
        reserves: Vec<u128>,
        /// Means of getting token rates, either constant or external contract call.
        token_rates: Vec<TokenRate>,
        /// Amplification coefficient.
        amp_coef: AmpCoef,
        /// Fees
        fees: Fees,
        /// Who receives protocol fees (if any).
        fee_receiver: Option<AccountId>,
    }

    #[ink(storage)]
    pub struct StablePoolContract {
        ownable: Ownable2StepData,
        pool: StablePoolData,
        psp22: PSP22Data,
    }

    impl StablePoolContract {
        pub fn new_pool(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            token_rates: Vec<TokenRate>,
            amp_coef: u128,
            owner: AccountId,
            fees: Option<Fees>,
            fee_receiver: Option<AccountId>,
        ) -> Result<Self, StablePoolError> {
            let mut unique_tokens = tokens.clone();
            unique_tokens.sort();
            unique_tokens.dedup();
            let token_count = tokens.len();
            ensure!(
                unique_tokens.len() == token_count,
                StablePoolError::IdenticalTokenId
            );
            ensure!(
                token_count == tokens_decimals.len()
                    && token_count == token_rates.len()
                    && (2..=MAX_COINS).contains(&token_count),
                StablePoolError::IncorrectTokenCount
            );

            ensure!(
                tokens_decimals.iter().all(|&d| d <= TOKEN_TARGET_DECIMALS),
                StablePoolError::TooLargeTokenDecimal
            );

            let precisions = tokens_decimals
                .iter()
                .map(|&decimal| {
                    10u128.pow(TOKEN_TARGET_DECIMALS.checked_sub(decimal).unwrap() as u32)
                })
                .collect();
            Ok(Self {
                ownable: Ownable2StepData::new(owner),
                pool: StablePoolData {
                    tokens,
                    reserves: vec![0; token_count],
                    precisions,
                    token_rates,
                    amp_coef: AmpCoef::new(amp_coef)?,
                    fees: fees.ok_or(StablePoolError::InvalidFee)?,
                    fee_receiver,
                },
                psp22: PSP22Data::default(),
            })
        }

        #[ink(constructor)]
        pub fn new_stable(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            init_amp_coef: u128,
            owner: AccountId,
            trade_fee: u32,
            protocol_fee: u32,
            fee_receiver: Option<AccountId>,
        ) -> Result<Self, StablePoolError> {
            let token_rates = vec![TokenRate::new_constant(RATE_PRECISION); tokens.len()];
            Self::new_pool(
                tokens,
                tokens_decimals,
                token_rates,
                init_amp_coef,
                owner,
                Fees::new(trade_fee, protocol_fee),
                fee_receiver,
            )
        }

        #[ink(constructor)]
        #[allow(clippy::too_many_arguments)]
        pub fn new_rated(
            tokens: Vec<AccountId>,
            tokens_decimals: Vec<u8>,
            external_rates: Vec<Option<AccountId>>,
            init_amp_coef: u128,
            owner: AccountId,
            trade_fee: u32,
            protocol_fee: u32,
            fee_receiver: Option<AccountId>,
        ) -> Result<Self, StablePoolError> {
            let token_rates: Vec<TokenRate> = external_rates
                .into_iter()
                .map(|rate| match rate {
                    Some(contract) => TokenRate::new_external(contract),
                    None => TokenRate::new_constant(RATE_PRECISION),
                })
                .collect();
            Self::new_pool(
                tokens,
                tokens_decimals,
                token_rates,
                init_amp_coef,
                owner,
                Fees::new(trade_fee, protocol_fee),
                fee_receiver,
            )
        }

        /// A helper function emitting events contained in a vector of PSP22Events.
        fn emit_events(&self, events: Vec<PSP22Event>) {
            for event in events {
                match event {
                    PSP22Event::Transfer { from, to, value } => {
                        self.env().emit_event(Transfer { from, to, value })
                    }
                    PSP22Event::Approval {
                        owner,
                        spender,
                        amount,
                    } => self.env().emit_event(Approval {
                        owner,
                        spender,
                        amount,
                    }),
                }
            }
        }

        #[inline]
        fn token_by_address(&self, address: AccountId) -> contract_ref!(PSP22) {
            address.into()
        }

        #[inline]
        fn token_by_id(&self, token_id: usize) -> contract_ref!(PSP22) {
            self.pool.tokens[token_id].into()
        }

        /// Scaled rates are rates multiplied by precision. They are assumed to fit in u128.
        /// If TOKEN_TARGET_DECIMALS is 18 and RATE_DECIMALS is 12, then rates not exceeding ~340282366 should fit.
        /// That's because if precision <= 10^18 and rate <= 10^12 * 340282366, then rate * precision < 2^128.
        fn get_scaled_rates(&mut self) -> Result<Vec<u128>, MathError> {
            self.pool
                .token_rates
                .iter_mut()
                .zip(self.pool.precisions.iter())
                .map(|(rate, &precision)| {
                    rate.get_rate()
                        .checked_mul(precision)
                        .ok_or(MathError::MulOverflow(104))
                })
                .collect()
        }

        fn token_id(&self, token: AccountId) -> Result<usize, StablePoolError> {
            self.pool
                .tokens
                .iter()
                .position(|&id| id == token)
                .ok_or(StablePoolError::InvalidTokenId(token))
        }

        /// Checks if tokens are valid and returns the tokens ids
        fn check_tokens(
            &self,
            token_in: AccountId,
            token_out: AccountId,
        ) -> Result<(usize, usize), StablePoolError> {
            ensure!(token_in != token_out, StablePoolError::IdenticalTokenId);
            //check token ids
            let token_in_id = self.token_id(token_in)?;
            let token_out_id = self.token_id(token_out)?;
            Ok((token_in_id, token_out_id))
        }
        /// Calculates lpt equivalent of the protocol fee and mints it to the `fee_to` if one is set.
        ///
        /// NOTE: Rates should be updated prior to calling this function
        fn mint_protocol_fee(&mut self, fee: u128, token_id: usize) -> Result<(), StablePoolError> {
            if let Some(fee_to) = self.fee_receiver() {
                let protocol_fee = self.pool.fees.protocol_trade_fee(fee)?;
                if protocol_fee > 0 {
                    let rates = self.get_scaled_rates()?;
                    let mut protocol_deposit_amounts = vec![0u128; self.pool.tokens.len()];
                    protocol_deposit_amounts[token_id] = protocol_fee;
                    let mut reserves = self.pool.reserves.clone();
                    reserves[token_id] = reserves[token_id]
                        .checked_sub(protocol_fee)
                        .ok_or(MathError::SubUnderflow(102))?;
                    let (protocol_fee_lp, _) = math::rated_compute_lp_amount_for_deposit(
                        &rates,
                        &protocol_deposit_amounts,
                        &reserves,
                        self.psp22.total_supply(),
                        None, // no fees
                        self.amp_coef()?,
                    )?;
                    // mint fee (shares) to protocol
                    let events = self.psp22.mint(fee_to, protocol_fee_lp)?;
                    self.emit_events(events);
                }
            }
            Ok(())
        }

        fn decrease_reserve(
            &mut self,
            token_id: usize,
            amount: u128,
        ) -> Result<(), StablePoolError> {
            self.pool.reserves[token_id] = self.pool.reserves[token_id]
                .checked_sub(amount)
                .ok_or(MathError::SubUnderflow(101))?;
            Ok(())
        }

        fn increase_reserve(
            &mut self,
            token_id: usize,
            amount: u128,
        ) -> Result<(), StablePoolError> {
            self.pool.reserves[token_id] = self.pool.reserves[token_id]
                .checked_add(amount)
                .ok_or(MathError::AddOverflow(101))?;
            Ok(())
        }

        fn _swap_exact_in(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: Option<u128>,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;

            // get transfered token_in amount
            let token_in_amount = self._transfer_in(token_in_id, token_in_amount)?;

            // Make sure rates are up to date before we attempt any calculations
            let rates = self.get_scaled_rates()?;

            // calc amount_out and fees
            let (token_out_amount, fee) = math::rated_swap_to(
                &rates,
                token_in_id,
                token_in_amount,
                token_out_id,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef()?,
            )?;

            // Check if swapped amount is not less than min_token_out_amount
            ensure!(
                token_out_amount >= min_token_out_amount,
                StablePoolError::InsufficientOutputAmount
            );
            // update reserves
            self.increase_reserve(token_in_id, token_in_amount)?;
            self.decrease_reserve(token_out_id, token_out_amount)?;

            // mint protocol fee
            self.mint_protocol_fee(fee, token_out_id)?;

            // transfer token_out
            self.token_by_address(token_out)
                .transfer(to, token_out_amount, vec![])?;

            self.env().emit_event(Swap {
                sender: self.env().caller(),
                token_in,
                amount_in: token_in_amount,
                token_out,
                amount_out: token_out_amount,
                to,
            });
            self.env().emit_event(Sync {
                reserves: self.reserves(),
            });
            Ok((token_out_amount, fee))
        }

        fn _swap_exact_out(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
            max_token_in_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            //check token ids
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;

            ensure!(
                token_out_amount > 0,
                StablePoolError::InsufficientOutputAmount
            );

            // Make sure rates are up to date before we attempt any calculations
            let rates = self.get_scaled_rates()?;

            // calc amount_out and fees
            let (token_in_amount, fee) = math::rated_swap_from(
                &rates,
                token_in_id,
                token_out_amount,
                token_out_id,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef()?,
            )?;

            // Check if in token_in_amount is as constrained by the user
            ensure!(
                token_in_amount <= max_token_in_amount,
                StablePoolError::InsufficientInputAmount
            );
            // update reserves
            self.increase_reserve(token_in_id, token_in_amount)?;
            self.decrease_reserve(token_out_id, token_out_amount)?;

            // mint protocol fee
            self.mint_protocol_fee(fee, token_out_id)?;

            // transfer token_in
            _ = self._transfer_in(token_in_id, Some(token_in_amount))?;

            // transfer token_out
            self.token_by_address(token_out)
                .transfer(to, token_out_amount, vec![])?;

            self.env().emit_event(Swap {
                sender: self.env().caller(),
                token_in,
                amount_in: token_in_amount,
                token_out,
                amount_out: token_out_amount,
                to,
            });
            self.env().emit_event(Sync {
                reserves: self.reserves(),
            });
            // note that fee is applied to token_out (same as in _swap_exact_in)
            Ok((token_in_amount, fee))
        }

        /// Handles PSP22 token transfer,
        ///
        /// If `amount` is `Some(amount)`, transfer this amount of `token_id`
        /// from the caller to this contract.
        ///
        /// If `amount` of `None`, calculate the difference between
        /// this contract balance and recorded reserve of `token_id`.
        fn _transfer_in(
            &self,
            token_id: usize,
            amount: Option<u128>,
        ) -> Result<u128, StablePoolError> {
            let mut token = self.token_by_id(token_id);
            let amount = if let Some(token_amount) = amount {
                token.transfer_from(
                    self.env().caller(),
                    self.env().account_id(),
                    token_amount,
                    vec![],
                )?;
                token_amount
            } else {
                token
                    .balance_of(self.env().account_id())
                    .checked_sub(self.pool.reserves[token_id])
                    .ok_or(MathError::SubUnderflow(103))?
            };
            ensure!(amount > 0, StablePoolError::InsufficientInputAmount);
            Ok(amount)
        }
    }

    impl StablePool for StablePoolContract {
        #[ink(message)]
        fn add_liquidity(
            &mut self,
            min_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            ensure!(
                amounts.len() == self.pool.tokens.len(),
                StablePoolError::IncorrectAmountsCount
            );
            // Check that at least one amount is non-zero
            ensure!(
                amounts.iter().any(|&amount| amount > 0),
                StablePoolError::ZeroAmounts
            );

            // Make sure rates are up to date before we attempt any calculations
            let rates = self.get_scaled_rates()?;

            // calc lp tokens (shares_to_mint, fee)
            let (shares, fee_part) = math::rated_compute_lp_amount_for_deposit(
                &rates,
                &amounts,
                &self.reserves(),
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )?;

            // Check min shares
            ensure!(
                shares >= min_share_amount,
                StablePoolError::InsufficientLiquidityMinted
            );

            // transfer amounts
            for (id, &token) in self.pool.tokens.iter().enumerate() {
                if amounts[id] > 0 {
                    self.token_by_address(token).transfer_from(
                        self.env().caller(),
                        self.env().account_id(),
                        amounts[id],
                        vec![],
                    )?;
                }
            }

            // mint shares
            let events = self.psp22.mint(to, shares)?;
            self.emit_events(events);

            // mint protocol fee
            if let Some(fee_to) = self.fee_receiver() {
                let protocol_fee = self.pool.fees.protocol_trade_fee(fee_part)?;
                if protocol_fee > 0 {
                    let events = self.psp22.mint(fee_to, protocol_fee)?;
                    self.emit_events(events);
                }
            }

            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.increase_reserve(i, amount)?;
            }

            self.env().emit_event(AddLiquidity {
                provider: self.env().caller(),
                token_amounts: amounts,
                shares,
                to,
            });
            self.env().emit_event(Sync {
                reserves: self.reserves(),
            });
            Ok((shares, fee_part))
        }

        // Note that this method does not require to update rates, neither it uses rates.
        // Thus it's always possible to call it, even if the rate is outdated, or the rate provider is down.
        #[ink(message)]
        fn remove_liquidity_by_shares(
            &mut self,
            shares: u128,
            min_amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<Vec<u128>, StablePoolError> {
            let amounts = math::compute_amounts_given_lp(
                shares,
                &self.reserves(),
                self.psp22.total_supply(),
            )?;

            // Check if enough tokens are withdrawn
            ensure!(
                amounts
                    .iter()
                    .zip(min_amounts.iter())
                    .all(|(amount, min_amount)| amount >= min_amount),
                StablePoolError::InsufficientOutputAmount
            );
            // Check that at least one amount is non-zero
            ensure!(
                amounts.iter().any(|&amount| amount > 0),
                StablePoolError::ZeroAmounts
            );

            // transfer tokens
            for (&token, &amount) in self.pool.tokens.iter().zip(amounts.iter()) {
                if amount > 0 {
                    self.token_by_address(token).transfer(to, amount, vec![])?;
                }
            }

            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.decrease_reserve(i, amount)?;
            }

            // Burn liquidity
            let events = self.psp22.burn(self.env().caller(), shares)?;
            self.emit_events(events);

            self.env().emit_event(RemoveLiquidity {
                provider: self.env().caller(),
                token_amounts: amounts.clone(),
                shares,
                to,
            });
            self.env().emit_event(Sync {
                reserves: self.reserves(),
            });
            Ok(amounts)
        }

        #[ink(message)]
        fn remove_liquidity_by_amounts(
            &mut self,
            max_share_amount: u128,
            amounts: Vec<u128>,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            ensure!(
                amounts.len() == self.pool.tokens.len(),
                StablePoolError::IncorrectAmountsCount
            );
            // Check that at least one amount is non-zero
            ensure!(
                amounts.iter().any(|&amount| amount > 0),
                StablePoolError::ZeroAmounts
            );

            let rates = self.get_scaled_rates()?;

            // calc comparable amounts
            let (shares_to_burn, fee_part) = math::rated_compute_lp_amount_for_withdraw(
                &rates,
                &amounts,
                &self.reserves(),
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )?;

            // check max shares
            ensure!(
                shares_to_burn <= max_share_amount,
                StablePoolError::InsufficientLiquidityBurned
            );
            // burn shares
            let events = self.psp22.burn(self.env().caller(), shares_to_burn)?;
            self.emit_events(events);
            // mint protocol fee
            if let Some(fee_to) = self.fee_receiver() {
                let protocol_fee = self.pool.fees.protocol_trade_fee(fee_part)?;
                if protocol_fee > 0 {
                    let events = self.psp22.mint(fee_to, protocol_fee)?;
                    self.emit_events(events);
                }
            }
            // transfer tokens
            for (&token, &amount) in self.pool.tokens.iter().zip(amounts.iter()) {
                if amount > 0 {
                    self.token_by_address(token).transfer(to, amount, vec![])?;
                }
            }
            // update reserves
            for (i, &amount) in amounts.iter().enumerate() {
                self.decrease_reserve(i, amount)?;
            }

            self.env().emit_event(RemoveLiquidity {
                provider: self.env().caller(),
                token_amounts: amounts,
                shares: shares_to_burn,
                to,
            });
            self.env().emit_event(Sync {
                reserves: self.reserves(),
            });
            Ok((shares_to_burn, fee_part))
        }

        #[ink(message)]
        fn swap_exact_in(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: u128,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            self._swap_exact_in(
                token_in,
                token_out,
                Some(token_in_amount),
                min_token_out_amount,
                to,
            )
        }

        #[ink(message)]
        fn swap_exact_out(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
            max_token_in_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            self._swap_exact_out(
                token_in,
                token_out,
                token_out_amount,
                max_token_in_amount,
                to,
            )
        }

        #[ink(message)]
        fn swap_received(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            min_token_out_amount: u128,
            to: AccountId,
        ) -> Result<(u128, u128), StablePoolError> {
            self._swap_exact_in(token_in, token_out, None, min_token_out_amount, to)
        }

        #[ink(message)]
        fn set_fee_receiver(
            &mut self,
            fee_receiver: Option<AccountId>,
        ) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            self.pool.fee_receiver = fee_receiver;
            self.env().emit_event(FeeReceiverChanged {
                new_fee_receiver: fee_receiver,
            });
            Ok(())
        }

        #[ink(message)]
        fn set_fees(&mut self, trade_fee: u32, protocol_fee: u32) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            self.pool.fees =
                Fees::new(trade_fee, protocol_fee).ok_or(StablePoolError::InvalidFee)?;
            self.env().emit_event(FeeChanged {
                trade_fee,
                protocol_fee,
            });
            Ok(())
        }

        #[ink(message)]
        fn ramp_amp_coef(
            &mut self,
            future_amp_coef: u128,
            future_time: u64,
        ) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            let init_amp_coef = self.amp_coef()?;
            self.pool
                .amp_coef
                .ramp_amp_coef(future_amp_coef, future_time)?;
            self.env().emit_event(AmpCoefChange {
                init_amp_coef,
                future_amp_coef,
                init_time: self.env().block_timestamp(),
                future_time,
            });
            Ok(())
        }

        #[ink(message)]
        fn stop_ramp_amp_coef(&mut self) -> Result<(), StablePoolError> {
            self.ensure_owner()?;
            self.pool.amp_coef.stop_ramp_amp_coef()?;
            self.env().emit_event(AmpCoefChangeStop {
                amp_coef: self.amp_coef()?,
                time: self.env().block_timestamp(),
            });
            Ok(())
        }

        #[ink(message)]
        fn tokens(&self) -> Vec<AccountId> {
            self.pool.tokens.clone()
        }

        #[ink(message)]
        fn reserves(&self) -> Vec<u128> {
            self.pool.reserves.clone()
        }

        #[ink(message)]
        fn amp_coef(&self) -> Result<u128, StablePoolError> {
            Ok(self.pool.amp_coef.compute_amp_coef()?)
        }

        #[ink(message)]
        fn future_amp_coef(&self) -> Option<(u128, u64)> {
            self.pool.amp_coef.future_amp_coef()
        }

        #[ink(message)]
        fn fees(&self) -> (u32, u32) {
            (self.pool.fees.trade_fee, self.pool.fees.protocol_fee)
        }

        #[ink(message)]
        fn fee_receiver(&self) -> Option<AccountId> {
            self.pool.fee_receiver
        }

        #[ink(message)]
        fn token_rates(&mut self) -> Vec<u128> {
            self.pool
                .token_rates
                .iter_mut()
                .map(|rate| rate.get_rate())
                .collect()
        }

        #[ink(message)]
        fn token_rates_providers(&self) -> Vec<Option<AccountId>> {
            self.pool
                .token_rates
                .iter()
                .map(|rate| rate.get_rate_provider())
                .collect()
        }

        #[ink(message)]
        fn get_swap_amount_out(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_in_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            let rates = self.get_scaled_rates()?;
            Ok(math::rated_swap_to(
                &rates,
                token_in_id,
                token_in_amount,
                token_out_id,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef()?,
            )?)
        }

        #[ink(message)]
        fn get_swap_amount_in(
            &mut self,
            token_in: AccountId,
            token_out: AccountId,
            token_out_amount: u128,
        ) -> Result<(u128, u128), StablePoolError> {
            let (token_in_id, token_out_id) = self.check_tokens(token_in, token_out)?;
            let rates = self.get_scaled_rates()?;
            Ok(math::rated_swap_from(
                &rates,
                token_in_id,
                token_out_amount,
                token_out_id,
                &self.reserves(),
                &self.pool.fees,
                self.amp_coef()?,
            )?)
        }

        #[ink(message)]
        fn get_mint_liquidity_for_amounts(
            &mut self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            ensure!(
                amounts.len() == self.pool.tokens.len(),
                StablePoolError::IncorrectAmountsCount
            );
            let rates = self.get_scaled_rates()?;
            Ok(math::rated_compute_lp_amount_for_deposit(
                &rates,
                &amounts,
                &self.reserves(),
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )?)
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_mint(
            &mut self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            Ok(math::compute_amounts_given_lp(
                liquidity,
                &self.reserves(),
                self.psp22.total_supply(),
            )?)
        }

        #[ink(message)]
        fn get_burn_liquidity_for_amounts(
            &mut self,
            amounts: Vec<u128>,
        ) -> Result<(u128, u128), StablePoolError> {
            ensure!(
                amounts.len() == self.pool.tokens.len(),
                StablePoolError::IncorrectAmountsCount
            );
            let rates = self.get_scaled_rates()?;
            math::rated_compute_lp_amount_for_withdraw(
                &rates,
                &amounts,
                &self.reserves(),
                self.psp22.total_supply(),
                Some(&self.pool.fees),
                self.amp_coef()?,
            )
            .map_err(StablePoolError::MathError)
        }

        #[ink(message)]
        fn get_amounts_for_liquidity_burn(
            &mut self,
            liquidity: u128,
        ) -> Result<Vec<u128>, StablePoolError> {
            ensure!(
                liquidity <= self.psp22.total_supply(),
                StablePoolError::InsufficientLiquidity
            );
            Ok(math::compute_amounts_given_lp(
                liquidity,
                &self.reserves(),
                self.psp22.total_supply(),
            )?)
        }
    }

    impl PSP22 for StablePoolContract {
        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.psp22.total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.psp22.balance_of(owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.psp22.allowance(owner, spender)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self.psp22.transfer(self.env().caller(), to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self
                .psp22
                .transfer_from(self.env().caller(), from, to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let events = self.psp22.approve(self.env().caller(), spender, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events =
                self.psp22
                    .increase_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events =
                self.psp22
                    .decrease_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }
    }

    impl PSP22Metadata for StablePoolContract {
        #[ink(message)]
        fn token_name(&self) -> Option<String> {
            Some("CommonStableSwap".to_string())
        }

        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            Some("CMNSS".to_string())
        }

        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            TOKEN_TARGET_DECIMALS
        }
    }

    impl Ownable2Step for StablePoolContract {
        #[ink(message)]
        fn get_owner(&self) -> Ownable2StepResult<AccountId> {
            self.ownable.get_owner()
        }

        #[ink(message)]
        fn get_pending_owner(&self) -> Ownable2StepResult<AccountId> {
            self.ownable.get_pending_owner()
        }

        #[ink(message)]
        fn transfer_ownership(&mut self, new_owner: AccountId) -> Ownable2StepResult<()> {
            self.ownable
                .transfer_ownership(self.env().caller(), new_owner)?;
            self.env()
                .emit_event(TransferOwnershipInitiated { new_owner });
            Ok(())
        }

        #[ink(message)]
        fn accept_ownership(&mut self) -> Ownable2StepResult<()> {
            let new_owner = self.env().caller();
            self.ownable.accept_ownership(new_owner)?;
            self.env()
                .emit_event(TransferOwnershipAccepted { new_owner });
            Ok(())
        }

        #[ink(message)]
        fn renounce_ownership(&mut self) -> Ownable2StepResult<()> {
            self.ownable
                .renounce_ownership(self.env().caller(), self.env().account_id())?;
            self.env().emit_event(OwnershipRenounced {});
            Ok(())
        }

        #[ink(message)]
        fn ensure_owner(&self) -> Ownable2StepResult<()> {
            self.ownable.ensure_owner(self.env().caller())
        }
    }
}
