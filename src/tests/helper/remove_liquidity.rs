use crate::tests::helper::init_pools::{
    calc_lp_balance_from_pool_init, test_amm_pool_init_fixture,
};
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{self as alkane_helpers};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use protorune_support::protostone::ProtostoneEdict;
use std::fmt::Write;

use super::common::*;

fn _insert_remove_liquidity_txs(
    amount: u128,
    test_block: &mut Block,
    pool_address: AlkaneId,
    input_outpoint: OutPoint,
    cellpack: Cellpack,
) {
    test_block
        .txdata
        .push(create_multiple_cellpack_with_witness_and_in_with_edicts(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(vec![ProtostoneEdict {
                    id: pool_address.into(),
                    amount: amount,
                    output: 0,
                }]),
                CellpackOrEdict::Cellpack(cellpack),
            ],
            input_outpoint,
            false,
        ));
}

pub fn insert_remove_liquidity_txs(
    amount: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
) {
    _insert_remove_liquidity_txs(
        amount,
        test_block,
        pool_address,
        input_outpoint,
        Cellpack {
            target: pool_address,
            inputs: vec![2],
        },
    )
}

pub fn insert_remove_liquidity_txs_w_router(
    amount: u128,
    test_block: &mut Block,
    deployment_ids: &AmmTestDeploymentIds,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
) {
    _insert_remove_liquidity_txs(
        amount,
        test_block,
        pool_address,
        input_outpoint,
        Cellpack {
            target: deployment_ids.amm_router_deployment,
            inputs: vec![
                2,
                token1_address.block,
                token1_address.tx,
                token2_address.block,
                token2_address.tx,
            ],
        },
    )
}

pub fn check_remove_liquidity_runtime_balance(
    runtime_balances: &mut BalanceSheet<IndexPointer>,
    removed_amount1: u128,
    removed_amount2: u128,
    lp_burned: u128,
    deployment_ids: &AmmTestDeploymentIds,
) -> Result<()> {
    runtime_balances.decrease(
        &deployment_ids.owned_token_1_deployment.into(),
        removed_amount1,
    );
    runtime_balances.decrease(
        &deployment_ids.owned_token_2_deployment.into(),
        removed_amount2,
    );
    runtime_balances.increase(&deployment_ids.amm_pool_1_deployment.into(), lp_burned);
    let sheet = get_sheet_for_runtime();

    assert_eq!(sheet, runtime_balances.clone());

    let sheet_lazy = get_lazy_sheet_for_runtime();

    assert_eq!(sheet_lazy, runtime_balances.clone());
    Ok(())
}

pub fn test_amm_burn_fixture(amount_burn: u128, use_router: bool, use_oyl: bool) -> Result<()> {
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let total_supply = (amount1 * amount2).sqrt();
    let (mut init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, use_oyl)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    if use_router {
        insert_remove_liquidity_txs_w_router(
            amount_burn,
            &mut test_block,
            &deployment_ids,
            input_outpoint,
            deployment_ids.amm_pool_1_deployment,
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        );
    } else {
        insert_remove_liquidity_txs(
            amount_burn,
            &mut test_block,
            input_outpoint,
            deployment_ids.amm_pool_1_deployment,
        );
    }

    index_block(&test_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&test_block)?;
    let amount_burned_true = std::cmp::min(amount_burn, total_lp);
    assert_eq!(
        sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into()),
        total_lp - amount_burned_true
    );

    let owned_alkane_sheets = get_last_outpoint_sheet(&test_block)?;
    let amount_returned_1 = amount_burned_true * amount1 / total_supply;
    assert_eq!(
        owned_alkane_sheets.get_cached(&deployment_ids.owned_token_1_deployment.into()),
        amount_returned_1
    );
    let amount_returned_2 = amount_burned_true * amount2 / total_supply;
    assert_eq!(
        owned_alkane_sheets.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        amount_returned_2
    );
    check_remove_liquidity_runtime_balance(
        &mut runtime_balances,
        amount_returned_1,
        amount_returned_2,
        amount_burned_true,
        &deployment_ids,
    )?;
    Ok(())
}
