use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::AMMPoolBase;
use alkanes_std_factory_support::MintableToken;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

#[derive(MessageDispatch)]
pub enum AMMPoolMessage {
    #[opcode(0)]
    InitPool {
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        factory: AlkaneId,
    },

    #[opcode(1)]
    AddLiquidity,

    #[opcode(2)]
    Burn,

    // this low level function should generally not be called directly unless the user is experienced with alkanes contracts
    #[opcode(3)]
    Swap {
        amount_0_out: u128,
        amount_1_out: u128,
        to: AlkaneId,
        data: Vec<u128>,
    },

    #[opcode(10)]
    CollectFees {},

    #[opcode(50)]
    ForwardIncoming,

    #[opcode(97)]
    #[returns(u128, u128)]
    GetReserves,

    #[opcode(98)]
    #[returns(u128, u128)]
    GetPriceCumulativeLast,

    #[opcode(99)]
    #[returns(String)]
    GetName,

    #[opcode(999)]
    #[returns(Vec<u8>)]
    PoolDetails,
}

#[derive(Default)]
pub struct AMMPool();

impl MintableToken for AMMPool {}
impl AMMPoolBase for AMMPool {}

impl AlkaneResponder for AMMPool {}
declare_alkane! {
    impl AlkaneResponder for AMMPool {
        type Message = AMMPoolMessage;
    }
}
