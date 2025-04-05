use crate::tests::helper::common::create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
use num::integer::Roots;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use protorune_support::protostone::ProtostoneEdict;

#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;

use super::common::*;

fn _insert_add_liquidity_txs(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    cellpack: Cellpack,
) {
    test_block.txdata.push(
        create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(vec![
                    ProtostoneEdict {
                        amount: amount1,
                        output: 0,
                        id: token1_address.into(),
                    },
                    ProtostoneEdict {
                        amount: amount2,
                        output: 0,
                        id: token2_address.into(),
                    },
                ]),
                CellpackOrEdict::Cellpack(cellpack),
            ],
            input_outpoint,
            false,
            true,
        ),
    );
}

pub fn insert_add_liquidity_txs(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    pool_address: AlkaneId,
    test_block: &mut Block,
    input_outpoint: OutPoint,
) {
    _insert_add_liquidity_txs(
        amount1,
        amount2,
        token1_address,
        token2_address,
        test_block,
        input_outpoint,
        Cellpack {
            target: pool_address,
            inputs: vec![1],
        },
    )
}

pub fn insert_add_liquidity_txs_w_router(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    test_block: &mut Block,
    deployment_ids: &AmmTestDeploymentIds,
    input_outpoint: OutPoint,
) {
    _insert_add_liquidity_txs(
        amount1,
        amount2,
        token1_address,
        token2_address,
        test_block,
        input_outpoint,
        Cellpack {
            target: deployment_ids.amm_router_deployment,
            inputs: vec![
                1,
                token1_address.block,
                token1_address.tx,
                token2_address.block,
                token2_address.tx,
            ],
        },
    )
}

pub fn calc_lp_balance_from_add_liquidity(
    prev_amount1: u128,
    prev_amount2: u128,
    added_amount1: u128,
    added_amount2: u128,
    total_supply: u128,
) -> u128 {
    let root_k = ((prev_amount1 + added_amount1) * (prev_amount2 + added_amount2)).sqrt();
    let root_k_last = (prev_amount1 * prev_amount2).sqrt();
    let numerator = total_supply * (root_k - root_k_last);
    let denominator = root_k * 5 + root_k_last;
    numerator / denominator
}

pub fn check_add_liquidity_lp_balance(
    prev_amount1: u128,
    prev_amount2: u128,
    added_amount1: u128,
    added_amount2: u128,
    total_supply: u128,
    test_block: &Block,
    pool_address: AlkaneId,
) -> Result<()> {
    let sheet = get_last_outpoint_sheet(test_block)?;
    let expected_amount = calc_lp_balance_from_add_liquidity(
        prev_amount1,
        prev_amount2,
        added_amount1,
        added_amount2,
        total_supply,
    );
    println!("expected amt from adding liquidity {:?}", expected_amount);
    assert_eq!(sheet.get_cached(&pool_address.into()), expected_amount);
    Ok(())
}

pub fn check_add_liquidity_runtime_balance(
    runtime_balances: &mut BalanceSheet<IndexPointer>,
    added_amount1: u128,
    added_amount2: u128,
    added_amount3: u128,
    deployment_ids: &AmmTestDeploymentIds,
) -> Result<()> {
    runtime_balances.increase(
        &deployment_ids.owned_token_1_deployment.into(),
        added_amount1,
    );
    runtime_balances.increase(
        &deployment_ids.owned_token_2_deployment.into(),
        added_amount2,
    );
    runtime_balances.increase(
        &deployment_ids.owned_token_3_deployment.into(),
        added_amount3,
    );

    let sheet = get_sheet_for_runtime();
    assert_eq!(sheet, runtime_balances.clone());

    let sheet_lazy = get_lazy_sheet_for_runtime();
    assert_eq!(sheet_lazy, runtime_balances.clone());

    Ok(())
}
