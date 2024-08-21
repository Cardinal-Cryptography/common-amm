#[ink::trait_definition]
pub trait RateProvider {
    // Get "rate" of a particular token with respect to a given base token.
    // For instance, in the context of liquid staking, the base token could be the native token of the chain and the rate,
    // at a particular point of time would be the price of the yield bearing liquid staking token in terms of the base token.
    // The rate is supposed to have precision of RATE_DECIMALS=12 decimal places. So if the rate is 1.5, it should be represented as 1.5 * 10^12.
    // Note that the rate is expected to be a number relatively close to 1.0. More specifically, with the selected precision, the maximum
    // supported rate is of the order of 10^8, but in practice one would expect (get_rate() / 10^12) \in [0.001, 1000.0].
    #[ink(message)]
    fn get_rate(&mut self) -> u128;
}
