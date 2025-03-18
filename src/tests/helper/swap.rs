use alkane_factory_support::factory::MintableToken;
use alkanes::tests::helpers::{self as alkane_helpers};
use alkanes_runtime_pool::{AMMPoolBase, DEFAULT_FEE_AMOUNT_PER_1000};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use protorune_support::protostone::ProtostoneEdict;
use ruint::Uint;
use std::fmt::Write;

use super::common::{
    create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers,
    get_last_outpoint_sheet, AmmTestDeploymentIds, CellpackOrEdict,
};

type U256 = Uint<256, 4>;

struct TestAMMPool {
    reserve_a: AlkaneTransfer,
    reserve_b: AlkaneTransfer,
}
impl MintableToken for TestAMMPool {}
impl AMMPoolBase for TestAMMPool {
    fn reserves(
        &self,
    ) -> (
        alkanes_support::parcel::AlkaneTransfer,
        alkanes_support::parcel::AlkaneTransfer,
    ) {
        (self.reserve_a.clone(), self.reserve_b.clone())
    }
}

fn _insert_swap_txs(
    amount: u128,
    swap_from_token: AlkaneId,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    cellpack: Cellpack,
) {
    test_block.txdata.push(
        create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(vec![ProtostoneEdict {
                    id: swap_from_token.into(),
                    amount: amount,
                    output: 0,
                }]),
                CellpackOrEdict::Cellpack(cellpack),
            ],
            input_outpoint,
            false,
            true,
        ),
    );
}

pub fn test_simulate_amount_out() -> Result<()> {
    // Set up test pool with initial reserves
    let token_a = AlkaneId::new(1, 1);
    let token_b = AlkaneId::new(2, 1);

    let test_pool = TestAMMPool {
        reserve_a: AlkaneTransfer {
            id: token_a.clone(),
            value: 1_000_000, // 1M tokens
        },
        reserve_b: AlkaneTransfer {
            id: token_b.clone(),
            value: 2_000_000, // 2M tokens
        },
    };

    // Test case 1: Swap token A for token B
    let input_amount_a = 100_000u128; // 100K tokens
    let result = test_pool.simulate_amount_out(token_a, input_amount_a)?;

    // Calculate expected output with 0.4% fee
    let amount_with_fee = U256::from(996) * U256::from(input_amount_a); // 0.4% fee
    let numerator = amount_with_fee * U256::from(2_000_000);
    let denominator = U256::from(1000) * U256::from(1_000_000) + amount_with_fee;
    let expected_output = (numerator / denominator).to_le_bytes_vec();

    println!("result amt out {:?}", result.data);
    println!("expected output {:?}", expected_output);

    assert_eq!(result.data, expected_output);

    // Test case 2: Swap token B for token A
    let input_amount_b = 200_000u128; // 200K tokens
    let result = test_pool.simulate_amount_out(token_b, input_amount_b)?;

    let amount_with_fee = U256::from(996) * U256::from(input_amount_b);
    let numerator = amount_with_fee * U256::from(1_000_000);
    let denominator = U256::from(1000) * U256::from(2_000_000) + amount_with_fee;
    let expected_output = (numerator / denominator).to_le_bytes_vec();

    assert_eq!(result.data, expected_output);

    // Test case 3: Invalid token (not in pool)
    let result = test_pool.simulate_amount_out(AlkaneId::new(3, 1), 100_000);
    assert!(result.is_err());

    Ok(())
}

pub fn insert_swap_txs(
    amount: u128,
    swap_from_token: AlkaneId,
    min_out: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
) {
    _insert_swap_txs(
        amount,
        swap_from_token,
        test_block,
        input_outpoint,
        Cellpack {
            target: pool_address,
            inputs: vec![3, min_out],
        },
    )
}

pub fn insert_swap_txs_w_router(
    amount: u128,
    swap_path: Vec<AlkaneId>,
    min_out: u128,
    test_block: &mut Block,
    deployment_ids: &AmmTestDeploymentIds,
    input_outpoint: OutPoint,
) {
    if swap_path.len() < 2 {
        panic!("Swap path must be at least two alkanes long");
    }
    let mut cellpack = Cellpack {
        target: deployment_ids.amm_router_deployment,
        inputs: vec![3, swap_path.len() as u128],
    };
    cellpack
        .inputs
        .extend(swap_path.iter().flat_map(|s| vec![s.block, s.tx]));
    cellpack.inputs.push(min_out);

    _insert_swap_txs(amount, swap_path[0], test_block, input_outpoint, cellpack)
}

fn calc_swapped_balance(amount: u128, reserve_from: u128, reserve_to: u128) -> Result<u128> {
    let amount_in_with_fee = (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount;
    Ok((amount_in_with_fee * reserve_to) / (1000 * reserve_from + amount_in_with_fee))
}

pub fn check_swap_lp_balance(
    prev_reserve_amount_in_path: Vec<u128>,
    swap_amount: u128,
    swap_target_token: AlkaneId,
    test_block: &Block,
) -> Result<()> {
    let sheet = get_last_outpoint_sheet(test_block)?;
    let mut current_swapped_amount = swap_amount;
    for i in 1..prev_reserve_amount_in_path.len() {
        current_swapped_amount = calc_swapped_balance(
            current_swapped_amount,
            prev_reserve_amount_in_path[i - 1],
            prev_reserve_amount_in_path[i],
        )?;
    }

    println!("expected amt from swapping {:?}", current_swapped_amount);
    assert_eq!(sheet.get(&swap_target_token.into()), current_swapped_amount);
    Ok(())
}
