#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod wnative {
    use ink::{
        codegen::{
            EmitEvent,
            Env,
        },
        prelude::vec::Vec,
    };
    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            Storage,
            String,
        },
    };
    use uniswap_v2::traits::wnative::{
        wnative_external,
        Wnative,
    };

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
    pub struct WnativeContract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl Wnative for WnativeContract {
        #[ink(message)]
        fn deposit(&mut self) -> Result<(), PSP22Error> {
            let transfer_value = self.env().transferred_value();
            let caller = self.env().caller();
            self._mint_to(caller, transfer_value)
        }

        #[ink(message)]
        fn withdraw(&mut self, amount: Balance) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            self._burn_from(caller, amount)?;
            self.env()
                .transfer(caller, amount)
                .map_err(|_| PSP22Error::Custom(String::from("WNATIVE: transfer failed")))
        }
    }

    impl PSP22Metadata for WnativeContract {
        #[ink(message)]
        fn token_name(&self) -> Option<String> {
            self.metadata.name.clone()
        }

        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            self.metadata.symbol.clone()
        }

        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            self.metadata.decimals
        }
    }

    impl PSP22 for WnativeContract {
        #[ink(message)]
        fn total_supply(&self) -> Balance {
            self._total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> Balance {
            self._balance_of(&owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self._allowance(&owner, &spender)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: Balance,
            data: Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let from = self.env().caller();
            self._transfer_from_to(from, to, value, data)?;
            Ok(())
        }

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

            if allowance < value {
                return Err(PSP22Error::InsufficientAllowance)
            }

            self._approve_from_to(from, caller, allowance - value)?;
            self._transfer_from_to(from, to, value, data)?;
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            self._approve_from_to(owner, spender, value)?;
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: Balance,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            self._approve_from_to(
                owner,
                spender,
                self._allowance(&owner, &spender) + delta_value,
            )
        }

        #[ink(message)]
        fn decrease_allowance(
            &mut self,
            spender: AccountId,
            delta_value: Balance,
        ) -> Result<(), PSP22Error> {
            let owner = self.env().caller();
            let allowance = self._allowance(&owner, &spender);

            if allowance < delta_value {
                return Err(PSP22Error::InsufficientAllowance)
            }

            self._approve_from_to(owner, spender, allowance - delta_value)
        }
    }

    impl psp22::Internal for WnativeContract {
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

        fn _emit_approval_event(&self, owner: AccountId, spender: AccountId, amount: Balance) {
            self.env().emit_event(Approval {
                owner,
                spender,
                value: amount,
            });
        }
    }

    impl WnativeContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            let mut instance = Self::default();
            instance.metadata.name = Some(String::from("Wrapped Native"));
            instance.metadata.symbol = Some(String::from("WNATIVE"));
            instance.metadata.decimals = 12;
            instance
        }
        /// For e2e testing purposes only. Do not use in production!
        #[cfg(feature = "e2e-tests")]
        #[ink(message)]
        pub fn terminate(&mut self) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            self.env().terminate_contract(caller)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn register_works() {
            let wnative_contract = WnativeContract::new();
            assert_eq!(
                wnative_contract.metadata.name,
                Some(String::from("Wrapped Native"))
            );
            assert_eq!(
                wnative_contract.metadata.symbol,
                Some(String::from("WNATIVE"))
            );
        }

        #[ink::test]
        fn test_deposit() {
            let accounts = default_accounts();
            let mut wnative_contract = create_contract(0);
            assert_eq!(deposit(&mut wnative_contract, 1000), Ok(()));
            let balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(balance, 1000, "balance not correct!");
            let native_balance: Balance = wnative_contract.env().balance();
            assert_eq!(native_balance, 1000, "native balance not correct!");
        }

        #[ink::test]
        fn test_withdraw() {
            let accounts = default_accounts();
            let mut wnative_contract = create_contract(1000);
            assert_eq!(get_balance(wnative_contract.env().account_id()), 1000);
            assert_eq!(
                wnative_contract._mint_to(accounts.alice, 1000),
                Ok(()),
                "mint failed"
            );
            let wnative_balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(wnative_balance, 1000, "balance not correct!");

            let before_balance = get_balance(accounts.alice);
            assert_eq!(wnative_contract.withdraw(800), Ok(()));
            assert_eq!(
                get_balance(accounts.alice),
                800 + before_balance,
                "withdraw should refund native token"
            );
            let wnative_balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(wnative_balance, 200, "balance not correct!");
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
            ink::env::test::default_accounts()
        }

        fn set_next_caller(caller: AccountId) {
            ink::env::test::set_caller::<Environment>(caller);
        }

        fn set_balance(account_id: AccountId, balance: Balance) {
            ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(account_id, balance)
        }

        /// Creates a new instance of `WnativeContract` with `initial_balance`.
        ///
        /// Returns the `contract_instance`.
        fn create_contract(initial_balance: Balance) -> WnativeContract {
            let accounts = default_accounts();
            set_next_caller(accounts.alice);
            set_balance(contract_id(), initial_balance);
            WnativeContract::new()
        }

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
        }

        fn deposit(contract: &mut WnativeContract, amount: Balance) -> Result<(), PSP22Error> {
            let sender = ink::env::caller::<ink::env::DefaultEnvironment>();
            let contract_id = contract.env().account_id();
            let sender_balance = get_balance(sender);
            let contract_balance = get_balance(contract_id);
            // ↓ doesn't work, is upstream issue: https://github.com/paritytech/ink/issues/1117
            // set_balance(sender, sender_balance - amount);
            set_balance(
                sender,
                if sender_balance > amount {
                    sender_balance - amount
                } else {
                    0
                },
            );
            set_balance(contract_id, contract_balance + amount);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(amount);
            contract.deposit()
        }
    }
}
