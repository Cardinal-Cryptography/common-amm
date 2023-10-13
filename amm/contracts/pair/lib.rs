#![cfg_attr(not(feature = "std"), no_std, no_main)]

mod data;

#[ink::contract]
pub mod pair {
    use crate::data::Data;
    use amm::{
        ensure,
        helpers::{
            helper::update_cumulative,
            transfer_helper::{
                balance_of,
                safe_transfer,
            },
            MINIMUM_LIQUIDITY,
            ZERO_ADDRESS,
        },
        traits::{
            factory::Factory,
            pair::{
                Pair,
                PairError,
            },
        },
    };
    use amm_helpers::{
        math::casted_mul,
        types::WrappedU256,
    };
    use ink::{
        contract_ref,
        prelude::vec::Vec,
    };
    use primitive_types::U256;
    use psp22::{
        PSP22Data,
        PSP22Error,
        PSP22Event,
        PSP22,
    };
    use sp_arithmetic::traits::IntegerSquareRoot;

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: Balance,
        pub amount_1: Balance,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: Balance,
        pub amount_1: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0_in: Balance,
        pub amount_1_in: Balance,
        pub amount_0_out: Balance,
        pub amount_1_out: Balance,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Sync {
        reserve_0: Balance,
        reserve_1: Balance,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: Balance,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct PairContract {
        psp22: PSP22Data,
        pair: Data,
    }

    impl PairContract {
        #[ink(constructor)]
        pub fn new(token_a: AccountId, token_b: AccountId) -> Self {
            let mut instance = Self::default();
            let caller = instance.env().caller();
            instance.pair.token_0 = token_a;
            instance.pair.token_1 = token_b;
            instance.pair.factory = caller;
            instance
        }

        fn mint_fee(&mut self, reserve_0: Balance, reserve_1: Balance) -> Result<bool, PairError> {
            let factory_ref: contract_ref!(Factory) = self.pair.factory.into();
            let fee_to = factory_ref.fee_to();
            let fee_on = fee_to != ZERO_ADDRESS.into();
            let k_last: U256 = self.pair.k_last.into();
            if fee_on {
                // Section 2.4 Protocol fee in the whitepaper.
                if !k_last.is_zero() {
                    let root_k: Balance = casted_mul(reserve_0, reserve_1)
                        .integer_sqrt()
                        .try_into()
                        .map_err(|_| PairError::CastOverflow1)?;
                    let root_k_last = k_last
                        .integer_sqrt()
                        .try_into()
                        .map_err(|_| PairError::CastOverflow2)?;
                    if root_k > root_k_last {
                        let total_supply = self.psp22.total_supply();
                        let numerator = total_supply
                            .checked_mul(
                                root_k
                                    .checked_sub(root_k_last)
                                    .ok_or(PairError::SubUnderFlow14)?,
                            )
                            .ok_or(PairError::MulOverFlow13)?;
                        let denominator = root_k
                            .checked_mul(5)
                            .ok_or(PairError::MulOverFlow13)?
                            .checked_add(root_k_last)
                            .ok_or(PairError::AddOverflow1)?;
                        let liquidity = numerator
                            .checked_div(denominator)
                            .ok_or(PairError::DivByZero5)?;
                        if liquidity > 0 {
                            let events = self.psp22.mint(fee_to, liquidity)?;
                            self.emit_events(events)
                        }
                    }
                }
            } else if !k_last.is_zero() {
                self.pair.k_last = 0.into();
            }
            Ok(fee_on)
        }

        fn update(
            &mut self,
            balance_0: Balance,
            balance_1: Balance,
            reserve_0: Balance,
            reserve_1: Balance,
        ) -> Result<(), PairError> {
            let now = Self::env().block_timestamp();
            let last_timestamp = self.pair.block_timestamp_last;
            if now != last_timestamp {
                let (price_0_cumulative_last, price_1_cumulative_last) = update_cumulative(
                    self.pair.price_0_cumulative_last,
                    self.pair.price_1_cumulative_last,
                    now.saturating_sub(last_timestamp).into(),
                    reserve_0,
                    reserve_1,
                );
                self.pair.price_0_cumulative_last = price_0_cumulative_last;
                self.pair.price_1_cumulative_last = price_1_cumulative_last;
            }
            self.pair.reserve_0 = balance_0;
            self.pair.reserve_1 = balance_1;
            self.pair.block_timestamp_last = now;

            self.env().emit_event(Sync {
                reserve_0: balance_0,
                reserve_1: balance_1,
            });
            Ok(())
        }

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
                    } => {
                        self.env().emit_event(Approval {
                            owner,
                            spender,
                            amount,
                        })
                    }
                }
            }
        }
    }

    impl Pair for PairContract {
        #[ink(message)]
        fn get_reserves(&self) -> (Balance, Balance, Timestamp) {
            (
                self.pair.reserve_0,
                self.pair.reserve_1,
                self.pair.block_timestamp_last,
            )
        }
        #[ink(message)]
        fn price_0_cumulative_last(&self) -> WrappedU256 {
            self.pair.price_0_cumulative_last
        }

        #[ink(message)]
        fn price_1_cumulative_last(&self) -> WrappedU256 {
            self.pair.price_1_cumulative_last
        }

        #[ink(message)]
        fn mint(&mut self, to: AccountId) -> Result<Balance, PairError> {
            let reserves = self.get_reserves();
            let contract = self.env().account_id();
            let balance_0 = balance_of(self.pair.token_0, contract);
            let balance_1 = balance_of(self.pair.token_1, contract);
            let amount_0_transferred = balance_0
                .checked_sub(reserves.0)
                .ok_or(PairError::SubUnderFlow1)?;
            let amount_1_transferred = balance_1
                .checked_sub(reserves.1)
                .ok_or(PairError::SubUnderFlow2)?;

            let fee_on = self.mint_fee(reserves.0, reserves.1)?;
            let total_supply = self.psp22.total_supply();

            let liquidity;
            if total_supply == 0 {
                let liq = amount_0_transferred
                    .checked_mul(amount_1_transferred)
                    .ok_or(PairError::MulOverFlow1)?;
                liquidity = liq
                    .integer_sqrt()
                    .checked_sub(MINIMUM_LIQUIDITY)
                    .ok_or(PairError::SubUnderFlow3)?;
                let events = self.psp22.mint(ZERO_ADDRESS.into(), MINIMUM_LIQUIDITY)?;
                self.emit_events(events)
            } else {
                let liquidity_1 = amount_0_transferred
                    .checked_mul(total_supply)
                    .ok_or(PairError::MulOverFlow2)?
                    .checked_div(reserves.0)
                    .ok_or(PairError::DivByZero1)?;
                let liquidity_2 = amount_1_transferred
                    .checked_mul(total_supply)
                    .ok_or(PairError::MulOverFlow3)?
                    .checked_div(reserves.1)
                    .ok_or(PairError::DivByZero2)?;
                liquidity = if liquidity_1 < liquidity_2 {
                    liquidity_1
                } else {
                    liquidity_2
                };
            }

            ensure!(liquidity > 0, PairError::InsufficientLiquidityMinted);

            let events = self.psp22.mint(to, liquidity)?;
            self.emit_events(events);

            self.update(balance_0, balance_1, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last = casted_mul(reserves.0, reserves.1).into();
            }

            self.env().emit_event(Mint {
                sender: self.env().caller(),
                amount_0: amount_0_transferred,
                amount_1: amount_1_transferred,
            });

            Ok(liquidity)
        }

        #[ink(message)]
        fn burn(&mut self, to: AccountId) -> Result<(Balance, Balance), PairError> {
            let reserves = self.get_reserves();
            let contract = self.env().account_id();
            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;
            let balance_0_before = balance_of(token_0, contract);
            let balance_1_before = balance_of(token_1, contract);
            let liquidity = self.balance_of(contract);

            let fee_on = self.mint_fee(reserves.0, reserves.1)?;
            let total_supply = self.psp22.total_supply();
            let amount_0 = liquidity
                .checked_mul(balance_0_before)
                .ok_or(PairError::MulOverFlow5)?
                .checked_div(total_supply)
                .ok_or(PairError::DivByZero3)?;
            let amount_1 = liquidity
                .checked_mul(balance_1_before)
                .ok_or(PairError::MulOverFlow6)?
                .checked_div(total_supply)
                .ok_or(PairError::DivByZero4)?;

            ensure!(
                amount_0 > 0 && amount_1 > 0,
                PairError::InsufficientLiquidityBurned
            );

            let events = self.psp22.burn(contract, liquidity)?;
            self.emit_events(events);

            safe_transfer(token_0, to, amount_0)?;
            safe_transfer(token_1, to, amount_1)?;

            let balance_0_after = balance_of(token_0, contract);
            let balance_1_after = balance_of(token_1, contract);

            self.update(balance_0_after, balance_1_after, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last = casted_mul(reserves.0, reserves.1).into();
            }

            self.env().emit_event(Burn {
                sender: self.env().caller(),
                amount_0,
                amount_1,
                to,
            });

            Ok((amount_0, amount_1))
        }

        #[ink(message)]
        fn swap(
            &mut self,
            amount_0_out: Balance,
            amount_1_out: Balance,
            to: AccountId,
        ) -> Result<(), PairError> {
            ensure!(
                amount_0_out > 0 || amount_1_out > 0,
                PairError::InsufficientOutputAmount
            );
            let reserves = self.get_reserves();
            ensure!(
                amount_0_out < reserves.0 && amount_1_out < reserves.1,
                PairError::InsufficientLiquidity
            );

            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;

            ensure!(to != token_0 && to != token_1, PairError::InvalidTo);
            if amount_0_out > 0 {
                safe_transfer(token_0, to, amount_0_out)?;
            }
            if amount_1_out > 0 {
                safe_transfer(token_1, to, amount_1_out)?;
            }
            let contract = self.env().account_id();
            let balance_0 = balance_of(token_0, contract);
            let balance_1 = balance_of(token_1, contract);

            let amount_0_in = if balance_0
                > reserves
                    .0
                    .checked_sub(amount_0_out)
                    .ok_or(PairError::SubUnderFlow4)?
            {
                balance_0
                    .checked_sub(
                        reserves
                            .0
                            .checked_sub(amount_0_out)
                            .ok_or(PairError::SubUnderFlow5)?,
                    )
                    .ok_or(PairError::SubUnderFlow6)?
            } else {
                0
            };
            let amount_1_in = if balance_1
                > reserves
                    .1
                    .checked_sub(amount_1_out)
                    .ok_or(PairError::SubUnderFlow7)?
            {
                balance_1
                    .checked_sub(
                        reserves
                            .1
                            .checked_sub(amount_1_out)
                            .ok_or(PairError::SubUnderFlow8)?,
                    )
                    .ok_or(PairError::SubUnderFlow9)?
            } else {
                0
            };

            ensure!(
                amount_0_in > 0 || amount_1_in > 0,
                PairError::InsufficientInputAmount
            );

            let balance_0_adjusted = balance_0
                .checked_mul(1000)
                .ok_or(PairError::MulOverFlow7)?
                .checked_sub(amount_0_in.checked_mul(3).ok_or(PairError::MulOverFlow8)?)
                .ok_or(PairError::SubUnderFlow10)?;
            let balance_1_adjusted = balance_1
                .checked_mul(1000)
                .ok_or(PairError::MulOverFlow9)?
                .checked_sub(amount_1_in.checked_mul(3).ok_or(PairError::MulOverFlow10)?)
                .ok_or(PairError::SubUnderFlow11)?;

            // Cast to U256 to prevent Overflow
            ensure!(
                casted_mul(balance_0_adjusted, balance_1_adjusted)
                    >= casted_mul(reserves.0, reserves.1)
                        .checked_mul(1000u128.pow(2).into())
                        .ok_or(PairError::MulOverFlow14)?,
                PairError::K
            );

            self.update(balance_0, balance_1, reserves.0, reserves.1)?;

            self.env().emit_event(Swap {
                sender: self.env().caller(),
                amount_0_in,
                amount_1_in,
                amount_0_out,
                amount_1_out,
                to,
            });
            Ok(())
        }

        #[ink(message)]
        fn skim(&mut self, to: AccountId) -> Result<(), PairError> {
            let contract = self.env().account_id();
            let reserve_0 = self.pair.reserve_0;
            let reserve_1 = self.pair.reserve_1;
            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;
            let balance_0 = balance_of(token_0, contract);
            let balance_1 = balance_of(token_1, contract);
            safe_transfer(
                token_0,
                to,
                balance_0
                    .checked_sub(reserve_0)
                    .ok_or(PairError::SubUnderFlow12)?,
            )?;
            safe_transfer(
                token_1,
                to,
                balance_1
                    .checked_sub(reserve_1)
                    .ok_or(PairError::SubUnderFlow13)?,
            )?;
            Ok(())
        }

        #[ink(message)]
        fn sync(&mut self) -> Result<(), PairError> {
            let contract = self.env().account_id();
            let reserve_0 = self.pair.reserve_0;
            let reserve_1 = self.pair.reserve_1;
            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;
            let balance_0 = balance_of(token_0, contract);
            let balance_1 = balance_of(token_1, contract);
            self.update(balance_0, balance_1, reserve_0, reserve_1)
        }

        #[ink(message)]
        fn get_token_0(&self) -> AccountId {
            self.pair.token_0
        }

        #[ink(message)]
        fn get_token_1(&self) -> AccountId {
            self.pair.token_1
        }
    }

    impl PSP22 for PairContract {
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
            let caller = self.env().caller();
            let infinite_allowance = self.psp22.allowance(from, caller) == u128::MAX;
            let mut events = self
                .psp22
                .transfer_from(self.env().caller(), from, to, value)?;
            if infinite_allowance {
                self.psp22.approve(from, caller, u128::MAX)?;
                events.remove(0);
            }
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

    #[cfg(test)]
    mod tests {
        use super::*;
        use sp_arithmetic::FixedU128;

        #[ink::test]
        fn initialize_works() {
            let token_0 = AccountId::from([0x03; 32]);
            let token_1 = AccountId::from([0x04; 32]);
            let pair = PairContract::new(token_0, token_1);
            assert_eq!(pair.get_token_0(), token_0);
            assert_eq!(pair.get_token_1(), token_1);
        }

        #[ink::test]
        fn update_cumulative_from_zero_time_elapsed() {
            let (cumulative0, cumulative1) =
                update_cumulative(0.into(), 0.into(), 0.into(), 10, 10);
            assert_eq!(cumulative0, 0.into());
            assert_eq!(cumulative1, 0.into());
        }

        #[ink::test]
        fn update_cumulative_from_one_time_elapsed() {
            let (cumulative0, cumulative1) =
                update_cumulative(0.into(), 0.into(), 1.into(), 10, 10);
            assert_eq!(
                FixedU128::from_inner(U256::from(cumulative0).as_u128()),
                1.into()
            );
            assert_eq!(
                FixedU128::from_inner(U256::from(cumulative1).as_u128()),
                1.into()
            );
        }
    }
}
