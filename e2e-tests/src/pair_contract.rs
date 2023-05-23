use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    10, 154, 141, 16, 193, 200, 83, 40, 64, 47, 35, 160, 16, 35, 144, 111, 175, 119, 48, 203, 170,
    197, 164, 245, 105, 99, 175, 15, 57, 215, 207, 116,
];

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct WrappedU256(pub U256);

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub struct U256(pub [u64; 4]);

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
pub enum OwnableError {
    CallerIsNotOwner(),
    NewOwnerIsZero(),
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
pub enum ReentrancyGuardError {
    ReentrantCall(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum NoChainExtension {}

pub mod event {
    #[allow(dead_code, clippy::large_enum_variant)]
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub enum Event {
        Mint {
            sender: ink_primitives::AccountId,
            amount_0: u128,
            amount_1: u128,
        },

        Burn {
            sender: ink_primitives::AccountId,
            amount_0: u128,
            amount_1: u128,
            to: ink_primitives::AccountId,
        },

        Swap {
            sender: ink_primitives::AccountId,
            amount_0_in: u128,
            amount_1_in: u128,
            amount_0_out: u128,
            amount_1_out: u128,
            to: ink_primitives::AccountId,
        },

        Sync {
            reserve_0: u128,
            reserve_1: u128,
        },

        Transfer {
            from: Option<ink_primitives::AccountId>,
            to: Option<ink_primitives::AccountId>,
            value: u128,
        },

        Approval {
            owner: ink_primitives::AccountId,
            spender: ink_primitives::AccountId,
            value: u128,
        },
    }
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
pub trait PSP22 {
    async fn total_supply<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E>;
    async fn balance_of<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E>;
    async fn increase_allowance<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        delta_value: u128,
    ) -> Result<TxInfo, E>;
    async fn decrease_allowance<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        delta_value: u128,
    ) -> Result<TxInfo, E>;
    async fn allowance<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
        spender: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E>;
    async fn transfer<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<TxInfo, E>;
    async fn transfer_from<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        from: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<TxInfo, E>;
    async fn approve<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        value: u128,
    ) -> Result<TxInfo, E>;
}

#[async_trait::async_trait]
impl PSP22 for Instance {
    ///  Returns the total token supply.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn total_supply<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E> {
        let data = vec![22, 45, 248, 194];
        conn.read(self.account_id, data).await
    }

    ///  Returns the account Balance for the specified `owner`.
    ///
    ///  Returns `0` if the account is non-existent.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn balance_of<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![101, 104, 56, 47];
            owner.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Atomically increases the allowance granted to `spender` by the caller.
    ///
    ///  An `Approval` event is emitted.
    ///
    ///  # Errors
    ///
    ///  Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    ///  Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn increase_allowance<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        delta_value: u128,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![150, 214, 181, 122];
            spender.encode_to(&mut data);
            delta_value.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Atomically decreases the allowance granted to `spender` by the caller.
    ///
    ///  An `Approval` event is emitted.
    ///
    ///  # Errors
    ///
    ///  Returns `InsufficientAllowance` error if there are not enough tokens allowed
    ///  by owner for `spender`.
    ///
    ///  Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    ///  Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn decrease_allowance<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        delta_value: u128,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![254, 203, 87, 213];
            spender.encode_to(&mut data);
            delta_value.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns the amount which `spender` is still allowed to withdraw from `owner`.
    ///
    ///  Returns `0` if no allowance has been set `0`.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn allowance<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
        spender: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![77, 71, 217, 33];
            owner.encode_to(&mut data);
            spender.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Transfers `value` amount of tokens from the caller's account to account `to`
    ///  with additional `data` in unspecified format.
    ///
    ///  On success a `Transfer` event is emitted.
    ///
    ///  # Errors
    ///
    ///  Returns `InsufficientBalance` error if there are not enough tokens on
    ///  the caller's account Balance.
    ///
    ///  Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    ///  Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn transfer<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<TxInfo, E> {
        let data_ = {
            let mut data_ = vec![219, 32, 249, 245];
            to.encode_to(&mut data_);
            value.encode_to(&mut data_);
            data.encode_to(&mut data_);
            data_
        };
        conn.exec(self.account_id, data_).await
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    async fn transfer_from<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        from: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<TxInfo, E> {
        let data_ = {
            let mut data_ = vec![84, 179, 199, 110];
            from.encode_to(&mut data_);
            to.encode_to(&mut data_);
            value.encode_to(&mut data_);
            data.encode_to(&mut data_);
            data_
        };
        conn.exec(self.account_id, data_).await
    }

    ///  Allows `spender` to withdraw from the caller's account multiple times, up to
    ///  the `value` amount.
    ///
    ///  If this function is called again it overwrites the current allowance with `value`.
    ///
    ///  An `Approval` event is emitted.
    ///
    ///  # Errors
    ///
    ///  Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    ///  Returns `ZeroRecipientAddress` error if recipient's address is zero.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn approve<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        value: u128,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![178, 15, 27, 189];
            spender.encode_to(&mut data);
            value.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }
}

#[async_trait::async_trait]
pub trait Ownable {
    async fn owner<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
    async fn transfer_ownership<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        new_owner: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn renounce_ownership<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E>;
}

