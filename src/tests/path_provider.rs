use alkanes_support::cellpack::Cellpack;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use init_pools::init_block_with_amm_pool;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations, ProtoruneRuneId};

use crate::tests::helper::path_provider::create_path_provider_insert_path_block;
use crate::tests::helper::*;
use crate::tests::std::path_provider_build;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_binary_deployed_to_id, assert_revert_context,
    get_last_outpoint_sheet,
};
use alkanes::view;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_path_provider() -> Result<()> {
    clear();
    let (test_block, deployment_ids) = init_block_with_amm_pool(false)?;

    let block_height = 840_000;

    index_block(&test_block, block_height)?;

    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_path_provider_deployment.clone(),
        path_provider_build::get_bytes(),
    );

    let sheet = get_last_outpoint_sheet(&test_block)?;
    assert_eq!(
        sheet.get(&ProtoruneRuneId { block: 2, tx: 11 }),
        10,
        "No authtoken found",
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_path_provider_set_and_get_path() -> Result<()> {
    clear();
    let (test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    let block_height = 840_000;
    index_block(&test_block, block_height)?;

    // Define start and end alkanes for our path
    let start_alkane = deployment_ids.owned_token_1_deployment;
    let end_alkane = deployment_ids.owned_token_2_deployment;

    // Define the path we want to set (a vector of AlkaneIds)
    let path = vec![
        start_alkane,
        deployment_ids.amm_pool_1_deployment,
        end_alkane,
    ];

    let previous_outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    // Create a new block for our path provider operations
    let path_block = create_path_provider_insert_path_block(
        start_alkane,
        end_alkane,
        path.clone(),
        &deployment_ids,
        previous_outpoint,
        block_height + 1,
    );

    // Index the block with the set path transaction
    index_block(&path_block, block_height + 1)?;

    // Create another block for getting the path
    let mut get_path_block =
        protorune::test_helpers::create_block_with_coinbase_tx(block_height + 2);

    // Create a transaction that gets the path
    get_path_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_path_provider_deployment,
                inputs: vec![
                    1, // opcode for GetOptimalPath
                    start_alkane.block,
                    start_alkane.tx,
                    end_alkane.block,
                    end_alkane.tx,
                ],
            }],
            OutPoint {
                txid: path_block.txdata[path_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    // Index the block with the get path transaction
    index_block(&get_path_block, block_height + 2)?;

    // Get the outpoint for the get path transaction
    let outpoint = OutPoint {
        txid: get_path_block.txdata[get_path_block.txdata.len() - 1].compute_txid(),
        vout: 3, // The response is in vout 3
    };

    // Get the trace data for the get path transaction
    let raw_trace_data = view::trace(&outpoint)?;
    let trace_data: Trace = raw_trace_data.clone().try_into()?;
    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();
    println!("trace_data: {:?}", trace_data);
    // Verify that we got the path we set
    if let Some(return_context) = last_trace_event {
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                let data = &trace_response.inner.data;
                println!("inner data: {:?}", data);

                // The data should contain the path AlkaneIds
                // Each AlkaneId is 2 u128s (block and tx)
                assert_eq!(data.len(), path.len() * 16 * 2, "Path length mismatch");

                // Verify each AlkaneId in the path
                for i in 0..path.len() {
                    let block_offset = i * 32;
                    let tx_offset = block_offset + 16;

                    // Extract the block and tx values from the data
                    let mut block_bytes = [0u8; 16];
                    let mut tx_bytes = [0u8; 16];
                    block_bytes.copy_from_slice(&data[block_offset..block_offset + 16]);
                    tx_bytes.copy_from_slice(&data[tx_offset..tx_offset + 16]);

                    let block = u128::from_le_bytes(block_bytes);
                    let tx = u128::from_le_bytes(tx_bytes);

                    // Verify that the AlkaneId matches what we set
                    assert_eq!(block, path[i].block, "Block mismatch at index {}", i);
                    assert_eq!(tx, path[i].tx, "Tx mismatch at index {}", i);
                }
            }
            _ => panic!("Expected ReturnContext variant, but got a different variant"),
        }
    } else {
        panic!("Failed to get last_trace_event from trace data");
    }

    // Create another block for getting the path
    let mut get_nonexistent_path_block =
        protorune::test_helpers::create_block_with_coinbase_tx(block_height + 3);

    // Create a transaction that gets the path
    get_nonexistent_path_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_path_provider_deployment,
                inputs: vec![
                    1, // opcode for GetOptimalPath
                    end_alkane.block,
                    end_alkane.tx,
                    start_alkane.block,
                    start_alkane.tx,
                ],
            }],
            OutPoint {
                txid: get_path_block.txdata[get_path_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    // Index the block with the get path transaction
    index_block(&get_nonexistent_path_block, block_height + 3)?;

    // Get the outpoint for the get path transaction
    let outpoint = OutPoint {
        txid: get_nonexistent_path_block.txdata[get_nonexistent_path_block.txdata.len() - 1]
            .compute_txid(),
        vout: 3, // The response is in vout 3
    };

    // Get the trace data for the get path transaction
    let raw_trace_data = view::trace(&outpoint)?;
    let trace_data: Trace = raw_trace_data.clone().try_into()?;
    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();

    if let Some(return_context) = last_trace_event {
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                let data = &trace_response.inner.data;
                assert_eq!(data.len(), 0, "Path length should be zero");
            }
            _ => panic!("Expected ReturnContext variant, but got a different variant"),
        }
    } else {
        panic!("Failed to get last_trace_event from trace data");
    }

    Ok(())
}

#[wasm_bindgen_test]
fn test_path_provider_only_owner() -> Result<()> {
    clear();
    let (test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    let block_height = 840_000;
    index_block(&test_block, block_height)?;

    // Define start and end alkanes for our path
    let start_alkane = deployment_ids.owned_token_1_deployment;
    let end_alkane = deployment_ids.owned_token_2_deployment;

    // Define the path we want to set (a vector of AlkaneIds)
    let path = vec![
        start_alkane,
        deployment_ids.amm_pool_1_deployment,
        end_alkane,
    ];

    let previous_outpoint = OutPoint {
        txid: test_block.txdata[0].compute_txid(), //use an outpoint that doesn't have the auth token
        vout: 0,
    };

    // Create a new block for our path provider operations
    let path_block = create_path_provider_insert_path_block(
        start_alkane,
        end_alkane,
        path.clone(),
        &deployment_ids,
        previous_outpoint,
        block_height + 1,
    );

    // Index the block with the set path transaction
    index_block(&path_block, block_height + 1)?;

    // Get the outpoint for the get path transaction
    let outpoint = OutPoint {
        txid: path_block.txdata[path_block.txdata.len() - 1].compute_txid(),
        vout: 3, // The response is in vout 3
    };

    assert_revert_context(&outpoint, "Auth token is not in incoming alkanes")?;

    Ok(())
}
