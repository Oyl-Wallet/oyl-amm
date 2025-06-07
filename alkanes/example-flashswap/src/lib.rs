use alkanes_runtime::runtime::AlkaneResponder;
use alkanes_runtime::{auth::AuthenticatedResponder, declare_alkane, message::MessageDispatch};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::{AlkaneTransfer, AlkaneTransferParcel};
use alkanes_support::response::CallResponse;
use anyhow::Result;
use metashrew_support::compat::{to_arraybuffer_layout, to_passback_ptr};
use oylswap_library::PoolInfo;

#[derive(Default)]
pub struct FlashSwap(());

impl AuthenticatedResponder for FlashSwap {}

#[derive(MessageDispatch)]
enum FlashSwapMessage {
    #[opcode(0)]
    Initialize {},

    #[opcode(1)]
    NoRefund {},

    #[opcode(10)]
    StaticAttack { pool: AlkaneId, liquidity: u128 },

    #[opcode(73776170)]
    Callback {
        caller: AlkaneId,
        amount_0_out: u128,
        amount_1_out: u128,
        data: Vec<u128>,
    },
}

impl FlashSwap {
    fn initialize(&self) -> Result<CallResponse> {
        self.observe_initialization()?;
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes);

        Ok(response)
    }

    fn no_refund(&self) -> Result<CallResponse> {
        let response: CallResponse = CallResponse::default();

        Ok(response)
    }

    fn refund(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::forward(&context.incoming_alkanes);
        Ok(response)
    }

    fn arb_call(&self, data: Vec<u128>) -> Result<CallResponse> {
        let context = self.context()?;
        self.call(
            &Cellpack {
                target: AlkaneId {
                    block: data[0],
                    tx: data[1],
                },
                inputs: data[2..].to_vec(),
            },
            &context.incoming_alkanes,
            self.fuel(),
        )
    }

    fn _get_pool_info(&self, pool: AlkaneId) -> Result<PoolInfo> {
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![999],
        };
        let response = self.call(&cellpack, &AlkaneTransferParcel(vec![]), self.fuel())?;
        Ok(PoolInfo::from_vec(&response.data)?)
    }

    fn static_attack(&self, pool: AlkaneId, liquidity: u128) -> Result<CallResponse> {
        let context = self.context()?;
        let pool_info = self._get_pool_info(pool)?;
        println!(
            "pool token balance start: {:?}",
            self.balance(&context.myself, &pool)
        );
        println!(
            "token a balance start: {:?}",
            self.balance(&context.myself, &pool_info.token_a)
        );
        println!(
            "token b balance start: {:?}",
            self.balance(&context.myself, &pool_info.token_b)
        );
        let input_transfer = AlkaneTransferParcel(vec![AlkaneTransfer {
            id: pool,
            value: liquidity,
        }]);
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![2],
        };
        let response = self.staticcall(&cellpack, &input_transfer, self.fuel());
        println!("call response: {:?}", response);
        println!(
            "pool token balance end: {:?}",
            self.balance(&context.myself, &pool)
        );
        println!(
            "token a balance end: {:?}",
            self.balance(&context.myself, &pool_info.token_a)
        );
        println!(
            "token b balance end: {:?}",
            self.balance(&context.myself, &pool_info.token_b)
        );

        response
    }

    fn callback(
        &self,
        caller: AlkaneId,
        amount_0_out: u128,
        amount_1_out: u128,
        data: Vec<u128>,
    ) -> Result<CallResponse> {
        if data[0] == 0 {
            self.no_refund()
        } else if data[0] == 1 {
            self.refund()
        } else {
            self.arb_call(data)
        }
    }
}

impl AlkaneResponder for FlashSwap {}

// Use the new macro format
declare_alkane! {
    impl AlkaneResponder for FlashSwap {
        type Message = FlashSwapMessage;
    }
}
