use add_liquidity::{check_add_liquidity_lp_balance, insert_add_liquidity_txs};
use alkanes_runtime_pool::PRECISION;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use init_pools::{
    calc_lp_balance_from_pool_init, init_block_with_amm_pool, insert_init_pool_liquidity_txs,
    test_amm_pool_init_fixture,
};
use metashrew_support::byte_view::ByteView;
use num::integer::Roots;
use oylswap_library::{StorableU256, U256};
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::protostone::ProtostoneEdict;
use remove_liquidity::test_amm_burn_fixture;
use swap::{
    check_swap_lp_balance, insert_swap_exact_tokens_for_tokens,
    insert_swap_exact_tokens_for_tokens_deadline,
};

use crate::tests::helper::add_liquidity::insert_add_liquidity_checked_txs;
use crate::tests::helper::common::DEPLOYMENT_IDS;
use crate::tests::helper::*;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_revert_context, assert_revert_context_at_index,
    assert_token_id_has_no_deployment, get_last_outpoint_sheet,
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
fn test_amm_pool_add_more_liquidity() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_add_liquidity_txs(
        amount1,
        amount2,
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        0,
        amount1,
        amount2,
        total_supply,
        &add_liquidity_block,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, amount1, amount2, 0)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_checked() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = add_liquidity_block.header.time as u128;
    insert_add_liquidity_checked_txs(
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        amount1,
        amount2,
        amount1,
        amount2,
        deadline,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        499000,
        amount1,
        amount2,
        total_supply,
        &add_liquidity_block,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, amount1, amount2, 0)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_one_sided() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_add_liquidity_txs(
        amount1,
        1,
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        0,
        amount1,
        1,
        total_supply,
        &add_liquidity_block,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, amount1, 1, 0)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_one_sided_checked_1() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = add_liquidity_block.header.time as u128;
    insert_add_liquidity_checked_txs(
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        amount1 / 2,
        amount2,
        amount1,
        amount2 / 2,
        deadline,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        499000,
        amount1 / 2,
        amount2 / 2,
        total_supply,
        &add_liquidity_block,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, amount1 / 2, amount2 / 2, 0)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_one_sided_checked_2() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = add_liquidity_block.header.time as u128;
    insert_add_liquidity_checked_txs(
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        amount1,
        amount2 / 2,
        amount1 / 2,
        amount2,
        deadline,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        499000,
        amount1 / 2,
        amount2 / 2,
        total_supply,
        &add_liquidity_block,
        DEPLOYMENT_IDS.amm_pool_1_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, amount1 / 2, amount2 / 2, 0)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_one_sided_checked_1_fail() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = add_liquidity_block.header.time as u128;
    insert_add_liquidity_checked_txs(
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        amount1 / 2,
        amount2,
        amount1,
        amount2,
        deadline,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    assert_revert_context(
        &(OutPoint {
            txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
        "ALKANES: revert: Error: INSUFFICIENT_B_AMOUNT",
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_one_sided_checked_2_fail() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, _) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = add_liquidity_block.header.time as u128;
    insert_add_liquidity_checked_txs(
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        amount1,
        amount2 / 2,
        amount1,
        amount2,
        deadline,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    assert_revert_context(
        &(OutPoint {
            txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
        "ALKANES: revert: Error: INSUFFICIENT_A_AMOUNT",
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_to_wrong_pool() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_add_liquidity_txs(
        amount1,
        amount2,
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        DEPLOYMENT_IDS.amm_pool_2_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        0,
        0,
        0,
        total_supply,
        &add_liquidity_block,
        DEPLOYMENT_IDS.amm_pool_2_deployment,
    )?;

    check_add_liquidity_runtime_balance(&mut runtime_balances, 0, 0, 0)?;

    assert_revert_context(
        &(OutPoint {
            txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
            vout: 5,
        }),
        "ALKANES: revert: Error: unsupported alkane sent to pool",
    )?;
    Ok(())
}
