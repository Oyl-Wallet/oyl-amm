use alkanes_runtime_factory::{sort_alkanes, take_two, AMMFactory, AMMFactoryBase};

use std::sync::Arc;

use alkanes_runtime::{declare_alkane, runtime::AlkaneResponder, storage::StoragePointer};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    cellpack::Cellpack,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::{shift_id_or_err, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_passback_ptr},
    index_pointer::KeyValuePointer,
};

struct OylAMMFactory {
    inner: AMMFactory,
}

impl Clone for OylAMMFactory {
    fn clone(&self) -> Self {
        OylAMMFactory {
            inner: self.inner.clone(),
        }
    }
}

impl OylAMMFactory {
    fn oyl_token(&self) -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/oyl_token")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::parse(&mut cursor)?)
    }

    pub fn default() -> Self {
        let inner = AMMFactory::default();
        let mut oyl_pool = OylAMMFactory { inner };
        oyl_pool.inner.set_delegate(Box::new(oyl_pool.clone())); // Override delegate with self
        oyl_pool
    }
}

impl AMMFactoryBase for OylAMMFactory {
    fn process_inputs_and_init_factory(
        &self,
        mut inputs: Vec<u128>,
        context: Context,
    ) -> Result<CallResponse> {
        println!("special process_inputs_and_init_factory for oyl");
        let pool_factory_id = shift_or_err(&mut inputs)?;
        let response = self.init_factory(pool_factory_id, context)?;

        let mut oyl_token_storage = StoragePointer::from_keyword("/oyl_token");
        let oyl_token: AlkaneId = shift_id_or_err(&mut inputs)?;
        oyl_token_storage.set(Arc::new(oyl_token.into()));
        println!("set oyl token storage");
        Ok(response)
    }
    fn create_new_pool(&self, context: Context) -> Result<CallResponse> {
        println!("special create_new_pool for oyl");
        if context.incoming_alkanes.0.len() != 2 {
            return Err(anyhow!("must send two runes to initialize a pool"));
        }
        // check that
        let (alkane_a, alkane_b) = take_two(&context.incoming_alkanes.0);
        let (a, b) = sort_alkanes((alkane_a.id.clone(), alkane_b.id.clone()));
        let oyl_token = self.oyl_token()?;
        let (a_oyl, b_oyl) = sort_alkanes((alkane_a.id.clone(), oyl_token.clone()));
        let next_sequence = self.sequence();
        self.pool_pointer(&a_oyl, &b_oyl)
            .set(Arc::new(AlkaneId::new(2, next_sequence).into()));
        let oyl_pool_deployment_response = self.call(
            &Cellpack {
                target: AlkaneId {
                    block: 6,
                    tx: self.pool_id()?,
                },
                inputs: vec![
                    0,
                    a_oyl.block,
                    a_oyl.tx,
                    b_oyl.block,
                    b_oyl.tx,
                    2,
                    next_sequence,
                ],
            },
            &AlkaneTransferParcel(vec![
                context.incoming_alkanes.0[0].clone(),
                context.incoming_alkanes.0[1].clone(),
            ]),
            self.fuel(),
        )?;
        self.pool_pointer(&a, &b)
            .set(Arc::new(AlkaneId::new(2, next_sequence + 1).into()));
        let pool_deployment_response = self.call(
            &Cellpack {
                target: AlkaneId {
                    block: 6,
                    tx: self.pool_id()?,
                },
                inputs: vec![0, a.block, a.tx, b.block, b.tx, 2, next_sequence],
            },
            &AlkaneTransferParcel(vec![
                context.incoming_alkanes.0[0].clone(),
                context.incoming_alkanes.0[1].clone(),
            ]),
            self.fuel(),
        )?;
        let mut response = CallResponse::default();
        response.alkanes = AlkaneTransferParcel(
            [
                oyl_pool_deployment_response.alkanes.0,
                pool_deployment_response.alkanes.0,
            ]
            .concat(),
        );
        Ok(response)
    }
}

impl AlkaneResponder for OylAMMFactory {
    fn execute(&self) -> Result<CallResponse> {
        self.inner.execute()
    }
}

declare_alkane! {OylAMMFactory}
