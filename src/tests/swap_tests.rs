use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_revert_context, get_last_outpoint_sheet,
};
use alkanes::view;
use alkanes_runtime_pool::PRECISION;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use alkanes_support::parcel::AlkaneTransfer;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use init_pools::{calc_lp_balance_from_pool_init, test_amm_pool_init_fixture};
use metashrew_support::byte_view::ByteView;
use oylswap_library::{StorableU256, DEFAULT_TOTAL_FEE_AMOUNT_PER_1000, U256};
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::balance_sheet::BalanceSheetOperations;
use protorune_support::protostone::ProtostoneEdict;
use wasm_bindgen_test::wasm_bindgen_test;

use super::helper::swap::{
    check_swap_runtime_balance, insert_low_level_swap_txs, insert_swap_tokens_for_exact_tokens_txs,
};
use crate::tests::helper::common::{check_input_tokens_refunded, AmmTestDeploymentIds};
use crate::tests::helper::swap::{
    check_swap_lp_balance, insert_swap_exact_tokens_for_tokens,
    insert_swap_exact_tokens_for_tokens_deadline, insert_swap_exact_tokens_for_tokens_no_split,
    insert_swap_tokens_for_exact_tokens_txs_no_split,
};
use crate::tests::helper::*;
use alkane_helpers::clear;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::collections::BTreeSet;
use std::fmt::Write;

#[wasm_bindgen_test]
fn test_amm_pool_swap() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_exact_tokens_for_tokens(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        0,
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
fn test_amm_pool_swap_no_split() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let init_balances = get_last_outpoint_sheet(&init_block)?;
    let amount_to_swap = 10000;
    insert_swap_exact_tokens_for_tokens_no_split(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let output_sheet = get_last_outpoint_sheet(&swap_block)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        init_balances.get(&deployment_ids.owned_token_2_deployment.into()),
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

    check_input_tokens_refunded(
        init_balances,
        output_sheet,
        BTreeSet::from_iter([
            deployment_ids.owned_token_1_deployment.into(),
            deployment_ids.owned_token_2_deployment.into(),
        ]),
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_deadline_fail() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    let deadline = (block_height - 1) as u128;

    insert_swap_exact_tokens_for_tokens_deadline(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        deadline,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    // Check the last trace event
    assert_revert_context(&outpoint, "EXPIRED deadline")?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_large() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 500000;
    insert_swap_exact_tokens_for_tokens(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );

    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        0,
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
fn test_amm_pool_swap_w_factory_middle_path() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_exact_tokens_for_tokens(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
            deployment_ids.owned_token_3_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2, amount2],
        amount_to_swap,
        0,
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

// Test swapping with zero output amounts (should fail)
#[wasm_bindgen_test]
fn test_amm_pool_swap_zero_output() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: 10000,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        0,
        AlkaneId::new(0, 0),
        vec![],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(
        &outpoint,
        "ALKANES: revert: Error: INSUFFICIENT_OUTPUT_AMOUNT",
    )?;

    Ok(())
}

// Test swapping more tokens than available in the pool (should fail)
#[wasm_bindgen_test]
fn test_amm_pool_swap_insufficient_liquidity() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: 10000,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        amount2 + 1,
        AlkaneId::new(0, 0),
        vec![],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: INSUFFICIENT_LIQUIDITY")?;

    Ok(())
}

// Test swapping with insufficient input amount (should fail)
#[wasm_bindgen_test]
fn test_amm_pool_swap_insufficient_input() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: 1, // Very small amount that won't satisfy the K equation
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        AlkaneId::new(0, 0),
        vec![],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: K is not increasing")?;

    Ok(())
}

// Test swapping with insufficient input amount (should fail)
#[wasm_bindgen_test]
fn test_amm_pool_swap_insufficient_input_2() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: 500000 * 10000 / (500000 - 10000), // satisfies the K equation without fees
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        AlkaneId::new(0, 0),
        vec![],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: K is not increasing")?;

    Ok(())
}
// Test swapping with insufficient input amount (should fail)
#[wasm_bindgen_test]
fn test_amm_pool_swap_insufficient_input_3() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: (1000 + DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * 500000 * 10000 / (500000 - 10000) / 1000, // barely doesn't satisfy the K equation with fees
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        AlkaneId::new(0, 0),
        vec![],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: K is not increasing")?;

    Ok(())
}
#[wasm_bindgen_test]
fn test_amm_pool_swap_sufficient_input() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: (1000 + DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * 500000 * 10000 / (500000 - 10000) / 1000
                + 1,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        AlkaneId::new(0, 0),
        vec![],
    );

    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        10000
    );
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_zero_to() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: (1000 + DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * 500000 * 10000 / (500000 - 10000) / 1000
                + 1,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        AlkaneId::new(0, 0),
        vec![1],
    );

    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        10000
    );
    Ok(())
}
// Test swapping with data parameter (callback functionality)
#[wasm_bindgen_test]
fn test_amm_pool_swap_with_data() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: 1,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        deployment_ids.example_flashswap,
        vec![0],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: K is not increasing")?;

    Ok(())
}

