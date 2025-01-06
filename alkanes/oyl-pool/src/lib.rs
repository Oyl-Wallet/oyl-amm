use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::{AMMPool, AMMPoolBase};
use alkanes_support::{
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
};
use anyhow::{anyhow, Result};
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

#[derive(Default)]
struct OylAMMPool {
    inner: AMMPool,
}

impl AMMPoolBase for OylAMMPool {
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
        println!("special execute for oyl");
        self.inner.execute()
    }
}

declare_alkane! {OylAMMPool}
