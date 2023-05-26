use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    111, 65, 128, 237, 148, 33, 212, 104, 146, 198, 37, 217, 229, 242, 37, 69, 55, 140, 63, 149,
    250, 3, 96, 64, 8, 202, 155, 10, 174, 216, 49, 55,
];

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
pub enum NoChainExtension {}

pub mod event {
    #[allow(dead_code, clippy::large_enum_variant)]
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub enum Event {
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
    async fn allowance<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
        spender: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E>;
    async fn balance_of<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        owner: ink_primitives::AccountId,
    ) -> Result<Result<u128, ink_wrapper_types::InkLangError>, E>;
    async fn decrease_allowance<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        delta_value: u128,
    ) -> Result<TxInfo, E>;
    async fn transfer_from<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        from: ink_primitives::AccountId,
        to: ink_primitives::AccountId,
        value: u128,
        data: Vec<u8>,
    ) -> Result<TxInfo, E>;
    async fn transfer<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
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
    async fn increase_allowance<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        spender: ink_primitives::AccountId,
        delta_value: u128,
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

    ///  Transfers `value` tokens on the behalf of `from` to the account `to`
    ///  with additional `data` in unspecified format.
    ///
    ///  This can be used to allow a contract to transfer tokens on ones behalf and/or
    ///  to charge fees in sub-currencies, for example.
    ///
    ///  On success a `Transfer` and `Approval` events are emitted.
    ///
    ///  # Errors
    ///
    ///  Returns `InsufficientAllowance` error if there are not enough tokens allowed
    ///  for the caller to withdraw from `from`.
    ///
    ///  Returns `InsufficientBalance` error if there are not enough tokens on
    ///  the the account Balance of `from`.
    ///
    ///  Returns `ZeroSenderAddress` error if sender's address is zero.
    ///
    ///  Returns `ZeroRecipientAddress` error if recipient's address is zero.
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
}

#[async_trait::async_trait]
pub trait Wnative {
    async fn withdraw<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        amount: u128,
    ) -> Result<TxInfo, E>;
    async fn deposit<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E>;
}

#[async_trait::async_trait]
impl Wnative for Instance {
    ///  Unwrap NATIVE
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn withdraw<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        amount: u128,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![87, 17, 232, 16];
            amount.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Deposit NATIVE to wrap it
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn deposit<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<TxInfo, E> {
        let data = vec![158, 29, 225, 29];
        conn.exec(self.account_id, data).await
    }
}

#[async_trait::async_trait]
pub trait PSP22Metadata {
    async fn token_symbol<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Option<String>, ink_wrapper_types::InkLangError>, E>;
    async fn token_name<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Option<String>, ink_wrapper_types::InkLangError>, E>;
    async fn token_decimals<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u8, ink_wrapper_types::InkLangError>, E>;
}

#[async_trait::async_trait]
impl PSP22Metadata for Instance {
    ///  Returns the token symbol.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn token_symbol<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Option<String>, ink_wrapper_types::InkLangError>, E> {
        let data = vec![52, 32, 91, 229];
        conn.read(self.account_id, data).await
    }

    ///  Returns the token name.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn token_name<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<Option<String>, ink_wrapper_types::InkLangError>, E> {
        let data = vec![61, 38, 27, 212];
        conn.read(self.account_id, data).await
    }

    ///  Returns the token decimals.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn token_decimals<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u8, ink_wrapper_types::InkLangError>, E> {
        let data = vec![114, 113, 183, 130];
        conn.read(self.account_id, data).await
    }
}

#[allow(dead_code)]
pub async fn upload<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
    conn: &C,
) -> Result<TxInfo, E> {
    let wasm = include_bytes!("../../target/ink/wnative_contract/wnative_contract.wasm");
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
