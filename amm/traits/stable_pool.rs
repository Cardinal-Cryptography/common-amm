use ink::prelude::vec::Vec;
use ink::primitives::AccountId;
use ink::LangError;
use psp22::PSP22Error;

use crate::{MathError, Ownable2StepError};

#[ink::trait_definition]
pub trait StablePool {
    /// Returns list of tokens in the pool.
    #[ink(message)]
    fn tokens(&self) -> Vec<AccountId>;

    /// Returns list of tokens reserves.
    #[ink(message)]
    fn reserves(&self) -> Vec<u128>;

    /// Returns current value of amplification coefficient.
    #[ink(message)]
    fn amp_coef(&self) -> Result<u128, StablePoolError>;

    /// Returns a tuple of the future amplification coefficient and the ramping end time.
    /// Returns `None` if the amplification coefficient is not in ramping period.
    #[ink(message)]
    fn future_amp_coef(&self) -> Option<(u128, u64)>;

    /// Returns current trade and protocol fees in 1e9 precision.
    #[ink(message)]
    fn fees(&self) -> (u32, u32);

    /// Protocol fees receiver (if any)
    #[ink(message)]
    fn fee_receiver(&self) -> Option<AccountId>;

    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns current tokens rates with precision of 12 decimal places.
    #[ink(message)]
    fn token_rates(&mut self) -> Vec<u128>;

    /// Returns list of RateProvider address for each token.
    /// If the rate is constant, returns None.
    #[ink(message)]
    fn token_rates_providers(&self) -> Vec<Option<AccountId>>;

