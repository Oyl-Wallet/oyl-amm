use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::{AMMPool, AMMPoolBase};
use alkanes_support::response::CallResponse;
use anyhow::Result;
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

#[derive(Default)]
struct OylAMMPool {
    inner: AMMPool,
}

impl AMMPoolBase for OylAMMPool {}

impl AlkaneResponder for OylAMMPool {
    fn execute(&self) -> Result<CallResponse> {
        self.inner.execute()
    }
}

declare_alkane! {OylAMMPool}
