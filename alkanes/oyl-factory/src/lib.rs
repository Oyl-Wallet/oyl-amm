use std::sync::Arc;

use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, storage::StoragePointer,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_factory::{sort_alkanes, take_two, AMMFactoryBase};
use alkanes_support::{
    cellpack::Cellpack, context::Context, id::AlkaneId, parcel::AlkaneTransferParcel,
    response::CallResponse,
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::to_arraybuffer_layout, index_pointer::KeyValuePointer, utils::consume_u128,
};

pub const FEE_TO_SWAP_TO_OYL_PER_1000: u128 = 3;
#[derive(MessageDispatch)]
pub enum OylAMMFactoryMessage {
    #[opcode(0)]
    InitFactory {
        pool_factory_id: u128,
        path_provider_id: AlkaneId,
        router_id: AlkaneId,
        oyl_token_id: AlkaneId,
    },

    #[opcode(1)]
    CreateNewPool,

    #[opcode(2)]
    FindExistingPoolId {
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    },

    #[opcode(3)]
    #[returns(Vec<u8>)]
    GetAllPools,

    #[opcode(4)]
    #[returns(Vec<u8>)]
    GetNumPools,

    #[opcode(5)]
    #[returns(AlkaneId)]
    GetPathProvider,

    #[opcode(6)]
    SwapToAndBurnOyl,
}

// Base implementation of AMMFactory that can be used directly or extended
#[derive(Default)]
pub struct OylAMMFactory();

impl OylAMMFactory {
    fn path_provider() -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/path_provider_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_u128(&mut cursor)?,
            consume_u128(&mut cursor)?,
        ))
    }
    fn set_path_provider(path_provider_id: AlkaneId) {
        let mut path_provider_id_pointer = StoragePointer::from_keyword("/path_provider_id");
        path_provider_id_pointer.set(Arc::new(path_provider_id.into()));
    }
    fn router() -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/router_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_u128(&mut cursor)?,
            consume_u128(&mut cursor)?,
        ))
    }
    fn set_router(router_id: AlkaneId) {
        let mut router_id_pointer = StoragePointer::from_keyword("/router_id");
        router_id_pointer.set(Arc::new(router_id.into()));
    }
    fn oyl_token() -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/oyl_token_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_u128(&mut cursor)?,
            consume_u128(&mut cursor)?,
        ))
    }
    fn set_oyl_token(oyl_token_id: AlkaneId) {
        let mut oyl_token_id_pointer = StoragePointer::from_keyword("/oyl_token_id");
        oyl_token_id_pointer.set(Arc::new(oyl_token_id.into()));
    }
    // External facing methods that implement the AMMFactoryMessage interface
    pub fn init_factory(
        &self,
        pool_factory_id: u128,
        path_provider_id: AlkaneId,
        router_id: AlkaneId,
        oyl_token_id: AlkaneId,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let response = AMMFactoryBase::init_factory(self, pool_factory_id, context)?;
        OylAMMFactory::set_path_provider(path_provider_id);
        OylAMMFactory::set_router(router_id);
        OylAMMFactory::set_oyl_token(oyl_token_id);
        Ok(response)
    }

    pub fn create_new_pool(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let (mut cellpack, parcel) =
            AMMFactoryBase::create_new_pool(self, context.clone(), self.sequence())?;
        cellpack.inputs.append(&mut context.clone().myself.into());
        self.call(&cellpack, &parcel, self.fuel())
    }

    pub fn find_existing_pool_id(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        response.data = AMMFactoryBase::find_existing_pool_id(self, alkane_a, alkane_b).into();
        Ok(response)
    }

    pub fn get_all_pools(&self) -> Result<CallResponse> {
        let context = self.context()?;
        AMMFactoryBase::get_all_pools(self, context)
    }

    pub fn get_num_pools(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        response.data = AMMFactoryBase::all_pools_length(self)?
            .to_le_bytes()
            .to_vec();
        Ok(response)
    }

    pub fn get_path_provider(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        let path_provider_id = StoragePointer::from_keyword("/path_provider_id")
            .get()
            .to_vec();
        response.data = path_provider_id;
        Ok(response)
    }

    fn _get_path_between(&self, alkane1: &AlkaneId, alkane2: &AlkaneId) -> Result<Vec<AlkaneId>> {
        let path_provider = OylAMMFactory::path_provider()?;
        let cellpack = Cellpack {
            target: path_provider,
            inputs: vec![1, alkane1.block, alkane1.tx, alkane2.block, alkane2.tx], // get optimal path
        };
        let response = self.call(&cellpack, &AlkaneTransferParcel::default(), self.fuel())?;
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(response.data);
        let mut path = Vec::new();

        // Keep reading pairs of u128s until consume_u128 returns an error
        loop {
            match (consume_u128(&mut cursor), consume_u128(&mut cursor)) {
                (Ok(block), Ok(tx)) => {
                    path.push(AlkaneId::new(block, tx));
                }
                _ => break, // Break the loop if either consume_u128 call fails
            }
        }

        Ok(path)
    }

    fn _swap_and_burn_oyl(&self, path: Vec<AlkaneId>) -> Result<()> {
        let router = OylAMMFactory::router()?;
        let mut input: Vec<u128> = vec![3, path.len() as u128];
        for id in path {
            input.append(&mut id.into());
        }
        self.call(
            &Cellpack {
                target: router,
                inputs: input,
            },
            &AlkaneTransferParcel::default(),
            self.fuel(),
        )?;
        Ok(())
    }

    pub fn swap_to_and_burn_oyl(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let oyl_token = OylAMMFactory::oyl_token()?;
        for alkane_transfer in context.incoming_alkanes.0 {
            let path = self._get_path_between(&alkane_transfer.id, &oyl_token)?;
            if path.len() != 0 {
                self._swap_and_burn_oyl(path);
            }
        }
        Ok(CallResponse::default())
    }
}
impl AMMFactoryBase for OylAMMFactory {}

impl AlkaneResponder for OylAMMFactory {
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
    impl AlkaneResponder for OylAMMFactory {
        type Message = OylAMMFactoryMessage;
    }
}
