use std::sync::Arc;

use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, storage::StoragePointer};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::{AMMPool, AMMPoolBase, AMMReserves};
use alkanes_support::{
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

struct OylAMMPool {
    inner: AMMPool,
}

impl Clone for OylAMMPool {
    fn clone(&self) -> Self {
        OylAMMPool {
            inner: self.inner.clone(),
        }
    }
}

impl OylAMMPool {
    pub fn default() -> Self {
        let inner = AMMPool::default();
        let mut oyl_pool = OylAMMPool { inner };
        oyl_pool.inner.set_delegate(Box::new(oyl_pool.clone())); // Override delegate with self
        oyl_pool
    }
}

impl AMMReserves for OylAMMPool {}
impl AMMPoolBase for OylAMMPool {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        AMMReserves::reserves(self)
    }
    fn init_pool(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        context: Context,
    ) -> Result<CallResponse> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set(Arc::new(vec![0x01]));
            StoragePointer::from_keyword("/alkane/0").set(Arc::new(alkane_a.into()));
            StoragePointer::from_keyword("/alkane/1").set(Arc::new(alkane_b.into()));
            self.mint(context.myself, context.incoming_alkanes)
        } else {
            Err(anyhow!("already initialized"))
        }
    }
    fn process_inputs_and_init_pool(
        &self,
        mut inputs: Vec<u128>,
        context: Context,
    ) -> Result<CallResponse> {
        let (a, b) = self.pull_ids_or_err(&mut inputs)?;
        // let oyl_token = shift_id_or_err(&mut inputs)?;
        // also input which alkane in this current pair is the token to swap to, to swap to OYL
        // also input the address that the OYL tokens should go to.
        self.init_pool(a, b, context)
    }
    fn swap(
        &self,
        parcel: AlkaneTransferParcel,
        amount_out_predicate: u128,
    ) -> Result<CallResponse> {
        println!("special swap for oyl");
        AMMPoolBase::swap(&self.inner, parcel, amount_out_predicate)
    }
}

impl AlkaneResponder for OylAMMPool {
    fn execute(&self) -> Result<CallResponse> {
        self.inner.execute()
    }
}

declare_alkane! {OylAMMPool}
