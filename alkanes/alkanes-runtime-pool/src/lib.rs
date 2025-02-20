use alkanes_runtime::{ runtime::AlkaneResponder, storage::StoragePointer };

#[allow(unused_imports)]
use alkanes_runtime::{ println, stdio::{ stdout, Write } };
use alkanes_support::{
    context::Context,
    id::AlkaneId,
    parcel::{ AlkaneTransfer, AlkaneTransferParcel },
    response::CallResponse,
    utils::{ overflow_error, shift, shift_or_err },
};
use anyhow::{ anyhow, Result };
use metashrew_support::index_pointer::KeyValuePointer;
use num::integer::Roots;
use protorune_support::balance_sheet::BalanceSheet;
use ruint::Uint;
use std::sync::Arc;

// per uniswap docs, the first 1e3 wei of lp token minted are burned to mitigate attacks where the value of a lp token is raised too high easily
pub const MINIMUM_LIQUIDITY: u128 = 1000;
pub const DEFAULT_FEE_AMOUNT_PER_1000: u128 = 4;

type U256 = Uint<256, 4>;

#[derive(Default)]
pub struct PoolInfo {
    pub token_a: AlkaneId,
    pub token_b: AlkaneId,
    pub reserve_a: u128,
    pub reserve_b: u128,
}

pub fn to_bytes(alkane: AlkaneId) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend(&alkane.block.to_le_bytes());
    bytes.extend(&alkane.tx.to_le_bytes());
    bytes
}

impl PoolInfo {
    pub fn try_to_vec(&self) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();

        bytes.extend_from_slice(&to_bytes(self.token_a));

        // Add token_b bytes
        bytes.extend_from_slice(&to_bytes(self.token_b));

        // Add reserve_a bytes (u128 -> 16 bytes)
        bytes.extend_from_slice(&self.reserve_a.to_le_bytes());

        // Add reserve_b bytes (u128 -> 16 bytes)
        bytes.extend_from_slice(&self.reserve_b.to_le_bytes());

        Ok(bytes)
    }
}

