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
use oylswap_library::PoolInfo;
use protorune_support::utils::consensus_decode;
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
    fn create_new_pool(&self) -> Result<CallResponse> {
        let context = self.context()?;
        if context.incoming_alkanes.0.len() != 2 {
            return Err(anyhow!(format!(
                "must send two runes to initialize a pool {:?}",
                context.incoming_alkanes.0
            )));
        }
        let (alkane_a, alkane_b) = take_two(&context.incoming_alkanes.0);
        let (a, b) = sort_alkanes((alkane_a.id.clone(), alkane_b.id.clone()));
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

        self.call(
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
            &AlkaneTransferParcel(vec![
                context.incoming_alkanes.0[0].clone(),
                context.incoming_alkanes.0[1].clone(),
            ]),
            self.fuel(),
        )
    }

    fn _find_existing_pool_id(&self, alkane_a: AlkaneId, alkane_b: AlkaneId) -> Result<AlkaneId> {
        let (a, b) = sort_alkanes((alkane_a, alkane_b));
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

    fn _check_deadline(&self, deadline: u128) -> Result<()> {
        let block = consensus_decode::<Block>(&mut std::io::Cursor::new(self.block()))?;
        if block.header.time as u128 > deadline {
            Err(anyhow!("EXPIRED deadline"))
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

    fn _get_pool_info(&self, pool: AlkaneId) -> Result<PoolInfo> {
        let cellpack = Cellpack {
            target: pool,
            inputs: vec![999],
        };
        let response = self.call(&cellpack, &AlkaneTransferParcel(vec![]), self.fuel())?;
        Ok(PoolInfo::from_vec(&response.data)?)
    }

    // note: for now, token_a and token_b must be in alphabetical order
    fn add_liquidity_checked(
        &self,
        token_a: AlkaneId,
        token_b: AlkaneId,
        amount_a_desired: u128,
        amount_b_desired: u128,
        amount_a_min: u128,
        amount_b_min: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        self._check_deadline(deadline)?;
        let context = self.context()?;
        let pool = self._find_existing_pool_id(token_a, token_b)?;
        let (previous_a, previous_b) = self._get_reserves(pool)?;
        let (amount_a, amount_b) = if previous_a == 0 && previous_b == 0 {
            (amount_a_desired, amount_b_desired)
        } else {
            let amount_b_optimal = amount_a_desired * previous_b / previous_a;
            if amount_b_optimal <= amount_b_desired {
                if amount_b_optimal < amount_b_min {
                    return Err(anyhow!("INSUFFICIENT_B_AMOUNT"));
                }
                (amount_a_desired, amount_b_optimal)
            } else {
                let amount_a_optimal = amount_b_desired * previous_a / previous_b;
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
        let mut result = self.call(&cellpack, &input_transfer, self.fuel())?;

        // return leftovers
        result.alkanes.pay(AlkaneTransfer {
            id: token_a,
            value: self.balance(&context.myself, &token_a),
        });
        result.alkanes.pay(AlkaneTransfer {
            id: token_b,
            value: self.balance(&context.myself, &token_b),
        });

        Ok(result)
    }

    fn _get_amount_out(
        &self,
        pool: AlkaneId,
        parcel: AlkaneTransferParcel,
    ) -> Result<(u128, u128)> {
        let transfer = parcel.0[0].clone();
        let pool_info = self._get_pool_info(pool)?;

        if &transfer.id == &pool_info.token_a {
            Ok((
                0,
                oylswap_library::get_amount_out(
                    transfer.value,
                    pool_info.reserve_a,
                    pool_info.reserve_b,
                )?,
            ))
        } else {
            Ok((
                oylswap_library::get_amount_out(
                    transfer.value,
                    pool_info.reserve_b,
                    pool_info.reserve_a,
                )?,
                0,
            ))
        }
    }

    fn _swap_exact_tokens_for_tokens(
        &self,
        pool: AlkaneId,
        amount_out_predicate: u128,
        parcel: AlkaneTransferParcel,
    ) -> Result<CallResponse> {
        let context = self.context()?;
        let (amount_0_out, amount_1_out) = self._get_amount_out(pool, parcel.clone())?;
        if amount_0_out < amount_out_predicate && amount_1_out < amount_out_predicate {
            return Err(anyhow!("predicate failed: insufficient output"));
        }
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
        self.call(&cellpack, &parcel, self.fuel())
    }

    fn swap_exact_tokens_for_tokens(
        &self,
        path: Vec<AlkaneId>,
        amount: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        self._check_deadline(deadline)?;
        let context = self.context()?;

        // swap
        if path.len() < 2 {
            return Err(anyhow!("Routing path must be at least two alkanes long"));
        }
        if context.incoming_alkanes.0.len() != 1 {
            return Err(anyhow!("Input must be 1 alkane"));
        }
        if path[0] != context.incoming_alkanes.0[0].id {
            return Err(anyhow!(
                "Routing path first element must be the input token"
            ));
        }
        let mut this_response = CallResponse {
            alkanes: context.incoming_alkanes.clone(),
            data: vec![],
        };

        for i in 1..path.len() {
            let pool = self._find_existing_pool_id(path[i - 1], path[i])?;
            let this_amount = if i == path.len() - 1 { amount } else { 0 };
            this_response =
                self._swap_exact_tokens_for_tokens(pool, this_amount, this_response.alkanes)?;
        }

        Ok(this_response)
    }

    fn _swap_tokens_for_exact_tokens(
        &self,
        token_in: AlkaneId,
        token_out: AlkaneId,
        desired_amount_out: u128,
        amount_in_max: u128,
    ) -> Result<CallResponse> {
        let context = self.context()?;

        let pool = self._find_existing_pool_id(token_in, token_out)?;

        let pool_info = self._get_pool_info(pool)?;

        let amount_in = if &token_in == &pool_info.token_a {
            oylswap_library::get_amount_in(
                desired_amount_out,
                pool_info.reserve_a,
                pool_info.reserve_b,
            )?
        } else {
            oylswap_library::get_amount_in(
                desired_amount_out,
                pool_info.reserve_b,
                pool_info.reserve_a,
            )?
        };
        if amount_in > amount_in_max {
            return Err(anyhow!("EXCESSIVE_INPUT_AMOUNT"));
        }
        let (amount_0_out, amount_1_out) = if &token_in == &pool_info.token_a {
            (0, desired_amount_out)
        } else {
            (desired_amount_out, 0)
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
        let transfer_in = AlkaneTransferParcel(vec![AlkaneTransfer {
            id: token_in,
            value: amount_in,
        }]);
        self.call(&cellpack, &transfer_in, self.fuel())
    }

    fn swap_tokens_for_exact_tokens(
        &self,
        path: Vec<AlkaneId>,
        desired_amount_out: u128,
        amount_in_max: u128,
        deadline: u128,
    ) -> Result<CallResponse> {
        self._check_deadline(deadline)?;
        let context = self.context()?;
        let parcel: AlkaneTransferParcel = context.clone().incoming_alkanes;

        if path.len() == 2 {
            return Err(anyhow!("Routing path must be at least two alkanes long"));
        }
        if parcel.0.len() != 1 {
            return Err(anyhow!("Input must be 1 alkane"));
        }
        if path[0] != parcel.0[0].id {
            return Err(anyhow!(
                "Routing path first element must be the input token"
            ));
        }
        if parcel.0[0].value < amount_in_max {
            return Err(anyhow!("amount_in_max is higher than input amount"));
        }
        let mut response = self._swap_tokens_for_exact_tokens(
            path[0],
            path[1],
            desired_amount_out,
            amount_in_max,
        )?;

        // refund remaining to user
        response.alkanes.pay(AlkaneTransfer {
            id: path[0],
            value: self.balance(&context.myself, &path[0]),
        });

        Ok(response)
    }
}
