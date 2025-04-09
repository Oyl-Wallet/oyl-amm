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
use super::helper::swap::check_swap_runtime_balance;

#[wasm_bindgen_test]
fn test_amm_pool_normal_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000, 1000000, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_double_init_fail() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![0],
            }],
            OutPoint {
                txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );
    index_block(&test_block, block_height)?;

    common::assert_revert_context(
        &(OutPoint {
            txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
        "ALKANES: revert: Error: already initialized",
    )?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_init_one_incoming_fail() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    test_block.txdata.push(
        common::create_multiple_cellpack_with_witness_and_in_with_edicts(
            Witness::new(),
            vec![
                common::CellpackOrEdict::Edict(vec![ProtostoneEdict {
                    id: deployment_ids.owned_token_1_deployment.into(),
                    amount: 1000000,
                    output: 0,
                }]),
                common::CellpackOrEdict::Cellpack(Cellpack {
                    target: deployment_ids.amm_factory_deployment,
                    inputs: vec![1],
                }),
            ],
            OutPoint {
                txid: test_block.txdata.last().unwrap().compute_txid(),
                vout: 0,
            },
            false,
        ),
    );
    index_block(&test_block, block_height)?;

    common::assert_revert_context(
        &(OutPoint {
            txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
            vout: 4,
        }),
        "ALKANES: revert: Error: must send two runes to initialize a pool",
    )?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_duplicate_pool_fail() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut init_block_2 = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        10000,
        10000,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut init_block_2,
        &deployment_ids,
        input_outpoint,
    );
    index_block(&init_block_2, block_height)?;

    let outpoint = OutPoint {
        txid: init_block_2.txdata[init_block_2.txdata.len() - 1].compute_txid(),
        vout: 4,
    };

    // For debugging purposes
    let trace_data: Trace = view::trace(&outpoint)?.try_into()?;
    println!(
        "last_trace_event: {:?}",
        trace_data.0.lock().expect("Mutex poisoned").last()
    );

    common::assert_revert_context(&outpoint, "ALKANES: revert: Error: pool already exists")?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_skewed_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000 / 2, 1000000, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_zero_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000, 1, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_bad_init() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    let previous_outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        10000,
        1,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut test_block,
        &deployment_ids,
        previous_outpoint,
    );
    index_block(&test_block, block_height)?;
    assert_token_id_has_no_deployment(deployment_ids.amm_pool_1_deployment)?;
    let sheet = get_last_outpoint_sheet(&test_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into()),
        0
    );

    let outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 4,
    };

    // Check the second-to-last trace event
    common::assert_revert_context_at_index(
        &outpoint,
        "Overflow error in expression: root_k.checked_sub(MINIMUM_LIQUIDITY)",
        Some(-2),
    )?;

    // Check the last trace event
    common::assert_revert_context(
        &outpoint,
        "Extcall failed: ALKANES: revert: Error: Overflow error in expression: root_k.checked_sub(MINIMUM_LIQUIDITY)"
    )?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_all() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp, false, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_some() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let burn_amount = total_lp / 3;
    test_amm_burn_fixture(burn_amount, false, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_more_than_owned() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp * 2, false, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_all_router() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp, true, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_add_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.amm_pool_1_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        amount1,
        amount2,
        total_supply,
        &add_liquidity_block,
        deployment_ids.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(
        &mut runtime_balances,
        amount1,
        amount2,
        0,
        &deployment_ids,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_to_wrong_pool() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_add_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.amm_pool_2_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        0,
        0,
        total_supply,
        &add_liquidity_block,
        deployment_ids.amm_pool_2_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, 0, 0, 0, &deployment_ids)?;

    common::assert_revert_context(
        &(OutPoint {
            txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
            vout: 5,
        }),
        "ALKANES: revert: Error: unsupported alkane sent to pool",
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_w_router() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_add_liquidity_txs_w_router(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut add_liquidity_block,
        &deployment_ids,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        amount1,
        amount2,
        total_supply,
        &add_liquidity_block,
        deployment_ids.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(
        &mut runtime_balances,
        amount1,
        amount2,
        0,
        &deployment_ids,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
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
fn test_amm_pool_swap_large() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 500000;
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
fn test_amm_pool_swap_w_router() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_txs_w_router(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        &deployment_ids,
        input_outpoint,
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
fn test_amm_pool_swap_w_router_middle_path() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_txs_w_router(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
            deployment_ids.owned_token_3_deployment,
        ],
        0,
        &mut swap_block,
        &deployment_ids,
        input_outpoint,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2, amount2],
        amount_to_swap,
        deployment_ids.owned_token_3_deployment,
        &swap_block,
    )?;

    check_swap_runtime_balance(
        vec![amount1, amount2, amount2],
        &mut runtime_balances,
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_3_deployment,
    )?;
    Ok(())
}

// #[wasm_bindgen_test]
// fn test_amm_pool_swap_oyl() -> Result<()> {
//     clear();
//     let (amount1, amount2) = (500000, 500000);
//     let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, true)?;
//     let block_height = 840_001;
//     let mut swap_block = create_block_with_coinbase_tx(block_height);
//     let input_outpoint = OutPoint {
//         txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
//         vout: 0,
//     };
//     let amount_to_swap = 10000;
//     insert_swap_txs(
//         amount_to_swap,
//         deployment_ids.owned_token_1_deployment,
//         0,
//         &mut swap_block,
//         input_outpoint,
//         deployment_ids.amm_pool_1_deployment,
//     );
//     index_block(&swap_block, block_height)?;

//     check_swap_lp_balance(
//         vec![amount1, amount2],
//         amount_to_swap,
//         deployment_ids.owned_token_2_deployment,
//         &swap_block,
//     )?;
//     Ok(())
// }

#[wasm_bindgen_test]
fn test_amm_pool_name() -> Result<()> {
    clear();
    // Initialize a pool
    let (block, deployment_ids, _) = test_amm_pool_init_fixture(1000000, 1000000, false)?;

    // Create a new block for testing the name
    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);

    // Call opcode 99 on the pool to get its name
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_pool_1_deployment,
                inputs: vec![99],
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height)?;

    // Get the trace data from the transaction
    let outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let trace_data = view::trace(&outpoint)?;

    // Convert trace data to string for easier searching
    let trace_str = String::from_utf8_lossy(&trace_data);

    // The expected pool name based on the feedback
    let expected_name = "OWNED / OWNED LP";

    // Check if the trace data contains the expected name
    assert!(
        trace_str.contains(expected_name),
        "Trace data should contain the name '{}', but it doesn't",
        expected_name
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_details() -> Result<()> {
    clear();
    // Initialize a pool
    let (block, deployment_ids, _) = test_amm_pool_init_fixture(1000000, 1000000, false)?;

    // Create a new block for testing the pool details
    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);

    // Call opcode 999 on the pool to get its pool details including the name
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_pool_1_deployment,
                inputs: vec![999],
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height)?;

    // Get the trace data from the transaction
    let outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let trace_data = view::trace(&outpoint)?;

    // Convert trace data to string for easier searching
    let trace_str = String::from_utf8_lossy(&trace_data);

    // The expected pool name
    let expected_name = "OWNED / OWNED LP";

    // Check if the trace data contains the expected name
    assert!(
        trace_str.contains(expected_name),
        "Trace data should contain the name '{}', but it doesn't",
        expected_name
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_get_num_pools() -> Result<()> {
    clear();
    let (block, deployment_ids, _) = test_amm_pool_init_fixture(1000000, 1000000, false)?;

    let block_height = 840_000;

    let mut test_block = protorune::test_helpers::create_block_with_coinbase_tx(block_height + 1);

    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![4],
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height + 1)?;

    let outpoint_3 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let raw_trace_data = view::trace(&outpoint_3)?;
    let trace_data: Trace = raw_trace_data.clone().try_into()?;

    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();

    // Access the data field from the trace response
    if let Some(return_context) = last_trace_event {
        // Use pattern matching to extract the data field from the TraceEvent enum
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                // Now we have the TraceResponse, access the data field
                let data = &trace_response.inner.data;

                // Assert that the first element of the data array is 2
                assert_eq!(
                    data[0], 2,
                    "Expected first element of data to be 2, but got {}",
                    data[0]
                );

                println!("Successfully verified data[0] = {}", data[0]);
            }
            _ => panic!("Expected ReturnContext variant, but got a different variant"),
        }
    } else {
        panic!("Failed to get last_trace_event from trace data");
    }

    Ok(())
}

