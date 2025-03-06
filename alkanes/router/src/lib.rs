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
}

impl AlkaneResponder for AMMRouter {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        let opcode = shift_or_err(&mut inputs)?;
        match opcode {
            0 => {
                let mut pointer = StoragePointer::from_keyword("/initialized");
                let mut factory = StoragePointer::from_keyword("/factory");
                if pointer.get().len() == 0 {
                    let factory_id =
                        AlkaneId::new(shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?);
                    factory.set(Arc::new(factory_id.into()));
                    pointer.set(Arc::new(vec![0x01]));
                    //placeholder
                    Ok(CallResponse::forward(&context.incoming_alkanes.clone()))
                } else {
                    Err(anyhow!("already initialized"))
                }
            }
            1..3 => {
                // add and remove liquidity
                let token1 = AlkaneId::new(shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?);
                let token2 = AlkaneId::new(shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?);

                let pool = self.get_pool_for(&token1, &token2)?;
                let cellpack = Cellpack {
                    target: pool,
                    inputs: vec![opcode],
                };
                let response = self.call(&cellpack, &context.incoming_alkanes, self.fuel())?;
                Ok(response)
            }
            3 => {
                // swap
                let num_alkanes_in_path: usize = shift_or_err(&mut inputs)? as usize;
                if num_alkanes_in_path < 2 {
                    return Err(anyhow!("Routing path must be at least two alkanes long"));
                }
                let mut path: Vec<AlkaneId> = vec![];
                for _ in 0..num_alkanes_in_path {
                    path.push(AlkaneId::new(
                        shift_or_err(&mut inputs)?,
                        shift_or_err(&mut inputs)?,
                    ));
                }
                let amount = shift_or_err(&mut inputs)?;
                let mut this_response = CallResponse {
                    alkanes: context.incoming_alkanes.clone(),
                    data: vec![],
                };

                for i in 1..num_alkanes_in_path {
                    let pool = self.get_pool_for(&path[i - 1], &path[i])?;
                    let this_amount = if i == num_alkanes_in_path - 1 {
                        amount
                    } else {
                        0
                    };
                    let cellpack = Cellpack {
                        target: pool,
                        inputs: vec![opcode, this_amount],
                    };
                    this_response = self.call(&cellpack, &this_response.alkanes, self.fuel())?;
                    println!("This response for pair {}: {:?}", i, this_response);
                }

                Ok(this_response)
            }
            4 => self.get_all_pools(),
            50 => Ok(CallResponse::forward(&context.incoming_alkanes)),

            _ => Err(anyhow!("unrecognized opcode {}", opcode)),
        }
    }
}

declare_alkane! {AMMRouter}
