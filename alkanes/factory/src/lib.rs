use alkanes_runtime::{
    auth::AuthenticatedResponder, declare_alkane, message::MessageDispatch,
    runtime::AlkaneResponder,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_factory::AMMFactoryBase;
use alkanes_support::{id::AlkaneId, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;

#[derive(MessageDispatch)]
pub enum AMMFactoryMessage {
    #[opcode(0)]
    InitFactory {
        pool_factory_id: u128,
        auth_token_units: u128,
    },

    #[opcode(1)]
    CreateNewPool,

    #[opcode(2)]
    FindExistingPoolId {
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    },

    #[opcode(3)]
    #[returns(Vec<u8>)]
    GetAllPools,

    #[opcode(4)]
    #[returns(Vec<u8>)]
    GetNumPools,

    #[opcode(7)]
    SetPoolFactoryId { pool_factory_id: u128 },

    #[opcode(10)]
    CollectFees { pool_id: AlkaneId },

    #[opcode(11)]
    AddLiquidity {
        token_a: AlkaneId,
        token_b: AlkaneId,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        deadline: u128,
    },

    #[opcode(12)]
    Burn {
        token_a: AlkaneId,
        token_b: AlkaneId,
        liquidity: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        deadline: u128,
    },
    #[opcode(13)]
    SwapExactTokensForTokens {
        amount_in: u128,
        path: Vec<AlkaneId>,
        amount_out_min: u128,
        deadline: u128,
    },

    #[opcode(14)]
    SwapTokensForExactTokens {
        path: Vec<AlkaneId>,
        amount_out: u128,
        amount_in_max: u128,
        deadline: u128,
    },
}

// Base implementation of AMMFactory that can be used directly or extended
#[derive(Default)]
pub struct AMMFactory();

impl AMMFactoryBase for AMMFactory {}

impl AlkaneResponder for AMMFactory {}
impl AuthenticatedResponder for AMMFactory {}

declare_alkane! {
    impl AlkaneResponder for AMMFactory {
        type Message = AMMFactoryMessage;
    }
}
