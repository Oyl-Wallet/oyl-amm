use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    cellpack::Cellpack,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::{overflow_error, shift, shift_id_or_err, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_ptr},
    index_pointer::KeyValuePointer,
    utils::{consume_sized_int, consume_u128},
};
use ruint::Uint;
use std::sync::Arc;

// per uniswap docs, the first 1e3 wei of lp token minted are burned to mitigate attacks where the value of a lp token is raised too high easily
pub const MINIMUM_LIQUIDITY: u128 = 1000;

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

    fn get_pool_for(&self, alkane1: AlkaneId, alkane2: AlkaneId) -> Result<AlkaneId> {
        let factory = Self::factory()?;
        let response = self.call(
            &Cellpack {
                target: factory,
                inputs: vec![1, alkane1.block, alkane1.tx, alkane2.block, alkane2.tx],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        )?;
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(response.data);
        //wrote this block with an angle for creating the pool here if it didnt find one,
        let pool = AlkaneId::new(consume_u128(&mut cursor)?, consume_u128(&mut cursor)?);
        Ok(pool)
    }

    fn add_liquidity(pool: AlkaneId, amount1: u128, amount2: u128) -> Result<(u128, u128)> {
        Ok((amount1, amount2))
    }

    fn remove_liquidity(pool: AlkaneId, amount1: u128, amount2: u128) -> Result<(u128, u128)> {
        Ok((amount1, amount2))
    }

    fn swap(pool: AlkaneId, token: AlkaneId, amount: u128, amountOutMin: u128) -> Result<(u128)> {
        Ok(amountOutMin)
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
                    let id = shift_id_or_err(&mut inputs)?;
                    factory.set(Arc::new(id.into()));
                    pointer.set(Arc::new(vec![0x01]));
                    //placeholder
                    Ok(CallResponse::default())
                } else {
                    Err(anyhow!("already initialized"))
                }
            }
            1..3 => {
                let response = CallResponse::default();
                let (alkane1, alkane2) =
                    (shift_id_or_err(&mut inputs)?, shift_id_or_err(&mut inputs)?);
                match opcode {
                    1 => {
                        //add_liquidity
                        let (amount1, amount2) =
                            (shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?);
                        Self::add_liquidity(
                            self.get_pool_for(alkane1, alkane2)?,
                            amount1,
                            amount2,
                        )?;
                    }
                    _ => {}
                }
                Ok(response)
            }
            50 => Ok(CallResponse::forward(&context.incoming_alkanes)),

            _ => Err(anyhow!("unrecognized opcode")),
        }
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&AMMRouter::default().run());
    to_ptr(&mut response) + 4
}
