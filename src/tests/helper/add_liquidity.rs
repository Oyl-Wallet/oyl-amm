use crate::tests::helper::common::create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers;
use alkanes::tests::helpers::{
    create_multiple_cellpack_with_witness_and_in, get_last_outpoint_sheet,
    get_lazy_sheet_for_runtime, get_sheet_for_runtime,
};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
use num::integer::Roots;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use protorune_support::protostone::ProtostoneEdict;

#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::cmp::min;
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

pub fn insert_add_liquidity_checked_txs(
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    amount_a_desired: u128,
    amount_b_desired: u128,
    amount_a_min: u128,
    amount_b_min: u128,
    deadline: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    deployment_ids: &AmmTestDeploymentIds,
) {
    test_block
        .txdata
        .push(create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_proxy,
                inputs: vec![
                    11,
                    2,
                    token1_address.tx,
                    2,
                    token2_address.tx,
                    amount_a_desired,
                    amount_b_desired,
                    amount_a_min,
                    amount_b_min,
                    deadline,
                ],
            }],
            input_outpoint,
            false,
        ));
}

pub fn calc_lp_balance_from_add_liquidity(
    prev_amount1: u128,
    prev_amount2: u128,
    added_amount1: u128,
    added_amount2: u128,
    total_supply: u128,
) -> u128 {
    min(
        total_supply * added_amount1 / prev_amount1,
        total_supply * added_amount2 / prev_amount2,
    )
}

pub fn check_add_liquidity_lp_balance(
    prev_amount1: u128,
    prev_amount2: u128,
    prev_lp_amount: u128,
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
    assert_eq!(
        sheet.get_cached(&pool_address.into()) - prev_lp_amount,
        expected_amount
    );
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
    )?;
    runtime_balances.increase(
        &deployment_ids.owned_token_2_deployment.into(),
        added_amount2,
    )?;
    runtime_balances.increase(
        &deployment_ids.owned_token_3_deployment.into(),
        added_amount3,
    )?;

    let sheet = get_sheet_for_runtime();
    assert_eq!(sheet, runtime_balances.clone());

    let sheet_lazy = get_lazy_sheet_for_runtime();
    assert_eq!(sheet_lazy, runtime_balances.clone());

    Ok(())
}
