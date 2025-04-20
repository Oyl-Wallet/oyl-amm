use alkanes_runtime::{
    message::MessageDispatch, runtime::AlkaneResponder, storage::StoragePointer,
};

#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_std_factory_support::MintableToken;
use alkanes_support::{
    cellpack::Cellpack,
    checked_expr,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
    utils::{overflow_error, shift, shift_or_err},
};
use anyhow::{anyhow, Result};
use metashrew_support::index_pointer::KeyValuePointer;
use num::integer::Roots;
use protorune_support::balance_sheet::{BalanceSheetOperations, CachedBalanceSheet};
use ruint::Uint;
use std::sync::Arc;

// per uniswap docs, the first 1e3 wei of lp token minted are burned to mitigate attacks where the value of a lp token is raised too high easily
pub const MINIMUM_LIQUIDITY: u128 = 1000;
pub const DEFAULT_FEE_AMOUNT_PER_1000: u128 = 5;

type U256 = Uint<256, 4>;

#[derive(Default)]
pub struct PoolInfo {
    pub token_a: AlkaneId,
    pub token_b: AlkaneId,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub pool_name: String,
}

impl PoolInfo {
    pub fn try_to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        let mut token_a_bytes: Vec<u8> = Vec::with_capacity(32);
        token_a_bytes.extend(&self.token_a.block.to_le_bytes());
        token_a_bytes.extend(&self.token_a.tx.to_le_bytes());

        let mut token_b_bytes: Vec<u8> = Vec::with_capacity(32);
        token_b_bytes.extend(&self.token_b.block.to_le_bytes());
        token_b_bytes.extend(&self.token_b.tx.to_le_bytes());

        bytes.extend_from_slice(&token_a_bytes);

        bytes.extend_from_slice(&token_b_bytes);

        bytes.extend_from_slice(&self.reserve_a.to_le_bytes());

        bytes.extend_from_slice(&self.reserve_b.to_le_bytes());
        bytes.extend_from_slice(&self.total_supply.to_le_bytes());

        // Add the pool name
        let name_bytes = self.pool_name.as_bytes();
        // Add the length of the name as a u32
        bytes.extend_from_slice(&(name_bytes.len() as u32).to_le_bytes());
        // Add the name bytes
        bytes.extend_from_slice(name_bytes);

        bytes
    }
}

pub trait AMMPoolBase: MintableToken {
    fn init_pool(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        context: Context,
    ) -> Result<CallResponse> {
        let mut pointer = StoragePointer::from_keyword("/initialized");
        if pointer.get().len() == 0 {
            pointer.set(Arc::new(vec![0x01]));
            StoragePointer::from_keyword("/alkane/0").set(Arc::new(alkane_a.into()));
            StoragePointer::from_keyword("/alkane/1").set(Arc::new(alkane_b.into()));
            self.add_liquidity(context.myself, context.incoming_alkanes)
        } else {
            Err(anyhow!("already initialized"))
        }
    }
    fn alkanes_for_self(&self) -> Result<(AlkaneId, AlkaneId)> {
        Ok((
            StoragePointer::from_keyword("/alkane/0")
                .get()
                .as_ref()
                .clone()
                .try_into()?,
            StoragePointer::from_keyword("/alkane/1")
                .get()
                .as_ref()
                .clone()
                .try_into()?,
        ))
    }
    fn check_inputs(
        &self,
        myself: &AlkaneId,
        parcel: &AlkaneTransferParcel,
        n: usize,
    ) -> Result<()> {
        if parcel.0.len() > n {
            Err(anyhow!(format!(
                "{} alkanes sent but maximum {} supported",
                parcel.0.len(),
                n
            )))
        } else {
            let (a, b) = self.alkanes_for_self()?;
            if let Some(_) = parcel
                .0
                .iter()
                .find(|v| myself != &v.id && v.id != a && v.id != b)
            {
                Err(anyhow!("unsupported alkane sent to pool"))
            } else {
                Ok(())
            }
        }
    }
    fn reserves(&self) -> (AlkaneTransfer, AlkaneTransfer);
    fn previous_reserves(
        &self,
        parcel: &AlkaneTransferParcel,
    ) -> Result<(AlkaneTransfer, AlkaneTransfer)> {
        let (reserve_a, reserve_b) = self.reserves();
        let incoming_sheet: CachedBalanceSheet = parcel.clone().into();
        Ok((
            AlkaneTransfer {
                id: reserve_a.id.clone(),
                value: reserve_a.value - incoming_sheet.get(&reserve_a.id.into()),
            },
            AlkaneTransfer {
                id: reserve_b.id.clone(),
                value: reserve_b.value - incoming_sheet.get(&reserve_b.id.into()),
            },
        ))
    }

