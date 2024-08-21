/// Address for which the private key is unknown.
/// This is used for sending MINIMUM_LIQUIDITY when minting tokens in Pair contract.
/// Result of sha512 hashing the ZERO_ADDERSS_MSG to curve (curve25519).
pub const BURN_ADDRESS: [u8; 32] = [
    58, 108, 115, 140, 64, 55, 232, 71, 183, 215, 14, 149, 138, 148, 201, 178, 212, 197, 99, 60,
    250, 175, 203, 88, 227, 37, 36, 127, 63, 212, 16, 72,
];

#[allow(unused)]
const BURN_ADDRESS_MSG: &str = "This is Aleph Zero DEX's burn address.";

/// Minimum liquidity threshold that is subtracted
/// from the minted liquidity and sent to the `BURN_ADDRESS`.
/// Prevents price manipulation and saturation.
/// See UniswapV2 whitepaper for more details.
/// NOTE: This value is taken from UniswapV2 whitepaper and is correct
/// only for liquidity tokens with precision = 18.
pub const MINIMUM_LIQUIDITY: u128 = 1000;

#[cfg(test)]
mod burn_address {
    use curve25519_dalek::ristretto::RistrettoPoint;
    use sha2::Sha512;

    use super::BURN_ADDRESS_MSG;

    #[test]
    fn test_burn_address() {
        let p = RistrettoPoint::hash_from_bytes::<Sha512>(BURN_ADDRESS_MSG.as_bytes());
        let burn_address = p.compress();
        assert_eq!(super::BURN_ADDRESS, burn_address.to_bytes());
    }
}

pub mod stable_pool {
    // Token amounts are rescaled so as if they have TOKEN_TARGET_DECIMALS decimal places.
    pub const TOKEN_TARGET_DECIMALS: u8 = 18;
    pub const TOKEN_TARGET_PRECISION: u128 = 10u128.pow(TOKEN_TARGET_DECIMALS as u32);

    // Precision for rate values. If the rate is 1.2, the rate provider should return 1.2 * RATE_PRECISION.
    pub const RATE_DECIMALS: u8 = 12;
    pub const RATE_PRECISION: u128 = 10u128.pow(RATE_DECIMALS as u32);

    /// Given as an integer with 1e9 precision (1%)
    pub const MAX_TRADE_FEE: u32 = 10_000_000;
    /// Given as an integer with 1e9 precision (50%)
    ///
    /// It is taken as part of the trade fee thus,
    /// a maximum 50% of 1% goes to the protocol (0.5% of the trade)
    pub const MAX_PROTOCOL_FEE: u32 = 500_000_000;
    /// Fee denominator
    pub const FEE_DENOM: u32 = 1_000_000_000;

    /// Maximum number coins (PSP22 token contracts) in the pool.
    pub const MAX_COINS: usize = 8;

    /// Minimum ramp duration, in milisec (24h).
    pub const MIN_RAMP_DURATION: u64 = 86400000;
    /// Min amplification coefficient.
    pub const MIN_AMP: u128 = 1;
    /// Max amplification coefficient.
    pub const MAX_AMP: u128 = 1_000_000;
    /// Max amplification change (how many times it can increase/decrease compared to current value).
    pub const MAX_AMP_CHANGE: u128 = 10;
}
