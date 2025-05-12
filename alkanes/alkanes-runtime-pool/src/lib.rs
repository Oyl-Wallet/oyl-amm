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
use metashrew_support::{index_pointer::KeyValuePointer, utils::consume_u128};
use num::integer::Roots;
use protorune_support::balance_sheet::{BalanceSheetOperations, CachedBalanceSheet};
use ruint::Uint;
use std::{cmp::min, sync::Arc};

// per uniswap docs, the first 1e3 wei of lp token minted are burned to mitigate attacks where the value of a lp token is raised too high easily
pub const MINIMUM_LIQUIDITY: u128 = 1000;
pub const DEFAULT_FEE_AMOUNT_PER_1000: u128 = 5;

type U256 = Uint<256, 4>;
struct Lock {}
impl Lock {
    fn lock_pointer() -> StoragePointer {
        StoragePointer::from_keyword("/lock")
    }
    fn get_lock() -> u128 {
        Lock::lock_pointer().get_value::<u128>()
    }
    fn set_lock(v: u128) {
        Lock::lock_pointer().set_value::<u128>(v);
    }
    fn lock<F>(func: F) -> Result<CallResponse>
    where
        F: FnOnce() -> Result<CallResponse>,
    {
        if Lock::lock_pointer().get().len() != 0 && Lock::get_lock() == 1 {
            return Err(anyhow!("LOCKED"));
        }

        // Locking the resource
        Lock::set_lock(1);

        // Run the function (this is the _ in Solidity)
        let ret = func();

        // Unlocking after function execution
        Lock::set_lock(0);

        ret
    }
}

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

