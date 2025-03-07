use std::sync::Arc;

use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, storage::StoragePointer};

use alkane_factory_support::factory::MintableToken;
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
impl MintableToken for OylAMMPool {}
impl AMMReserves for OylAMMPool {}
impl AMMPoolBase for OylAMMPool {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        AMMReserves::reserves(self)
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
