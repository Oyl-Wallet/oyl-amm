use crate::tests::helper::init_pools::{
    calc_lp_balance_from_pool_init, test_amm_pool_init_fixture, INIT_AMT_TOKEN1, INIT_AMT_TOKEN2,
};
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, get_last_outpoint_sheet, get_lazy_sheet_for_runtime,
    get_sheet_for_runtime,
};
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

pub fn insert_remove_liquidity_txs(
    amount: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
    separate_leftovers: bool,
) {
    test_block.txdata.push(
        create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(vec![ProtostoneEdict {
                    id: pool_address.into(),
                    amount: amount,
                    output: 0,
                }]),
                CellpackOrEdict::Cellpack(Cellpack {
                    target: pool_address,
                    inputs: vec![2],
                }),
            ],
            input_outpoint,
            false,
            separate_leftovers,
        ),
    );
}

pub fn check_remove_liquidity_runtime_balance(
    runtime_balances: &mut BalanceSheet<IndexPointer>,
    removed_amount1: u128,
    removed_amount2: u128,
    lp_burned: u128,
) -> Result<()> {
    runtime_balances.decrease(
        &DEPLOYMENT_IDS.owned_token_1_deployment.into(),
        removed_amount1,
    );
    runtime_balances.decrease(
        &DEPLOYMENT_IDS.owned_token_2_deployment.into(),
        removed_amount2,
    );
    runtime_balances.increase(&DEPLOYMENT_IDS.amm_pool_1_deployment.into(), lp_burned);
    let sheet = get_sheet_for_runtime();

    assert_eq!(sheet, runtime_balances.clone());

    let sheet_lazy = get_lazy_sheet_for_runtime();

    assert_eq!(sheet_lazy, runtime_balances.clone());
    Ok(())
}

pub fn test_amm_burn_fixture(amount_burn: u128) -> Result<()> {
    let (amount1, amount2) = (1000000, 1000000);
    let (amount1_leftover, amount2_leftover) =
        (INIT_AMT_TOKEN1 - amount1, INIT_AMT_TOKEN2 - 2 * amount2);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let total_supply = (amount1 * amount2).sqrt();
    let (mut init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_remove_liquidity_txs(
        amount_burn,
        &mut test_block,
        input_outpoint,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
        false,
    );

    index_block(&test_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&test_block)?;
    let amount_burned_true = std::cmp::min(amount_burn, total_lp);
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.amm_pool_1_deployment.into()),
        total_lp - amount_burned_true
    );

    let amount_returned_1 = amount_burned_true * amount1 / total_supply;
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.owned_token_1_deployment.into()) - amount1_leftover,
        amount_returned_1
    );
    let amount_returned_2 = amount_burned_true * amount2 / total_supply;
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.owned_token_2_deployment.into()) - amount2_leftover,
        amount_returned_2
    );
    check_remove_liquidity_runtime_balance(
        &mut runtime_balances,
        amount_returned_1,
        amount_returned_2,
        amount_burned_true,
    )?;
    Ok(())
}