    fn pool_details(&self, context: &Context) -> Result<CallResponse> {
        println!("in pool details");
        let (reserve_a, reserve_b) = self.previous_reserves(&context.incoming_alkanes)?;
        println!("after previous reserves");
        let (token_a, token_b) = self.alkanes_for_self()?;

        let pool_info = PoolInfo {
            token_a,
            token_b,
            reserve_a: reserve_a.value,
            reserve_b: reserve_b.value,
            total_supply: self.total_supply(),
            pool_name: self.name(),
        };

        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        response.data = pool_info.try_to_vec();

        Ok(response)
    }

    fn add_liquidity(
        &self,
        myself: AlkaneId,
        parcel: AlkaneTransferParcel,
    ) -> Result<CallResponse> {
        self.check_inputs(&myself, &parcel, 2)?;
        let mut total_supply = self.total_supply();
        let (reserve_a, reserve_b) = self.reserves();
        let (previous_a, previous_b) = self.previous_reserves(&parcel)?;
        let root_k_last = checked_expr!(previous_a.value.checked_mul(previous_b.value))?.sqrt();
        let root_k = checked_expr!(reserve_a.value.checked_mul(reserve_b.value))?.sqrt();
        if root_k > root_k_last || root_k_last == 0 {
            let liquidity;
            if total_supply == 0 {
                liquidity = checked_expr!(root_k.checked_sub(MINIMUM_LIQUIDITY))?;
                total_supply = total_supply + MINIMUM_LIQUIDITY;
            } else {
                let root_k_diff = checked_expr!(root_k.checked_sub(root_k_last))?;
                let numerator = checked_expr!(total_supply.checked_mul(root_k_diff))?;
                let root_k_times_5 = checked_expr!(root_k.checked_mul(5))?; // constant 5 is assuming 1/6 of LP fees goes as protocol fees
                let denominator = checked_expr!(root_k_times_5.checked_add(root_k_last))?;
                liquidity = numerator / denominator;
            }
            self.set_total_supply(checked_expr!(total_supply.checked_add(liquidity))?);
            let mut response = CallResponse::default();
            response.alkanes = AlkaneTransferParcel(vec![AlkaneTransfer {
                id: myself,
                value: liquidity,
            }]);
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
        let amount_a = checked_expr!(liquidity.checked_mul(reserve_a.value))? / total_supply;
        let amount_b = checked_expr!(liquidity.checked_mul(reserve_b.value))? / total_supply;
        if amount_a == 0 || amount_b == 0 {
            return Err(anyhow!("insufficient liquidity!"));
        }
        self.set_total_supply(checked_expr!(total_supply.checked_sub(liquidity))?);
        response.alkanes = AlkaneTransferParcel(vec![
            AlkaneTransfer {
                id: reserve_a.id,
                value: amount_a,
            },
            AlkaneTransfer {
                id: reserve_b.id,
                value: amount_b,
            },
        ]);
        Ok(response)
    }
    fn get_amount_out(
        &self,
        amount: u128,
        reserve_from: u128,
        reserve_to: u128,
        use_fees: bool,
    ) -> Result<u128> {
        let amount_in_with_fee = if use_fees {
            U256::from(1000 - DEFAULT_FEE_AMOUNT_PER_1000) * U256::from(amount)
        } else {
            U256::from(1000) * U256::from(amount)
        };

        let numerator = amount_in_with_fee * U256::from(reserve_to);
        let denominator = U256::from(1000) * U256::from(reserve_from) + amount_in_with_fee;
        Ok((numerator / denominator).try_into()?)
    }
    fn get_transfer_out_from_swap(
        &self,
        parcel: AlkaneTransferParcel,
        use_fees: bool,
    ) -> Result<AlkaneTransfer> {
        if parcel.0.len() != 1 {
            return Err(anyhow!(format!(
                "payload can only include 1 alkane, sent {}",
                parcel.0.len()
            )));
        }
        let transfer = parcel.0[0].clone();
        println!("transfer {:?}", transfer);
        let (previous_a, previous_b) = self.previous_reserves(&parcel)?;
        println!("previous {:?} {:?}", previous_a, previous_b);
        let (reserve_a, reserve_b) = self.reserves();
        println!("now {:?} {:?}", reserve_a, reserve_b);

        if &transfer.id == &reserve_a.id {
            Ok(AlkaneTransfer {
                id: reserve_b.id,
                value: self.get_amount_out(
                    transfer.value,
                    previous_a.value,
                    previous_b.value,
                    use_fees,
                )?,
            })
        } else {
            Ok(AlkaneTransfer {
                id: reserve_a.id,
                value: self.get_amount_out(
                    transfer.value,
                    previous_b.value,
                    previous_a.value,
                    use_fees,
                )?,
            })
        }
    }

    fn swap(
        &self,
        parcel: AlkaneTransferParcel,
        amount_out_predicate: u128,
    ) -> Result<CallResponse> {
        let output = self.get_transfer_out_from_swap(parcel, true)?;
        println!("output from swap: {:?}", output);
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