pub trait AMMPoolBase {
    fn init_pool(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        context: Context
    ) -> Result<CallResponse> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set(Arc::new(vec![0x01]));
            StoragePointer::from_keyword("/alkane/0").set(Arc::new(alkane_a.into()));
            StoragePointer::from_keyword("/alkane/1").set(Arc::new(alkane_b.into()));
            self.mint(context.myself, context.incoming_alkanes)
        } else {
            Err(anyhow!("already initialized"))
        }
    }
    fn process_inputs_and_init_pool(
        &self,
        mut inputs: Vec<u128>,
        context: Context
    ) -> Result<CallResponse> {
        let (a, b) = self.pull_ids_or_err(&mut inputs)?;
        self.init_pool(a, b, context)
    }
    fn alkanes_for_self(&self) -> Result<(AlkaneId, AlkaneId)> {
        Ok((
            StoragePointer::from_keyword("/alkane/0").get().as_ref().clone().try_into()?,
            StoragePointer::from_keyword("/alkane/1").get().as_ref().clone().try_into()?,
        ))
    }
    fn check_inputs(
        &self,
        myself: &AlkaneId,
        parcel: &AlkaneTransferParcel,
        n: usize
    ) -> Result<()> {
        if parcel.0.len() > n {
            Err(anyhow!(format!("{} alkanes sent but maximum {} supported", parcel.0.len(), n)))
        } else {
            let (a, b) = self.alkanes_for_self()?;
            if let Some(_) = parcel.0.iter().find(|v| myself != &v.id && v.id != a && v.id != b) {
                Err(anyhow!("unsupported alkane sent to pool"))
            } else {
                Ok(())
            }
        }
    }
    fn total_supply(&self) -> u128 {
        StoragePointer::from_keyword("/totalsupply").get_value::<u128>()
    }
    fn set_total_supply(&self, v: u128) {
        StoragePointer::from_keyword("/totalsupply").set_value::<u128>(v);
    }
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer);
    fn previous_reserves(&self, parcel: &AlkaneTransferParcel) -> (AlkaneTransfer, AlkaneTransfer) {
        let (reserve_a, reserve_b) = self.reserves();
        let mut reserve_sheet: BalanceSheet = AlkaneTransferParcel(
            vec![reserve_a.clone(), reserve_b.clone()]
        ).into();
        let incoming_sheet: BalanceSheet = parcel.clone().into();
        reserve_sheet.debit(&incoming_sheet).unwrap();
        (
            AlkaneTransfer {
                id: reserve_a.id.clone(),
                value: reserve_sheet.get(&reserve_a.id.clone().into()),
            },
            AlkaneTransfer {
                id: reserve_b.id.clone(),
                value: reserve_sheet.get(&reserve_b.id.clone().into()),
            },
        )
    }

    fn pool_details(&self) -> Result<CallResponse> {
        let (reserve_a, reserve_b) = self.reserves();
        let (token_a, token_b) = self.alkanes_for_self().unwrap();
        let pool_info = PoolInfo {
            token_a,
            token_b,
            reserve_a: reserve_a.value,
            reserve_b: reserve_b.value,
        };

        let mut response = CallResponse::default();
        response.data = pool_info.try_to_vec()?;

        Ok(response)
    }

    fn mint(&self, myself: AlkaneId, parcel: AlkaneTransferParcel) -> Result<CallResponse> {
        self.check_inputs(&myself, &parcel, 2)?;
        let mut total_supply = self.total_supply();
        let (reserve_a, reserve_b) = self.reserves();
        let (previous_a, previous_b) = self.previous_reserves(&parcel);
        let root_k_last = overflow_error(previous_a.value.checked_mul(previous_b.value))?.sqrt();
        let root_k = overflow_error(reserve_a.value.checked_mul(reserve_b.value))?.sqrt();
        if root_k > root_k_last || root_k_last == 0 {
            let liquidity;
            if total_supply == 0 {
                liquidity = overflow_error(root_k.checked_sub(MINIMUM_LIQUIDITY))?;
                total_supply = total_supply + MINIMUM_LIQUIDITY;
            } else {
                let numerator = overflow_error(
                    total_supply.checked_mul(overflow_error(root_k.checked_sub(root_k_last))?)
                )?;
                let denominator = overflow_error(
                    overflow_error(root_k.checked_mul(5))?.checked_add(root_k_last) // constant 5 is assuming 1/6 of LP fees goes as protocol fees
                )?;
                liquidity = numerator / denominator;
            }
            self.set_total_supply(overflow_error(total_supply.checked_add(liquidity))?);
            let mut response = CallResponse::default();
            response.alkanes = AlkaneTransferParcel(
                vec![AlkaneTransfer {
                    id: myself,
                    value: liquidity,
                }]
            );
            Ok(response)
        } else {
            Err(anyhow!("root k is less than previous root k"))
        }
    }
    fn burn(&self, myself: AlkaneId, parcel: AlkaneTransferParcel) -> Result<CallResponse> {
        self.check_inputs(&myself, &parcel, 1)?;
        let incoming = parcel.0[0].clone();
        if incoming.id != myself {
            return Err(anyhow!("can only burn LP alkane for this pair"));
        }
        let liquidity = incoming.value;
        let (reserve_a, reserve_b) = self.reserves();
        let total_supply = self.total_supply();
        let mut response = CallResponse::default();
        let amount_a = overflow_error(liquidity.checked_mul(reserve_a.value))? / total_supply;
        let amount_b = overflow_error(liquidity.checked_mul(reserve_b.value))? / total_supply;
        if amount_a == 0 || amount_b == 0 {
            return Err(anyhow!("insufficient liquidity!"));
        }
        self.set_total_supply(overflow_error(total_supply.checked_sub(liquidity))?);
        response.alkanes = AlkaneTransferParcel(
            vec![
                AlkaneTransfer {
                    id: reserve_a.id,
                    value: amount_a,
                },
                AlkaneTransfer {
                    id: reserve_b.id,
                    value: amount_b,
                }
            ]
        );
        Ok(response)
    }
    fn get_amount_out(&self, amount: u128, reserve_from: u128, reserve_to: u128) -> Result<u128> {
        let amount_in_with_fee =
            U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(amount);
        let numerator = amount_in_with_fee * U256::from(reserve_to);
        let denominator = U256::from(1000) * U256::from(reserve_from) + amount_in_with_fee;
        Ok((numerator / denominator).try_into()?)
    }
    fn simulate_amount_out(&self, mut inputs: Vec<u128>) -> Result<CallResponse> {
        let token: AlkaneId = AlkaneId::new(shift_or_err(&mut inputs)?, shift_or_err(&mut inputs)?);
        let amount: u128 = shift_or_err(&mut inputs)?;
        let input = AlkaneTransferParcel(
            vec![AlkaneTransfer {
                id: token,
                value: amount,
            }]
        );
        let (previous_a, previous_b) = self.previous_reserves(&input);

        println!("previous a {}", previous_a.value);
        println!("previous b{}", previous_b.value);

        let amount_in_with_fee =
            U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(amount);

        println!("{}", amount_in_with_fee);

        let mut response = CallResponse::default();

        if &token == &previous_a.id {
            println!("passed token a");
            let numerator = amount_in_with_fee * U256::from(previous_b.value);
            let denominator = U256::from(1000) * U256::from(previous_a.value) + amount_in_with_fee;
            response.data = (numerator / denominator).to_le_bytes_vec();
            return Ok(response);
        } else {
            println!("passed token b");
            let numerator = amount_in_with_fee * U256::from(previous_a.value);
            let denominator = U256::from(1000) * U256::from(previous_b.value) + amount_in_with_fee;
            response.data = (numerator / denominator).to_le_bytes_vec();
            return Ok(response);
        }
    }
    fn swap(
        &self,
        parcel: AlkaneTransferParcel,
        amount_out_predicate: u128
    ) -> Result<CallResponse> {
        if parcel.0.len() != 1 {
            return Err(
                anyhow!(format!("payload can only include 1 alkane, sent {}", parcel.0.len()))
            );
        }
        let transfer = parcel.0[0].clone();
        let (previous_a, previous_b) = self.previous_reserves(&parcel);
        let (reserve_a, reserve_b) = self.reserves();

        println!(
            "amount out for b {}",
            self.get_amount_out(transfer.value, previous_a.value, previous_b.value)?
        );
        println!(
            "amount out fot a, {}",
            self.get_amount_out(transfer.value, previous_b.value, previous_a.value)?
        );
        let output = if &transfer.id == &reserve_a.id {
            AlkaneTransfer {
                id: reserve_b.id,
                value: self.get_amount_out(transfer.value, previous_a.value, previous_b.value)?,
            }
        } else {
            AlkaneTransfer {
                id: reserve_a.id,
                value: self.get_amount_out(transfer.value, previous_b.value, previous_a.value)?,
            }
        };
        if output.value < amount_out_predicate {
            return Err(anyhow!("predicate failed: insufficient output"));
        }
        let mut response = CallResponse::default();
        response.alkanes = AlkaneTransferParcel(vec![output]);
        Ok(response)
    }
    fn pull_ids(&self, v: &mut Vec<u128>) -> Option<(AlkaneId, AlkaneId)> {
        let a_block = shift(v)?;
        let a_tx = shift(v)?;
        let b_block = shift(v)?;
        let b_tx = shift(v)?;
        Some((AlkaneId::new(a_block, a_tx), AlkaneId::new(b_block, b_tx)))
    }
    fn pull_ids_or_err(&self, v: &mut Vec<u128>) -> Result<(AlkaneId, AlkaneId)> {
        self.pull_ids(v)
            .ok_or("")
            .map_err(|_| anyhow!("AlkaneId values for pair missing from list"))
    }
}

