use scale::Encode as _;

// This file was auto-generated with ink-wrapper (https://crates.io/crates/ink-wrapper).

#[allow(dead_code)]
pub const CODE_HASH: [u8; 32] = [
    129, 250, 156, 149, 21, 117, 128, 66, 225, 241, 169, 181, 220, 206, 235, 214, 224, 209, 157,
    228, 168, 218, 243, 220, 64, 186, 236, 48, 167, 6, 226, 162,
];
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
pub enum ReentrancyGuardError {
    ReentrantCall(),
}

#[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
pub enum NoChainExtension {}

pub mod event {
    #[allow(dead_code, clippy::large_enum_variant)]
    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode)]
    pub enum Event {
        PairCreated {
            token_0: ink_primitives::AccountId,
            token_1: ink_primitives::AccountId,
            pair: ink_primitives::AccountId,
            pair_len: u64,
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
pub trait Factory {
    async fn create_pair<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn all_pairs<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        pid: u64,
    ) -> Result<Result<Option<ink_primitives::AccountId>, ink_wrapper_types::InkLangError>, E>;
    async fn get_pair<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
    ) -> Result<Result<Option<ink_primitives::AccountId>, ink_wrapper_types::InkLangError>, E>;
    async fn fee_to<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
    async fn pair_contract_code_hash<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::Hash, ink_wrapper_types::InkLangError>, E>;
    async fn fee_to_setter<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E>;
    async fn set_fee_to_setter<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        fee_to_setter: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
    async fn all_pairs_length<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u64, ink_wrapper_types::InkLangError>, E>;
    async fn set_fee_to<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        fee_to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E>;
}
#[async_trait::async_trait]
impl Factory for Instance {
    ///  Creates an instance of the `Pair` contract for the `(token_a, token_b)` pair.

    ///  Returns the address of the contract instance if successful.
    ///  Fails if the `Pair` instance of the token pair already exists
    ///  or the token pair is illegal.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn create_pair<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![199, 127, 75, 2];
            token_a.encode_to(&mut data);
            token_b.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns address of the pair contract identified by `pid` id.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn all_pairs<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        pid: u64,
    ) -> Result<Result<Option<ink_primitives::AccountId>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![129, 1, 194, 87];
            pid.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Returns addres of `Pair` contract instance (if any) for `(token_a, token_b)` pair.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn get_pair<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
        token_a: ink_primitives::AccountId,
        token_b: ink_primitives::AccountId,
    ) -> Result<Result<Option<ink_primitives::AccountId>, ink_wrapper_types::InkLangError>, E> {
        let data = {
            let mut data = vec![69, 163, 192, 246];
            token_a.encode_to(&mut data);
            token_b.encode_to(&mut data);
            data
        };
        conn.read(self.account_id, data).await
    }

    ///  Returns recipient address of the trading fees.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn fee_to<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![214, 131, 50, 243];
        conn.read(self.account_id, data).await
    }

    ///  Returns code hash of the `Pair` contract this factory instance uses.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn pair_contract_code_hash<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::Hash, ink_wrapper_types::InkLangError>, E> {
        let data = vec![32, 190, 88, 163];
        conn.read(self.account_id, data).await
    }

    ///  Returns account allowed to call `set_fee_to_setter`.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn fee_to_setter<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<ink_primitives::AccountId, ink_wrapper_types::InkLangError>, E> {
        let data = vec![157, 8, 231, 17];
        conn.read(self.account_id, data).await
    }

    ///  Sets the address eligible for calling `set_foo_to` method.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn set_fee_to_setter<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        fee_to_setter: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![128, 153, 149, 89];
            fee_to_setter.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }

    ///  Returns number of token pairs created by the factory contract.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn all_pairs_length<TxInfo, E, C: ink_wrapper_types::Connection<TxInfo, E>>(
        &self,
        conn: &C,
    ) -> Result<Result<u64, ink_wrapper_types::InkLangError>, E> {
        let data = vec![249, 45, 204, 63];
        conn.read(self.account_id, data).await
    }

    ///  Sets the address for receiving protocol's share of trading fees.
    #[allow(dead_code, clippy::too_many_arguments)]
    async fn set_fee_to<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        &self,
        conn: &C,
        fee_to: ink_primitives::AccountId,
    ) -> Result<TxInfo, E> {
        let data = {
            let mut data = vec![62, 242, 5, 167];
            fee_to.encode_to(&mut data);
            data
        };
        conn.exec(self.account_id, data).await
    }
}

#[allow(dead_code)]
pub async fn upload<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
    conn: &C,
) -> Result<TxInfo, E> {
    let wasm = include_bytes!("../../target/ink/factory_contract/factory_contract.wasm");
    let tx_info = conn.upload((*wasm).into(), CODE_HASH.into()).await?;
    Ok(tx_info)
}
impl Instance {
    #[allow(dead_code, clippy::too_many_arguments)]
    pub async fn new<TxInfo, E, C: ink_wrapper_types::SignedConnection<TxInfo, E>>(
        conn: &C,
        salt: Vec<u8>,
        fee_to_setter: ink_primitives::AccountId,
        pair_code_hash: ink_primitives::Hash,
    ) -> Result<Self, E> {
        let data = {
            let mut data = vec![155, 174, 157, 94];
            fee_to_setter.encode_to(&mut data);
            pair_code_hash.encode_to(&mut data);
            data
        };
        let account_id = conn.instantiate(CODE_HASH, salt, data).await?;
        Ok(Self { account_id })
    }
}
