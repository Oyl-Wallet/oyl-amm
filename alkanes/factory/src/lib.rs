use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_factory::AMMFactory;
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

declare_alkane! {AMMFactory}