// Test swapping with data parameter (callback functionality)
#[wasm_bindgen_test]
fn test_amm_pool_swap_with_data_2() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: 1,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        10000,
        deployment_ids.example_flashswap,
        vec![1],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: K is not increasing")?;

    Ok(())
}

// Test swapping with data parameter (callback functionality)
#[wasm_bindgen_test]
fn test_amm_pool_swap_with_data_3() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * swap_out / 1000);

    println!("amount needed to cover fee: {}", amount_fee_cover);

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: amount_fee_cover,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        swap_out,
        deployment_ids.example_flashswap,
        vec![1],
    );

    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: K is not increasing")?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_with_data_4() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * swap_out / 1000)
        + 1;

    println!("amount needed to cover fee: {}", amount_fee_cover);

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: amount_fee_cover,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        swap_out,
        deployment_ids.example_flashswap,
        vec![1],
    );

    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(sheet.cached.balances.len(), 0);
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_with_reentrancy_add_liquidity() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * swap_out / 1000)
        + 1;

    println!("amount needed to cover fee: {}", amount_fee_cover);

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: amount_fee_cover,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        swap_out,
        deployment_ids.example_flashswap,
        vec![2, deployment_ids.amm_pool_1_deployment.tx, 1], // add liquidity
    );

    index_block(&swap_block, block_height)?;

    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: LOCKED")?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_with_reentrancy_burn() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * swap_out / 1000)
        + 1;

    println!("amount needed to cover fee: {}", amount_fee_cover);

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: amount_fee_cover,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        swap_out,
        deployment_ids.example_flashswap,
        vec![2, deployment_ids.amm_pool_1_deployment.tx, 2], // burn
    );

    index_block(&swap_block, block_height)?;

    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: LOCKED")?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_with_reentrancy_swap() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_TOTAL_FEE_AMOUNT_PER_1000) * DEFAULT_TOTAL_FEE_AMOUNT_PER_1000 * swap_out / 1000)
        + 1;

    println!("amount needed to cover fee: {}", amount_fee_cover);
    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: amount_fee_cover,
            output: 0,
        }],
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
        0,
        swap_out,
        deployment_ids.example_flashswap,
        vec![2, deployment_ids.amm_pool_1_deployment.tx, 3, 0, 0, 0, 0, 0], // swap
    );

    index_block(&swap_block, block_height)?;

    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: LOCKED")?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_no_split() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let init_balances = get_last_outpoint_sheet(&init_block)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_swap_tokens_for_exact_tokens_txs_no_split(
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        5000,
        10000,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_2_deployment.into())
            - init_balances.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        5000
    );
    assert_eq!(
        init_balances.get_cached(&deployment_ids.owned_token_1_deployment.into())
            - sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()),
        5076
    );
    check_input_tokens_refunded(
        init_balances,
        sheet,
        BTreeSet::from_iter([
            deployment_ids.owned_token_1_deployment.into(),
            deployment_ids.owned_token_2_deployment.into(),
        ]),
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_1() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_tokens_for_exact_tokens_txs(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        5000,
        10000,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        5000
    );
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()),
        4924
    );
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_2() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_tokens_for_exact_tokens_txs(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        5000,
        5076,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        5000
    );
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()),
        4924
    );
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_3() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_tokens_for_exact_tokens_txs(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        5000,
        5075,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(&outpoint, "ALKANES: revert: Error: EXCESSIVE_INPUT_AMOUNT")?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_4() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_tokens_for_exact_tokens_txs(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        5000,
        10001,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()),
        5000
    );
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()),
        4924
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_5() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_tokens_for_exact_tokens_txs(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        10000,
        10256,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(
        &outpoint,
        &format!(
            "Extcall failed: balance underflow, transferring({:?}), from({:?}), balance(10000)",
            AlkaneTransfer {
                id: deployment_ids.owned_token_1_deployment,
                value: 10256
            },
            deployment_ids.amm_factory_proxy,
        ),
    )?;

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_tokens_for_exact_middle() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;
    insert_swap_tokens_for_exact_tokens_txs(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
            deployment_ids.owned_token_3_deployment,
        ],
        5000,
        7000,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&swap_block)?;
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_3_deployment.into()),
        5000
    );
    assert_eq!(
        sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()),
        4846
    );
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_price_swap() -> Result<()> {
    clear();
    // Initialize a pool
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(1000000, 1000000)?;

    // Create a new block for testing the pool details
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    swap_block.header.time = init_block.header.time + 100;
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;

    insert_swap_exact_tokens_for_tokens(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let mut test_block = create_block_with_coinbase_tx(block_height + 1);

    // Call opcode 999 on the pool to get its pool details including the name
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_pool_1_deployment,
                inputs: vec![98],
            }],
            OutPoint {
                txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height + 1)?;

    // Get the trace data from the transaction
    let outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let trace_data: Trace = view::trace(&outpoint)?.try_into()?;

    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();

    // Access the data field from the trace response
    if let Some(return_context) = last_trace_event {
        // Use pattern matching to extract the data field from the TraceEvent enum
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                // Now we have the TraceResponse, access the data field
                let data = &trace_response.inner.data;
                assert_eq!(
                    data[16], 100,
                    "first price event should be 1:1, * time(100)"
                );
                assert_eq!(
                    data[48], 100,
                    "first price event should be 1:1, * time(100)"
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
fn test_amm_price_swap_2() -> Result<()> {
    clear();
    // Initialize a pool
    let (init_block, _, deployment_ids) = test_amm_pool_init_fixture(1000000, 1000000)?;

    // Create a new block for testing the pool details
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    swap_block.header.time = init_block.header.time + 100;
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000;

    insert_swap_exact_tokens_for_tokens(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block, block_height)?;

    let mut swap_block2 = create_block_with_coinbase_tx(block_height + 1);
    swap_block2.header.time = swap_block.header.time + 100;
    let input_outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 2,
    };
    let amount_to_swap = 10000;

    insert_swap_exact_tokens_for_tokens(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block2,
        input_outpoint,
        &deployment_ids,
    );
    index_block(&swap_block2, block_height + 1)?;

    let mut test_block = create_block_with_coinbase_tx(block_height + 2);

    // Call opcode 999 on the pool to get its pool details including the name
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_pool_1_deployment,
                inputs: vec![98],
            }],
            OutPoint {
                txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );

    index_block(&test_block, block_height + 2)?;

    // Get the trace data from the transaction
    let outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };

    let trace_data: Trace = view::trace(&outpoint)?.try_into()?;

    let last_trace_event = trace_data.0.lock().expect("Mutex poisoned").last().cloned();

    // Access the data field from the trace response
    if let Some(return_context) = last_trace_event {
        // Use pattern matching to extract the data field from the TraceEvent enum
        match return_context {
            TraceEvent::ReturnContext(trace_response) => {
                // Now we have the TraceResponse, access the data field
                let data = &trace_response.inner.data;
                println!("{:?}", data);
                let p0: U256 = StorableU256::from_bytes(data[0..32].to_vec()).into();
                let p1: U256 = StorableU256::from_bytes(data[32..64].to_vec()).into();

                println!(
                    "{:?}.{:?}",
                    p0 >> U256::from(PRECISION),
                    p0 & U256::from(u128::MAX)
                );
                println!(
                    "{:?}.{:?}",
                    p1 >> U256::from(PRECISION),
                    p1 & U256::from(u128::MAX)
                );

                assert_eq!(p0 >> U256::from(PRECISION), U256::from(198));
                assert_eq!(
                    p0 & U256::from(u128::MAX),
                    U256::from(11758271886674012252348290890464069812u128)
                );
                assert_eq!(p1 >> U256::from(PRECISION), U256::from(202));
                assert_eq!(
                    p1 & U256::from(u128::MAX),
                    U256::from(1650292961922242512542177858976124688u128)
                );
            }
            _ => panic!("Expected ReturnContext variant, but got a different variant"),
        }
    } else {
        panic!("Failed to get last_trace_event from trace data");
    }

    Ok(())
}
