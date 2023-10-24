#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub mod factory;
pub mod pair;
pub mod router;
pub mod wnative;

pub use ink::env::DefaultEnvironment as Env;
pub type Balance = <Env as ink::env::Environment>::Balance;
pub type Timestamp = <Env as ink::env::Environment>::Timestamp;

/// Zero address for which the private key is unknown.
/// This is used for the feeTo and feeToSetter addresses in the factory contract a
/// and for sending MINIMUM_LIQUIDITY when minting tokens in Pair contract.
/// Result of sha512 hashing the ZERO_ADDERSS_MSG to curve (curve25519).
pub const ZERO_ADDRESS: [u8; 32] = [
    58, 108, 115, 140, 64, 55, 232, 71, 183, 215, 14, 149, 138, 148, 201, 178, 212, 197, 99, 60,
    250, 175, 203, 88, 227, 37, 36, 127, 63, 212, 16, 72,
];

#[allow(unused)]
const ZERO_ADDRESS_MSG: &str = "This is Aleph Zero DEX's zero address.";

/// Minimum liquidity threshold that is subtracted
/// from the minted liquidity and sent to the `ZERO_ADDRESS`.
/// Prevents price manipulation and saturation.
/// See UniswapV2 whitepaper for more details.
/// NOTE: This value is taken from UniswapV2 whitepaper and is correct
/// only for liquidity tokens with precision = 18.
pub const MINIMUM_LIQUIDITY: u128 = 1000;

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr $(,)? ) => {{
        if !$x {
            return Err($y.into())
        }
    }};
}

#[cfg(test)]
mod zero_address {
    use curve25519_dalek::ristretto::RistrettoPoint;
    use sha2::Sha512;

    use super::ZERO_ADDRESS_MSG;

    #[test]
    fn test_zero_address() {
        let p = RistrettoPoint::hash_from_bytes::<Sha512>(ZERO_ADDRESS_MSG.as_bytes());
        let zero_address_address = p.compress();
        assert_eq!(super::ZERO_ADDRESS, zero_address_address.to_bytes());
    }
}
