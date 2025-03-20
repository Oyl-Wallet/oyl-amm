use alkanes_runtime::{
    declare_alkane, message::MessageDispatch, runtime::AlkaneResponder, storage::StoragePointer,
};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    cellpack::Cellpack,
    context::Context,
    id::AlkaneId,
    parcel::AlkaneTransferParcel,
    response::CallResponse,
    utils::{shift_id_or_err, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_passback_ptr},
    index_pointer::KeyValuePointer,
    utils::consume_u128,
};
use std::sync::Arc;

#[derive(MessageDispatch)]
enum AMMRouterMessage {
    #[opcode(0)]
    Initialize { factory_id: AlkaneId },

    #[opcode(1)]
    AddLiquidity { token1: AlkaneId, token2: AlkaneId },

    #[opcode(2)]
    RemoveLiquidity { token1: AlkaneId, token2: AlkaneId },

    #[opcode(3)]
    Swap { path: Vec<AlkaneId>, amount: u128 },

    #[opcode(4)]
    #[returns(Vec<u8>)]
    GetAllPools,

    #[opcode(50)]
    ForwardIncoming,
}

#[derive(Default)]
struct AMMRouter(());

impl AMMRouter {
    fn factory() -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/factory")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_u128(&mut cursor)?,
            consume_u128(&mut cursor)?,
        ))
    }

    fn get_pool_for(&self, alkane1: &AlkaneId, alkane2: &AlkaneId) -> Result<AlkaneId> {
        let factory = Self::factory()?;
        let response = self.call(
            &Cellpack {
                target: factory,
                inputs: vec![2, alkane1.block, alkane1.tx, alkane2.block, alkane2.tx],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        )?;
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(response.data);
        //wrote this block with an angle for creating the pool here if it didnt find one,
        let pool = AlkaneId::new(consume_u128(&mut cursor)?, consume_u128(&mut cursor)?);
        Ok(pool)
    }

    fn get_all_pools(&self) -> Result<CallResponse> {
        let factory = Self::factory()?;
        self.call(
            &Cellpack {
                target: factory,
                inputs: vec![3],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        )
    }

    fn initialize(&self, factory_id: AlkaneId) -> Result<CallResponse> {
        let context = self.context()?;

        let mut pointer = StoragePointer::from_keyword("/initialized");
        let mut factory = StoragePointer::from_keyword("/factory");
        if pointer.get().len() == 0 {
            factory.set(Arc::new(factory_id.into()));
            pointer.set(Arc::new(vec![0x01]));
            //placeholder
            Ok(CallResponse::forward(&context.incoming_alkanes.clone()))
        } else {
            Err(anyhow!("already initialized"))
        }
    }

    fn add_or_remove_liquidity(
        &self,
        opcode: u128,
        token1: AlkaneId,
        token2: AlkaneId,
    ) -> Result<CallResponse> {
        let context = self.context()?;

        let pool = self.get_pool_for(&token1, &token2)?;
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![opcode],
        };
        let response = self.call(&cellpack, &context.incoming_alkanes, self.fuel())?;
        Ok(response)
    }

    fn add_liquidity(&self, token1: AlkaneId, token2: AlkaneId) -> Result<CallResponse> {
        self.add_or_remove_liquidity(1, token1, token2)
    }

    fn remove_liquidity(&self, token1: AlkaneId, token2: AlkaneId) -> Result<CallResponse> {
        self.add_or_remove_liquidity(2, token1, token2)
    }

    fn swap(&self, path: Vec<AlkaneId>, amount: u128) -> Result<CallResponse> {
        let context = self.context()?;

        // swap
        if path.len() < 2 {
            return Err(anyhow!("Routing path must be at least two alkanes long"));
        }
        let mut this_response = CallResponse {
            alkanes: context.incoming_alkanes.clone(),
            data: vec![],
        };

        for i in 1..path.len() {
            let pool = self.get_pool_for(&path[i - 1], &path[i])?;
            let this_amount = if i == path.len() - 1 { amount } else { 0 };
            let cellpack = Cellpack {
                target: pool,
                inputs: vec![3, this_amount],
            };
            this_response = self.call(&cellpack, &this_response.alkanes, self.fuel())?;
            println!("This response for pair {}: {:?}", i, this_response);
        }

        Ok(this_response)
    }

    fn forward_incoming(&self) -> Result<CallResponse> {
        let context = self.context()?;
        Ok(CallResponse::forward(&context.incoming_alkanes))
    }
}

impl AlkaneResponder for AMMRouter {
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
    impl AlkaneResponder for AMMRouter {
        type Message = AMMRouterMessage;
    }
}
