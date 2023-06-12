use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    38, 30, 23, 90, 132, 176, 35, 198, 229, 58, 80, 203, 247, 177, 238, 88, 0, 7, 99, 156, 120,
    149, 86, 7, 204, 200, 20, 194, 236, 40, 2, 4,
];

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum RouterError {
    PSP22Error(PSP22Error),
    FactoryError(FactoryError),
    PairError(PairError),
    HelperError(HelperError),
    TransferHelperError(TransferHelperError),
    LangError(ink_wrapper_types::InkLangError),
    TransferError(),
    PairNotFound(),
    InsufficientAmount(),
    InsufficientAAmount(),
    InsufficientOutputAmount(),
    ExcessiveInputAmount(),
    InsufficientBAmount(),
    InsufficientLiquidity(),
    ZeroAddress(),
    IdenticalAddresses(),
    Expired(),
    SubUnderFlow(),
    MulOverFlow(),
    DivByZero(),
    TransferFailed(),
    InvalidPath(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum PSP22Error {
    Custom(String),
    InsufficientBalance(),
    InsufficientAllowance(),
    ZeroRecipientAddress(),
    ZeroSenderAddress(),
    SafeTransferCheckFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum FactoryError {
    PairError(PairError),
    CallerIsNotFeeSetter(),
    ZeroAddress(),
    IdenticalAddresses(),
    PairExists(),
    PairInstantiationFailed(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum PairError {
    PSP22Error(PSP22Error),
    OwnableError(OwnableError),
    ReentrancyGuardError(ReentrancyGuardError),
    LangError(ink_wrapper_types::InkLangError),
    TransferError(),
    K(),
    InsufficientLiquidityMinted(),
    InsufficientLiquidityBurned(),
    InsufficientOutputAmount(),
    InsufficientLiquidity(),
    InsufficientInputAmount(),
    SafeTransferFailed(),
    InvalidTo(),
    Overflow(),
    Locked(),
    SubUnderFlow1(),
    SubUnderFlow2(),
    SubUnderFlow3(),
    SubUnderFlow4(),
    SubUnderFlow5(),
    SubUnderFlow6(),
    SubUnderFlow7(),
    SubUnderFlow8(),
    SubUnderFlow9(),
    SubUnderFlow10(),
    SubUnderFlow11(),
    SubUnderFlow12(),
    SubUnderFlow13(),
    SubUnderFlow14(),
    MulOverFlow1(),
    MulOverFlow2(),
    MulOverFlow3(),
    MulOverFlow4(),
    MulOverFlow5(),
    MulOverFlow6(),
    MulOverFlow7(),
    MulOverFlow8(),
    MulOverFlow9(),
    MulOverFlow10(),
    MulOverFlow11(),
    MulOverFlow12(),
    MulOverFlow13(),
    MulOverFlow14(),
    DivByZero1(),
    DivByZero2(),
    DivByZero3(),
    DivByZero4(),
    DivByZero5(),
    AddOverflow1(),
    CastOverflow1(),
    CastOverflow2(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum OwnableError {
    CallerIsNotOwner(),
    NewOwnerIsZero(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum ReentrancyGuardError {
    ReentrantCall(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum HelperError {
    IdenticalAddresses(),
    ZeroAddress(),
    InsufficientAmount(),
    InsufficientLiquidity(),
    DivByZero(),
    CastOverflow(),
    MulOverFlow(),
    AddOverFlow(),
    DivByZero2(),
    CastOverflow2(),
    InvalidPath(),
    SubUnderFlow(),
    PairNotFound(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum TransferHelperError {
    TransferFailed(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum NoChainExtension {}

pub mod event {
    #[allow(dead_code, clippy::large_enum_variant)]
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub enum Event {}
}

#[derive(Debug, Clone, Copy)]
pub struct Instance {
    account_id: ink_primitives::AccountId,
}

impl From<ink_primitives::AccountId> for Instance {
    fn from(account_id: ink_primitives::AccountId) -> Self {
        Self { account_id }
    }
}

impl From<Instance> for ink_primitives::AccountId {
    fn from(instance: Instance) -> Self {
        instance.account_id
    }
}

impl ink_wrapper_types::EventSource for Instance {
    type Event = event::Event;
}

#[async_trait::async_trait]
pub trait Router {
    async fn swap_tokens_for_exact_native<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn add_liquidity<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn swap_exact_tokens_for_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn swap_tokens_for_exact_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn swap_native_for_exact_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn remove_liquidity<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
        liquidity: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn factory<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
    async fn remove_liquidity_native<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token: ink_primitives::AccountId,
        liquidity: u128,
        amount_token_min: u128,
        amount_native_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn add_liquidity_native<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token: ink_primitives::AccountId,
        amount_token_desired: u128,
        amount_token_min: u128,
        amount_native_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn swap_exact_tokens_for_native<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn quote<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_a: u128,
        reserve_a: u128,
        reserve_b: u128,
    ) -> Result<Result<Result<u128, RouterError>, ink_wrapper_types::InkLangError>, E>;
    async fn get_amount_out<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_in: u128,
        reserve_in: u128,
        reserve_out: u128,
    ) -> Result<Result<Result<u128, RouterError>, ink_wrapper_types::InkLangError>, E>;
    async fn get_amount_in<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_out: u128,
        reserve_in: u128,
        reserve_out: u128,
    ) -> Result<Result<Result<u128, RouterError>, ink_wrapper_types::InkLangError>, E>;
    async fn get_amounts_in<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_out: u128,
        path: Vec<ink_primitives::AccountId>,
    ) -> Result<Result<Result<Vec<u128>, RouterError>, ink_wrapper_types::InkLangError>, E>;
    async fn wnative<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
    async fn swap_exact_native_for_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out_min: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E>;
    async fn get_amounts_out<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_in: u128,
        path: Vec<ink_primitives::AccountId>,
    ) -> Result<Result<Result<Vec<u128>, RouterError>, ink_wrapper_types::InkLangError>, E>;
}

#[async_trait::async_trait]
impl Router for Instance {
    ///  Exchanges tokens along `path` token pairs
    ///  so that at the end caller receives `amount_out`
    ///  worth of native tokens and pays no more than `amount_in_max`
    ///  of the starting token. Fails if any of these conditions
    ///  is not satisfied.
    ///  Transfers tokens to account under `to` address.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap_tokens_for_exact_native<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![178, 178, 143, 146];
            amount_out.encode_to(&mut data);
            amount_in_max.encode_to(&mut data);
            path.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Adds liquidity to `(token_a, token_b)` pair.
    ///
    ///  Will add at least `*_min` amount of tokens and up to `*_desired`
    ///  while still maintaining the constant `k` product of the pair.
    ///
    ///  If succesful, liquidity tokens will be minted for `to` account.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn add_liquidity<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![165, 183, 213, 151];
            token_a.encode_to(&mut data);
            token_b.encode_to(&mut data);
            amount_a_desired.encode_to(&mut data);
            amount_b_desired.encode_to(&mut data);
            amount_a_min.encode_to(&mut data);
            amount_b_min.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Exchanges tokens along `path` tokens.
    ///  Starts with `amount_in` and pair under `(path[0], path[1])` address.
    ///  Fails if output amount is less than `amount_out_min`.
    ///  Transfers tokens to account under `to` address.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap_exact_tokens_for_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![175, 10, 136, 54];
            amount_in.encode_to(&mut data);
            amount_out_min.encode_to(&mut data);
            path.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Exchanges tokens along `path` token pairs
    ///  so that at the end caller receives `amount_out`
    ///  worth of tokens and pays no more than `amount_in_max`
    ///  of the starting token. Fails if any of these conditions
    ///  is not satisfied.
    ///  Transfers tokens to account under `to` address.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap_tokens_for_exact_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out: u128,
        amount_in_max: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![216, 234, 253, 103];
            amount_out.encode_to(&mut data);
            amount_in_max.encode_to(&mut data);
            path.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Exchanges tokens along `path` token pairs
    ///  so that at the end caller receives `amount_out`
    ///  worth of tokens and pays no more than `amount_in_max`
    ///  of the native token. Fails if any of these conditions
    ///  is not satisfied.
    ///  Transfers tokens to account under `to` address.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap_native_for_exact_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![18, 153, 253, 242];
            amount_out.encode_to(&mut data);
            path.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Removes `liquidity` amount of tokens from `(token_a, token_b)`
    ///  pair and transfers tokens `to` account.
    ///
    ///  Fails if any of the balances is lower than respective `*_min` amount.
    ///
    ///  Returns withdrawn balances of both tokens.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn remove_liquidity<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
        liquidity: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![211, 171, 229, 163];
            token_a.encode_to(&mut data);
            token_b.encode_to(&mut data);
            liquidity.encode_to(&mut data);
            amount_a_min.encode_to(&mut data);
            amount_b_min.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns address of the `Factory` contract for this `Router` instance.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn factory<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![172, 58, 76, 24];
        conn.read(self.account_id, data).await
    }

    ///  Removes `liquidity` amount of tokens from `(token, wrapped_native)`
    ///  pair and transfers tokens `to` account.
    ///
    ///  Fails if any of the balances is lower than respective `*_min` amount.
    ///
    ///  Returns withdrawn balances of both tokens.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn remove_liquidity_native<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        token: ink_primitives::AccountId,
        liquidity: u128,
        amount_token_min: u128,
        amount_native_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![52, 72, 187, 92];
            token.encode_to(&mut data);
            liquidity.encode_to(&mut data);
            amount_token_min.encode_to(&mut data);
            amount_native_min.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Adds liquidity to `(token, native token)` pair.
    ///
    ///  Will add at least `*_min` amount of tokens and up to `*_desired`
    ///  while still maintaining the constant `k` product of the pair.
    ///
    ///  If succesful, liquidity tokens will be minted for `to` account.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn add_liquidity_native<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token: ink_primitives::AccountId,
        amount_token_desired: u128,
        amount_token_min: u128,
        amount_native_min: u128,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![41, 45, 114, 33];
            token.encode_to(&mut data);
            amount_token_desired.encode_to(&mut data);
            amount_token_min.encode_to(&mut data);
            amount_native_min.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Exchanges exact amount of token,
    ///  along the `path` token pairs, and expects
    ///  to receive at least `amount_out_min` of native tokens
    ///  at the end of execution. Fails if the output
    ///  amount is less than `amount_out_min`.
    ///  Transfers tokens to account under `to` address.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap_exact_tokens_for_native<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_in: u128,
        amount_out_min: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![203, 87, 116, 35];
            amount_in.encode_to(&mut data);
            amount_out_min.encode_to(&mut data);
            path.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns amount of `B` tokens that have to be supplied
    ///  , with the `amount_a` amount of tokens `A, to maintain
    ///  constant `k` product of `(A, B)` token pair.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn quote<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_a: u128,
        reserve_a: u128,
        reserve_b: u128,
    ) -> Result<Result<Result<u128, RouterError>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![22, 52, 123, 16];
            amount_a.encode_to(&mut data);
            reserve_a.encode_to(&mut data);
            reserve_b.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Returns amount of `B` tokens received
    ///  for `amount_in` of `A` tokens that maintains
    ///  the constant product of `reserve_in / reserve_out`.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_amount_out<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_in: u128,
        reserve_in: u128,
        reserve_out: u128,
    ) -> Result<Result<Result<u128, RouterError>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![65, 227, 21, 253];
            amount_in.encode_to(&mut data);
            reserve_in.encode_to(&mut data);
            reserve_out.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Returns amount of `A` tokens user has to supply
    ///  to get exactly `amount_out` of `B` token while maintaining
    ///  pool's constant product.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_amount_in<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_out: u128,
        reserve_in: u128,
        reserve_out: u128,
    ) -> Result<Result<Result<u128, RouterError>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![234, 74, 200, 93];
            amount_out.encode_to(&mut data);
            reserve_in.encode_to(&mut data);
            reserve_out.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Returns amounts of tokens user has to supply.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_amounts_in<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_out: u128,
        path: Vec<ink_primitives::AccountId>,
    ) -> Result<Result<Result<Vec<u128>, RouterError>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![112, 121, 152, 252];
            amount_out.encode_to(&mut data);
            path.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Returns address of the `WrappedNative` contract for this `Router` instance.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn wnative<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![85, 147, 234, 182];
        conn.read(self.account_id, data).await
    }

    ///  Exchanges exact amount of native token,
    ///  along the `path` token pairs, and expects
    ///  to receive at least `amount_out_min` of tokens
    ///  at the end of execution. Fails if the output
    ///  amount is less than `amount_out_min`.
    ///  Transfers tokens to account under `to` address.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap_exact_native_for_tokens<
        TxInfo,
        E,
        C: ink_wrapper_types::SignedConnection<TxInfo, E>,
    >(
        &self,
        conn: &C,
        amount_out_min: u128,
        path: Vec<ink_primitives::AccountId>,
        to: ink_primitives::AccountId,
        deadline: u64,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![10, 120, 226, 81];
            amount_out_min.encode_to(&mut data);
            path.encode_to(&mut data);
            to.encode_to(&mut data);
            deadline.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns amounts of tokens received for `amount_in`.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_amounts_out<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_in: u128,
        path: Vec<ink_primitives::AccountId>,
    ) -> Result<Result<Result<Vec<u128>, RouterError>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![113, 112, 184, 246];
            amount_in.encode_to(&mut data);
            path.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }
}

#[allow(dead_code)]
pub async fn upload<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
    conn: &C,
) -> Result<TxInfo, E> {
    let wasm = include_bytes!("../../target/ink/router_contract/router_contract.wasm");
    let tx_info = conn.upload((*wasm).into(), CODE_HASH.into()).await?;

    Ok(tx_info)
}

impl Instance {
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn new<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        conn: &C,
        salt: Vec<u8>,
        factory: ink_primitives::AccountId,
        wnative: ink_primitives::AccountId,
    ) -> Result<Self, E> {
        let data = {
            let mut data = vec![155, 174, 157, 94];
            factory.encode_to(&mut data);
            wnative.encode_to(&mut data);
            data
        };
        let account_id = conn.instantiate(CODE_HASH, salt, data).await?;
        Ok(Self { account_id })
    }
}
