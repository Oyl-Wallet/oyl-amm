use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{auth::AuthenticatedResponder, declare_alkane, message::MessageDispatch};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_std_factory_support::MintableToken;
use alkanes_support::response::CallResponse;
use anyhow::Result;
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};

#[derive(Default)]
pub struct OylToken(());

impl MintableToken for OylToken {}

impl AuthenticatedResponder for OylToken {}

#[derive(MessageDispatch)]
enum OylTokenMessage {
    #[opcode(0)]
    Initialize {
        total_supply: u128,
        name: String,
        symbol: String,
    },

    #[opcode(99)]
    #[returns(String)]
    GetName,

    #[opcode(100)]
    #[returns(String)]
    GetSymbol,

    #[opcode(101)]
    #[returns(u128)]
    GetTotalSupply,

    #[opcode(1000)]
    #[returns(Vec<u8>)]
    GetData,
}

impl OylToken {
    fn initialize(&self, total_supply: u128, name: String, symbol: String) -> Result<CallResponse> {
        self.observe_initialization()?;
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        <Self as MintableToken>::set_name_and_symbol_str(self, name, symbol);

        response.alkanes.0.push(self.mint(&context, total_supply)?);

        Ok(response)
    }

    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.name().into_bytes().to_vec();

        Ok(response)
    }

    fn get_symbol(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.symbol().into_bytes().to_vec();

        Ok(response)
    }

    fn get_total_supply(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.total_supply().to_le_bytes().to_vec();

        Ok(response)
    }

    fn get_data(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self.data();

        Ok(response)
    }
}

impl AlkaneResponder for OylToken {}

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for OylToken {
        type Message = OylTokenMessage;
    }
}