#[async_trait::async_trait]
impl Ownable for Instance {
    ///  Returns the address of the current owner.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn owner<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![79, 164, 60, 140];
        conn.read(self.account_id, data).await
    }

    ///  Transfers ownership of the contract to a `new_owner`.
    ///  Can only be called by the current owner.
    ///
    ///  On success a `OwnershipTransferred` event is emitted.
    ///
    ///  # Errors
    ///
    ///  Panics with `CallerIsNotOwner` error if caller is not owner.
    ///
    ///  Panics with `NewOwnerIsZero` error if new owner's address is zero.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn transfer_ownership<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        new_owner: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![17, 244, 62, 253];
            new_owner.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Leaves the contract without owner. It will not be possible to call
    ///  owner's functions anymore. Can only be called by the current owner.
    ///
    ///  NOTE: Renouncing ownership will leave the contract without an owner,
    ///  thereby removing any functionality that is only available to the owner.
    ///
    ///  On success a `OwnershipTransferred` event is emitted.
    ///
    ///  # Errors
    ///
    ///  Panics with `CallerIsNotOwner` error if caller is not owner
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn renounce_ownership<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E> {
        let data = vec![94, 34, 135, 83];
        conn.exec(self.account_id, data).await
    }
}

#[async_trait::async_trait]
pub trait Pair {
    async fn mint<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn get_token_1<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
    async fn get_reserves<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<(u128, u128, u64), ink_wrapper_types::InkLangError>, E>;
    async fn swap<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_0_out: u128,
        amount_1_out: u128,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn price_0_cumulative_last<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<WrappedU256, ink_wrapper_types::InkLangError>, E>;
    async fn price_1_cumulative_last<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<WrappedU256, ink_wrapper_types::InkLangError>, E>;
    async fn burn<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn sync<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E>;
    async fn skim<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn initialize<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_0: ink_primitives::AccountId,
        token_1: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn get_token_0<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
}

#[async_trait::async_trait]
impl Pair for Instance {
    ///  Mints liquidity tokens `to` account.
    ///  The amount minted is equivalent to the excess of contract's balance and reserves.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn mint<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![78, 170, 247, 34];
            to.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns address of the second token.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_token_1<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![165, 176, 97, 111];
        conn.read(self.account_id, data).await
    }

    ///  Returns amounts of tokens this pair holds at `Timestamp`.
    ///  
    ///  NOTE: This does not include the tokens that were transferred to the contract
    ///  as part of the _current_ transaction.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_reserves<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<(u128, u128, u64), ink_wrapper_types::InkLangError>, E> {
        let data = vec![90, 33, 227, 252];
        conn.read(self.account_id, data).await
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    async fn swap<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        amount_0_out: u128,
        amount_1_out: u128,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![196, 182, 14, 216];
            amount_0_out.encode_to(&mut data);
            amount_1_out.encode_to(&mut data);
            to.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns cumulative prive of the first token.
    ///  
    ///  NOTE: Cumulative price is the sum of token price,
    ///  recorded at the end of the block (in the last transaction),
    ///  since the beginning of the token pair.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn price_0_cumulative_last<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<WrappedU256, ink_wrapper_types::InkLangError>, E> {
        let data = vec![244, 217, 153, 81];
        conn.read(self.account_id, data).await
    }

    ///  Returns cumulative prive of the second token.
    ///  
    ///  NOTE: Cumulative price is the sum of token price,
    ///  recorded at the end of the block (in the last transaction),
    ///  since the beginning of the token pair.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn price_1_cumulative_last<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<WrappedU256, ink_wrapper_types::InkLangError>, E> {
        let data = vec![29, 211, 141, 82];
        conn.read(self.account_id, data).await
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    async fn burn<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![2, 33, 197, 36];
            to.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    async fn sync<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E> {
        let data = vec![121, 38, 29, 147];
        conn.exec(self.account_id, data).await
    }

    #[allow(dead_code, clippy::too_many_arguments)]
    async fn skim<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![81, 195, 39, 129];
            to.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Initializes the pair with given token IDs.
    ///  
    ///  NOTE: Why do we need it at all? Why not put in the constructor?
    ///  Potentialy dangerous in case of a hack where initial caller/owner
    ///  of the contract can re-initialize with a different pair.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn initialize<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_0: ink_primitives::AccountId,
        token_1: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![211, 114, 2, 28];
            token_0.encode_to(&mut data);
            token_1.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns address of the first token.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_token_0<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![122, 235, 152, 168];
        conn.read(self.account_id, data).await
    }
}

#[allow(dead_code)]
pub async fn upload<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
    conn: &C,
) -> Result<TxInfo, E> {
    let wasm = include_bytes!("../../target/ink/pair_contract/pair_contract.wasm");
    let tx_info = conn.upload((*wasm).into(), CODE_HASH.into()).await?;

    Ok(tx_info)
}

impl Instance {
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn new<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        conn: &C,
        salt: Vec<u8>,
    ) -> Result<Self, E> {
        let data = vec![155, 174, 157, 94];
        let account_id = conn.instantiate(CODE_HASH, salt, data).await?;
        Ok(Self { account_id })
    }
}