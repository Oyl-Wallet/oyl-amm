use add_liquidity::{
    check_add_liquidity_lp_balance, insert_add_liquidity_txs, insert_add_liquidity_txs_w_router,
};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::response::ExtendedCallResponse;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use common::{get_last_outpoint_sheet, get_sheet_for_outpoint};
use init_pools::{
    assert_contracts_correct_ids, calc_lp_balance_from_pool_init, init_block_with_amm_pool,
    insert_init_pool_liquidity_txs, test_amm_pool_init_fixture,
};
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations, ProtoruneRuneId};
use protorune_support::protostone::ProtostoneEdict;
use remove_liquidity::test_amm_burn_fixture;
use swap::{check_swap_lp_balance, insert_swap_txs, insert_swap_txs_w_router};

use crate::tests::helper::*;
use crate::tests::std::path_provider_build;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_binary_deployed_to_id, assert_token_id_has_no_deployment,
};
use alkanes::view;
use alkanes_support::id::AlkaneId;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

use super::helper::add_liquidity::check_add_liquidity_runtime_balance;
use super::helper::path_provider::create_path_provider_insert_path_block;
use super::helper::swap::check_swap_runtime_balance;

#[wasm_bindgen_test]
fn test_amm_pool_swap_no_oyl() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, true)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        0,
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        &swap_block,
    )?;

    check_swap_runtime_balance(
        vec![amount1, amount2],
        &mut runtime_balances,
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_small_no_fee_burn() -> Result<()> {
    clear();
    let (amount1, amount2) = (50000, 50000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, true)?;

    // Define start and end alkanes for our path
    let start_alkane = deployment_ids.owned_token_2_deployment;
    let end_alkane = deployment_ids.oyl_token_deployment;

    // Define the path we want to set (a vector of AlkaneIds)
    let path = vec![start_alkane, end_alkane];

    let mut previous_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    // Create a new block for our path provider operations
    let mut path_block = create_path_provider_insert_path_block(
        start_alkane,
        end_alkane,
        path.clone(),
        &deployment_ids,
        previous_outpoint,
        840_001,
    );

    previous_outpoint = OutPoint {
        txid: path_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.oyl_token_deployment,
        &mut path_block,
        &deployment_ids,
        previous_outpoint,
    );

    runtime_balances.increase(&deployment_ids.owned_token_2_deployment.into(), amount1);
    runtime_balances.increase(&deployment_ids.oyl_token_deployment.into(), amount2);

    // Index the block with the set path transaction
    index_block(&path_block, 840_001)?;

    let block_height = 840_002;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: path_block.txdata[path_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 1;
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        0,
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        &swap_block,
    )?;

    check_swap_runtime_balance(
        vec![amount1, amount2],
        &mut runtime_balances,
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_oyl() -> Result<()> {
    clear();
    let (amount1, amount2) = (100000, 100000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, true)?;

    // Define start and end alkanes for our path
    let start_alkane = deployment_ids.owned_token_2_deployment;
    let end_alkane = deployment_ids.oyl_token_deployment;

    // Define the path we want to set (a vector of AlkaneIds)
    let path = vec![start_alkane, end_alkane];

    let mut previous_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    // Create a new block for our path provider operations
    let mut path_block = create_path_provider_insert_path_block(
        start_alkane,
        end_alkane,
        path.clone(),
        &deployment_ids,
        previous_outpoint,
        840_001,
    );

    previous_outpoint = OutPoint {
        txid: path_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.oyl_token_deployment,
        &mut path_block,
        &deployment_ids,
        previous_outpoint,
    );

    runtime_balances.increase(&deployment_ids.owned_token_2_deployment.into(), amount1);
    runtime_balances.increase(&deployment_ids.oyl_token_deployment.into(), amount2);

    // Index the block with the set path transaction
    index_block(&path_block, 840_001)?;

    let block_height = 840_002;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: path_block.txdata[path_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 50000;
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        0,
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        &swap_block,
    )?;

    check_swap_runtime_balance(
        vec![amount1, amount2],
        &mut runtime_balances,
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
    )?;

    // TODO: check that the oyl pool now has less oyl tokens
    Ok(())
}
