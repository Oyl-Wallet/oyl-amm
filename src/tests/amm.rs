use alkanes_support::cellpack::Cellpack;
use alkanes_support::trace::Trace;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;

use crate::tests::helper::*;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{self as alkane_helpers, assert_token_id_has_no_deployment};
use alkanes::view;
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_amm_pool_normal_init() -> Result<()> {
    clear();
    let (block, _ids) = test_amm_pool_init_fixture(1000000, 1000000)?;
    let trace_result: Trace = view::trace(&OutPoint {
        txid: block.txdata[block.txdata.len() - 1].compute_txid(),
        vout: 3,
    })?
    .try_into()?;
    println!("trace: {:?}", trace_result);
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_double_init_fail() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool()?;
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_init_one_incoming_fail() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool()?;
    let input_outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_single_edict_split_tx(
        // should fail since init pool requires two alkanes, this only creates a tx with one
        1000000,
        deployment_ids.amm_pool_1_deployment.clone(),
        &mut test_block,
        input_outpoint,
    );
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![1],
            }],
            OutPoint {
                txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );
    index_block(&test_block, block_height)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_skewed_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000 / 2, 1000000)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_zero_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000, 1)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_bad_init() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool()?;
    let input_output = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        10000,
        1,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut test_block,
        &deployment_ids,
        input_output,
    );
    index_block(&test_block, block_height)?;
    assert_token_id_has_no_deployment(deployment_ids.amm_pool_1_deployment);
    let sheet = get_last_outpoint_sheet(&test_block)?;
    assert_eq!(sheet.get(&deployment_ids.amm_pool_1_deployment.into()), 0);
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_all() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_some() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let burn_amount = total_lp / 3;
    test_amm_burn_fixture(burn_amount, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_more_than_owned() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp * 2, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_all_router() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp, true)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_to_wrong_pool() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_w_router() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_large() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_w_router() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_w_router_middle_path() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    Ok(())
}
