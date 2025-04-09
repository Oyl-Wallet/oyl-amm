use crate::tests::std::{factory_build, oyl_pool_build, pool_build, router_build};
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
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use num::integer::Roots;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use protorune_support::protostone::ProtostoneEdict;
use std::fmt::Write;

use super::common::*;

pub const OYL_AMM_POOL_FACTORY_ID: u128 = 0xf041;
pub const INIT_AMT_TOKEN1: u128 = 1000000;
pub const INIT_AMT_TOKEN2: u128 = 2000000;
pub const INIT_AMT_TOKEN3: u128 = 1000000;

pub fn init_block_with_amm_pool(use_oyl: bool) -> Result<(Block, AmmTestDeploymentIds)> {
    let pool_id = if use_oyl {
        OYL_AMM_POOL_FACTORY_ID
    } else {
        AMM_FACTORY_ID
    };
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
            inputs: vec![0, pool_id],
        },
        // token 1 init 1 auth token and mint 1000000 owned tokens. Also deploys owned token contract at {2,2}
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0, 1, INIT_AMT_TOKEN1],
        },
        // token 2 init 1 auth token and mint 2000000 owned tokens
        Cellpack {
            target: AlkaneId { block: 5, tx: 2 }, // factory creation of owned token using {2, 2} as the factory
            inputs: vec![0, 1, INIT_AMT_TOKEN2],
        },
        // token 3 init 1 auth token and mint 1000000 owned tokens
        Cellpack {
            target: AlkaneId { block: 5, tx: 2 }, // factory creation of owned token using {2, 2} as the factory
            inputs: vec![0, 1, INIT_AMT_TOKEN1],
        },
        // oyl token init 1 auth token and mint 1000000 owned tokens.
        Cellpack {
            target: AlkaneId { block: 5, tx: 2 }, // factory creation of owned token using {2, 2} as the factory
            inputs: vec![0, 1, 1000000],
        },
        // router
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0, 2, 1],
        },
        // path provider
        Cellpack {
            target: AlkaneId { block: 1, tx: 0 },
            inputs: vec![0],
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
            [].into(),
            router_build::get_bytes(),
            path_provider_build::get_bytes(),
        ]
        .into(),
        cellpacks,
    );
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
        amm_factory_deployment: AlkaneId { block: 2, tx: 1 },
        owned_token_1_deployment: AlkaneId { block: 2, tx: 2 },
        auth_token_1_deployment: AlkaneId { block: 2, tx: 3 },
        owned_token_2_deployment: AlkaneId { block: 2, tx: 4 },
        auth_token_2_deployment: AlkaneId { block: 2, tx: 5 },
        owned_token_3_deployment: AlkaneId { block: 2, tx: 6 },
        auth_token_3_deployment: AlkaneId { block: 2, tx: 7 },
        oyl_token_deployment: AlkaneId { block: 2, tx: 8 },
        oyl_auth_token_deployment: AlkaneId { block: 2, tx: 9 },
        amm_router_deployment: AlkaneId { block: 2, tx: 10 },
        amm_pool_1_deployment: AlkaneId { block: 2, tx: 13 },
        amm_pool_2_deployment: AlkaneId { block: 2, tx: 14 },
        amm_path_provider_deployment: AlkaneId { block: 2, tx: 11 },
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
        deployment_ids.owned_token_3_deployment.clone(),
        alkanes_std_owned_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.oyl_token_deployment.clone(),
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
        deployment_ids.oyl_auth_token_deployment.clone(),
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
        deployment_ids.amm_path_provider_deployment.clone(),
        path_provider_build::get_bytes(),
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
    previous_output: OutPoint,
) {
    test_block
        .txdata
        .push(create_multiple_cellpack_with_witness_and_in_with_edicts(
            Witness::new(),
            vec![
                CellpackOrEdict::Edict(vec![
                    ProtostoneEdict {
                        id: token1_address.into(),
                        amount: amount1,
                        output: 0,
                    },
                    ProtostoneEdict {
                        id: token2_address.into(),
                        amount: amount2,
                        output: 0,
                    },
                ]),
                CellpackOrEdict::Cellpack(Cellpack {
                    target: deployment_ids.amm_factory_deployment,
                    inputs: vec![1],
                }),
            ],
            previous_output,
            false,
        ));
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
    let sheet = get_last_outpoint_sheet(test_block)?;
    let expected_amount = calc_lp_balance_from_pool_init(amount1, amount2);
    println!(
        "expected amt from init {:?} {:?}",
        sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into()),
        expected_amount
    );
    assert_eq!(
        sheet.get_cached(&deployment_ids.amm_pool_1_deployment.into()),
        expected_amount
    );
    assert_eq!(
        sheet.get(&deployment_ids.owned_token_1_deployment.into()),
        INIT_AMT_TOKEN1 - amount1
    );
    assert_eq!(
        sheet.get(&deployment_ids.owned_token_2_deployment.into()),
        INIT_AMT_TOKEN2 - amount1 - amount2
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
        sheet.get_cached(&deployment_ids.amm_pool_2_deployment.into()),
        expected_amount
    );
    assert_eq!(
        sheet.get(&deployment_ids.owned_token_2_deployment.into()),
        INIT_AMT_TOKEN2 - amount1 - amount2
    );
    assert_eq!(
        sheet.get(&deployment_ids.owned_token_3_deployment.into()),
        INIT_AMT_TOKEN3 - amount2
    );
    Ok(())
}

pub fn check_and_get_init_liquidity_runtime_balance(
    amount1: u128,
    amount2: u128,
    deployment_ids: &AmmTestDeploymentIds,
) -> Result<BalanceSheet<IndexPointer>> {
    let mut initial_runtime_balances: BalanceSheet<IndexPointer> =
        BalanceSheet::<IndexPointer>::new();
    initial_runtime_balances.set(&deployment_ids.owned_token_1_deployment.into(), amount1);
    initial_runtime_balances.set(
        &deployment_ids.owned_token_2_deployment.into(),
        amount1 + amount2,
    );
    initial_runtime_balances.set(&deployment_ids.owned_token_3_deployment.into(), amount2);
    let sheet = get_sheet_for_runtime();
    assert_eq!(sheet, initial_runtime_balances);
    let lazy_sheet = get_lazy_sheet_for_runtime();
    assert_eq!(lazy_sheet, initial_runtime_balances);
    Ok(initial_runtime_balances)
}

pub fn test_amm_pool_init_fixture(
    amount1: u128,
    amount2: u128,
    use_oyl: bool,
) -> Result<(Block, AmmTestDeploymentIds, BalanceSheet<IndexPointer>)> {
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(use_oyl)?;
    let mut previous_outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut test_block,
        &deployment_ids,
        previous_outpoint,
    );

    previous_outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.owned_token_3_deployment,
        &mut test_block,
        &deployment_ids,
        previous_outpoint,
    );

    index_block(&test_block, block_height)?;
    assert_contracts_correct_ids(&deployment_ids, use_oyl)?;
    check_init_liquidity_lp_1_balance(amount1, amount2, &test_block, &deployment_ids)?;
    check_init_liquidity_lp_2_balance(amount1, amount2, &test_block, &deployment_ids)?;
    let init_runtime_balance =
        check_and_get_init_liquidity_runtime_balance(amount1, amount2, &deployment_ids)?;
    Ok((test_block, deployment_ids, init_runtime_balance))
}
