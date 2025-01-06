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
        println!("special swap for oyl!");
        if parcel.0.len() != 1 {
            return Err(anyhow!(format!(
                "payload can only include 1 alkane, sent {}",
                parcel.0.len()
            )));
        }
        let transfer = parcel.0[0].clone();
        let (previous_a, previous_b) = self.previous_reserves(&parcel);
        let (reserve_a, reserve_b) = self.reserves();
        let output = if &transfer.id == &reserve_a.id {
            AlkaneTransfer {
                id: reserve_b.id,
                value: self.get_amount_out(transfer.value, previous_a.value, previous_b.value)?,
            }
        } else {
            AlkaneTransfer {
                id: reserve_a.id,
                value: self.get_amount_out(transfer.value, previous_b.value, previous_a.value)?,
            }
        };
        if output.value < amount_out_predicate {
            return Err(anyhow!("predicate failed: insufficient output"));
        }
        let mut response = CallResponse::default();
        response.alkanes = AlkaneTransferParcel(vec![output]);
        Ok(response)
    }
}

impl AlkaneResponder for OylAMMPool {
    fn execute(&self) -> Result<CallResponse> {
        self.inner.execute()
    }
}

declare_alkane! {OylAMMPool}
