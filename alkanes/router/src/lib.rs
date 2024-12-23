use alkanes_runtime::{runtime::AlkaneResponder, storage::StoragePointer};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::{overflow_error, shift, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::{
    compat::{to_arraybuffer_layout, to_ptr},
    index_pointer::KeyValuePointer,
};
use num::integer::Roots;
use protorune_support::balance_sheet::BalanceSheet;
use ruint::Uint;
use std::sync::Arc;

// per uniswap docs, the first 1e3 wei of lp token minted are burned to mitigate attacks where the value of a lp token is raised too high easily
pub const MINIMUM_LIQUIDITY: u128 = 1000;

type U256 = Uint<256, 4>;

#[derive(Default)]
struct AMMRouter(());

impl AMMRouter {}

impl AlkaneResponder for AMMRouter {
    fn execute(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut inputs = context.inputs.clone();
        match shift_or_err(&mut inputs)? {
            0 => {
                let mut pointer = StoragePointer::from_keyword("/initialized");
                if pointer.get().len() == 0 {
                    pointer.set(Arc::new(vec![0x01]));
                    //placeholder
                    Ok(CallResponse::default())
                } else {
                    Err(anyhow!("already initialized"))
                }
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
