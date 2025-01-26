use crate::tests::std::{
    factory_build, oyl_factory_build, oyl_pool_build, pool_build, router_build,
};
use alkanes::indexer::index_block;
use alkanes::precompiled::{alkanes_std_auth_token_build, alkanes_std_owned_token_build};
use alkanes::tests::helpers::{self as alkane_helpers, assert_binary_deployed_to_id};
use alkanes_runtime_pool::MINIMUM_LIQUIDITY;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::constants::{AMM_FACTORY_ID, AUTH_TOKEN_FACTORY_ID};
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use num::integer::Roots;
use std::fmt::Write;

use super::common::*;

pub const OYL_AMM_POOL_FACTORY_ID: u128 = 0xf041;

pub fn init_block_with_amm_pool() -> Result<(Block, AmmTestDeploymentIds)> {
    let cellpacks: Vec<Cellpack> = [
        //amm pool init (in factory space so new pools can copy this code)
        Cellpack {
            target: AlkaneId {
                block: 3,
                tx: AMM_FACTORY_ID,
            },
            inputs: vec![50],
        },
        Cellpack {
            target: AlkaneId {
                block: 3,
                tx: OYL_AMM_POOL_FACTORY_ID,
            },
            inputs: vec![50],
        },
        //auth token factory init
        Cellpack {
            target: AlkaneId {
                block: 3,
                tx: AUTH_TOKEN_FACTORY_ID,
            },
            inputs: vec![100],
        },
        //amm factory
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0, AMM_FACTORY_ID],
        },
        // token 1 init 1 auth token and mint 1000000 owned tokens
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0, 1, 1000000],
        },
        // token 2 init 1 auth token and mint 1000000 owned tokens
        Cellpack {
            target: AlkaneId { block: 5, tx: 2 }, // factory creation of owned token using {2, 2} as the factory. Then it deploys to {2,4}
            inputs: vec![0, 1, 2000000],
        },
        // token 2 init 1 auth token and mint 1000000 owned tokens
        Cellpack {
            target: AlkaneId { block: 5, tx: 2 }, // factory creation of owned token using {2, 2} as the factory. Then it deploys to {2,6}
            inputs: vec![0, 1, 1000000],
        },
        // router
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0, 2, 1],
        },
        //oyl amm factory
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0, OYL_AMM_POOL_FACTORY_ID],
        },
    ]
    .into();
    let test_block = alkane_helpers::init_with_multiple_cellpacks_with_tx(
        [
            pool_build::get_bytes(),
            oyl_pool_build::get_bytes(),
            alkanes_std_auth_token_build::get_bytes(),
            factory_build::get_bytes(),
            alkanes_std_owned_token_build::get_bytes(),
            [].into(),
            [].into(),
            router_build::get_bytes(),
            oyl_factory_build::get_bytes(),
        ]
        .into(),
        cellpacks,
    );
    let mut tx_iterator = (1..).into_iter();
    // note: the order that these are defined matters, since the tx_terator will increment by one
    let deployed_ids = AmmTestDeploymentIds {
        amm_pool_factory: AlkaneId {
            block: 4,
            tx: AMM_FACTORY_ID,
        },
        oyl_amm_pool_factory: AlkaneId {
            block: 4,
            tx: OYL_AMM_POOL_FACTORY_ID,
        },
        auth_token_factory: AlkaneId {
            block: 4,
            tx: AUTH_TOKEN_FACTORY_ID,
        },
        amm_factory_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        owned_token_1_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        auth_token_1_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        owned_token_2_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        auth_token_2_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        owned_token_3_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        auth_token_3_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        amm_router_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        oyl_amm_factory_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        amm_pool_1_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
        amm_pool_2_deployment: AlkaneId {
            block: 2,
            tx: tx_iterator.next().unwrap(),
        },
    };
    return Ok((test_block, deployed_ids));
}

pub fn assert_contracts_correct_ids(
    deployment_ids: &AmmTestDeploymentIds,
    use_oyl: bool,
) -> Result<()> {
    let pool_binary = if use_oyl {
        oyl_pool_build::get_bytes()
    } else {
        pool_build::get_bytes()
    };
    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_pool_factory.clone(),
        pool_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.oyl_amm_pool_factory.clone(),
        oyl_pool_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.auth_token_factory.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );

    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_factory_deployment.clone(),
        factory_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.owned_token_1_deployment.clone(),
        alkanes_std_owned_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.owned_token_2_deployment.clone(),
        alkanes_std_owned_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.auth_token_1_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.auth_token_2_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.auth_token_3_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.auth_token_3_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_pool_1_deployment.clone(),
        pool_binary.clone(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_pool_2_deployment.clone(),
        pool_binary.clone(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_router_deployment.clone(),
        router_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.oyl_amm_factory_deployment.clone(),
        oyl_factory_build::get_bytes(),
    );
    Ok(())
}

pub fn insert_init_pool_liquidity_txs(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    test_block: &mut Block,
    deployment_ids: &AmmTestDeploymentIds,
    input_outpoint_for_split: OutPoint,
) {
    insert_two_edict_split_tx(
        amount1,
        amount2,
        token1_address,
        token2_address,
        test_block,
        input_outpoint_for_split,
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
}

pub fn calc_lp_balance_from_pool_init(amount1: u128, amount2: u128) -> u128 {
    if (amount1 * amount2).sqrt() < MINIMUM_LIQUIDITY {
        return 0;
    }
    return (amount1 * amount2).sqrt() - MINIMUM_LIQUIDITY;
}

pub fn check_init_liquidity_lp_1_balance(
    amount1: u128,
    amount2: u128,
    test_block: &Block,
    deployment_ids: &AmmTestDeploymentIds,
) -> Result<()> {
    let sheet = get_sheet_with_pool_1_init(test_block)?;
    let expected_amount = calc_lp_balance_from_pool_init(amount1, amount2);
    println!("expected amt from init {:?}", expected_amount);
    assert_eq!(
        sheet.get(&deployment_ids.amm_pool_1_deployment.into()),
        expected_amount
    );
    Ok(())
}

pub fn check_init_liquidity_lp_2_balance(
    amount1: u128,
    amount2: u128,
    test_block: &Block,
    deployment_ids: &AmmTestDeploymentIds,
) -> Result<()> {
    let sheet = get_last_outpoint_sheet(test_block)?;
    let expected_amount = calc_lp_balance_from_pool_init(amount1, amount2);
    println!("expected amt from init {:?}", expected_amount);
    assert_eq!(
        sheet.get(&deployment_ids.amm_pool_2_deployment.into()),
        expected_amount
    );
    Ok(())
}

pub fn test_amm_pool_init_fixture(
    amount1: u128,
    amount2: u128,
    use_oyl: bool,
) -> Result<(Block, AmmTestDeploymentIds)> {
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool()?;
    let input_output_pool1 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut test_block,
        &deployment_ids,
        input_output_pool1,
    );
    let input_output_pool2 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.owned_token_3_deployment,
        &mut test_block,
        &deployment_ids,
        input_output_pool2,
    );
    index_block(&test_block, block_height)?;
    assert_contracts_correct_ids(&deployment_ids, use_oyl)?;
    check_init_liquidity_lp_1_balance(amount1, amount2, &test_block, &deployment_ids)?;
    check_init_liquidity_lp_2_balance(amount1, amount2, &test_block, &deployment_ids)?;
    Ok((test_block, deployment_ids))
}
