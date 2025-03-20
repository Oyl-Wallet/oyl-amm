use alkanes_runtime::storage::StoragePointer;
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    context::Context, id::AlkaneId, parcel::AlkaneTransfer, response::CallResponse,
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    byte_view::ByteView,
    index_pointer::KeyValuePointer,
    utils::{consume_sized_int, consume_u128},
};
use std::sync::Arc;

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

pub trait AMMFactoryBase {
    fn pool_id(&self) -> Result<u128> {
        let ptr = StoragePointer::from_keyword("/pool_factory_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(consume_u128(&mut cursor)?)
    }

    fn pool_pointer(&self, a: &AlkaneId, b: &AlkaneId) -> StoragePointer {
        StoragePointer::from_keyword("/pools/")
            .select(&a.clone().into())
            .keyword("/")
            .select(&b.clone().into())
    }
    fn _pull_incoming(&self, context: &mut Context) -> Option<AlkaneTransfer> {
        let i = context
            .incoming_alkanes
            .0
            .iter()
            .position(|v| v.id == context.myself)?;
        Some(context.incoming_alkanes.0.remove(i))
    }
    fn _only_owner(&self, v: Option<AlkaneTransfer>) -> Result<()> {
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
    fn init_factory(&self, pool_factory_id: u128, context: Context) -> Result<CallResponse> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        let mut pool_factory_id_pointer = StoragePointer::from_keyword("/pool_factory_id");
        if pointer.get().len() == 0 {
            pointer.set(Arc::new(vec![0x01]));
            // set the address for the implementation for AMM pool
            pool_factory_id_pointer.set(Arc::new(pool_factory_id.to_bytes()));
            Ok(CallResponse::forward(&context.incoming_alkanes.clone()))
        } else {
            Err(anyhow!("already initialized"))
        }
    }
    fn create_new_pool(&self, context: Context) -> Result<CallResponse>;

    fn find_existing_pool_id(&self, alkane_a: AlkaneId, alkane_b: AlkaneId) -> AlkaneId {
        let (a, b) = sort_alkanes((alkane_a, alkane_b));
        let mut cursor =
            std::io::Cursor::<Vec<u8>>::new(self.pool_pointer(&a, &b).get().as_ref().clone());
        AlkaneId::new(
            consume_sized_int::<u128>(&mut cursor).unwrap(),
            consume_sized_int::<u128>(&mut cursor).unwrap(),
        )
    }
    // Get the total number of pools
    fn all_pools_length(&self) -> Result<u128> {
        let ptr = StoragePointer::from_keyword("/all_pools_length")
            .get()
            .as_ref()
            .clone();

        if ptr.len() == 0 {
            return Ok(0);
        }

        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(consume_u128(&mut cursor)?)
    }

    // Get a pool by index
    fn all_pools(&self, index: u128) -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/all_pools/")
            .select(&index.to_le_bytes().to_vec())
            .get()
            .as_ref()
            .clone();

        if ptr.len() == 0 {
            return Err(anyhow!("pool not found at index {}", index));
        }

        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_sized_int::<u128>(&mut cursor)?,
            consume_sized_int::<u128>(&mut cursor)?,
        ))
    }

    // Get all pools (returns a list of pool IDs)
    fn get_all_pools(&self, context: Context) -> Result<CallResponse> {
        let length = self.all_pools_length()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        let mut all_pools_data = Vec::new();

        // Add the total count as the first element
        all_pools_data.extend_from_slice(&length.to_le_bytes());

        for i in 0..length {
            match self.all_pools(i) {
                Ok(pool_id) => {
                    all_pools_data.extend_from_slice(&pool_id.block.to_le_bytes());
                    all_pools_data.extend_from_slice(&pool_id.tx.to_le_bytes());
                }
                Err(_) => {
                    continue;
                }
            }
        }

        response.data = all_pools_data;
        Ok(response)
    }
}
