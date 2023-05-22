pub mod helper;
pub mod math;
pub mod transfer_helper;

/// Zero address for which the private key is unknown.
/// This is used for the feeTo and feeToSetter addresses in the factory contract a
/// and for sending MINIMUM_LIQUIDITY when minting tokens in Pair contract.
/// Result of sha512 hashing the ZERO_ADDERSS_MSG to RistrettoPoint.
pub const ZERO_ADDRESS: [u8; 32] = [
    58, 108, 115, 140, 64, 55, 232, 71, 183, 215, 14, 149, 138, 148, 201, 178, 212, 197, 99, 60,
    250, 175, 203, 88, 227, 37, 36, 127, 63, 212, 16, 72,
];

#[allow(unused)]
const ZERO_ADDRESS_MSG : &str = "This is Aleph Zero DEX's zero address.";

#[cfg(test)]
mod zero_address {
    use curve25519_dalek::ristretto::RistrettoPoint;
    use sha2::Sha512;

    #[test]
    fn test_zero_address() {
        let P = RistrettoPoint::hash_from_bytes::<Sha512>(ZERO_ADDRESS_MSG.as_bytes());
        let zero_address_address = P.compress();
        assert_eq!(super::ZERO_ADDRESS, zero_address_address.to_bytes());
    }
}
