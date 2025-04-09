use alkanes_runtime::{
    auth::AuthenticatedResponder, declare_alkane, message::MessageDispatch,
    runtime::AlkaneResponder,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_path_provider::AMMPathProviderBase;
use alkanes_support::response::CallResponse;
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;

#[derive(MessageDispatch)]
pub enum AMMPathProviderMessage {
    #[opcode(0)]
    InitPathProvider,
}

// Base implementation of AMMFactory that can be used directly or extended
#[derive(Default)]
pub struct AMMPathProvider();

impl AuthenticatedResponder for AMMPathProvider {}

impl AMMPathProviderBase for AMMPathProvider {}

impl AMMPathProvider {
    // External facing methods that implement the AMMFactoryMessage interface
    pub fn init_path_provider(&self) -> Result<CallResponse> {
        let context = self.context()?;
        AMMPathProviderBase::init_path_provider(self, context)
    }
}

impl AlkaneResponder for AMMPathProvider {
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
    impl AlkaneResponder for AMMPathProvider {
        type Message = AMMPathProviderMessage;
    }
}
