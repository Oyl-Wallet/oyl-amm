use add_liquidity::{check_add_liquidity_lp_balance, insert_add_liquidity_txs};
use alkanes_runtime_pool::{MINIMUM_LIQUIDITY, PRECISION};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use init_pools::{calc_lp_balance_from_pool_init, test_amm_pool_init_fixture};
use metashrew_support::byte_view::ByteView;
use num::integer::Roots;
use oylswap_library::{StorableU256, U256};
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::protostone::ProtostoneEdict;
use remove_liquidity::test_amm_burn_fixture;

use crate::tests::helper::add_liquidity::insert_add_liquidity_checked_txs;
use crate::tests::helper::remove_liquidity::{
    check_burn_balances, check_remove_liquidity_runtime_balance,
    insert_remove_liquidity_checked_txs,
};
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
fn test_amm_pool_burn_all() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_some() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let burn_amount = total_lp / 3;
    test_amm_burn_fixture(burn_amount)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_more_than_owned() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp * 2)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_checked() -> Result<()> {
    clear();
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let amount_burn = total_lp / 2;
    let (mut init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = test_block.header.time as u128;
    insert_remove_liquidity_checked_txs(
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        amount_burn,
        (amount1 - MINIMUM_LIQUIDITY) / 2,
        (amount2 - MINIMUM_LIQUIDITY) / 2,
        deadline,
        &mut test_block,
        input_outpoint,
        &deployment_ids,
    );

    index_block(&test_block, block_height)?;

    let amount_burned_true = std::cmp::min(amount_burn, total_lp);

    let (amount_returned_1, amount_returned_2) = check_burn_balances(
        &test_block,
        amount_burned_true,
        total_lp,
        amount1,
        amount2,
        &deployment_ids,
    )?;

    check_remove_liquidity_runtime_balance(
        &mut runtime_balances,
        amount_returned_1,
        amount_returned_2,
        amount_burned_true,
        &deployment_ids,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_checked_fail_1() -> Result<()> {
    clear();
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let amount_burn = total_lp / 2;
    let (mut init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = test_block.header.time as u128;
    insert_remove_liquidity_checked_txs(
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        amount_burn,
        (amount1 - MINIMUM_LIQUIDITY) / 2 + 1,
        (amount2 - MINIMUM_LIQUIDITY) / 2,
        deadline,
        &mut test_block,
        input_outpoint,
        &deployment_ids,
    );

    index_block(&test_block, block_height)?;

    assert_revert_context(
        &(OutPoint {
            txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
        "ALKANES: revert: Error: INSUFFICIENT_A_AMOUNT",
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_checked_fail_2() -> Result<()> {
    clear();
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let amount_burn = total_lp / 2;
    let (mut init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = test_block.header.time as u128;
    insert_remove_liquidity_checked_txs(
        deployment_ids.owned_token_2_deployment,
        deployment_ids.owned_token_1_deployment,
        amount_burn,
        (amount2 - MINIMUM_LIQUIDITY) / 2,
        (amount1 - MINIMUM_LIQUIDITY) / 2 + 1,
        deadline,
        &mut test_block,
        input_outpoint,
        &deployment_ids,
    );

    index_block(&test_block, block_height)?;

    assert_revert_context(
        &(OutPoint {
            txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
        "ALKANES: revert: Error: INSUFFICIENT_B_AMOUNT",
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_checked_fail_3() -> Result<()> {
    clear();
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let amount_burn = total_lp / 2;
    let (mut init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let deadline = test_block.header.time as u128;
    insert_remove_liquidity_checked_txs(
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        amount_burn,
        (amount1 - MINIMUM_LIQUIDITY) / 2,
        (amount2 - MINIMUM_LIQUIDITY) / 2 + 1,
        deadline,
        &mut test_block,
        input_outpoint,
        &deployment_ids,
    );

    index_block(&test_block, block_height)?;

    assert_revert_context(
        &(OutPoint {
            txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
        "ALKANES: revert: Error: INSUFFICIENT_B_AMOUNT",
    )?;
    Ok(())
}