#[wasm_bindgen_test]
fn test_find_existing_pool_id() -> Result<()> {
    clear();
    let (block, deployment_ids, _) = test_amm_pool_init_fixture(1000000, 1000000, false)?;

    let block_height = 840_000;

    let mut test_block = protorune::test_helpers::create_block_with_coinbase_tx(block_height + 1);

    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![
                    2,
                    deployment_ids.owned_token_1_deployment.block,
                    deployment_ids.owned_token_1_deployment.tx,
                    deployment_ids.owned_token_2_deployment.block,
                    deployment_ids.owned_token_2_deployment.tx,
                ],
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height + 1)?;

    let outpoint_3 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let raw_trace_data = view::trace(&outpoint_3)?;
    let trace_data: Trace = raw_trace_data.clone().try_into()?;
    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();
    // Access the data field from the trace response
    if let Some(return_context) = last_trace_event {
        // Use pattern matching to extract the data field from the TraceEvent enum
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                // Now we have the TraceResponse, access the data field
                let data = &trace_response.inner.data;

                println!("Last return data = {:?}", data);

                // Assert that the first element of the data array is 2
                assert_eq!(
                    data[0], 2,
                    "Expected first u128 of data to be 2, but got {}",
                    data[0]
                );
                assert_eq!(
                    data[16], 11,
                    "Expected second u128 of data to be 11, but got {}",
                    data[16]
                );
            }
            _ => panic!("Expected ReturnContext variant, but got a different variant"),
        }
    } else {
        panic!("Failed to get last_trace_event from trace data");
    }

    Ok(())
}

