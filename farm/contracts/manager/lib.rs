#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod dummy {

    #[derive(Default)]
    #[ink(storage)]
    pub struct Dummy {}

    impl Dummy {
        #[ink(constructor)]
        pub fn new() -> Self {
            Dummy::default()
        }

        #[ink(message)]
        pub fn dummy(&self) {}
    }
}
