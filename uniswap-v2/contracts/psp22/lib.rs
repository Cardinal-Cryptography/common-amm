#![cfg_attr(not(feature = "std"), no_std, no_main)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod token {
    use ink::codegen::{
        EmitEvent,
        Env,
    };
    use openbrush::{
        contracts::psp22::extensions::metadata::*,
        traits::{
            Storage,
            String,
        },
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
    pub struct MyPSP22 {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        metadata: metadata::Data,
    }

    impl psp22::Internal for MyPSP22 {
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

    impl MyPSP22 {
        #[ink(constructor)]
        pub fn new(
            total_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimals: u8,
        ) -> Self {
            let mut instance = Self::default();
            instance.metadata.name = name;
            instance.metadata.symbol = symbol;
            instance.metadata.decimals = decimals;
            instance
                ._mint_to(instance.env().caller(), total_supply)
                .expect("Should mint");
            instance
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn initialize_works() {
            let name = "TOKEN_A".to_string();
            let symbol = "TKNA".to_string();
            let decimals = 18;
            let token = MyPSP22::new(
                1_000_000_000,
                Some(name.clone()),
                Some(symbol.clone()),
                decimals,
            );
            assert_eq!(token.metadata.name.unwrap(), name);
            assert_eq!(token.metadata.symbol.unwrap(), symbol);
            assert_eq!(token.metadata.decimals, decimals);
        }
    }
}