#[wasm_bindgen_test]
fn test_find_nonexisting_pool_id() -> Result<()> {
    clear();
    let (block, deployment_ids, _) = test_amm_pool_init_fixture(1000000, 1000000, false)?;

    let block_height = 840_000;

    let mut test_block = protorune::test_helpers::create_block_with_coinbase_tx(block_height + 1);

    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![2, 12, 100, 13, 101],
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height + 1)?;

    let outpoint_3 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    // Print the trace event for debugging purposes
    let raw_trace_data = view::trace(&outpoint_3)?;
    let trace_data: Trace = raw_trace_data.clone().try_into()?;
    println!(
        "last trace event {:?}",
        trace_data.0.lock().expect("Mutex poisoned").last()
    );

    common::assert_revert_context(
        &outpoint_3,
        "ALKANES: revert: wasm `unreachable` instruction executed",
    )?;

    Ok(())
}

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

    let sheet = get_sheet_for_outpoint(&test_block, test_block.txdata.len() - 1, 0)?;
    assert_eq!(
        sheet.get(&ProtoruneRuneId { block: 2, tx: 12 }),
        1,
        "No authtoken found",
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_get_all_pools() -> Result<()> {
    clear();
    let (block, deployment_ids, _) = test_amm_pool_init_fixture(1000000, 1000000, false)?;

    let block_height = 840_000;

    let mut test_block = protorune::test_helpers::create_block_with_coinbase_tx(block_height + 1);

    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![3],
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height + 1)?;

    let outpoint_3 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let raw_trace_data = view::trace(&outpoint_3)?;
    println!("Raw trace data length: {}", raw_trace_data.len());

    let trace_data: Trace = raw_trace_data.clone().try_into()?;
    println!("Trace data: {:?}", trace_data);

    let mut data_start = None;
    for i in 0..raw_trace_data.len().saturating_sub(16) {
        if raw_trace_data[i] == 2 && raw_trace_data[i + 1..i + 16].iter().all(|&b| b == 0) {
            data_start = Some(i);
            break;
        }
    }

    let start_idx =
        data_start.ok_or_else(|| anyhow::anyhow!("Could not find pool count in trace data"))?;
    println!("Found pool data at offset: {}", start_idx);

    let count_bytes: [u8; 16] = raw_trace_data[start_idx..start_idx + 16].try_into()?;
    let pool_count = u128::from_le_bytes(count_bytes) as usize;
    println!("Pool count: {}", pool_count);

    assert!(
        pool_count > 0,
        "Expected at least one pool, but got {}",
        pool_count
    );

    let expected_data_len = 16 + (pool_count * 32); // 16 bytes for count + 32 bytes per pool
    assert!(
        start_idx + expected_data_len <= raw_trace_data.len(),
        "Not enough data for {} pools. Expected at least {} bytes, but got {}",
        pool_count,
        expected_data_len,
        raw_trace_data.len() - start_idx
    );

    let mut pools = Vec::new();
    for i in 0..pool_count {
        let pool_start = start_idx + 16 + (i * 32);

        let block_bytes: [u8; 16] = raw_trace_data[pool_start..pool_start + 16].try_into()?;
        let tx_bytes: [u8; 16] = raw_trace_data[pool_start + 16..pool_start + 32].try_into()?;

        let block = u128::from_le_bytes(block_bytes);
        let tx = u128::from_le_bytes(tx_bytes);

        println!("Pool ID {}: (block={}, tx={})", i, block, tx);
        pools.push(AlkaneId::new(block, tx));
    }

    assert_eq!(
        pools.len(),
        pool_count,
        "Expected {} pool IDs, but got {}",
        pool_count,
        pools.len()
    );

    Ok(())
}
