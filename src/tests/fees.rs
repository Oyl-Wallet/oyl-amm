use add_liquidity::insert_add_liquidity_txs;
use alkanes_support::cellpack::Cellpack;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use init_pools::test_amm_pool_init_fixture;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::protostone::ProtostoneEdict;

use crate::tests::helper::common::divide_round_u128;
use crate::tests::helper::remove_liquidity::insert_remove_liquidity_txs;
use crate::tests::helper::swap::insert_swap_exact_tokens_for_tokens;
use crate::tests::helper::*;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, get_last_outpoint_sheet, get_sheet_for_outpoint,
};
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use wasm_bindgen_test::wasm_bindgen_test;

use oylswap_library::{DEFAULT_TOTAL_FEE_AMOUNT_PER_1000, PROTOCOL_FEE_AMOUNT_PER_1000};

fn test_fee_fixture(custom_fee: u128) -> Result<()> {
    let (amount1, amount2) = (500000000, 500000000);
    let (init_block, mut runtime_balances, deployment_ids) =
        test_amm_pool_init_fixture(amount1, amount2)?;
    let mut add_liquidity_block = create_block_with_coinbase_tx(840_001);
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
    index_block(&add_liquidity_block, 840_001)?;

    let mut change_fee_block = create_block_with_coinbase_tx(840_001);
    let input_outpoint = OutPoint {
        txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
        vout: 2,
    };
    change_fee_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_proxy,
                inputs: vec![
                    21,
                    deployment_ids.amm_pool_1_deployment.block,
                    deployment_ids.amm_pool_1_deployment.tx,
                    custom_fee,
                ],
            }],
            input_outpoint,
            false,
        ),
    );
    index_block(&change_fee_block, 840_001)?;

    let block_height = 840_002;

    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: change_fee_block.txdata[change_fee_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    let amount_to_swap = 10000000;

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
    let swap2_input_outpoint = OutPoint {
        txid: swap_block.txdata[swap_block.txdata.len() - 1].compute_txid(),
        vout: 2,
    };
    let first_swap_sheet = get_last_outpoint_sheet(&swap_block)?;

    insert_swap_exact_tokens_for_tokens(
        first_swap_sheet.get_cached(&deployment_ids.owned_token_2_deployment.into())
            * (1000 + custom_fee)
            / 1000,
        vec![
            deployment_ids.owned_token_2_deployment,
            deployment_ids.owned_token_1_deployment,
        ],
        0,
        &mut swap_block2,
        swap2_input_outpoint,
        &deployment_ids,
    );

    index_block(&swap_block2, block_height + 1)?;

    let mut collect_block = create_block_with_coinbase_tx(block_height + 2);
    collect_block.txdata.push(
        common::create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
            Witness::new(),
            vec![
                common::CellpackOrEdict::Edict(vec![ProtostoneEdict {
                    id: deployment_ids.amm_factory_auth_token.into(),
                    amount: 1,
                    output: 0,
                }]),
                common::CellpackOrEdict::Cellpack(Cellpack {
                    target: deployment_ids.amm_factory_proxy,
                    inputs: vec![10, 2, deployment_ids.amm_pool_1_deployment.tx],
                }),
            ],
            OutPoint {
                txid: swap_block2.txdata[swap_block2.txdata.len() - 1].compute_txid(),
                vout: 2,
            },
            false,
            true,
        ),
    );
    index_block(&collect_block, block_height + 2)?;

    let sheet = get_last_outpoint_sheet(&collect_block)?;

    let mut burn_block = create_block_with_coinbase_tx(block_height + 3);

    insert_remove_liquidity_txs(
        sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into()),
        &mut burn_block,
        OutPoint {
            txid: collect_block.txdata[collect_block.txdata.len() - 1].compute_txid(),
            vout: 0,
        },
        deployment_ids.amm_pool_1_deployment,
        true,
    );
    insert_remove_liquidity_txs(
        amount1,
        &mut burn_block,
        OutPoint {
            txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
            vout: 0,
        },
        deployment_ids.amm_pool_1_deployment,
        true,
    );

    index_block(&burn_block, block_height + 3)?;

    let fees_sheet = get_sheet_for_outpoint(&burn_block, burn_block.txdata.len() - 2, 0)?;
    let lp_sheet = get_last_outpoint_sheet(&burn_block)?;

    let user_total_fees_earned = lp_sheet
        .get_cached(&deployment_ids.owned_token_1_deployment.into())
        + lp_sheet.get_cached(&deployment_ids.owned_token_2_deployment.into())
        - amount1
        - amount2;

    let implied_total_fees = (fees_sheet
        .get_cached(&deployment_ids.owned_token_1_deployment.into())
        + fees_sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()))
        * custom_fee
        / PROTOCOL_FEE_AMOUNT_PER_1000;

    // 60% goes to LPs, half of that goes to this LP position (recall init also has a lp position that isn't unraveled)
    let implied_user_fees_earned =
        implied_total_fees * (custom_fee - PROTOCOL_FEE_AMOUNT_PER_1000) / custom_fee / 2;

    assert!(
        implied_user_fees_earned.abs_diff(user_total_fees_earned) * 100 / implied_user_fees_earned
            < 5 // 5% difference tolerance allowed
    );

    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_fee_claim() -> Result<()> {
    clear();
    test_fee_fixture(DEFAULT_TOTAL_FEE_AMOUNT_PER_1000)
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_fee_claim_large_fee() -> Result<()> {
    clear();
    test_fee_fixture(200)
}
