use alkanes_runtime_factory::{sort_alkanes, take_two, AMMFactory, AMMFactoryBase};

use std::sync::Arc;

use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, storage::StoragePointer};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    cellpack::Cellpack,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::shift_id_or_err,
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_passback_ptr},
    index_pointer::KeyValuePointer,
};

struct OylAMMFactory {
    inner: AMMFactory,
}

impl Clone for OylAMMFactory {
    fn clone(&self) -> Self {
        OylAMMFactory {
            inner: self.inner.clone(),
        }
    }
}

impl OylAMMFactory {
    pub fn default() -> Self {
        let inner = AMMFactory::default();
        let mut oyl_pool = OylAMMFactory { inner };
        oyl_pool.inner.set_delegate(Box::new(oyl_pool.clone())); // Override delegate with self
        oyl_pool
    }
}

impl AMMFactoryBase for OylAMMFactory {
    fn create_new_pool(&self, context: Context) -> Result<CallResponse> {
        if context.incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("must send two runes to initialize a pool"));
        }
        // check that
        let (alkane_a, alkane_b) = take_two(&context.incoming_alkanes.0);
        let (a, b) = sort_alkanes((alkane_a.id.clone(), alkane_b.id.clone()));
        let next_sequence = self.sequence();
        self.pool_pointer(&a, &b)
            .set(Arc::new(AlkaneId::new(2, next_sequence).into()));
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

impl AlkaneResponder for OylAMMFactory {
    fn execute(&self) -> Result<CallResponse> {
        self.inner.execute()
    }
}

declare_alkane! {OylAMMFactory}
