use alkanes_support::cellpack::Cellpack;
use alkanes_support::response::ExtendedCallResponse;
use alkanes_support::trace::{Trace, TraceEvent};
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
use common::{get_last_outpoint_sheet, get_sheet_for_outpoint};
use init_pools::{
    assert_contracts_correct_ids, calc_lp_balance_from_pool_init, init_block_with_amm_pool,
    insert_init_pool_liquidity_txs, test_amm_pool_init_fixture,
};
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations, ProtoruneRuneId};

use crate::tests::helper::*;
use crate::tests::std::path_provider_build;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_binary_deployed_to_id, assert_token_id_has_no_deployment,
};
use alkanes::view;
use alkanes_support::id::AlkaneId;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;

use super::common::AmmTestDeploymentIds;

pub fn create_path_provider_insert_path_block(
    start_alkane: AlkaneId,
    end_alkane: AlkaneId,
    path: Vec<AlkaneId>,
    deployment_ids: &AmmTestDeploymentIds,
    previous_outpoint: OutPoint,
    height: u32,
) -> Block {
    // Create a new block for our path provider operations
    let mut path_block = protorune::test_helpers::create_block_with_coinbase_tx(height);

    let mut input = vec![
        2, // opcode for SetPath
        start_alkane.block,
        start_alkane.tx,
        end_alkane.block,
        end_alkane.tx,
        // Add the path AlkaneIds
        path.len() as u128,
    ];
    for id in path {
        input.append(&mut id.into());
    }

    // Create a transaction that sets the path
    path_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_path_provider_deployment,
                inputs: input,
            }],
            previous_outpoint,
            false,
        ),
    );
    path_block
}
