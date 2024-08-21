use ink::{contract_ref, env::DefaultEnvironment, primitives::AccountId};
use scale::{Decode, Encode};
use traits::RateProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct ExternalTokenRate {
    rate_provider: AccountId,
    cached_token_rate: u128,
    last_update_block_no: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum TokenRate {
    Constant(u128),
    External(ExternalTokenRate),
}

impl TokenRate {
    pub fn new_constant(rate: u128) -> Self {
        Self::Constant(rate)
    }

    pub fn new_external(rate_provider: AccountId) -> Self {
        Self::External(ExternalTokenRate::new(rate_provider))
    }

    /// Get current rate and update the cache.
    pub fn get_rate(&mut self) -> u128 {
        match self {
            Self::External(external) => external.get_rate_update(),
            Self::Constant(rate) => *rate,
        }
    }

    pub fn get_rate_provider(&self) -> Option<AccountId> {
        match self {
            Self::External(external) => Some(external.rate_provider),
            Self::Constant(_) => None,
        }
    }
}

impl ExternalTokenRate {
    pub fn new(rate_provider: AccountId) -> Self {
        Self {
            rate_provider,
            cached_token_rate: 0,
            last_update_block_no: 0,
        }
    }

    pub fn get_rate_update(&mut self) -> u128 {
        let current_block_no = ink::env::block_number::<DefaultEnvironment>();
        if self.last_update_block_no < current_block_no {
            self.cached_token_rate = self.query_rate();
            self.last_update_block_no = current_block_no;
        }
        self.cached_token_rate
    }

    fn query_rate(&self) -> u128 {
        let mut rate_provider: contract_ref!(RateProvider, DefaultEnvironment) =
            self.rate_provider.into();
        rate_provider.get_rate()
    }
}
