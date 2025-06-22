use crate::tests::std::{example_flashswap_build, factory_build, oyl_token_build, pool_build};
use alkanes::indexer::index_block;
use alkanes::precompiled::{
    alkanes_std_auth_token_build, alkanes_std_beacon_proxy_build, alkanes_std_owned_token_build,
    alkanes_std_upgradeable_beacon_build, alkanes_std_upgradeable_build,
};
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_binary_deployed_to_id,
    create_multiple_cellpack_with_witness_and_in, get_last_outpoint_sheet,
    get_lazy_sheet_for_runtime, get_sheet_for_runtime, BinaryAndCellpack,
};
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

pub const INIT_AMT_TOKEN1: u128 = 1_000_000_000_000_000_000_000u128;
pub const INIT_AMT_TOKEN2: u128 = 2_000_000_000_000_000_000_000u128;
pub const INIT_AMT_TOKEN3: u128 = 1_000_000_000_000_000_000_000u128;
pub const INIT_AMT_OYL: u128 = 1_000_000_000_000_000_000_000u128;

pub fn init_block_with_amm_pool() -> Result<Block> {
    let cellpack_pairs: Vec<BinaryAndCellpack> = [
        //amm pool init (in factory space so new pools can copy this code)
        BinaryAndCellpack {
            binary: pool_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: AMM_FACTORY_ID,
                },
                inputs: vec![50],
            },
        },
        //auth token factory init
        BinaryAndCellpack {
            binary: alkanes_std_auth_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: AUTH_TOKEN_FACTORY_ID,
                },
                inputs: vec![100],
            },
        },
        //amm factory initial deploy, no initialize call since behind proxy
        BinaryAndCellpack {
            binary: factory_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId { block: 3, tx: 2 },
                inputs: vec![50],
            },
        },
        // deploy the proxy and point to factory logic impl
        BinaryAndCellpack {
            binary: alkanes_std_upgradeable_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: DEPLOYMENT_IDS.amm_factory_proxy.tx,
                },
                inputs: vec![
                    0x7fff,
                    DEPLOYMENT_IDS.amm_factory_logic_impl.block,
                    DEPLOYMENT_IDS.amm_factory_logic_impl.tx,
                    1,
                ],
            },
        },
        // now do init with the proxy
        BinaryAndCellpack::cellpack_only(Cellpack {
            target: DEPLOYMENT_IDS.amm_factory_proxy,
            inputs: vec![
                0,
                AMM_FACTORY_ID,
                10, // 10 auth tokens
            ],
        }),
        // token 1 init 1 auth token and mint 1000000 owned tokens. Also deploys owned token contract at {2,2}
        BinaryAndCellpack {
            binary: alkanes_std_owned_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId { block: 1, tx: 0 },
                inputs: vec![0, 1, INIT_AMT_TOKEN1],
            },
        },
        // token 2 init 1 auth token and mint 2000000 owned tokens
        BinaryAndCellpack::cellpack_only(Cellpack {
            target: AlkaneId {
                block: 5,
                tx: DEPLOYMENT_IDS.owned_token_1_deployment.tx,
            }, // factory creation of owned token using {2, 2} as the factory
            inputs: vec![0, 1, INIT_AMT_TOKEN2],
        }),
        // token 3 init 1 auth token and mint 1000000 owned tokens
        BinaryAndCellpack::cellpack_only(Cellpack {
            target: AlkaneId {
                block: 5,
                tx: DEPLOYMENT_IDS.owned_token_1_deployment.tx,
            }, // factory creation of owned token using {2, 2} as the factory
            inputs: vec![0, 1, INIT_AMT_TOKEN1],
        }),
        // oyl token init 1 auth token and mint 1000000 owned tokens.
        BinaryAndCellpack {
            binary: oyl_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId { block: 1, tx: 0 }, // factory creation of owned token using {2, 2} as the factory
                inputs: vec![
                    0,
                    INIT_AMT_OYL,
                    u128::from_le_bytes(*b"OYL Token\0\0\0\0\0\0\0"),
                    u128::from_le_bytes(*b"OYL\0\0\0\0\0\0\0\0\0\0\0\0\0"),
                ],
            },
        },
        BinaryAndCellpack {
            binary: example_flashswap_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId { block: 1, tx: 0 },
                inputs: vec![0],
            },
        },
    ]
    .into();
    let test_block = alkane_helpers::init_with_cellpack_pairs(cellpack_pairs);

    return Ok(test_block);
}

