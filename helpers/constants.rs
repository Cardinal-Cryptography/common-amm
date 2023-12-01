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
