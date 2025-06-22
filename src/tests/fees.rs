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

#[wasm_bindgen_test]
fn test_amm_pool_swap_fee_claim() -> Result<()> {
    clear();
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

    let block_height = 840_002;

    let mut swap_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: add_liquidity_block.txdata[add_liquidity_block.txdata.len() - 1].compute_txid(),
        vout: 2,
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
        first_swap_sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()) * 1005 / 1000,
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

    let user_fees_earned_a =
        lp_sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()) - amount1;
    let user_fees_earned_b =
        lp_sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()) - amount2;

    let implied_total_fees_a =
        fees_sheet.get_cached(&deployment_ids.owned_token_1_deployment.into()) * 10 / 4;
    let implied_total_fees_b =
        fees_sheet.get_cached(&deployment_ids.owned_token_2_deployment.into()) * 10 / 4;

    assert_eq!(
        divide_round_u128(implied_total_fees_a * 6 / 10 / 2, 1000), // 60% goes to LPs, half of that goes to this LP position, then we take 3 digits of
        divide_round_u128(user_fees_earned_a, 1000)
    );
    assert_eq!(
        divide_round_u128(implied_total_fees_b * 6 / 10 / 2, 1000),
        divide_round_u128(user_fees_earned_b, 1000)
    );

    Ok(())
}
