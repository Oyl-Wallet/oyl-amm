use std::sync::Arc;

use alkanes_runtime::message::MessageDispatch;
use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, storage::StoragePointer};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_pool::AMMPoolBase;
use alkanes_std_factory_support::MintableToken;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::{
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::shift_id_or_err,
};
use anyhow::{anyhow, Result};
use metashrew_support::utils::consume_u128;
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_passback_ptr},
    index_pointer::KeyValuePointer,
};

pub const FEE_TO_SWAP_TO_OYL_PER_10: u128 = 5;
// Define a new message type for OYL-specific functionality if needed
#[derive(MessageDispatch)]
pub enum OylAMMPoolMessage {
    #[opcode(0)]
    InitPool {
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        factory: AlkaneId,
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
    #[returns(AlkaneId, AlkaneId, u128, u128, u128, String)]
    PoolDetails,
}

#[derive(Default)]
pub struct OylAMMPool();

impl OylAMMPool {
    fn factory() -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/factory_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_u128(&mut cursor)?,
            consume_u128(&mut cursor)?,
        ))
    }
    fn set_factory(factory_id: AlkaneId) {
        let mut factory_id_pointer = StoragePointer::from_keyword("/factory_id");
        factory_id_pointer.set(Arc::new(factory_id.into()));
    }
    pub fn init_pool(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        factory: AlkaneId,
    ) -> Result<CallResponse> {
        OylAMMPool::set_factory(factory.into());
        AMMPoolBase::init_pool(self, alkane_a, alkane_b)
    }

    fn _handle_oyl_swap_and_burn(&self, alkane_out_with_fees: AlkaneTransfer) -> Result<()> {
        let context = self.context()?;
        let alkane_out_no_fees =
            self.get_transfer_out_from_swap(context.incoming_alkanes.clone(), false)?;

        let factory = OylAMMPool::factory()?;
        let amount_to_burn = (alkane_out_no_fees.value - alkane_out_with_fees.value)
            * FEE_TO_SWAP_TO_OYL_PER_10
            / 10;
        println!("amount_to_burn: {}", amount_to_burn);
        if amount_to_burn != 0 {
            self.call(
                &Cellpack {
                    target: factory,
                    inputs: vec![6], // swap to and burn oyl
                },
                &AlkaneTransferParcel(vec![AlkaneTransfer {
                    id: alkane_out_with_fees.id,
                    value: amount_to_burn,
                }]),
                self.fuel(),
            )?;
        }
        Ok(())
    }

    pub fn swap(&self, amount_out_predicate: u128) -> Result<CallResponse> {
        let response = AMMPoolBase::swap(self, amount_out_predicate)?;

        self._handle_oyl_swap_and_burn(response.alkanes.0[0])?;

        Ok(response)
    }
}

impl MintableToken for OylAMMPool {}
impl AMMPoolBase for OylAMMPool {}

impl AlkaneResponder for OylAMMPool {
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
    impl AlkaneResponder for OylAMMPool {
        type Message = OylAMMPoolMessage;
    }
}
