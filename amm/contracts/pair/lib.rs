#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod pair {
    // 2^112
    const Q112: u128 = 5192296858534827628530496329220096;
    // From the UniswapV2 whitepaper. Section 3.7.
    // This number is high enough to support 18-decimal-place tokens
    // with a totalSupply over 1 quadrillion.
    // RESERVES_UPPER_BOUND cannot be u128::MAX because reserve_0*1000*reserve_1*1000
    // must fit in U256 for swap to work correctly.
    //
    // 2^112 - 1
    const RESERVES_UPPER_BOUND: u128 = Q112 - 1;

    // Numbers used in the equations below, derived from the UniswapV2 paper.
    // They have different meaning depending on the context so please consult the WP.
    // Adjustments made to not deal with floating point numbers.

    // Whitepaper 3.2.1, equation (11)
    const TRADING_FEE_ADJ_RESERVES: u128 = 1000;
    const TRADING_FEE_ADJ_AMOUNTS: u128 = 3;

    // Whitepaper 2.4, equation (7)
    const PROTOCOL_FEE_ADJ_DENOM: u128 = 5;

    const TWO_POW_32: u64 = 4294967296;

    use amm_helpers::{
        constants::{BURN_ADDRESS, MINIMUM_LIQUIDITY},
        ensure,
        math::casted_mul,
        types::WrappedU256,
    };
    use ink::{
        contract_ref,
        prelude::{
            string::{String, ToString},
            vec::Vec,
        },
    };

    use primitive_types::U256;
    use psp22::{PSP22Data, PSP22Error, PSP22Event, PSP22Metadata, PSP22};
    use traits::{Factory, MathError, Pair, PairError, SwapCallee};

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
        pub block_timestamp_last: u32,
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
                    let root_k: U256 = casted_mul(reserve_0, reserve_1).integer_sqrt();
                    let root_k_last = k_last.integer_sqrt();
                    if root_k > root_k_last {
                        let total_supply: U256 = self.psp22.total_supply().into();
                        let numerator = total_supply
                            .checked_mul(
                                root_k
                                    .checked_sub(root_k_last)
                                    .ok_or(MathError::SubUnderflow(1))?,
                            )
                            .ok_or(MathError::MulOverflow(1))?;
                        let denominator = root_k
                            .checked_mul(PROTOCOL_FEE_ADJ_DENOM.into())
                            .ok_or(MathError::MulOverflow(2))?
                            .checked_add(root_k_last)
                            .ok_or(MathError::AddOverflow(1))?;
                        let liquidity: u128 = numerator
                            .checked_div(denominator)
                            .ok_or(MathError::DivByZero(1))?
                            .try_into()
                            .map_err(|_| MathError::CastOverflow(1))?;
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
            ensure!(
                balance_0 <= RESERVES_UPPER_BOUND && balance_1 <= RESERVES_UPPER_BOUND,
                PairError::ReservesOverflow
            );

            let now_seconds = u32::try_from(
                self.env()
                    .block_timestamp()
                    .checked_div(1000)
                    .unwrap_or_default()
                    % TWO_POW_32,
            )
            .unwrap(); // mod u32::MAX is guaranteed to not exceed 2^32-1

            // Wrapping subtraction so that the time_elapsed works correctly over the 2^32 boundary.
            // i.e. (1 - (2^32 - 1) = 2
            let time_elapsed = now_seconds.wrapping_sub(self.pair.block_timestamp_last);
            if time_elapsed > 0 && reserve_0 > 0 && reserve_1 > 0 {
                self.pair.price_0_cumulative_last = price_cumulative(
                    reserve_1,
                    reserve_0,
                    time_elapsed,
                    self.pair.price_0_cumulative_last,
                )?
                .into();
                self.pair.price_1_cumulative_last = price_cumulative(
                    reserve_0,
                    reserve_1,
                    time_elapsed,
                    self.pair.price_1_cumulative_last,
                )?
                .into();
            }
            self.pair.reserve_0 = balance_0;
            self.pair.reserve_1 = balance_1;
            self.pair.block_timestamp_last = now_seconds;

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
        fn get_reserves(&self) -> (u128, u128, u32) {
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

            let liquidity = if total_supply == 0 {
                let liq = casted_mul(amount_0_transferred, amount_1_transferred);
                let liquidity: u128 = u128::try_from(liq.integer_sqrt())
                    .map_err(|_| MathError::CastOverflow(2))?
                    .checked_sub(MINIMUM_LIQUIDITY)
                    .ok_or(MathError::SubUnderflow(4))?;
                let events = self.psp22.mint(BURN_ADDRESS.into(), MINIMUM_LIQUIDITY)?;
                self.emit_events(events);
                liquidity
            } else {
                let liquidity_0: u128 = casted_mul(amount_0_transferred, total_supply)
                    .checked_div(reserves.0.into())
                    .ok_or(MathError::DivByZero(2))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(3))?;

                let liquidity_1 = casted_mul(amount_1_transferred, total_supply)
                    .checked_div(reserves.1.into())
                    .ok_or(MathError::DivByZero(3))?
                    .try_into()
                    .map_err(|_| MathError::CastOverflow(4))?;

                liquidity_0.min(liquidity_1)
            };

            ensure!(liquidity > 0, PairError::InsufficientLiquidityMinted);

            let events = self.psp22.mint(to, liquidity)?;
            self.emit_events(events);

            self.update(balance_0, balance_1, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last =
                    Some(casted_mul(self.pair.reserve_0, self.pair.reserve_1).into());
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
            let total_supply = self.psp22.total_supply().into();
            let amount_0 = casted_mul(liquidity, balance_0_before)
                .checked_div(total_supply)
                .ok_or(MathError::DivByZero(4))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(5))?;
            let amount_1 = casted_mul(liquidity, balance_1_before)
                .checked_div(total_supply)
                .ok_or(MathError::DivByZero(5))?
                .try_into()
                .map_err(|_| MathError::CastOverflow(6))?;

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
            data: Option<Vec<u8>>,
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

            // Optimistically transfer tokens.
            if amount_0_out > 0 {
                self.token_0().transfer(to, amount_0_out, Vec::new())?;
            }
            if amount_1_out > 0 {
                self.token_1().transfer(to, amount_1_out, Vec::new())?;
            }

            if let Some(data) = data {
                // Call the callback.
                let mut swap_callee: contract_ref!(SwapCallee) = to.into();
                swap_callee.swap_call(self.env().caller(), amount_0_out, amount_1_out, data);
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
                .ok_or(MathError::MulOverflow(3))?
                .checked_sub(
                    amount_0_in
                        .checked_mul(TRADING_FEE_ADJ_AMOUNTS)
                        .ok_or(MathError::MulOverflow(4))?,
                )
                .ok_or(MathError::SubUnderflow(9))?;
            let balance_1_adjusted = balance_1
                .checked_mul(TRADING_FEE_ADJ_RESERVES)
                .ok_or(MathError::MulOverflow(5))?
                .checked_sub(
                    amount_1_in
                        .checked_mul(TRADING_FEE_ADJ_AMOUNTS)
                        .ok_or(MathError::MulOverflow(6))?,
                )
                .ok_or(MathError::SubUnderflow(10))?;

            // Cast to U256 to prevent Overflow
            ensure!(
                casted_mul(balance_0_adjusted, balance_1_adjusted)
                    >= casted_mul(reserves.0, reserves.1)
                        .checked_mul(TRADING_FEE_ADJ_RESERVES.pow(2).into())
                        .ok_or(MathError::MulOverflow(7))?,
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

    impl PSP22Metadata for PairContract {
        #[ink(message)]
        fn token_name(&self) -> Option<String> {
            Some("CommonAMM-V2".to_string())
        }

        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            Some("CMNAMM-V2".to_string())
        }

        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            12
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

    // Reserves are at most 2^112 - 1.
    // Consumer of the `price_cumulative_last` should use `overflowing_sub` to get the correct value.
    #[inline]
    fn price_cumulative(
        num: u128,
        denom: u128,
        time_elapsed_seconds: u32,
        last_price: WrappedU256,
    ) -> Result<U256, PairError> {
        // We use overflowing_add below to make the algorithm work correctly across the 2^256 boundary.
        Ok(
            UQ112x112::from_frac(num, denom).ok_or(PairError::ReservesOverflow)? // u224.div(u112) at most 2^224
            .saturating_mul(time_elapsed_seconds.into()) // so 2^224 * 2^32 never overflows 2^256.
            .overflowing_add(last_price.into())
            .0, // We don't care about the overflow flag, we just want the value.
        )
    }

    struct UQ112x112;

    impl UQ112x112 {
        fn from_frac(num: u128, denom: u128) -> Option<U256> {
            if num >= Q112 || denom >= Q112 {
                None
            } else {
                #[allow(clippy::arithmetic_side_effects)]
                Some(U256::from(num).checked_mul(Q112.into()).unwrap() / U256::from(denom))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn consts() {
            assert_eq!(Q112, 2u128.pow(112))
        }

        #[test]
        fn u112x112() {
            assert!(
                UQ112x112::from_frac(Q112, 1).is_none(),
                "Should not work with num >= Q112"
            );
            assert!(
                UQ112x112::from_frac(1, Q112).is_none(),
                "Should not work with denom >= Q112"
            );

            assert_eq!(
                UQ112x112::from_frac(1u128, 1u128).unwrap(),
                U256::from(Q112)
            );

            assert_eq!(
                UQ112x112::from_frac(1u128, 2u128).unwrap(),
                U256::from(Q112 / 2)
            );

            assert_eq!(
                UQ112x112::from_frac(Q112 - 1, Q112 - 1).unwrap(), // (n * (n-1)) / (n - 1) = n
                U256::from(Q112),
            );

            assert_eq!(
                UQ112x112::from_frac(Q112 - 1, 1).unwrap(),
                U256::from(2).pow(224.into()) - U256::from(Q112),
            );
        }

        #[ink::test]
        fn initialize_works() {
            let token_0 = AccountId::from([0x03; 32]);
            let token_1 = AccountId::from([0x04; 32]);
            let pair = PairContract::new(token_0, token_1);
            assert_eq!(pair.get_token_0(), token_0);
            assert_eq!(pair.get_token_1(), token_1);
        }

        #[ink::test]
        fn price_cumulative_from_zero_time_elapsed() {
            let cumulative = price_cumulative(1, 1, 0, 0.into()).unwrap();
            assert_eq!(cumulative, 0.into());
        }

        #[ink::test]
        fn price_cumulative_from_one_time_elapsed() {
            let cumulative = price_cumulative(1, 1, 1, 0.into()).unwrap();
            assert_eq!(cumulative, U256::from(Q112).into());
        }

        #[ink::test]
        fn price_cumulative_biggies() {
            assert_eq!(
                price_cumulative(
                    RESERVES_UPPER_BOUND,
                    RESERVES_UPPER_BOUND,
                    u32::MAX,
                    0.into(),
                )
                .unwrap(),
                U256::from(2).pow(144.into()) - U256::from(2).pow(112.into())
            );
            let max_cumulative_without_overflow =
                U256::MAX - U256::from(2).pow(144.into()) - U256::from(2).pow(224.into())
                    + Q112
                    + 1; // Add 1 since u256::MAX is 2^256-1
            assert_eq!(
                // max reserve 0, min reserve 1, max time elapsed.
                // [(2^112 - 1) * 2^112] / 1 * (2^32 - 1)
                price_cumulative(RESERVES_UPPER_BOUND, 1, u32::MAX, 0.into()).unwrap(),
                max_cumulative_without_overflow,
            );
            let new_cumulative_overflow = price_cumulative(
                RESERVES_UPPER_BOUND,
                1,
                u32::MAX,
                max_cumulative_without_overflow.into(),
            )
            .unwrap();
            assert!(
                new_cumulative_overflow < max_cumulative_without_overflow,
                "value after overflow should be lower"
            );
            let diff = U256::MAX - max_cumulative_without_overflow + new_cumulative_overflow;
            assert_eq!(diff + 1, max_cumulative_without_overflow); // +1 to account for overflow; u256::MAX = 2^256 - 1
        }
    }
}
