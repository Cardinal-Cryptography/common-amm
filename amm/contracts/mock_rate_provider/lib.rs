#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod mock_rate_provider {
    #[ink(storage)]
    pub struct MockRateProviderContract {
        rate: u128,
    }

    impl MockRateProviderContract {
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                rate: 10u128.pow(12u32),
            }
        }

        #[ink(message)]
        pub fn set_rate(&mut self, rate: u128) {
            self.rate = rate;
        }
    }

    impl traits::RateProvider for MockRateProviderContract {
        #[ink(message)]
        fn get_rate(&mut self) -> u128 {
            self.rate
        }
    }
}