pub struct AMMPool {
    delegate: Option<Box<dyn AMMPoolBase>>,
}

impl Clone for AMMPool {
    fn clone(&self) -> Self {
        AMMPool { delegate: None }
    }
}

impl AMMPool {
    pub fn default() -> Self {
        let mut pool = AMMPool { delegate: None };
        pool.set_delegate(Box::new(pool.clone()));
        pool
    }

    pub fn set_delegate(&mut self, delegate: Box<dyn AMMPoolBase>) {
        self.delegate = Some(delegate);
    }
}

pub trait AMMReserves: AlkaneResponder + AMMPoolBase {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        let (a, b) = self.alkanes_for_self().unwrap();
        let context = self.context().unwrap();
        (
            AlkaneTransfer {
                id: a,
                value: self.balance(&context.myself, &a),
            },
            AlkaneTransfer {
                id: b,
                value: self.balance(&context.myself, &b),
            },
        )
    }
}

impl AMMReserves for AMMPool {}
impl AMMPoolBase for AMMPool {
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer) {
        AMMReserves::reserves(self)
    }
}

impl AlkaneResponder for AMMPool {
    fn execute(&self) -> Result<CallResponse> {
        if let Some(delegate) = &self.delegate {
            let context = self.context()?;
            let mut inputs = context.inputs.clone();
            match shift_or_err(&mut inputs)? {
                0 => delegate.process_inputs_and_init_pool(inputs, context),
                1 => delegate.mint(context.myself, context.incoming_alkanes),
                2 => delegate.burn(context.myself, context.incoming_alkanes),
                3 => delegate.swap(context.incoming_alkanes, shift_or_err(&mut inputs)?),
                4 => delegate.simulate_amount_out(inputs),
                5 => delegate.pool_details(),
                50 => Ok(CallResponse::forward(&context.incoming_alkanes)),

                _ => Err(anyhow!("unrecognized opcode")),
            }
        } else {
            Err(anyhow!("No delegate set"))
        }
    }
}
