use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_revert_context, get_last_outpoint_sheet,
};
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use init_pools::{
    calc_lp_balance_from_pool_init, init_block_with_amm_pool, insert_init_pool_liquidity_txs,
    test_amm_pool_init_fixture,
};
use oylswap_library::DEFAULT_FEE_AMOUNT_PER_1000;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::protostone::ProtostoneEdict;
use wasm_bindgen_test::wasm_bindgen_test;

use super::helper::swap::{
    check_swap_runtime_balance, insert_low_level_swap_txs, insert_swap_tokens_for_exact_tokens_txs,
};
use crate::tests::helper::*;
use alkane_helpers::clear;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;

// Test swapping with zero output amounts (should fail)
#[wasm_bindgen_test]
fn test_amm_pool_swap_zero_output() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: (1000 + DEFAULT_FEE_AMOUNT_PER_1000) * 500000 * 10000 / (500000 - 10000) / 1000, // barely doesn't satisfy the K equation with fees
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: (1000 + DEFAULT_FEE_AMOUNT_PER_1000) * 500000 * 10000 / (500000 - 10000) / 1000
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    insert_low_level_swap_txs(
        vec![ProtostoneEdict {
            id: deployment_ids.owned_token_1_deployment.into(),
            amount: (1000 + DEFAULT_FEE_AMOUNT_PER_1000) * 500000 * 10000 / (500000 - 10000) / 1000
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * DEFAULT_FEE_AMOUNT_PER_1000 * swap_out / 1000);

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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * DEFAULT_FEE_AMOUNT_PER_1000 * swap_out / 1000)
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * DEFAULT_FEE_AMOUNT_PER_1000 * swap_out / 1000)
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * DEFAULT_FEE_AMOUNT_PER_1000 * swap_out / 1000)
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
    let (init_block, deployment_ids, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    let swap_out = 10000;
    let amount_fee_cover = DEFAULT_FEE_AMOUNT_PER_1000 * amount1 * swap_out
        / ((1000 - DEFAULT_FEE_AMOUNT_PER_1000) * amount2
            - (1000 - DEFAULT_FEE_AMOUNT_PER_1000) * DEFAULT_FEE_AMOUNT_PER_1000 * swap_out / 1000)
        + 1;

    println!("amount needed to cover fee: {}", amount_fee_cover);
    let deadline = swap_block.header.time as u128;
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
        vec![2, deployment_ids.amm_pool_1_deployment.tx, 3, 0, deadline], // swap
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
fn test_amm_pool_swap_tokens_for_exact_1() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids, mut runtime_balances) =
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
        &deployment_ids,
        input_outpoint,
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
    let (init_block, deployment_ids, mut runtime_balances) =
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
        &deployment_ids,
        input_outpoint,
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
    let (init_block, deployment_ids, mut runtime_balances) =
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
        &deployment_ids,
        input_outpoint,
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
    let (init_block, deployment_ids, mut runtime_balances) =
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
        &deployment_ids,
        input_outpoint,
    );
    index_block(&swap_block, block_height)?;

    // Check that the transaction reverted with the expected error
    let outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 5,
    };

    assert_revert_context(
        &outpoint,
        "ALKANES: revert: Error: amount_in_max is higher than input amount",
    )?;

    Ok(())
}
