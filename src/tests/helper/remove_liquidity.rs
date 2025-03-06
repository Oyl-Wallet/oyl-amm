use crate::tests::helper::init_pools::{
    calc_lp_balance_from_pool_init, test_amm_pool_init_fixture,
};
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{self as alkane_helpers};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;
use std::fmt::Write;

use super::common::*;

fn _insert_remove_liquidity_txs(
    amount: u128,
    test_block: &mut Block,
    pool_address: AlkaneId,
    input_outpoint: OutPoint,
    cellpack: Cellpack,
) {
    insert_single_edict_split_tx(amount, pool_address, test_block, input_outpoint);
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![cellpack],
            OutPoint {
                txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );
}

pub fn insert_remove_liquidity_txs(
    amount: u128,
    test_block: &mut Block,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
) {
    _insert_remove_liquidity_txs(
        amount,
        test_block,
        pool_address,
        input_outpoint,
        Cellpack {
            target: pool_address,
            inputs: vec![2],
        },
    )
}

pub fn insert_remove_liquidity_txs_w_router(
    amount: u128,
    test_block: &mut Block,
    deployment_ids: &AmmTestDeploymentIds,
    input_outpoint: OutPoint,
    pool_address: AlkaneId,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
) {
    _insert_remove_liquidity_txs(
        amount,
        test_block,
        pool_address,
        input_outpoint,
        Cellpack {
            target: deployment_ids.amm_router_deployment,
            inputs: vec![
                2,
                token1_address.block,
                token1_address.tx,
                token2_address.block,
                token2_address.tx,
            ],
        },
    )
}

pub fn test_amm_burn_fixture(amount_burn: u128, use_router: bool, use_oyl: bool) -> Result<()> {
    let (amount1, amount2) = (1000000, 1000000);
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let total_supply = (amount1 * amount2).sqrt();
    let (mut init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, use_oyl)?;

    let block_height = 840_001;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    if use_router {
        insert_remove_liquidity_txs_w_router(
            amount_burn,
            &mut test_block,
            &deployment_ids,
            input_outpoint,
            deployment_ids.amm_pool_1_deployment,
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        );
    } else {
        insert_remove_liquidity_txs(
            amount_burn,
            &mut test_block,
            input_outpoint,
            deployment_ids.amm_pool_1_deployment,
        );
    }

    index_block(&test_block, block_height)?;

    let sheet = get_sheet_with_remaining_lp_after_burn(&test_block)?;
    let amount_burned_true = std::cmp::min(amount_burn, total_lp);
    assert_eq!(
        sheet.get(&deployment_ids.amm_pool_1_deployment.into()),
        total_lp - amount_burned_true
    );

    let owned_alkane_sheets = get_last_outpoint_sheet(&test_block)?;
    assert_eq!(
        owned_alkane_sheets.get(&deployment_ids.owned_token_1_deployment.into()),
        amount_burned_true * amount1 / total_supply
    );
    assert_eq!(
        owned_alkane_sheets.get(&deployment_ids.owned_token_2_deployment.into()),
        amount_burned_true * amount2 / total_supply
    );
    Ok(())
}
