use std::sync::Arc;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, storage::StoragePointer,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::AMMPoolBase;
use alkanes_std_factory_support::MintableToken;
use alkanes_support::{
    cellpack::Cellpack,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::{overflow_error, shift, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};
use metashrew_support::index_pointer::KeyValuePointer;

#[derive(MessageDispatch)]
pub enum AMMPoolMessage {
    #[opcode(0)]
    InitPool {
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    },

    #[opcode(1)]
    AddLiquidity,

    #[opcode(2)]
    Burn,

    #[opcode(3)]
    Swap { amount_out_predicate: u128 },

    #[opcode(50)]
    ForwardIncoming,

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

impl AlkaneResponder for AMMPool {
    fn execute(&self) -> Result<CallResponse> {
        // The opcode extraction and dispatch logic is now handled by the declare_alkane macro
        // This method is still required by the AlkaneResponder trait, but we can just return an error
        // indicating that it should not be called directly
        Err(anyhow!(
            "This method should not be called directly. Use the declare_alkane macro instead."
        ))
    }
}
declare_alkane! {
    impl AlkaneResponder for AMMPool {
        type Message = AMMPoolMessage;
    }
}
