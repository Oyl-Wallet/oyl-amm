#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer};
use alkanes_support::{
    cellpack::Cellpack,
    constants::AMM_FACTORY_ID,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::shift_or_err,
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_ptr},
    index_pointer::KeyValuePointer,
    utils::consume_sized_int,
};
use std::sync::Arc;

#[derive(Default)]
struct AMMFactory(());

pub fn take_two<T: Clone>(v: &Vec<T>) -> (T, T) {
    (v[0].clone(), v[1].clone())
}

pub fn sort_alkanes((a, b): (AlkaneId, AlkaneId)) -> (AlkaneId, AlkaneId) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

pub fn join_ids(a: AlkaneId, b: AlkaneId) -> Vec<u8> {
    let mut result: Vec<u8> = a.into();
    let value: Vec<u8> = b.into();
    result.extend_from_slice(&value);
    result
}

pub fn join_ids_from_tuple(v: (AlkaneId, AlkaneId)) -> Vec<u8> {
    join_ids(v.0, v.1)
}

impl AMMFactory {
    pub fn pool_pointer(&self, a: &AlkaneId, b: &AlkaneId) -> StoragePointer {
        StoragePointer::from_keyword("/pools/")
            .select(&a.clone().into())
            .keyword("/")
            .select(&b.clone().into())
    }
    pub fn _pull_incoming(&self, context: &mut Context) -> Option<AlkaneTransfer> {
        let i = context
            .incoming_alkanes
            .0
            .iter()
            .position(|v| v.id == context.myself)?;
        Some(context.incoming_alkanes.0.remove(i))
    }
    pub fn _only_owner(&self, v: Option<AlkaneTransfer>) -> Result<()> {
        if let Some(auth) = v {
            if auth.value < 1 {
                Err(anyhow!(
                    "must spend a balance of this alkane to the alkane to use as a proxy"
                ))
            } else {
                Ok(())
            }
        } else {
            Err(anyhow!(
                "must spend a balance of this alkane to the alkane to use as a proxy"
            ))
        }
    }
}

impl AlkaneResponder for AMMFactory {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        match shift_or_err(&mut inputs)? {
            0 => {
                let mut pointer = StoragePointer::from_keyword("/initialized");
                if pointer.get().len() == 0 {
                    pointer.set(Arc::new(vec![0x01]));
                    Ok(CallResponse::default())
                } else {
                    Err(anyhow!("already initialized"))
                }
            }
            1 => {
                if context.incoming_alkanes.0.len() != 2 {
                    panic!("must send two runes to initialize a pool");
                } else {
                    let (alkane_a, alkane_b) = take_two(&context.incoming_alkanes.0);
                    let (a, b) = sort_alkanes((alkane_a.id.clone(), alkane_b.id.clone()));
                    let next_sequence = self.sequence();
                    self.pool_pointer(&a, &b)
                        .set(Arc::new(AlkaneId::new(2, next_sequence).into()));
                    self.call(
                        &Cellpack {
                            target: AlkaneId {
                                block: 6,
                                tx: AMM_FACTORY_ID,
                            },
                            inputs: vec![0, a.block, a.tx, b.block, b.tx],
                        },
                        &AlkaneTransferParcel(vec![
                            context.incoming_alkanes.0[0].clone(),
                            context.incoming_alkanes.0[1].clone(),
                        ]),
                        self.fuel(),
                    )
                }
            }
            2 => {
                let mut response = CallResponse::default();
                response.alkanes = context.incoming_alkanes.clone();
                let (alkane_a, alkane_b) = (
                    AlkaneId::new(shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?),
                    AlkaneId::new(shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?),
                );
                let (a, b) = sort_alkanes((alkane_a, alkane_b));
                let mut cursor = std::io::Cursor::<Vec<u8>>::new(
                    self.pool_pointer(&a, &b).get().as_ref().clone(),
                );
                let id = AlkaneId::new(
                    consume_sized_int::<u128>(&mut cursor).unwrap(),
                    consume_sized_int::<u128>(&mut cursor).unwrap(),
                );
                response.data = id.into();
                Ok(response)
            }
            _ => Err(anyhow!("unrecognized opcode")),
        }
    }
}

#[no_mangle]
pub extern "C" fn __execute() -> i32 {
    let mut response = to_arraybuffer_layout(&AMMFactory::default().run());
    to_ptr(&mut response) + 4
}
