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
use alkanes_support::{id::AlkaneId, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::compat::to_arraybuffer_layout;

#[derive(MessageDispatch)]
pub enum AMMPathProviderMessage {
    #[opcode(0)]
    InitPathProvider,
    #[opcode(1)]
    #[returns(Vec<AlkaneId>)]
    GetOptimalPath { start: AlkaneId, end: AlkaneId },
    #[opcode(2)]
    SetPath {
        start: AlkaneId,
        end: AlkaneId,
        path: Vec<AlkaneId>,
    },
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

    pub fn get_optimal_path(&self, start: AlkaneId, end: AlkaneId) -> Result<CallResponse> {
        let context = self.context()?;
        let data: Vec<u8> = AMMPathProviderBase::path_bytes(self, &start, &end);
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = data;
        Ok(response)
    }

    pub fn set_path(
        &self,
        start: AlkaneId,
        end: AlkaneId,
        path: Vec<AlkaneId>,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        AMMPathProviderBase::set_optimal_path(self, start, end, path)?;

        Ok(CallResponse::forward(&context.incoming_alkanes))
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
