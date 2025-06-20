use alkanes_runtime::{auth::AuthenticatedResponder, storage::StoragePointer};
#[allow(unused_imports)]
use alkanes_runtime::{
    println,
    stdio::{stdout, Write},
};
use alkanes_support::{
    cellpack::Cellpack,
    context::Context,
    id::AlkaneId,
    parcel::{AlkaneTransfer, AlkaneTransferParcel},
    response::CallResponse,
};
use anyhow::{anyhow, Result};
use bitcoin::Block;
use metashrew_support::{
    byte_view::ByteView,
    index_pointer::KeyValuePointer,
    utils::{consume_sized_int, consume_u128},
};
use oylswap_library::{PoolInfo, U256};
use protorune_support::utils::consensus_decode;
use std::{collections::BTreeSet, sync::Arc};

pub fn join_ids(a: AlkaneId, b: AlkaneId) -> Vec<u8> {
    let mut result: Vec<u8> = a.into();
    let value: Vec<u8> = b.into();
    result.extend_from_slice(&value);
    result
}

pub fn join_ids_from_tuple(v: (AlkaneId, AlkaneId)) -> Vec<u8> {
    join_ids(v.0, v.1)
}

pub trait AMMFactoryBase: AuthenticatedResponder {
    fn pool_id(&self) -> Result<u128> {
        let ptr = StoragePointer::from_keyword("/pool_factory_id")
            .get()
            .as_ref()
            .clone();
        let mut cursor = std::io::Cursor::<Vec<u8>>::new(ptr);
        Ok(consume_u128(&mut cursor)?)
    }
    fn set_pool_id(&self, pool_factory_id: u128) {
        let mut pool_factory_id_pointer = StoragePointer::from_keyword("/pool_factory_id");
        // set the address for the implementation for AMM pool
        pool_factory_id_pointer.set(Arc::new(pool_factory_id.to_bytes()));
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
    fn init_factory(&self, pool_factory_id: u128, auth_token_units: u128) -> Result<CallResponse> {
        self.observe_initialization()?;
        let context = self.context()?;
        let mut pool_factory_id_pointer = StoragePointer::from_keyword("/pool_factory_id");
        // set the address for the implementation for AMM pool
        pool_factory_id_pointer.set(Arc::new(pool_factory_id.to_bytes()));
        let auth_token = self.deploy_auth_token(auth_token_units)?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        response.alkanes.pay(auth_token);
        Ok(response)
    }
    fn set_pool_factory_id(&self, pool_factory_id: u128) -> Result<CallResponse> {
        self.only_owner()?;
        let context = self.context()?;
        self.set_pool_id(pool_factory_id);
        Ok(CallResponse::forward(&context.incoming_alkanes.clone()))
    }
    fn create_new_pool(
        &self,
        token_a: AlkaneId,
        token_b: AlkaneId,
        amount_a: u128,
        amount_b: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        if token_a == token_b {
            return Err(anyhow!("tokens to create the pool cannot be the same"));
        }
        if amount_a == 0 || amount_b == 0 {
            return Err(anyhow!("input amount cannot be zero"));
        }
        let (a, b) = oylswap_library::sort_alkanes((token_a.clone(), token_b.clone()));
        let pool_id = AlkaneId::new(2, self.sequence());
        // check if this pool already exists
        if self.pool_pointer(&a, &b).get().len() == 0 {
            self.pool_pointer(&a, &b).set(Arc::new(pool_id.into()));
        } else {
            return Err(anyhow!("pool already exists"));
        }

        // Add the new pool to the registry
        let length = self.all_pools_length()?;

        // Store the pool ID at the current index
        StoragePointer::from_keyword("/all_pools/")
            .select(&length.to_le_bytes().to_vec())
            .set(Arc::new(pool_id.into()));

        // Update the length
        StoragePointer::from_keyword("/all_pools_length")
            .set(Arc::new((length + 1).to_le_bytes().to_vec()));

        let input_transfer = AlkaneTransferParcel(vec![
            AlkaneTransfer {
                id: token_a,
                value: amount_a,
            },
            AlkaneTransfer {
                id: token_b,
                value: amount_b,
            },
        ]);

        let result = self.call(
            &Cellpack {
                target: AlkaneId {
                    block: 6,
                    tx: self.pool_id()?,
                },
                inputs: vec![
                    0,
                    a.block,
                    a.tx,
                    b.block,
                    b.tx,
                    context.myself.block,
                    context.myself.tx,
                ],
            },
            &input_transfer,
            self.fuel(),
        )?;
        self._return_leftovers(context.myself, result, context.incoming_alkanes)
    }

    fn _find_existing_pool_id(&self, alkane_a: AlkaneId, alkane_b: AlkaneId) -> Result<AlkaneId> {
        let (a, b) = oylswap_library::sort_alkanes((alkane_a, alkane_b));
        if self.pool_pointer(&a, &b).get().len() == 0 {
            return Err(anyhow!(format!(
                "the pool {:?} {:?} doesn't exist in the factory",
                alkane_a, alkane_b
            )));
        }
        let mut cursor =
            std::io::Cursor::<Vec<u8>>::new(self.pool_pointer(&a, &b).get().as_ref().clone());
        Ok(AlkaneId::new(
            consume_sized_int::<u128>(&mut cursor)?,
            consume_sized_int::<u128>(&mut cursor)?,
        ))
    }

    fn find_existing_pool_id(
        &self,
        alkane_a: AlkaneId,
        alkane_b: AlkaneId,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());

        response.data = self._find_existing_pool_id(alkane_a, alkane_b)?.into();
        Ok(response)
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
    fn get_all_pools(&self) -> Result<CallResponse> {
        let context = self.context()?;
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
    fn get_num_pools(&self) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response = CallResponse::forward(&context.incoming_alkanes.clone());
        response.data = AMMFactoryBase::all_pools_length(self)?
            .to_le_bytes()
            .to_vec();
        Ok(response)
    }

    fn collect_fees(&self, pool_id: AlkaneId) -> Result<CallResponse> {
        self.only_owner()?;
        let context = self.context()?;
        self.call(
            &Cellpack {
                target: pool_id,
                inputs: vec![10],
            },
            &context.incoming_alkanes.clone(),
            self.fuel(),
        )
    }

    fn _check_deadline(&self, height: u64, deadline: u128) -> Result<()> {
        if deadline != 0 && height as u128 > deadline {
            Err(anyhow!(format!(
                "EXPIRED deadline: block height ({}) > deadline({})",
                height, deadline
            )))
        } else {
            Ok(())
        }
    }

    fn _get_reserves(&self, pool: AlkaneId) -> Result<(u128, u128)> {
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![97],
        };
        let response = self.call(&cellpack, &AlkaneTransferParcel(vec![]), self.fuel())?;
        let mut offset = 0;

        let reserve_a = u128::from_le_bytes(response.data[offset..offset + 16].try_into()?);
        offset += 16;
        let reserve_b = u128::from_le_bytes(response.data[offset..offset + 16].try_into()?);

        Ok((reserve_a, reserve_b))
    }

