use alkanes_runtime::storage::StoragePointer;
#[allow(unused_imports)]
use alkanes_runtime::{
    auth::AuthenticatedResponder,
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{context::Context, id::AlkaneId, response::CallResponse};
use anyhow::{anyhow, Result};
use metashrew_support::{
    index_pointer::KeyValuePointer,
    utils::{consume_sized_int, consume_u128},
};
use std::sync::Arc;

pub trait AMMPathProviderBase: AuthenticatedResponder {
    fn init_path_provider(&self, context: Context) -> Result<CallResponse> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set(Arc::new(vec![0x01]));
            let auth_token = self.deploy_auth_token(1u128)?;
            let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
            response.alkanes.pay(auth_token);
            Ok(response)
        } else {
            Err(anyhow!("already initialized"))
        }
    }

    fn path(&self, alkane_a: &AlkaneId, alkane_b: &AlkaneId) -> StoragePointer {
        StoragePointer::from_keyword("/path")
            .select(&alkane_a.into())
            .select(&alkane_b.into())
    }
    fn get_path(&self, alkane_a: &AlkaneId, alkane_b: &AlkaneId) -> Vec<AlkaneId> {
        let data = self.path_bytes(alkane_a, alkane_b);
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(data);
        let mut path = Vec::new();
        while let (Ok(block), Ok(tx)) = (consume_u128(&mut cursor), consume_u128(&mut cursor)) {
            path.push(AlkaneId { block, tx });
        }
        path
    }

    fn path_bytes(&self, alkane_a: &AlkaneId, alkane_b: &AlkaneId) -> Vec<u8> {
        let data = self.path(alkane_a, alkane_b).get();
        let path = data.to_vec();
        path
    }

    fn set_optimal_path(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        optimal_path: Vec<AlkaneId>,
    ) -> Result<()> {
        self.only_owner()?;
        let data: Vec<u8> = optimal_path
            .into_iter()
            .map(|alkane| Into::<Vec<u8>>::into(&alkane))
            .flatten()
            .collect::<_>();

        self.path(&alkane_a, &alkane_b).set(Arc::from(data));
        Ok(())
    }
}
