#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
pub mod wnative {
    use ink::prelude::{
        string::String,
        vec::Vec,
    };
    use psp22::{
        PSP22Data,
        PSP22Error,
        PSP22Event,
        PSP22Metadata,
        PSP22,
    };
    use traits::Wnative;

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        amount: u128,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: u128,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct WnativeContract {
        data: PSP22Data,
    }

    impl WnativeContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
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

        /// For e2e testing purposes only. Do not use in production!
        #[cfg(feature = "e2e-tests")]
        #[ink(message)]
        pub fn terminate(&mut self) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            self.env().terminate_contract(caller)
        }
    }

    impl Wnative for WnativeContract {
        #[ink(message)]
        fn deposit(&mut self) -> Result<(), PSP22Error> {
            let events = self
                .data
                .mint(self.env().caller(), self.env().transferred_value())?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn withdraw(&mut self, value: u128) -> Result<(), PSP22Error> {
            let caller = self.env().caller();
            let events = self.data.burn(caller, value)?;
            self.env()
                .transfer(caller, value)
                .map_err(|_| PSP22Error::Custom(String::from("Wrapper AZERO: withdraw failed")))?;
            self.emit_events(events);
            Ok(())
        }
    }

    impl PSP22Metadata for WnativeContract {
        #[ink(message)]
        fn token_name(&self) -> Option<String> {
            Some(String::from("Wrapped AZERO"))
        }

        #[ink(message)]
        fn token_symbol(&self) -> Option<String> {
            Some(String::from("WAZERO"))
        }

        #[ink(message)]
        fn token_decimals(&self) -> u8 {
            12
        }
    }

    impl PSP22 for WnativeContract {
        #[ink(message)]
        fn total_supply(&self) -> u128 {
            self.data.total_supply()
        }

        #[ink(message)]
        fn balance_of(&self, owner: AccountId) -> u128 {
            self.data.balance_of(owner)
        }

        #[ink(message)]
        fn allowance(&self, owner: AccountId, spender: AccountId) -> u128 {
            self.data.allowance(owner, spender)
        }

        #[ink(message)]
        fn transfer(
            &mut self,
            to: AccountId,
            value: u128,
            _data: ink::prelude::vec::Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self.data.transfer(self.env().caller(), to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: u128,
            _data: ink::prelude::vec::Vec<u8>,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
                .transfer_from(self.env().caller(), from, to, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn approve(&mut self, spender: AccountId, value: u128) -> Result<(), PSP22Error> {
            let events = self.data.approve(self.env().caller(), spender, value)?;
            self.emit_events(events);
            Ok(())
        }

        #[ink(message)]
        fn increase_allowance(
            &mut self,
            spender: AccountId,
            delta_value: u128,
        ) -> Result<(), PSP22Error> {
            let events = self
                .data
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
            let events = self
                .data
                .decrease_allowance(self.env().caller(), spender, delta_value)?;
            self.emit_events(events);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn register_works() {
            let wnative_contract = WnativeContract::new();
            assert_eq!(
                wnative_contract.token_name(),
                Some(String::from("Wrapped AZERO"))
            );
            assert_eq!(
                wnative_contract.token_symbol(),
                Some(String::from("WAZERO"))
            );
        }

        #[ink::test]
        fn test_deposit() {
            let accounts = default_accounts();
            let mut wnative_contract = create_contract(0);
            assert_eq!(deposit(&mut wnative_contract, 1000), Ok(()));
            let balance = wnative_contract.balance_of(accounts.alice);
            assert_eq!(balance, 1000, "balance not correct!");
            let native_balance: Balance = get_balance(contract_id());
            assert_eq!(native_balance, 1000, "native balance not correct!");
        }

        #[ink::test]
        fn test_withdraw() {
            let accounts = default_accounts();
            let mut wnative_contract = create_contract(1000);
            assert_eq!(get_balance(contract_id()), 1000);
            assert!(
                wnative_contract.data.mint(accounts.alice, 1000).is_ok(),
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

        fn contract_id() -> AccountId {
            ink::env::test::callee::<ink::env::DefaultEnvironment>()
        }

        fn get_balance(account_id: AccountId) -> Balance {
            ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(account_id)
                .expect("Cannot get account balance")
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

        fn deposit(contract: &mut WnativeContract, amount: Balance) -> Result<(), PSP22Error> {
            let sender = ink::env::caller::<ink::env::DefaultEnvironment>();
            let contract_id = contract_id();
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