    fn _get_reserves_ordered(&self, token_a: AlkaneId, token_b: AlkaneId) -> Result<(u128, u128)> {
        let (token_0, _) = oylswap_library::sort_alkanes((token_a, token_b));
        let pool = self._find_existing_pool_id(token_a, token_b)?;
        let (reserve_0, reserve_1) = self._get_reserves(pool)?;
        if token_a == token_0 {
            Ok((reserve_0, reserve_1))
        } else {
            Ok((reserve_1, reserve_0))
        }
    }

    fn _get_pool_info(&self, pool: AlkaneId) -> Result<PoolInfo> {
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![999],
        };
        let response = self.call(&cellpack, &AlkaneTransferParcel(vec![]), self.fuel())?;
        Ok(PoolInfo::from_vec(&response.data)?)
    }

    fn _return_leftovers(
        &self,
        myself: AlkaneId,
        result: CallResponse,
        input_alkanes: AlkaneTransferParcel,
    ) -> Result<CallResponse> {
        let mut response = CallResponse::default();
        let mut unique_ids: BTreeSet<AlkaneId> = BTreeSet::new();
        for transfer in input_alkanes.0 {
            unique_ids.insert(transfer.id);
        }
        for transfer in result.alkanes.0 {
            unique_ids.insert(transfer.id);
        }
        for id in unique_ids {
            response.alkanes.pay(AlkaneTransfer {
                id: id,
                value: self.balance(&myself, &id),
            });
        }
        Ok(response)
    }

    // note: for now, token_a and token_b must be in alphabetical order
    fn add_liquidity(
        &self,
        token_a: AlkaneId,
        token_b: AlkaneId,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        self._check_deadline(context.height, deadline)?;
        let pool = self._find_existing_pool_id(token_a, token_b)?;
        let (previous_a, previous_b) = self._get_reserves(pool)?;
        let (amount_a, amount_b) = if previous_a == 0 && previous_b == 0 {
            (amount_a_desired, amount_b_desired)
        } else {
            let amount_b_optimal: u128 = (U256::from(amount_a_desired) * U256::from(previous_b)
                / U256::from(previous_a))
            .try_into()?;
            if amount_b_optimal <= amount_b_desired {
                if amount_b_optimal < amount_b_min {
                    return Err(anyhow!("INSUFFICIENT_B_AMOUNT"));
                }
                (amount_a_desired, amount_b_optimal)
            } else {
                let amount_a_optimal = (U256::from(amount_b_desired) * U256::from(previous_a)
                    / U256::from(previous_b))
                .try_into()?;
                if amount_a_optimal > amount_a_desired || amount_a_optimal < amount_a_min {
                    return Err(anyhow!("INSUFFICIENT_A_AMOUNT"));
                }
                (amount_a_optimal, amount_b_desired)
            }
        };

        let input_transfer = AlkaneTransferParcel(vec![
            AlkaneTransfer {
                id: token_a,
                value: amount_a,
            },
            AlkaneTransfer {
                id: token_b,
                value: amount_b,
            },
        ]);
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![1],
        };
        let result = self.call(&cellpack, &input_transfer, self.fuel())?;
        self._return_leftovers(context.myself, result, context.incoming_alkanes)
    }

    fn burn(
        &self,
        token_a: AlkaneId,
        token_b: AlkaneId,
        liquidity: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        self._check_deadline(context.height, deadline)?;
        let parcel = context.incoming_alkanes;
        let pool = self._find_existing_pool_id(token_a, token_b)?;
        let input_transfer = AlkaneTransferParcel(vec![AlkaneTransfer {
            id: pool,
            value: liquidity,
        }]);
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![2],
        };
        let result = self.call(&cellpack, &input_transfer, self.fuel())?;
        if result.alkanes.0[0].id == token_a {
            if result.alkanes.0[0].value < amount_a_min {
                return Err(anyhow!("INSUFFICIENT_A_AMOUNT"));
            }
            if result.alkanes.0[1].value < amount_b_min {
                return Err(anyhow!("INSUFFICIENT_B_AMOUNT"));
            }
        } else {
            if result.alkanes.0[0].value < amount_b_min {
                return Err(anyhow!("INSUFFICIENT_B_AMOUNT"));
            }
            if result.alkanes.0[1].value < amount_a_min {
                return Err(anyhow!("INSUFFICIENT_A_AMOUNT"));
            }
        }
        self._return_leftovers(context.myself, result, parcel)
    }

    fn _swap(&self, amounts: &Vec<u128>, path: &Vec<AlkaneId>) -> Result<CallResponse> {
        let context = self.context()?;
        let mut response: CallResponse = CallResponse::default();
        for i in 1..path.len() {
            let pool = self._find_existing_pool_id(path[i - 1], path[i])?;
            let (token_a, _) = oylswap_library::sort_alkanes((path[i - 1], path[i]));
            let amount_out = amounts[i];
            let (amount_0_out, amount_1_out) = if path[i - 1] == token_a {
                (0, amount_out)
            } else {
                (amount_out, 0)
            };
            let cellpack = Cellpack {
                target: pool,
                inputs: vec![
                    3,
                    amount_0_out,
                    amount_1_out,
                    context.caller.block,
                    context.caller.tx,
                    0,
                ],
            };
            let parcel = AlkaneTransferParcel(vec![AlkaneTransfer {
                id: path[i - 1],
                value: amounts[i - 1],
            }]);
            response = self.call(&cellpack, &parcel, self.fuel())?;
        }
        Ok(response)
    }

    fn get_amounts_out(&self, amount_in: u128, path: &Vec<AlkaneId>) -> Result<Vec<u128>> {
        let n = path.len();
        if n < 2 {
            return Err(anyhow!("Routing path must be at least two alkanes long"));
        }
        let mut amounts: Vec<u128> = vec![0; n];
        amounts[0] = amount_in;
        for i in 1..n {
            let (reserve_in, reserve_out) = self._get_reserves_ordered(path[i - 1], path[i])?;
            amounts[i] = oylswap_library::get_amount_out(amounts[i - 1], reserve_in, reserve_out)?;
        }
        Ok(amounts)
    }

    fn swap_exact_tokens_for_tokens(
        &self,
        amount_in: u128,
        path: Vec<AlkaneId>,
        amount_out_min: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        self._check_deadline(context.height, deadline)?;
        let parcel = context.incoming_alkanes;

        let amounts = self.get_amounts_out(amount_in, &path)?;
        if amounts[amounts.len() - 1] < amount_out_min {
            return Err(anyhow!("predicate failed: insufficient output"));
        }

        let result = self._swap(&amounts, &path)?;
        self._return_leftovers(context.myself, result, parcel)
    }

    fn get_amounts_in(&self, amount_out: u128, path: &Vec<AlkaneId>) -> Result<Vec<u128>> {
        let n = path.len();
        if n < 2 {
            return Err(anyhow!("Routing path must be at least two alkanes long"));
        }
        let mut amounts: Vec<u128> = vec![0; n];
        amounts[n - 1] = amount_out;
        for i in 1..n {
            let (reserve_in, reserve_out) =
                self._get_reserves_ordered(path[n - i - 1], path[n - i])?;
            amounts[n - i - 1] =
                oylswap_library::get_amount_in(amounts[n - i], reserve_in, reserve_out)?;
        }
        Ok(amounts)
    }

    fn swap_tokens_for_exact_tokens(
        &self,
        path: Vec<AlkaneId>,
        desired_amount_out: u128,
        amount_in_max: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        self._check_deadline(context.height, deadline)?;
        let parcel: AlkaneTransferParcel = context.clone().incoming_alkanes;

        let amounts = self.get_amounts_in(desired_amount_out, &path)?;
        if amounts[0] > amount_in_max {
            return Err(anyhow!(format!(
                "EXCESSIVE_INPUT_AMOUNT: required({}) > amount_in_max({})",
                amounts[0], amount_in_max
            )));
        }

        let result = self._swap(&amounts, &path)?;
        self._return_leftovers(context.myself, result, parcel)
    }
}
