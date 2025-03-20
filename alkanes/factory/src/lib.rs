use std::sync::Arc;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, storage::StoragePointer,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_factory::{sort_alkanes, take_two, AMMFactoryBase};
use alkanes_support::{
    cellpack::Cellpack, context::Context, id::AlkaneId, parcel::AlkaneTransferParcel,
    response::CallResponse,
};
use anyhow::{anyhow, Result};
use metashrew_support::{compat::to_arraybuffer_layout, index_pointer::KeyValuePointer};

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
}

// Base implementation of AMMFactory that can be used directly or extended
#[derive(Default)]
pub struct AMMFactory();

impl AMMFactory {
    // External facing methods that implement the AMMFactoryMessage interface
    pub fn init_factory(&self, pool_factory_id: u128) -> Result<CallResponse> {
        let context = self.context()?;
        AMMFactoryBase::init_factory(self, pool_factory_id, context)
    }

    pub fn create_new_pool(&self) -> Result<CallResponse> {
        let context = self.context()?;
        AMMFactoryBase::create_new_pool(self, context)
    }

    pub fn find_existing_pool_id(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        AMMFactoryBase::find_existing_pool_id(self, alkane_a, alkane_b, context)
    }

    pub fn get_all_pools(&self) -> Result<CallResponse> {
        AMMFactoryBase::get_all_pools(self)
    }
}

impl AMMFactoryBase for AMMFactory {
    fn create_new_pool(&self, context: Context) -> Result<CallResponse> {
        if context.incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("must send two runes to initialize a pool"));
        }
        // check that
        let (alkane_a, alkane_b) = take_two(&context.incoming_alkanes.0);
        let (a, b) = sort_alkanes((alkane_a.id.clone(), alkane_b.id.clone()));
        let next_sequence = self.sequence();
        let pool_id = AlkaneId::new(2, next_sequence);

        self.pool_pointer(&a, &b).set(Arc::new(pool_id.into()));

        // Add the new pool to the registry
        let length = self.all_pools_length()?;

        // Store the pool ID at the current index
        StoragePointer::from_keyword("/all_pools/")
            .select(&length.to_le_bytes().to_vec())
            .set(Arc::new(pool_id.into()));

        // Update the length
        StoragePointer::from_keyword("/all_pools_length")
            .set(Arc::new((length + 1).to_le_bytes().to_vec()));

        self.call(
            &Cellpack {
                target: AlkaneId {
                    block: 6,
                    tx: self.pool_id()?,
                },
                inputs: vec![0, a.block, a.tx, b.block, b.tx],
            },
            &AlkaneTransferParcel(vec![
                context.incoming_alkanes.0[0].clone(),
                context.incoming_alkanes.0[1].clone(),
            ]),
            self.fuel(),
        )
    }
}

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
