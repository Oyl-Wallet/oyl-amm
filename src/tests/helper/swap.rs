use alkanes::tests::helpers::{self as alkane_helpers};
use alkanes_runtime_pool::{AMMPoolBase, DEFAULT_FEE_AMOUNT_PER_1000};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use protorune_support::balance_sheet::BalanceSheetOperations;
use protorune_support::protostone::ProtostoneEdict;
use ruint::Uint;
use std::fmt::Write;

use super::common::{
    create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers,
    get_last_outpoint_sheet, AmmTestDeploymentIds, CellpackOrEdict,
};

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
    assert_eq!(
        sheet.get_cached(&swap_target_token.into()),
        current_swapped_amount
    );
    Ok(())
}