    /// Calculate swap amount of `token_out`
    /// given `token_in amount`.
    ///
    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns a tuple of (amount out, fee)
    /// NOTE: fee is applied on `token_out`
    #[ink(message)]
    fn get_swap_amount_out(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate required swap amount of `token_in`
    /// to get `token_out_amount`.
    ///
    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns a tuple of (amount in, fee)
    /// NOTE: fee is applied on `token_out`
    #[ink(message)]
    fn get_swap_amount_in(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate how many lp tokens will be minted
    /// given deposit `amounts`.
    ///
    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns a tuple of (lpt amount, fee)
    #[ink(message)]
    fn get_mint_liquidity_for_amounts(
        &mut self,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate ideal deposit amounts required
    /// to mint `liquidity` amount of lp tokens
    ///
    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns required deposit amounts
    #[ink(message)]
    fn get_amounts_for_liquidity_mint(
        &mut self,
        liquidity: u128,
    ) -> Result<Vec<u128>, StablePoolError>;

    /// Calculate how many lp tokens will be burned
    /// given withdraw `amounts`.
    ///
    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns a tuple of (lpt amount, fee part)
    #[ink(message)]
    fn get_burn_liquidity_for_amounts(
        &mut self,
        amounts: Vec<u128>,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Calculate ideal withdraw amounts for
    /// burning `liquidity` amount of lp tokens
    ///
    /// Updates cached token rates if there was a new block since the previous update.
    ///
    /// Returns withdraw amounts
    #[ink(message)]
    fn get_amounts_for_liquidity_burn(
        &mut self,
        liquidity: u128,
    ) -> Result<Vec<u128>, StablePoolError>;

    /// Deposit `amounts` of tokens to receive lpt tokens to `to` account.
    /// Caller must allow enough spending allowance of underlying tokens
    /// for this contract.
    /// Returns an error if the minted LP tokens amount is less
    /// than `min_share_amount`.
    /// Returns a tuple of (minted lpt amount, fee)
    #[ink(message)]
    fn add_liquidity(
        &mut self,
        min_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Burns LP tokens and withdraws underlying tokens to `to` account
    /// in imbalanced `amounts`.
    /// Returns a tuple of (burned lpt amount, fee part)
    #[ink(message)]
    fn remove_liquidity_by_amounts(
        &mut self,
        max_share_amount: u128,
        amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Burns lp tokens and withdraws underlying tokens in balanced amounts to `to` account.
    /// Fails if any of the amounts received is less than in `min_amounts`.
    /// Returns withdrawal amounts
    #[ink(message)]
    fn remove_liquidity_by_shares(
        &mut self,
        shares: u128,
        min_amounts: Vec<u128>,
        to: AccountId,
    ) -> Result<Vec<u128>, StablePoolError>;

    /// Swaps token_in to token_out.
    /// Swapped tokens are transferred to the `to` account.
    /// caller account must allow enough spending allowance of `token_in`
    /// for this contract.
    /// Returns an error if swapped `token_out` amount is less than
    /// `min_token_out_amount`.
    /// NOTE: Fee is applied on `token_out`.
    /// Returns a tuple of (token out amount, fee amount)
    #[ink(message)]
    fn swap_exact_in(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_in_amount: u128,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Swaps token_in to token_out.
    /// Swapped tokens are transferred to the `to` account.
    /// Caller account must allow enough spending allowance of `token_in`
    /// for this contract.
    /// Returns an error if it is required to spend more than
    /// `max_token_in_amount`  to get `token_out_amount`.
    /// NOTE: Fee is applied on `token_out`.
    /// Returns a tuple of (token in amount, fee amount)
    #[ink(message)]
    fn swap_exact_out(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        token_out_amount: u128,
        max_token_in_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    /// Swaps excess reserve balance of `token_in` to `token_out`.
    /// Swapped tokens are transferred to the `to` account.
    /// Returns a tuple of (token out amount, fee amount)
    #[ink(message)]
    fn swap_received(
        &mut self,
        token_in: AccountId,
        token_out: AccountId,
        min_token_out_amount: u128,
        to: AccountId,
    ) -> Result<(u128, u128), StablePoolError>;

    // --- OWNER RESTRICTED FUNCTIONS --- //

    #[ink(message)]
    fn set_fee_receiver(&mut self, fee_receiver: Option<AccountId>) -> Result<(), StablePoolError>;

    /// Set fees
    /// - trade_fee given as an integer with 1e9 precision. The the maximum is 1% (10000000)
    /// - protocol_fee given as an integer with 1e9 precision. The maximum is 50% (500000000)
    #[ink(message)]
    fn set_fees(&mut self, trade_fee: u32, protocol_fee: u32) -> Result<(), StablePoolError>;

    /// Ramp amplification coeficient to `future_amp_coef`. The ramping should finish at `future_time`
    #[ink(message)]
    fn ramp_amp_coef(
        &mut self,
        future_amp_coef: u128,
        future_time: u64,
    ) -> Result<(), StablePoolError>;

    /// Stop ramping amplification coefficient.
    /// If ramping is not in progress, it does not influence the A.
    #[ink(message)]
    fn stop_ramp_amp_coef(&mut self) -> Result<(), StablePoolError>;
}

#[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum StablePoolError {
    Ownable2StepError(Ownable2StepError),
    MathError(MathError),
    PSP22Error(PSP22Error),
    LangError(LangError),
    InvalidTokenId(AccountId),
    IdenticalTokenId,
    IncorrectAmountsCount,
    ZeroAmounts,
    InsufficientLiquidityMinted,
    InsufficientLiquidityBurned,
    InsufficientOutputAmount,
    InsufficientLiquidity,
    InsufficientInputAmount,
    IncorrectTokenCount,
    TooLargeTokenDecimal,
    InvalidFee,
    AmpCoefTooLow,
    AmpCoefTooHigh,
    AmpCoefRampDurationTooShort,
    AmpCoefChangeTooLarge,
}

impl From<PSP22Error> for StablePoolError {
    fn from(error: PSP22Error) -> Self {
        StablePoolError::PSP22Error(error)
    }
}

impl From<LangError> for StablePoolError {
    fn from(error: LangError) -> Self {
        StablePoolError::LangError(error)
    }
}

impl From<MathError> for StablePoolError {
    fn from(error: MathError) -> Self {
        StablePoolError::MathError(error)
    }
}

impl From<Ownable2StepError> for StablePoolError {
    fn from(error: Ownable2StepError) -> Self {
        StablePoolError::Ownable2StepError(error)
    }
}
