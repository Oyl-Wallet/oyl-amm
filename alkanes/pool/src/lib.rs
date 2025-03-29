use std::sync::Arc;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, storage::StoragePointer,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::{AMMPoolBase, AMMReserves};
use alkanes_std_factory_support::MintableToken;
use alkanes_support::{
    cellpack::Cellpack,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::{overflow_error, shift, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};
use metashrew_support::index_pointer::KeyValuePointer;

#[derive(MessageDispatch)]
pub enum AMMPoolMessage {
    #[opcode(0)]
    InitPool {
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    },

    #[opcode(1)]
    AddLiquidity,

    #[opcode(2)]
    Burn,

    #[opcode(3)]
    Swap { amount_out_predicate: u128 },

    #[opcode(50)]
    ForwardIncoming,

    #[opcode(99)]
    #[returns(String)]
    GetName,

    #[opcode(999)]
    #[returns(Vec<u8>)]
    PoolDetails,
}

#[derive(Default)]
pub struct AMMPool();

impl AMMPool {
    pub fn set_pool_name_and_symbol(&self) -> Result<()> {
        let (alkane_a, alkane_b) = self.alkanes_for_self()?;

        // Get name for alkane_a
        let name_a = match self.call(
            &Cellpack {
                target: alkane_a,
                inputs: vec![99],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        ) {
            Ok(response) => {
                if response.data.is_empty() {
                    format!("{},{}", alkane_a.block, alkane_a.tx)
                } else {
                    String::from_utf8_lossy(&response.data).to_string()
                }
            }
            Err(_) => format!("{},{}", alkane_a.block, alkane_a.tx),
        };

        // Get name for alkane_b
        let name_b = match self.call(
            &Cellpack {
                target: alkane_b,
                inputs: vec![99],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        ) {
            Ok(response) => {
                if response.data.is_empty() {
                    format!("{},{}", alkane_b.block, alkane_b.tx)
                } else {
                    String::from_utf8_lossy(&response.data).to_string()
                }
            }
            Err(_) => format!("{},{}", alkane_b.block, alkane_b.tx),
        };

        // Format the pool name
        let pool_name = format!("{} / {} LP", name_a, name_b);

        // Set the name using MintableToken trait
        MintableToken::name_pointer(self).set(Arc::new(pool_name.into_bytes()));

        Ok(())
    }

    // External facing methods that implement the AMMPoolMessage interface
    pub fn init_pool(&self, alkane_a: AlkaneId, alkane_b: AlkaneId) -> Result<CallResponse> {
        let context = self.context()?;
        let result = AMMPoolBase::init_pool(self, alkane_a, alkane_b, context);

        if result.is_ok() {
            // Ignore errors from set_pool_name_and_symbol to avoid failing the initialization
            let _ = self.set_pool_name_and_symbol();
        }

        result
    }

    pub fn add_liquidity(&self) -> Result<CallResponse> {
        let context = self.context()?;
        AMMPoolBase::add_liquidity(self, context.myself, context.incoming_alkanes)
    }

    pub fn burn(&self) -> Result<CallResponse> {
        let context = self.context()?;
        AMMPoolBase::burn(self, context.myself, context.incoming_alkanes)
    }

    pub fn swap(&self, amount_out_predicate: u128) -> Result<CallResponse> {
        let context = self.context()?;
        AMMPoolBase::swap(self, context.incoming_alkanes, amount_out_predicate)
    }

    pub fn forward_incoming(&self) -> Result<CallResponse> {
        let context = self.context()?;
        Ok(CallResponse::forward(&context.incoming_alkanes))
    }

    pub fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.name().into_bytes().to_vec();
        Ok(response)
    }

    pub fn pool_details(&self) -> Result<CallResponse> {
        AMMPoolBase::pool_details(self)
    }
}

impl MintableToken for AMMPool {}
impl AMMReserves for AMMPool {}
impl AMMPoolBase for AMMPool {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        AMMReserves::reserves(self)
    }
}

impl AlkaneResponder for AMMPool {
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
    impl AlkaneResponder for AMMPool {
        type Message = AMMPoolMessage;
    }
}
