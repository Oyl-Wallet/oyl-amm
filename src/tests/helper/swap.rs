use alkanes::tests::helpers::{
    self as alkane_helpers, create_multiple_cellpack_with_witness_and_in, get_last_outpoint_sheet,
    get_lazy_sheet_for_runtime, get_sheet_for_runtime,
};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use oylswap_library::DEFAULT_FEE_AMOUNT_PER_1000;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use protorune_support::protostone::ProtostoneEdict;
use ruint::Uint;
use std::fmt::Write;

use super::common::{
    create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers, AmmTestDeploymentIds,
    CellpackOrEdict,
};

fn _insert_swap_txs(
    input_edicts: Vec<ProtostoneEdict>,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    cellpack: Cellpack,
) {
    test_block.txdata.push(
        create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(input_edicts),
                CellpackOrEdict::Cellpack(cellpack),
            ],
            input_outpoint,
            false,
            true,
        ),
    );
}

fn _insert_swap_txs_no_split(test_block: &mut Block, input_outpoint: OutPoint, cellpack: Cellpack) {
    test_block
        .txdata
        .push(create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![cellpack],
            input_outpoint,
            false,
        ));
}

pub fn insert_low_level_swap_txs(
    input_edicts: Vec<ProtostoneEdict>,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
    amount_0_out: u128,
    amount_1_out: u128,
    to: AlkaneId,
    data: Vec<u128>,
) {
    let mut inputs: Vec<u128> = vec![3];

    inputs.push(amount_0_out);
    inputs.push(amount_1_out);
    inputs.append(&mut to.clone().into());
    inputs.push(data.len() as u128);
    inputs.append(&mut data.clone());
    _insert_swap_txs(
        input_edicts,
        test_block,
        input_outpoint,
        Cellpack {
            target: pool_address,
            inputs: inputs,
        },
    )
}

pub fn _prepare_swap_tokens_for_exact_tokens_cellpack(
    swap_path: Vec<AlkaneId>,
    amount_out: u128,
    amount_in_max: u128,
    test_block: &mut Block,
    deployment_ids: &AmmTestDeploymentIds,
) -> Cellpack {
    if swap_path.len() < 2 {
        panic!("Swap path must be at least two alkanes long");
    }
    let mut cellpack = Cellpack {
        target: deployment_ids.amm_factory_proxy,
        inputs: vec![14, swap_path.len() as u128],
    };
    cellpack
        .inputs
        .extend(swap_path.iter().flat_map(|s| vec![s.block, s.tx]));
    cellpack.inputs.push(amount_out);
    cellpack.inputs.push(amount_in_max);
    cellpack.inputs.push(test_block.header.time as u128);
    cellpack
}

pub fn insert_swap_tokens_for_exact_tokens_txs(
    amount: u128,
    swap_path: Vec<AlkaneId>,
    amount_out: u128,
    amount_in_max: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    deployment_ids: &AmmTestDeploymentIds,
) {
    let cellpack = _prepare_swap_tokens_for_exact_tokens_cellpack(
        swap_path.clone(),
        amount_out,
        amount_in_max,
        test_block,
        deployment_ids,
    );

    _insert_swap_txs(
        vec![ProtostoneEdict {
            id: swap_path[0].into(),
            amount: amount,
            output: 0,
        }],
        test_block,
        input_outpoint,
        cellpack,
    )
}

pub fn insert_swap_tokens_for_exact_tokens_txs_no_split(
    swap_path: Vec<AlkaneId>,
    amount_out: u128,
    amount_in_max: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    deployment_ids: &AmmTestDeploymentIds,
) {
    let cellpack = _prepare_swap_tokens_for_exact_tokens_cellpack(
        swap_path,
        amount_out,
        amount_in_max,
        test_block,
        deployment_ids,
    );

    _insert_swap_txs_no_split(test_block, input_outpoint, cellpack)
}

