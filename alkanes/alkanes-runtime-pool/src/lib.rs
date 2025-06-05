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
use bitcoin::Block;
use metashrew_support::{byte_view::ByteView, index_pointer::KeyValuePointer, utils::consume_u128};
use num::integer::Roots;
use oylswap_library::{Lock, PoolInfo, Sqrt, StorableU256, DEFAULT_FEE_AMOUNT_PER_1000, U256};
use protorune_support::balance_sheet::{BalanceSheetOperations, CachedBalanceSheet};
use protorune_support::utils::consensus_decode;
use std::{cmp::min, sync::Arc};

// per uniswap docs, the first 1e3 wei of lp token minted are burned to mitigate attacks where the value of a lp token is raised too high easily
pub const MINIMUM_LIQUIDITY: u128 = 1000;
pub const SWAP_EXTCALL_OPCODE: u128 = 73776170;
pub const PRECISION: u32 = 128;

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
    fn k_last(&self) -> U256 {
        self.k_last_pointer().get_value::<StorableU256>().into()
    }
    fn set_k_last(&self, v: U256) {
        self.k_last_pointer().set_value::<StorableU256>(v.into());
    }
    fn block_timestamp_last_pointer(&self) -> StoragePointer {
        StoragePointer::from_keyword("/blockTimestampLast")
    }
    fn block_timestamp_last(&self) -> u32 {
        self.block_timestamp_last_pointer().get_value::<u32>()
    }
    fn set_block_timestamp_last(&self, v: u32) {
        self.block_timestamp_last_pointer().set_value::<u32>(v);
    }
    fn price_cumulative_pointers(&self) -> (StoragePointer, StoragePointer) {
        (
            StoragePointer::from_keyword("/price0CumLast"),
            StoragePointer::from_keyword("/price1CumLast"),
        )
    }
    fn price_cumulative(&self) -> (U256, U256) {
        let (p0, p1) = self.price_cumulative_pointers();
        (
            p0.get_value::<StorableU256>().into(),
            p1.get_value::<StorableU256>().into(),
        )
    }
    fn increase_price_cumulative(&self, v0: U256, v1: U256) {
        let (mut p0, mut p1) = self.price_cumulative_pointers();
        let (p0_val, p1_val) = self.price_cumulative();
        p0.set_value::<StorableU256>((p0_val + v0).into());
        p1.set_value::<StorableU256>((p1_val + v1).into());
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
        self.set_k_last(U256::from(0));
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
        if parcel.0.len() != n {
            Err(anyhow!(format!(
                "{} alkanes sent but expected {} alkane inputs",
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
        if !k_last.is_zero() {
            let root_k_last = k_last.sqrt();
            let root_k = (U256::from(previous_a) * U256::from(previous_b)).sqrt();
            if (root_k > root_k_last) {
                let numerator = U256::from(total_supply) * (root_k - root_k_last);
                let root_k_fee_adj = root_k * U256::from(3) / U256::from(2); // assuming 2/5 of 0.5% fee goes to protocol
                let denominator = root_k_fee_adj + root_k_last;
                let liquidity: u128 = (numerator / denominator).try_into()?; // guaranteed to be storable in u128
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
        let new_k = U256::from(previous_a.value) * U256::from(previous_b.value);
        self.set_k_last(new_k);
        Ok(response)
    }
    fn forward_incoming(&self) -> Result<CallResponse> {
        let context = self.context()?;
        Ok(CallResponse::forward(&context.incoming_alkanes))
    }

    fn _update_cum_prices(&self, reserve0: u128, reserve1: u128) -> Result<()> {
        let block = consensus_decode::<Block>(&mut std::io::Cursor::new(self.block()))?;
        let current_timestamp = block.header.time;
        let last_timestamp = self.block_timestamp_last();
        let time_elapsed = current_timestamp - last_timestamp;
        if time_elapsed > 0 && reserve0 != 0 && reserve1 != 0 {
            self.increase_price_cumulative(
                ((U256::from(reserve1) << U256::from(PRECISION)) / U256::from(reserve0)
                    * U256::from(time_elapsed))
                .try_into()?,
                ((U256::from(reserve0) << U256::from(PRECISION)) / U256::from(reserve1)
                    * U256::from(time_elapsed))
                .try_into()?,
            );
        }
        self.set_block_timestamp_last(current_timestamp);
        Ok(())
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
            self._update_cum_prices(previous_a.value, previous_b.value)?;
            let new_k = U256::from(reserve_a.value) * U256::from(reserve_b.value);
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

            self._update_cum_prices(previous_a.value, previous_b.value)?;
            let new_k =
                U256::from(reserve_a.value - amount_a) * U256::from(reserve_b.value - amount_b);
            self.set_k_last(new_k);
            Ok(response)
        })
    }

    fn swap(
        &self,
        amount_0_out: u128,
        amount_1_out: u128,
        to: AlkaneId, // goes to this address if not zero and if data is not empty, otherwise goes to caller
        data: Vec<u128>,
    ) -> Result<CallResponse> {
        Lock::lock(|| {
            if amount_0_out == 0 && amount_1_out == 0 {
                return Err(anyhow!("INSUFFICIENT_OUTPUT_AMOUNT"));
            }
            let context = self.context()?;
            let parcel = context.incoming_alkanes.clone();

            let (reserve_0, reserve_1) = self.previous_reserves(&parcel)?;
            if amount_0_out >= reserve_0.value || amount_1_out >= reserve_1.value {
                return Err(anyhow!("INSUFFICIENT_LIQUIDITY"));
            }

            if to == reserve_0.id || to == reserve_1.id {
                return Err(anyhow!("INVALID_TO"));
            }

            let mut alkane_transfer = AlkaneTransferParcel::default();

            // Optimistically transfer tokens
            if amount_0_out > 0 {
                alkane_transfer.0.push(AlkaneTransfer {
                    id: reserve_0.id.clone(),
                    value: amount_0_out,
                });
            }

            if amount_1_out > 0 {
                alkane_transfer.0.push(AlkaneTransfer {
                    id: reserve_1.id.clone(),
                    value: amount_1_out,
                });
            }

            let mut response = CallResponse::default();
            // If data is provided, call the recipient with the data
            let should_send_to_extcall = !data.is_empty() && to != AlkaneId::new(0, 0);
            if should_send_to_extcall {
                let mut extcall_input: Vec<u128> = vec![SWAP_EXTCALL_OPCODE];
                extcall_input.append(&mut context.caller.clone().into());
                extcall_input.push(amount_0_out);
                extcall_input.push(amount_1_out);
                extcall_input.push(data.len() as u128);
                extcall_input.append(&mut data.clone());

                self.call(
                    &Cellpack {
                        target: to.clone(),
                        inputs: extcall_input,
                    },
                    &alkane_transfer.clone(),
                    self.fuel(),
                )?;
            } else {
                response.alkanes = alkane_transfer;
            }

            // Get the new balances after transfers
            let (mut balance_0, mut balance_1) = self.reserves()?;
            if !should_send_to_extcall {
                // haven't sent the tokens yet in this case
                balance_0.value -= amount_0_out;
                balance_1.value -= amount_1_out;
            }

            // Calculate input amounts.
            let mut amount_0_in = if balance_0.value > reserve_0.value - amount_0_out {
                balance_0.value - (reserve_0.value - amount_0_out)
            } else {
                0
            };

            let mut amount_1_in = if balance_1.value > reserve_1.value - amount_1_out {
                balance_1.value - (reserve_1.value - amount_1_out)
            } else {
                0
            };

            // Check that at least one input amount is greater than 0
            if amount_0_in == 0 && amount_1_in == 0 {
                return Err(anyhow!("INSUFFICIENT_INPUT_AMOUNT"));
            }

            // Check K value (constant product formula)
            // In Uniswap: balance0Adjusted.mul(balance1Adjusted) >= uint(_reserve0).mul(_reserve1).mul(1000**2)
            let balance_0_adjusted = U256::from(balance_0.value) * U256::from(1000)
                - U256::from(amount_0_in) * U256::from(DEFAULT_FEE_AMOUNT_PER_1000);
            let balance_1_adjusted = U256::from(balance_1.value) * U256::from(1000)
                - U256::from(amount_1_in) * U256::from(DEFAULT_FEE_AMOUNT_PER_1000);

            if balance_0_adjusted * balance_1_adjusted
                < U256::from(reserve_0.value)
                    * U256::from(reserve_1.value)
                    * U256::from(1000 * 1000)
            {
                return Err(anyhow!("K is not increasing"));
            }

            self._update_cum_prices(reserve_0.value, reserve_1.value)?;

            // Return response with transfers
            Ok(response)
        })
    }

    fn get_price_cumulative_last(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let mut bytes = Vec::new();
        let (p0, p1) = self.price_cumulative();
        bytes.extend_from_slice(&p0.to_le_bytes::<32>());
        bytes.extend_from_slice(&p1.to_le_bytes::<32>());
        response.data = bytes;
        Ok(response)
    }

    fn get_reserves(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let parcel = context.incoming_alkanes.clone();
        let (reserve_a, reserve_b) = self.previous_reserves(&parcel)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&reserve_a.value.to_le_bytes());
        bytes.extend_from_slice(&reserve_b.value.to_le_bytes());
        response.data = bytes;
        Ok(response)
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