pub fn assert_contracts_correct_ids() -> Result<()> {
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.amm_pool_factory.clone(),
        pool_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.auth_token_factory.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );

    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.amm_factory_proxy.clone(),
        alkanes_std_upgradeable_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.amm_factory_logic_impl.clone(),
        factory_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.owned_token_1_deployment.clone(),
        alkanes_std_owned_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.owned_token_2_deployment.clone(),
        alkanes_std_owned_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.owned_token_3_deployment.clone(),
        alkanes_std_owned_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.oyl_token_deployment.clone(),
        oyl_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.auth_token_1_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.auth_token_2_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.auth_token_3_deployment.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.amm_pool_1_deployment.clone(),
        pool_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        DEPLOYMENT_IDS.amm_pool_2_deployment.clone(),
        pool_build::get_bytes(),
    );
    Ok(())
}

pub fn insert_init_pool_liquidity_txs(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    test_block: &mut Block,
    previous_output: OutPoint,
) {
    test_block
        .txdata
        .push(create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: DEPLOYMENT_IDS.amm_factory_proxy,
                inputs: vec![
                    1,
                    token1_address.block,
                    token1_address.tx,
                    token2_address.block,
                    token2_address.tx,
                    amount1,
                    amount2,
                ],
            }],
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

pub fn check_init_liquidity_balance(
    amount1: u128,
    amount2: u128,
    test_block: &Block,
) -> Result<()> {
    let sheet = get_last_outpoint_sheet(test_block)?;
    let expected_amount = calc_lp_balance_from_pool_init(amount1, amount2);
    println!(
        "expected amt from init {:?} {:?}",
        sheet.get_cached(&DEPLOYMENT_IDS.amm_pool_1_deployment.into()),
        expected_amount
    );
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.amm_pool_1_deployment.into()),
        expected_amount
    );
    assert_eq!(
        sheet.get_cached(&DEPLOYMENT_IDS.amm_pool_2_deployment.into()),
        expected_amount
    );
    assert_eq!(
        sheet.get(&DEPLOYMENT_IDS.owned_token_1_deployment.into()),
        INIT_AMT_TOKEN1 - amount1
    );
    assert_eq!(
        sheet.get(&DEPLOYMENT_IDS.owned_token_2_deployment.into()),
        INIT_AMT_TOKEN2 - amount1 - amount2
    );
    assert_eq!(
        sheet.get(&DEPLOYMENT_IDS.owned_token_3_deployment.into()),
        INIT_AMT_TOKEN3 - amount2
    );

    Ok(())
}

pub fn check_and_get_init_liquidity_runtime_balance(
    amount1: u128,
    amount2: u128,
) -> Result<BalanceSheet<IndexPointer>> {
    let mut initial_runtime_balances: BalanceSheet<IndexPointer> =
        BalanceSheet::<IndexPointer>::new();
    initial_runtime_balances.set(&DEPLOYMENT_IDS.owned_token_1_deployment.into(), amount1);
    initial_runtime_balances.set(
        &DEPLOYMENT_IDS.owned_token_2_deployment.into(),
        amount1 + amount2,
    );
    initial_runtime_balances.set(&DEPLOYMENT_IDS.owned_token_3_deployment.into(), amount2);
    let sheet = get_sheet_for_runtime();
    assert_eq!(sheet, initial_runtime_balances);
    let lazy_sheet = get_lazy_sheet_for_runtime();
    assert_eq!(lazy_sheet, initial_runtime_balances);
    Ok(initial_runtime_balances)
}

pub fn test_amm_pool_init_fixture(
    amount1: u128,
    amount2: u128,
) -> Result<(Block, BalanceSheet<IndexPointer>)> {
    let block_height = 840_000;
    let mut test_block = init_block_with_amm_pool()?;
    let mut previous_outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        DEPLOYMENT_IDS.owned_token_1_deployment,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        &mut test_block,
        previous_outpoint,
    );

    previous_outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        amount1,
        amount2,
        DEPLOYMENT_IDS.owned_token_2_deployment,
        DEPLOYMENT_IDS.owned_token_3_deployment,
        &mut test_block,
        previous_outpoint,
    );

    index_block(&test_block, block_height)?;
    assert_contracts_correct_ids()?;
    check_init_liquidity_balance(amount1, amount2, &test_block)?;
    let init_runtime_balance = check_and_get_init_liquidity_runtime_balance(amount1, amount2)?;
    Ok((test_block, init_runtime_balance))
}
