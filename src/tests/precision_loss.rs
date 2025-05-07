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
    let mut attacker_token1 = 1_000_000_000_000_000_0u128;
    let mut attacker_token2 = 1_000u128;
    let (init_token1, init_token2) = (attacker_token1, attacker_token2);

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
        let add_amount1 = attacker_token1;
        let add_amount2 = 1u128;

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

    // Assert that the attacker has gained tokens or the invariant has increased
    assert!(attacker_token1 == init_token1, "Attacker gained token1");
    assert!(attacker_token2 == init_token2, "Attacker gained token2");
    assert!(
        final_invariant == initial_invariant,
        "Attacker changed invariant"
    );

    Ok(())
}
