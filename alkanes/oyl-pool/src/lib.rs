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

    fn oyl_swap_contract(&self) -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/oyl_swap_contract")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::parse(&mut cursor)?)
    }
}

impl AMMReserves for OylAMMPool {}
impl AMMPoolBase for OylAMMPool {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        AMMReserves::reserves(self)
    }
    fn process_inputs_and_init_pool(
        &self,
        mut inputs: Vec<u128>,
        context: Context,
    ) -> Result<CallResponse> {
        let (a, b) = self.pull_ids_or_err(&mut inputs)?;
        let response = self.init_pool(a, b, context)?;

        let mut oyl_swap_storage = StoragePointer::from_keyword("/oyl_swap_contract");
        let oyl_swap_contract = shift_id_or_err(&mut inputs)?;
        oyl_swap_storage.set(Arc::new(oyl_swap_contract.into()));
        Ok(response)
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
