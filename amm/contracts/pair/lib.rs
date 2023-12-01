#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod pair {
    // Numbers used in the equations below, derived from the UniswapV2 paper.
    // They have different meaning depending on the context so please consult the WP.
    // Adjustments made to not deal with floating point numbers.

    // Whitepaper 3.2.1, equation (11)
    const TRADING_FEE_ADJ_RESERVES: u128 = 1000;
    const TRADING_FEE_ADJ_AMOUNTS: u128 = 3;

    // Whitepaper 2.4, equation (7)
    const PROTOCOL_FEE_ADJ_DENOM: u128 = 5;

    use amm_helpers::{
        constants::{BURN_ADDRESS, MINIMUM_LIQUIDITY},
        ensure,
        math::casted_mul,
        types::WrappedU256,
    };
    use ink::{contract_ref, prelude::vec::Vec};
    use primitive_types::U256;
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22};
    use sp_arithmetic::{traits::IntegerSquareRoot, FixedPointNumber, FixedU128};
    use traits::{Factory, MathError, Pair, PairError};

    #[ink(event)]
    pub struct Mint {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: u128,
        pub amount_1: u128,
    }

    #[ink(event)]
    pub struct Burn {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0: u128,
        pub amount_1: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Swap {
        #[ink(topic)]
        pub sender: AccountId,
        pub amount_0_in: u128,
        pub amount_1_in: u128,
        pub amount_0_out: u128,
        pub amount_1_out: u128,
        #[ink(topic)]
        pub to: AccountId,
    }

    #[ink(event)]
    pub struct Sync {
        reserve_0: u128,
        reserve_1: u128,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: u128,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: u128,
    }

    #[ink::storage_item]
    #[derive(Debug)]
    pub struct PairData {
        pub factory: AccountId,
        pub token_0: AccountId,
        pub token_1: AccountId,
        pub reserve_0: u128,
        pub reserve_1: u128,
        pub block_timestamp_last: u64,
        pub price_0_cumulative_last: WrappedU256,
        pub price_1_cumulative_last: WrappedU256,
        pub k_last: Option<WrappedU256>,
    }

    impl PairData {
        fn new(token_0: AccountId, token_1: AccountId, factory: AccountId) -> Self {
            Self {
                factory,
                token_0,
                token_1,
                reserve_0: 0,
                reserve_1: 0,
                block_timestamp_last: 0,
                price_0_cumulative_last: 0.into(),
                price_1_cumulative_last: 0.into(),
                k_last: None,
            }
        }
    }

    #[ink(storage)]
    pub struct PairContract {
        psp22: PSP22Data,
        pair: PairData,
    }

    impl PairContract {
        #[ink(constructor)]
        pub fn new(token_0: AccountId, token_1: AccountId) -> Self {
            let pair = PairData::new(token_0, token_1, Self::env().caller());
            Self {
                psp22: PSP22Data::default(),
                pair,
            }
        }

        #[inline]
        fn token_0(&self) -> contract_ref!(PSP22) {
            self.pair.token_0.into()
        }

        #[inline]
        fn token_1(&self) -> contract_ref!(PSP22) {
            self.pair.token_1.into()
        }

        #[inline]
        fn factory(&self) -> contract_ref!(Factory) {
            self.pair.factory.into()
        }

        #[inline]
        fn token_balances(&self, who: AccountId) -> (u128, u128) {
            (
                self.token_0().balance_of(who),
                self.token_1().balance_of(who),
            )
        }

        fn mint_fee(&mut self, reserve_0: u128, reserve_1: u128) -> Result<bool, PairError> {
            if let Some(fee_to) = self.factory().fee_to() {
                // Section 2.4 Protocol fee in the whitepaper.
                if let Some(k_last) = self.pair.k_last.map(Into::<U256>::into) {
                    let root_k: u128 = casted_mul(reserve_0, reserve_1)
                        .integer_sqrt()
                        .try_into()
                        .map_err(|_| MathError::CastOverflow(1))?;
                    let root_k_last = k_last
                        .integer_sqrt()
                        .try_into()
                        .map_err(|_| MathError::CastOverflow(2))?;
                    if root_k > root_k_last {
                        let total_supply = self.psp22.total_supply();
                        let numerator = total_supply
                            .checked_mul(
                                root_k
                                    .checked_sub(root_k_last)
                                    .ok_or(MathError::SubUnderflow(1))?,
                            )
                            .ok_or(MathError::MulOverflow(1))?;
                        let denominator = root_k
                            .checked_mul(PROTOCOL_FEE_ADJ_DENOM)
                            .ok_or(MathError::MulOverflow(2))?
                            .checked_add(root_k_last)
                            .ok_or(MathError::AddOverflow(1))?;
                        let liquidity = numerator
                            .checked_div(denominator)
                            .ok_or(MathError::DivByZero(1))?;
                        if liquidity > 0 {
                            let events = self.psp22.mint(fee_to, liquidity)?;
                            self.emit_events(events)
                        }
                    }
                }
                Ok(true)
            } else if self.pair.k_last.is_some() {
                self.pair.k_last = None;
                Ok(false)
            } else {
                Ok(false)
            }
        }

        fn update(
            &mut self,
            balance_0: u128,
            balance_1: u128,
            reserve_0: u128,
            reserve_1: u128,
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
                    } => self.env().emit_event(Approval {
                        owner,
                        spender,
                        amount,
                    }),
                }
            }
        }
    }

    impl Pair for PairContract {
        #[ink(message)]
        fn get_factory(&self) -> AccountId {
            self.pair.factory
        }

        #[ink(message)]
        fn get_minimum_liquidity(&self) -> u128 {
            MINIMUM_LIQUIDITY
        }

        #[ink(message)]
        fn get_reserves(&self) -> (u128, u128, u64) {
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
        fn mint(&mut self, to: AccountId) -> Result<u128, PairError> {
            let reserves = self.get_reserves();
            let contract = self.env().account_id();
            let (balance_0, balance_1) = self.token_balances(contract);
            let amount_0_transferred = balance_0
                .checked_sub(reserves.0)
                .ok_or(MathError::SubUnderflow(2))?;
            let amount_1_transferred = balance_1
                .checked_sub(reserves.1)
                .ok_or(MathError::SubUnderflow(3))?;

            let fee_on = self.mint_fee(reserves.0, reserves.1)?;
            let total_supply = self.psp22.total_supply();

            let liquidity;
            if total_supply == 0 {
                let liq = amount_0_transferred
                    .checked_mul(amount_1_transferred)
                    .ok_or(MathError::MulOverflow(3))?;
                liquidity = liq
                    .integer_sqrt()
                    .checked_sub(MINIMUM_LIQUIDITY)
                    .ok_or(MathError::SubUnderflow(4))?;
                let events = self.psp22.mint(BURN_ADDRESS.into(), MINIMUM_LIQUIDITY)?;
                self.emit_events(events)
            } else {
                let liquidity_0 = amount_0_transferred
                    .checked_mul(total_supply)
                    .ok_or(MathError::MulOverflow(4))?
                    .checked_div(reserves.0)
                    .ok_or(MathError::DivByZero(2))?;
                let liquidity_1 = amount_1_transferred
                    .checked_mul(total_supply)
                    .ok_or(MathError::MulOverflow(5))?
                    .checked_div(reserves.1)
                    .ok_or(MathError::DivByZero(3))?;
                liquidity = if liquidity_0 < liquidity_1 {
                    liquidity_0
                } else {
                    liquidity_1
                };
            }

            ensure!(liquidity > 0, PairError::InsufficientLiquidityMinted);

            let events = self.psp22.mint(to, liquidity)?;
            self.emit_events(events);

            self.update(balance_0, balance_1, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last = Some(casted_mul(reserves.0, reserves.1).into());
            }

            self.env().emit_event(Mint {
                sender: self.env().caller(),
                amount_0: amount_0_transferred,
                amount_1: amount_1_transferred,
            });

            Ok(liquidity)
        }

        #[ink(message)]
        fn burn(&mut self, to: AccountId) -> Result<(u128, u128), PairError> {
            let reserves = self.get_reserves();
            let contract = self.env().account_id();
            let (balance_0_before, balance_1_before) = self.token_balances(contract);
            let liquidity = self.balance_of(contract);

            let fee_on = self.mint_fee(reserves.0, reserves.1)?;
            let total_supply = self.psp22.total_supply();
            let amount_0 = liquidity
                .checked_mul(balance_0_before)
                .ok_or(MathError::MulOverflow(6))?
                .checked_div(total_supply)
                .ok_or(MathError::DivByZero(4))?;
            let amount_1 = liquidity
                .checked_mul(balance_1_before)
                .ok_or(MathError::MulOverflow(7))?
                .checked_div(total_supply)
                .ok_or(MathError::DivByZero(5))?;

            ensure!(
                amount_0 > 0 && amount_1 > 0,
                PairError::InsufficientLiquidityBurned
            );

            let events = self.psp22.burn(contract, liquidity)?;
            self.emit_events(events);

            self.token_0().transfer(to, amount_0, Vec::new())?;
            self.token_1().transfer(to, amount_1, Vec::new())?;

            let (balance_0_after, balance_1_after) = self.token_balances(contract);

            self.update(balance_0_after, balance_1_after, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last = Some(casted_mul(reserves.0, reserves.1).into());
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
            amount_0_out: u128,
            amount_1_out: u128,
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
                self.token_0().transfer(to, amount_0_out, Vec::new())?;
            }
            if amount_1_out > 0 {
                self.token_1().transfer(to, amount_1_out, Vec::new())?;
            }
            let contract = self.env().account_id();
            let (balance_0, balance_1) = self.token_balances(contract);

            let liquidity_depth_0 = reserves
                .0
                .checked_sub(amount_0_out)
                .ok_or(MathError::SubUnderflow(5))?;

            let amount_0_in = if balance_0 > liquidity_depth_0 {
                balance_0
                    .checked_sub(liquidity_depth_0)
                    .ok_or(MathError::SubUnderflow(6))?
            } else {
                0
            };

            let liquidity_depth_1 = reserves
                .1
                .checked_sub(amount_1_out)
                .ok_or(MathError::SubUnderflow(7))?;

            let amount_1_in = if balance_1 > liquidity_depth_1 {
                balance_1
                    .checked_sub(liquidity_depth_1)
                    .ok_or(MathError::SubUnderflow(8))?
            } else {
                0
            };

            ensure!(
                amount_0_in > 0 || amount_1_in > 0,
                PairError::InsufficientInputAmount
            );

            let balance_0_adjusted = balance_0
                .checked_mul(TRADING_FEE_ADJ_RESERVES)
                .ok_or(MathError::MulOverflow(8))?
                .checked_sub(
                    amount_0_in
                        .checked_mul(TRADING_FEE_ADJ_AMOUNTS)
                        .ok_or(MathError::MulOverflow(9))?,
                )
                .ok_or(MathError::SubUnderflow(9))?;
            let balance_1_adjusted = balance_1
                .checked_mul(TRADING_FEE_ADJ_RESERVES)
                .ok_or(MathError::MulOverflow(10))?
                .checked_sub(
                    amount_1_in
                        .checked_mul(TRADING_FEE_ADJ_AMOUNTS)
                        .ok_or(MathError::MulOverflow(11))?,
                )
                .ok_or(MathError::SubUnderflow(10))?;

            // Cast to U256 to prevent Overflow
            ensure!(
                casted_mul(balance_0_adjusted, balance_1_adjusted)
                    >= casted_mul(reserves.0, reserves.1)
                        .checked_mul(TRADING_FEE_ADJ_RESERVES.pow(2).into())
                        .ok_or(MathError::MulOverflow(12))?,
                PairError::KInvariantChanged
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
            let (balance_0, balance_1) = self.token_balances(contract);
            let (amount_0, amount_1) = (
                balance_0
                    .checked_sub(reserve_0)
                    .ok_or(MathError::SubUnderflow(11))?,
                balance_1
                    .checked_sub(reserve_1)
                    .ok_or(MathError::SubUnderflow(12))?,
            );
            self.token_0().transfer(to, amount_0, Vec::new())?;
            self.token_1().transfer(to, amount_1, Vec::new())?;
            Ok(())
        }

        #[ink(message)]
        fn sync(&mut self) -> Result<(), PairError> {
            let contract = self.env().account_id();
            let reserve_0 = self.pair.reserve_0;
            let reserve_1 = self.pair.reserve_1;
            let (balance_0, balance_1) = self.token_balances(contract);
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

    #[inline]
    pub fn update_cumulative(
        price_0_cumulative_last: WrappedU256,
        price_1_cumulative_last: WrappedU256,
        time_elapsed: U256,
        reserve_0: u128,
        reserve_1: u128,
    ) -> (WrappedU256, WrappedU256) {
        let price_cumulative_last_0: WrappedU256 = U256::from(
            FixedU128::checked_from_rational(reserve_1, reserve_0)
                .unwrap_or_default()
                .into_inner(),
        )
        .saturating_mul(time_elapsed)
        .saturating_add(price_0_cumulative_last.into())
        .into();
        let price_cumulative_last_1: WrappedU256 = U256::from(
            FixedU128::checked_from_rational(reserve_0, reserve_1)
                .unwrap_or_default()
                .into_inner(),
        )
        .saturating_mul(time_elapsed)
        .saturating_add(price_1_cumulative_last.into())
        .into();
        (price_cumulative_last_0, price_cumulative_last_1)
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
