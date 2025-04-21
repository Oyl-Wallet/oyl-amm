use alkanes_runtime::{declare_alkane, message::MessageDispatch, runtime::AlkaneResponder};
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
    InitFactory { pool_factory_id: u128 },

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
}

// Base implementation of AMMFactory that can be used directly or extended
#[derive(Default)]
pub struct AMMFactory();

impl AMMFactory {
    pub fn create_new_pool(&self) -> Result<CallResponse> {
        let (cellpack, parcel) = AMMFactoryBase::create_new_pool(self)?;
        self.call(&cellpack, &parcel, self.fuel())
    }
}

impl AMMFactoryBase for AMMFactory {}

impl AlkaneResponder for AMMFactory {
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
    impl AlkaneResponder for AMMFactory {
        type Message = AMMFactoryMessage;
    }
}