pub trait AMMPoolBase: MintableToken + AlkaneResponder {
    fn factory(&self) -> Result<AlkaneId> {
        let ptr = StoragePointer::from_keyword("/factory_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(AlkaneId::new(
            consume_u128(&mut cursor)?,
            consume_u128(&mut cursor)?,
        ))
    }
    fn set_factory(&self, factory_id: AlkaneId) {
        let mut factory_id_pointer = StoragePointer::from_keyword("/factory_id");
        factory_id_pointer.set(Arc::new(factory_id.into()));
    }
    fn claimable_fees_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/claimablefees")
    }
    fn claimable_fees(&self) -> u128 {
        self.claimable_fees_pointer().get_value::<u128>()
    }
    fn set_claimable_fees(&self, v: u128) {
        self.claimable_fees_pointer().set_value::<u128>(v);
    }
    fn k_last_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/klast")
    }
    fn k_last(&self) -> u128 {
        self.k_last_pointer().get_value::<u128>()
    }
    fn set_k_last(&self, v: u128) {
        self.k_last_pointer().set_value::<u128>(v);
    }
    fn _only_factory_caller(&self) -> Result<()> {
        if self.context()?.caller != self.factory()? {
            return Err(anyhow!("Caller is not factory"));
        }
        Ok(())
    }
    fn init_pool(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
        factory: AlkaneId,
    ) -> Result<CallResponse> {
        self.observe_initialization()?;
        StoragePointer::from_keyword("/alkane/0").set(Arc::new(alkane_a.into()));
        StoragePointer::from_keyword("/alkane/1").set(Arc::new(alkane_b.into()));
        self.set_factory(factory.into());
        let _ = self.set_pool_name_and_symbol();
        self.set_k_last(0);
        self.add_liquidity()
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

    fn set_pool_name_and_symbol(&self) -> Result<()> {
        let (alkane_a, alkane_b) = self.alkanes_for_self()?;

        // Get name for alkane_a
        let name_a = match self.call(
            &Cellpack {
                target: alkane_a,
                inputs: vec![99],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        ) {
            Ok(response) => {
                if response.data.is_empty() {
                    format!("{},{}", alkane_a.block, alkane_a.tx)
                } else {
                    String::from_utf8_lossy(&response.data).to_string()
                }
            }
            Err(_) => format!("{},{}", alkane_a.block, alkane_a.tx),
        };

        // Get name for alkane_b
        let name_b = match self.call(
            &Cellpack {
                target: alkane_b,
                inputs: vec![99],
            },
            &AlkaneTransferParcel(vec![]),
            self.fuel(),
        ) {
            Ok(response) => {
                if response.data.is_empty() {
                    format!("{},{}", alkane_b.block, alkane_b.tx)
                } else {
                    String::from_utf8_lossy(&response.data).to_string()
                }
            }
            Err(_) => format!("{},{}", alkane_b.block, alkane_b.tx),
        };

        // Format the pool name
        let pool_name = format!("{} / {} LP", name_a, name_b);

        // Set the name using MintableToken trait
        MintableToken::name_pointer(self).set(Arc::new(pool_name.into_bytes()));

        Ok(())
    }

    fn reserves(&self) -> Result<(AlkaneTransfer, AlkaneTransfer)> {
        let (a, b) = self.alkanes_for_self()?;
        let context = self.context()?;
        Ok((
            AlkaneTransfer {
                id: a,
                value: self.balance(&context.myself, &a),
            },
            AlkaneTransfer {
                id: b,
                value: self.balance(&context.myself, &b),
            },
        ))
    }
    fn previous_reserves(
        &self,
        parcel: &AlkaneTransferParcel,
    ) -> Result<(AlkaneTransfer, AlkaneTransfer)> {
        let (reserve_a, reserve_b) = self.reserves()?;
        let incoming_sheet: CachedBalanceSheet = parcel.clone().try_into()?;
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

    fn _mint_fee(&self, previous_a: u128, previous_b: u128) -> Result<()> {
        let total_supply = self.total_supply();
        let k_last = self.k_last();
        if k_last != 0 {
            let root_k_last = k_last.sqrt();
            let root_k = checked_expr!(previous_a.checked_mul(previous_b))?.sqrt();
            if (root_k > root_k_last) {
                let numerator = checked_expr!(total_supply.checked_mul(root_k - root_k_last))?;
                let root_k_fee_adj = checked_expr!(root_k.checked_mul(3))? / 2; // assuming 2/5 of 0.5% fee goes to protocol
                let denominator = checked_expr!(root_k_fee_adj.checked_add(root_k_last))?;
                let liquidity = numerator / denominator;
                self.increase_total_supply(liquidity)?;
                self.set_claimable_fees(checked_expr!(self
                    .claimable_fees()
                    .checked_add(liquidity))?);
            }
        }
        Ok(())
    }

    fn collect_fees(&self) -> Result<CallResponse> {
        self._only_factory_caller()?;
        let context = self.context()?;
        let (previous_a, previous_b) = self.previous_reserves(&context.incoming_alkanes)?;
        self._mint_fee(previous_a.value, previous_b.value)?;
        let myself = context.myself;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.alkanes.pay(AlkaneTransfer {
            id: myself,
            value: self.claimable_fees(),
        });
        self.set_claimable_fees(0);
        let new_k = checked_expr!(previous_a.value.checked_mul(previous_b.value))?;
        self.set_k_last(new_k);
        Ok(response)
    }

    fn add_liquidity(&self) -> Result<CallResponse> {
        Lock::lock(|| {
            let context = self.context()?;
            let myself = context.myself;
            let parcel = context.incoming_alkanes.clone();
            self.check_inputs(&myself, &parcel, 2)?;
            let (reserve_a, reserve_b) = self.reserves()?;
            let (previous_a, previous_b) = self.previous_reserves(&parcel)?;
            let (amount_a_in, amount_b_in) = (
                reserve_a.value - previous_a.value,
                reserve_b.value - previous_b.value,
            );
            self._mint_fee(previous_a.value, previous_b.value)?;
            let total_supply = self.total_supply(); // must be defined here since totalSupply can update in _mintFee
            let liquidity;
            if total_supply == 0 {
                let root_k = checked_expr!(amount_a_in.checked_mul(amount_b_in))?.sqrt();
                liquidity = checked_expr!(root_k.checked_sub(MINIMUM_LIQUIDITY))?;
                self.set_total_supply(MINIMUM_LIQUIDITY);
            } else {
                let liquidity_a = checked_expr!(amount_a_in.checked_mul(total_supply))?;
                let liquidity_b = checked_expr!(amount_b_in.checked_mul(total_supply))?;
                liquidity = min(
                    liquidity_a / previous_a.value,
                    liquidity_b / previous_b.value,
                );
            }
            if liquidity == 0 {
                return Err(anyhow!("INSUFFICIENT_LIQUIDITY_MINTED"));
            }
            let mut response = CallResponse::default();
            response.alkanes.pay(self.mint(&context, liquidity)?);
            let new_k = checked_expr!(reserve_a.value.checked_mul(reserve_b.value))?;
            self.set_k_last(new_k);
            Ok(response)
        })
    }
    fn burn(&self) -> Result<CallResponse> {
        Lock::lock(|| {
            let context = self.context()?;
            let myself = context.myself;
            let parcel = context.incoming_alkanes;
            self.check_inputs(&myself, &parcel, 1)?;
            let incoming = parcel.0[0].clone();
            if incoming.id != myself {
                return Err(anyhow!("can only burn LP alkane for this pair"));
            }
            let (previous_a, previous_b) = self.previous_reserves(&parcel)?;
            self._mint_fee(previous_a.value, previous_b.value)?;
            let liquidity = incoming.value;
            let (reserve_a, reserve_b) = self.reserves()?;
            let total_supply = self.total_supply();
            let mut response = CallResponse::default();
            let amount_a = checked_expr!(liquidity.checked_mul(reserve_a.value))? / total_supply;
            let amount_b = checked_expr!(liquidity.checked_mul(reserve_b.value))? / total_supply;
            if amount_a == 0 || amount_b == 0 {
                return Err(anyhow!("INSUFFICIENT_LIQUIDITY_BURNED"));
            }
            self.decrease_total_supply(liquidity)?;
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

            let new_k = checked_expr!(
                (reserve_a.value - amount_a).checked_mul(reserve_b.value - amount_b)
            )?;
            self.set_k_last(new_k);
            Ok(response)
        })
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
        let (previous_a, previous_b) = self.previous_reserves(&parcel)?;
        let (reserve_a, reserve_b) = self.reserves()?;

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

    fn _check_k_increasing(
        &self,
        input_parcel: &AlkaneTransferParcel,
        output_parcel: &AlkaneTransferParcel,
    ) -> Result<()> {
        let (a, b) = self.reserves()?;
        let (previous_a, previous_b) = self.previous_reserves(&input_parcel)?;
        let outgoing_sheet: CachedBalanceSheet = output_parcel.clone().try_into()?;
        let prev_k = U256::from(previous_a.value) * U256::from(previous_b.value);
        let new_a = U256::from(a.value) - U256::from(outgoing_sheet.get(&a.id.into()));
        let new_b = U256::from(b.value) - U256::from(outgoing_sheet.get(&b.id.into()));
        let new_k = new_a * new_b;
        if new_k < prev_k {
            return Err(anyhow!(format!(
                "New k ({}) is not >= prev k ({})",
                new_k, prev_k
            )));
        }
        Ok(())
    }

    fn swap(&self, amount_out_predicate: u128) -> Result<CallResponse> {
        Lock::lock(|| {
            let context = self.context()?;
            let parcel: AlkaneTransferParcel = context.incoming_alkanes;
            let output = self.get_transfer_out_from_swap(parcel.clone(), true)?;
            if output.value < amount_out_predicate {
                return Err(anyhow!("predicate failed: insufficient output"));
            }
            let output_parcel: AlkaneTransferParcel = AlkaneTransferParcel(vec![output]);
            self._check_k_increasing(&parcel, &output_parcel)?;
            let mut response = CallResponse::default();
            response.alkanes = output_parcel;
            Ok(response)
        })
    }

    fn forward_incoming(&self) -> Result<CallResponse> {
        let context = self.context()?;
        Ok(CallResponse::forward(&context.incoming_alkanes))
    }

    fn get_name(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        response.data = self.name().into_bytes().to_vec();
        Ok(response)
    }

    fn pool_details(&self) -> Result<CallResponse> {
        let context = self.context()?;
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
