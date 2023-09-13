#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod pair {
    use amm::{
        ensure,
        helpers::{
            transfer_helper::safe_transfer,
            ZERO_ADDRESS,
        },
        impls::pair::{
            pair::{
                Internal,
                MINIMUM_LIQUIDITY,
            },
            *,
        },
        traits::pair::*,
    };
    use amm_helpers::{
        math::casted_mul,
        types::WrappedU256,
    };
    use ink::{
        codegen::{
            EmitEvent,
            Env,
        },
        prelude::vec::Vec,
    };
    use openbrush::{
        contracts::{
            psp22::*,
            reentrancy_guard::*,
        },
        modifiers,
        traits::Storage,
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
        value: Balance,
    }

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct PairContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        guard: reentrancy_guard::Data,
        #[storage_field]
        pair: data::Data,
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

        #[modifiers(non_reentrant)]
        #[ink(message)]
        fn mint(&mut self, to: AccountId) -> Result<Balance, PairError> {
            let reserves = self.get_reserves();
            let contract = self.env().account_id();
            let balance_0 = PSP22Ref::balance_of(&self.pair.token_0, contract);
            let balance_1 = PSP22Ref::balance_of(&self.pair.token_1, contract);
            let amount_0_transferred = balance_0
                .checked_sub(reserves.0)
                .ok_or(PairError::SubUnderFlow1)?;
            let amount_1_transferred = balance_1
                .checked_sub(reserves.1)
                .ok_or(PairError::SubUnderFlow2)?;

            let fee_on = self._mint_fee(reserves.0, reserves.1)?;
            let total_supply = self.psp22.supply;

            let liquidity;
            if total_supply == 0 {
                let liq = amount_0_transferred
                    .checked_mul(amount_1_transferred)
                    .ok_or(PairError::MulOverFlow1)?;
                liquidity = liq
                    .integer_sqrt()
                    .checked_sub(MINIMUM_LIQUIDITY)
                    .ok_or(PairError::SubUnderFlow3)?;
                self._mint_to(ZERO_ADDRESS.into(), MINIMUM_LIQUIDITY)?;
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
                liquidity = min(liquidity_1, liquidity_2);
            }

            ensure!(liquidity > 0, PairError::InsufficientLiquidityMinted);

            self._mint_to(to, liquidity)?;

            self._update(balance_0, balance_1, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last = casted_mul(reserves.0, reserves.1).into();
            }

            self._emit_mint_event(
                self.env().caller(),
                amount_0_transferred,
                amount_1_transferred,
            );

            Ok(liquidity)
        }

        #[modifiers(non_reentrant)]
        #[ink(message)]
        fn burn(&mut self, to: AccountId) -> Result<(Balance, Balance), PairError> {
            let reserves = self.get_reserves();
            let contract = self.env().account_id();
            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;
            let balance_0_before = PSP22Ref::balance_of(&token_0, contract);
            let balance_1_before = PSP22Ref::balance_of(&token_1, contract);
            let liquidity = self._balance_of(&contract);

            let fee_on = self._mint_fee(reserves.0, reserves.1)?;
            let total_supply = self.psp22.supply;
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

            self._burn_from(contract, liquidity)?;

            safe_transfer(token_0, to, amount_0)?;
            safe_transfer(token_1, to, amount_1)?;

            let balance_0_after = PSP22Ref::balance_of(&token_0, contract);
            let balance_1_after = PSP22Ref::balance_of(&token_1, contract);

            self._update(balance_0_after, balance_1_after, reserves.0, reserves.1)?;

            if fee_on {
                self.pair.k_last = casted_mul(reserves.0, reserves.1).into();
            }

            self._emit_burn_event(self.env().caller(), amount_0, amount_1, to);

            Ok((amount_0, amount_1))
        }

        #[modifiers(non_reentrant)]
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
            let balance_0 = PSP22Ref::balance_of(&token_0, contract);
            let balance_1 = PSP22Ref::balance_of(&token_1, contract);

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

            self._update(balance_0, balance_1, reserves.0, reserves.1)?;

            self._emit_swap_event(
                self.env().caller(),
                amount_0_in,
                amount_1_in,
                amount_0_out,
                amount_1_out,
                to,
            );
            Ok(())
        }

        #[modifiers(non_reentrant)]
        #[ink(message)]
        fn skim(&mut self, to: AccountId) -> Result<(), PairError> {
            let contract = self.env().account_id();
            let reserve_0 = self.pair.reserve_0;
            let reserve_1 = self.pair.reserve_1;
            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;
            let balance_0 = PSP22Ref::balance_of(&token_0, contract);
            let balance_1 = PSP22Ref::balance_of(&token_1, contract);
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

        #[modifiers(non_reentrant)]
        #[ink(message)]
        fn sync(&mut self) -> Result<(), PairError> {
            let contract = self.env().account_id();
            let reserve_0 = self.pair.reserve_0;
            let reserve_1 = self.pair.reserve_1;
            let token_0 = self.pair.token_0;
            let token_1 = self.pair.token_1;
            let balance_0 = PSP22Ref::balance_of(&token_0, contract);
            let balance_1 = PSP22Ref::balance_of(&token_1, contract);
            self._update(balance_0, balance_1, reserve_0, reserve_1)
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

    fn min(x: u128, y: u128) -> u128 {
        if x < y {
            return x
        }
        y
    }

    impl PSP22 for PairContract {
        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let allowance = self._allowance(&from, &caller);

            // In uniswapv2 max allowance never decrease
            if allowance != u128::MAX {
                ensure!(allowance >= value, PSP22Error::InsufficientAllowance);
                self._approve_from_to(from, caller, allowance - value)?;
            }
            self._transfer_from_to(from, to, value, data)?;
            Ok(())
        }
    }

    impl psp22::Internal for PairContract {
        // in uniswapv2 no check for zero account
        fn _mint_to(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut new_balance = self._balance_of(&account);
            new_balance += amount;
            self.psp22.balances.insert(&account, &new_balance);
            self.psp22.supply += amount;
            self._emit_transfer_event(None, Some(account), amount);
            Ok(())
        }

        fn _burn_from(&mut self, account: AccountId, amount: Balance) -> Result<(), PSP22Error> {
            let mut from_balance = self._balance_of(&account);

            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            from_balance -= amount;
            self.psp22.balances.insert(&account, &from_balance);
            self.psp22.supply -= amount;
            self._emit_transfer_event(Some(account), None, amount);
            Ok(())
        }

        fn _approve_from_to(
            &mut self,
            owner: AccountId,
            spender: AccountId,
            amount: Balance,
        ) -> Result<(), PSP22Error> {
            self.psp22.allowances.insert(&(&owner, &spender), &amount);
            self._emit_approval_event(owner, spender, amount);
            Ok(())
        }

        fn _transfer_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            amount: Balance,
            _data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from_balance = self._balance_of(&from);

            ensure!(from_balance >= amount, PSP22Error::InsufficientBalance);

            self.psp22.balances.insert(&from, &(from_balance - amount));
            let to_balance = self._balance_of(&to);
            self.psp22.balances.insert(&to, &(to_balance + amount));

            self._emit_transfer_event(Some(from), Some(to), amount);
            Ok(())
        }

        fn _emit_approval_event(&self, owner: AccountId, spender: AccountId, amount: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
        }

        fn _emit_transfer_event(
            &self,
            from: Option<AccountId>,
            to: Option<AccountId>,
            amount: Balance,
        ) {
            self.env().emit_event(Transfer {
                from,
                to,
                value: amount,
            });
        }
    }

    impl pair::Internal for PairContract {
        fn _emit_mint_event(&self, sender: AccountId, amount_0: Balance, amount_1: Balance) {
            self.env().emit_event(Mint {
                sender,
                amount_0,
                amount_1,
            })
        }

        fn _emit_burn_event(
            &self,
            sender: AccountId,
            amount_0: Balance,
            amount_1: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Burn {
                sender,
                amount_0,
                amount_1,
                to,
            })
        }

        fn _emit_swap_event(
            &self,
            sender: AccountId,
            amount_0_in: Balance,
            amount_1_in: Balance,
            amount_0_out: Balance,
            amount_1_out: Balance,
            to: AccountId,
        ) {
            self.env().emit_event(Swap {
                sender,
                amount_0_in,
                amount_1_in,
                amount_0_out,
                amount_1_out,
                to,
            })
        }

        fn _emit_sync_event(&self, reserve_0: Balance, reserve_1: Balance) {
            self.env().emit_event(Sync {
                reserve_0,
                reserve_1,
            })
        }
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
    }
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn initialize_works() {
            let token_0 = AccountId::from([0x03; 32]);
            let token_1 = AccountId::from([0x04; 32]);
            let pair = PairContract::new(token_0, token_1);
            assert_eq!(pair.get_token_0(), token_0);
            assert_eq!(pair.get_token_1(), token_1);
        }
    }
}