fn _prepare_swap_exact_tokens_for_tokens_cellpack(
    amount: u128,
    swap_path: Vec<AlkaneId>,
    min_out: u128,
    deadline: u128,
    deployment_ids: &AmmTestDeploymentIds,
) -> Cellpack {
    if swap_path.len() < 2 {
        panic!("Swap path must be at least two alkanes long");
    }
    let mut cellpack = Cellpack {
        target: deployment_ids.amm_factory_proxy,
        inputs: vec![13, amount, swap_path.len() as u128],
    };
    cellpack
        .inputs
        .extend(swap_path.iter().flat_map(|s| vec![s.block, s.tx]));
    cellpack.inputs.push(min_out);
    cellpack.inputs.push(deadline);
    cellpack
}

pub fn insert_swap_exact_tokens_for_tokens_deadline(
    amount: u128,
    swap_path: Vec<AlkaneId>,
    min_out: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    deadline: u128,
    deployment_ids: &AmmTestDeploymentIds,
) {
    let cellpack = _prepare_swap_exact_tokens_for_tokens_cellpack(
        amount,
        swap_path.clone(),
        min_out,
        deadline,
        deployment_ids,
    );

    _insert_swap_txs(
        vec![ProtostoneEdict {
            id: swap_path[0].into(),
            amount: amount,
            output: 0,
        }],
        test_block,
        input_outpoint,
        cellpack,
    )
}

pub fn insert_swap_exact_tokens_for_tokens_no_split(
    amount: u128,
    swap_path: Vec<AlkaneId>,
    min_out: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    deployment_ids: &AmmTestDeploymentIds,
) {
    let cellpack = _prepare_swap_exact_tokens_for_tokens_cellpack(
        amount,
        swap_path.clone(),
        min_out,
        u128::MAX,
        deployment_ids,
    );

    _insert_swap_txs_no_split(test_block, input_outpoint, cellpack)
}

pub fn insert_swap_exact_tokens_for_tokens(
    amount: u128,
    swap_path: Vec<AlkaneId>,
    min_out: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    deployment_ids: &AmmTestDeploymentIds,
) {
    insert_swap_exact_tokens_for_tokens_deadline(
        amount,
        swap_path,
        min_out,
        test_block,
        input_outpoint,
        test_block.header.time as u128,
        deployment_ids,
    )
}

fn calc_swapped_balance(amount: u128, reserve_from: u128, reserve_to: u128) -> Result<u128> {
    let amount_in_with_fee = (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount;
    Ok((amount_in_with_fee * reserve_to) / (1000 * reserve_from + amount_in_with_fee))
}

fn calc_swapped_balance_from_path(
    prev_reserve_amount_in_path: Vec<u128>,
    swap_amount: u128,
) -> Result<u128> {
    let mut current_swapped_amount = swap_amount;
    for i in 1..prev_reserve_amount_in_path.len() {
        current_swapped_amount = calc_swapped_balance(
            current_swapped_amount,
            prev_reserve_amount_in_path[i - 1],
            prev_reserve_amount_in_path[i],
        )?;
    }
    Ok(current_swapped_amount)
}

pub fn check_swap_lp_balance(
    prev_reserve_amount_in_path: Vec<u128>,
    swap_amount: u128,
    original_amount: u128,
    swap_target_token: AlkaneId,
    test_block: &Block,
) -> Result<()> {
    let sheet = get_last_outpoint_sheet(test_block)?;
    let swapped_amount = calc_swapped_balance_from_path(prev_reserve_amount_in_path, swap_amount)?;
    println!("expected amt from swapping {:?}", swapped_amount);
    assert_eq!(
        sheet.get_cached(&swap_target_token.into()) - original_amount,
        swapped_amount
    );
    Ok(())
}

pub fn check_swap_runtime_balance(
    prev_reserve_amount_in_path: Vec<u128>,
    runtime_balances: &mut BalanceSheet<IndexPointer>,
    swap_amount: u128,
    swap_starting_token: AlkaneId,
    swap_target_token: AlkaneId,
) -> Result<()> {
    runtime_balances.increase(&swap_starting_token.into(), swap_amount);
    let swapped_amount = calc_swapped_balance_from_path(prev_reserve_amount_in_path, swap_amount)?;
    runtime_balances.decrease(&swap_target_token.into(), swapped_amount);
    let sheet = get_sheet_for_runtime();
    assert_eq!(sheet, runtime_balances.clone());
    let lazy_sheet = get_lazy_sheet_for_runtime();
    assert_eq!(lazy_sheet, runtime_balances.clone());
    Ok(())
}
