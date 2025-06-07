use add_liquidity::{check_add_liquidity_lp_balance, insert_add_liquidity_txs};
use alkanes_runtime_pool::{MINIMUM_LIQUIDITY, PRECISION};
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

use crate::tests::helper::add_liquidity::insert_add_liquidity_checked_txs;
use crate::tests::helper::common::{
    create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers, CellpackOrEdict,
    DEPLOYMENT_IDS,
};
use crate::tests::helper::remove_liquidity::{
    check_burn_balances, check_remove_liquidity_runtime_balance,
    insert_remove_liquidity_checked_txs,
};
use crate::tests::helper::*;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_revert_context, assert_revert_context_at_index,
    assert_token_id_has_no_deployment, create_multiple_cellpack_with_witness_and_in,
    get_last_outpoint_sheet,
};
use alkanes::view;
use alkanes_support::id::AlkaneId;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_amm_static_attack() -> Result<()> {
    clear();
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let amount_burn = total_lp;
    let (mut init_block, mut runtime_balances) = test_amm_pool_init_fixture(amount1, amount2)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    test_block.txdata.push(
        create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(vec![ProtostoneEdict {
                    id: DEPLOYMENT_IDS.amm_pool_1_deployment.into(),
                    amount: amount_burn,
                    output: 0,
                }]),
                CellpackOrEdict::Cellpack(Cellpack {
                    target: DEPLOYMENT_IDS.example_flashswap,
                    inputs: vec![10, 2, DEPLOYMENT_IDS.amm_pool_1_deployment.tx, amount_burn],
                }),
            ],
            input_outpoint,
            false,
            true,
        ),
    );

    index_block(&test_block, block_height)?;

    let sheet = get_last_outpoint_sheet(&test_block)?;
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.amm_pool_1_deployment.into()),
        total_lp
    );
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.owned_token_1_deployment.into()),
        0
    );
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.owned_token_2_deployment.into()),
        0
    );

    check_remove_liquidity_runtime_balance(&mut runtime_balances, 0, 0, 0)?;
    Ok(())
}
