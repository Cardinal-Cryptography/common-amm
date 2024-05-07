use primitive_types::U256;

pub fn casted_mul(a: u128, b: u128) -> U256 {
    U256::from(a) * U256::from(b)
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum MathError {
    AddOverflow(u8),
    CastOverflow(u8),
    DivByZero(u8),
    MulOverflow(u8),
    SubUnderflow(u8),
}
