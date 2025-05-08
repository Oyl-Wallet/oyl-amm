use std::sync::Arc;

use alkanes_runtime::{
    auth::AuthenticatedResponder, declare_alkane, message::MessageDispatch,
    runtime::AlkaneResponder, storage::StoragePointer,
};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime_factory::AMMFactoryBase;
use alkanes_support::{
    cellpack::Cellpack,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::to_arraybuffer_layout, index_pointer::KeyValuePointer, utils::consume_u128,
};

#[derive(MessageDispatch)]
pub enum OylAMMFactoryMessage {
    #[opcode(0)]
    InitFactory {
        pool_factory_id: u128,
        auth_token_units: u128,
        path_provider_id: AlkaneId,
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

    #[opcode(7)]
    SetPoolFactoryId { pool_factory_id: u128 },

    #[opcode(8)]
    SetPathProviderId { path_provider_id: AlkaneId },

    #[opcode(9)]
    SetOylTokenId { oyl_token_id: AlkaneId },

    #[opcode(10)]
    CollectFees { pool_id: AlkaneId },

    #[opcode(20)]
    SwapAlongPath { path: Vec<AlkaneId>, amount: u128 },
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
        auth_token_units: u128,
        path_provider_id: AlkaneId,
        oyl_token_id: AlkaneId,
    ) -> Result<CallResponse> {
        let response = AMMFactoryBase::init_factory(self, pool_factory_id, auth_token_units)?;
        OylAMMFactory::set_path_provider(path_provider_id);
        OylAMMFactory::set_oyl_token(oyl_token_id);
        Ok(response)
    }

    pub fn set_path_provider_id(&self, path_provider_id: AlkaneId) -> Result<CallResponse> {
        self.only_owner()?;
        let context = self.context()?;
        OylAMMFactory::set_path_provider(path_provider_id);
        Ok(CallResponse::forward(&context.incoming_alkanes.clone()))
    }

    pub fn set_oyl_token_id(&self, oyl_token_id: AlkaneId) -> Result<CallResponse> {
        self.only_owner()?;
        let context = self.context()?;
        OylAMMFactory::set_oyl_token(oyl_token_id);
        Ok(CallResponse::forward(&context.incoming_alkanes.clone()))
    }

    pub fn create_new_pool(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let (mut cellpack, parcel) = AMMFactoryBase::create_new_pool(self)?;
        cellpack.inputs.append(&mut context.clone().myself.into());
        self.call(&cellpack, &parcel, self.fuel())
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

    pub fn swap_to_and_burn_oyl(&self) -> Result<CallResponse> {
        let context = self.context()?;
        if context.incoming_alkanes.0.len() != 1 {
            return Err(anyhow!(format!(
                "payload can only include 1 alkane, sent {}",
                context.incoming_alkanes.0.len()
            )));
        }
        let oyl_token = OylAMMFactory::oyl_token()?;
        let alkane_transfer = context.incoming_alkanes.0[0];
        if alkane_transfer.id != oyl_token {
            let path = self._get_path_between(&alkane_transfer.id, &oyl_token)?;
            if path.len() != 0 {
                let _ = self.swap_along_path(path, 0); // soft fail if swapping to oyl fails
            }
        }

        Ok(CallResponse::default())
    }
}
impl AMMFactoryBase for OylAMMFactory {}

impl AlkaneResponder for OylAMMFactory {}
impl AuthenticatedResponder for OylAMMFactory {}

declare_alkane! {
    impl AlkaneResponder for OylAMMFactory {
        type Message = OylAMMFactoryMessage;
    }
}
