use psp22::PSP22Error;

#[ink::trait_definition]
pub trait Wnative {
    /// Deposit NATIVE to wrap it
    #[ink(message, payable)]
    fn deposit(&mut self) -> Result<(), PSP22Error>;

    /// Unwrap NATIVE
    #[ink(message)]
    fn withdraw(&mut self, value: u128) -> Result<(), PSP22Error>;
}
