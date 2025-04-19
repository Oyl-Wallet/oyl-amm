use add_liquidity::{
    check_add_liquidity_lp_balance, insert_add_liquidity_txs, insert_add_liquidity_txs_w_router,
};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::response::ExtendedCallResponse;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
use common::{get_last_outpoint_sheet, get_sheet_for_outpoint};
use init_pools::{
    assert_contracts_correct_ids, calc_lp_balance_from_pool_init, init_block_with_amm_pool,
    insert_init_pool_liquidity_txs, test_amm_pool_init_fixture,
};
use metashrew_support::utils::consume_u128;
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

fn check_reserves_amount(amount1: u128, amount2: u128, check_block: &Block) -> Result<()> {
    // Get the outpoint for the get path transaction
    let outpoint = OutPoint {
        txid: check_block.txdata[check_block.txdata.len() - 1].compute_txid(),
        vout: 3, // The response is in vout 3
    };

    // Get the trace data for the get path transaction
    let raw_trace_data = view::trace(&outpoint)?;
    let trace_data: Trace = raw_trace_data.clone().try_into()?;
    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();
    // Verify that we got the path we set
    if let Some(return_context) = last_trace_event {
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                let data = &trace_response.inner.data;
                let mut cursor = std::io::Cursor::<Vec<u8>>::new(data.clone());
                let alkane_a =
                    AlkaneId::new(consume_u128(&mut cursor)?, consume_u128(&mut cursor)?);
                let alkane_b =
                    AlkaneId::new(consume_u128(&mut cursor)?, consume_u128(&mut cursor)?);
                let reserve_a = consume_u128(&mut cursor)?;
                let reserve_b = consume_u128(&mut cursor)?;
                println!("{:?} {} {:?} {}", alkane_a, reserve_a, alkane_b, reserve_b);
                assert_eq!(reserve_a, amount1);
                assert_eq!(reserve_b, amount2);
            }
            _ => panic!("Expected ReturnContext variant, but got a different variant"),
        }
    } else {
        panic!("Failed to get last_trace_event from trace data");
    }
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

    let oyl_token2_pool = AlkaneId { block: 2, tx: 15 };

    let mut check_block = protorune::test_helpers::create_block_with_coinbase_tx(840_003);
    check_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: oyl_token2_pool,
                inputs: vec![999],
            }],
            OutPoint {
                txid: swap_block.txdata.last().unwrap().compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&check_block, 840_003)?;

    check_reserves_amount(133277, 99946, &check_block)?;

    // swap some from oyl pool to ensure no infinite loop occurs
    let mut swap_oyl_block = protorune::test_helpers::create_block_with_coinbase_tx(840_004);
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        0,
        &mut swap_oyl_block,
        OutPoint {
            txid: check_block.txdata.last().unwrap().compute_txid(),
            vout: 0,
        },
        oyl_token2_pool,
    );
    let last_outpoint = OutPoint {
        txid: swap_oyl_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.oyl_token_deployment,
        0,
        &mut swap_oyl_block,
        last_outpoint,
        oyl_token2_pool,
    );
    swap_oyl_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: oyl_token2_pool,
                inputs: vec![999],
            }],
            OutPoint {
                txid: swap_oyl_block.txdata.last().unwrap().compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&swap_oyl_block, 840_004)?;

    check_reserves_amount(133277, 99853, &swap_oyl_block)?;

    Ok(())
}
