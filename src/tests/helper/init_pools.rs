use crate::tests::std::{example_flashswap_build, factory_build, oyl_token_build, pool_build};
use alkanes::indexer::index_block;
use alkanes::precompiled::{
    alkanes_std_auth_token_build, alkanes_std_beacon_proxy_build, alkanes_std_owned_token_build,
    alkanes_std_upgradeable_beacon_build, alkanes_std_upgradeable_build,
};
use alkanes::tests::helpers::{
    self as alkane_helpers, assert_binary_deployed_to_id, assert_id_points_to_alkane_id,
    create_multiple_cellpack_with_witness_and_in, get_last_outpoint_sheet,
    get_lazy_sheet_for_runtime, get_sheet_for_runtime, BinaryAndCellpack,
};
use alkanes::vm::utils::sequence_pointer;
use alkanes_runtime_pool::MINIMUM_LIQUIDITY;
use alkanes_support::cellpack::Cellpack;
use alkanes_support::constants::{AMM_FACTORY_ID, AUTH_TOKEN_FACTORY_ID};
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Block, Witness};
use metashrew_core::index_pointer::AtomicPointer;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use metashrew_support::index_pointer::KeyValuePointer;
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;
use protorune_support::balance_sheet::{BalanceSheet, BalanceSheetOperations};
use protorune_support::protostone::ProtostoneEdict;
use std::fmt::Write;

use super::common::*;

pub const INIT_AMT_TOKEN1: u128 = 1_000_000_000_000_000_000_000u128;
pub const INIT_AMT_TOKEN2: u128 = 2_000_000_000_000_000_000_000u128;
pub const INIT_AMT_TOKEN3: u128 = 1_000_000_000_000_000_000_000u128;
pub const INIT_AMT_OYL: u128 = 1_000_000_000_000_000_000_000u128;

pub fn init_factories(deployment_ids: &AmmTestDeploymentIds) -> Result<Block> {
    let block_height = 840_000;
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
                target: AlkaneId {
                    block: 3,
                    tx: AMM_FACTORY_LOGIC_IMPL_TX,
                },
                inputs: vec![50],
            },
        },
        // deploy the proxy and point to factory logic impl
        BinaryAndCellpack {
            binary: alkanes_std_upgradeable_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.amm_factory_proxy.tx,
                },
                inputs: vec![
                    0x7fff,
                    deployment_ids.amm_factory_logic_impl.block,
                    deployment_ids.amm_factory_logic_impl.tx,
                    1,
                ],
            },
        },
        // token 1 init 1 auth token and mint 1000000 owned tokens. Also deploys owned token contract at {2,2}
        BinaryAndCellpack {
            binary: alkanes_std_owned_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.owned_token_1_deployment.tx,
                },
                inputs: vec![0, 1, INIT_AMT_TOKEN1],
            },
        },
        // token 2 init 1 auth token and mint 2000000 owned tokens
        BinaryAndCellpack {
            binary: alkanes_std_owned_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.owned_token_2_deployment.tx,
                },
                inputs: vec![0, 1, INIT_AMT_TOKEN2],
            },
        },
        // token 3 init 1 auth token and mint 1000000 owned tokens
        BinaryAndCellpack {
            binary: alkanes_std_owned_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.owned_token_3_deployment.tx,
                },
                inputs: vec![0, 1, INIT_AMT_TOKEN1],
            },
        },
        // oyl token init 1 auth token and mint 1000000 owned tokens.
        BinaryAndCellpack {
            binary: oyl_token_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.oyl_token_deployment.tx,
                },
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
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.example_flashswap.tx,
                },
                inputs: vec![0],
            },
        },
        BinaryAndCellpack {
            binary: alkanes_std_beacon_proxy_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.pool_beacon_proxy.tx,
                },
                inputs: vec![0x8fff],
            },
        },
        BinaryAndCellpack {
            binary: alkanes_std_upgradeable_beacon_build::get_bytes(),
            cellpack: Cellpack {
                target: AlkaneId {
                    block: 3,
                    tx: deployment_ids.pool_upgradeable_beacon.tx,
                },
                inputs: vec![
                    0x7fff,
                    deployment_ids.amm_pool_logic_impl.block,
                    deployment_ids.amm_pool_logic_impl.tx,
                    1,
                ],
            },
        },
    ]
    .into();
    let test_block = alkane_helpers::init_with_cellpack_pairs(cellpack_pairs);
    index_block(&test_block, block_height)?;
    return Ok(test_block);
}

pub fn init_factory_proxy(
    input_outpoint: OutPoint,
    deployment_ids: &mut AmmTestDeploymentIds,
) -> Result<Block> {
    let block_height = 840_000;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let mut next_sequence_pointer = sequence_pointer(&mut AtomicPointer::default());
    let auth_sequence = next_sequence_pointer.get_value::<u128>();

    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_proxy,
                inputs: vec![
                    0,
                    POOL_BEACON_PROXY_TX,
                    deployment_ids.pool_upgradeable_beacon.block,
                    deployment_ids.pool_upgradeable_beacon.tx,
                    10, // 10 auth tokens
                ],
            }],
            input_outpoint,
            false,
        ),
    );

    index_block(&test_block, block_height)?;

    deployment_ids.amm_factory_auth_token = AlkaneId {
        block: 2,
        tx: auth_sequence,
    };

    return Ok(test_block);
}

pub fn assert_contracts_correct_ids(deployment_ids: &AmmTestDeploymentIds) -> Result<()> {
    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_pool_logic_impl.clone(),
        pool_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.auth_token_factory.clone(),
        alkanes_std_auth_token_build::get_bytes(),
    );

    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_factory_proxy.clone(),
        alkanes_std_upgradeable_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.amm_factory_logic_impl.clone(),
        factory_build::get_bytes(),
    );
    let _ = assert_binary_deployed_to_id(
        deployment_ids.oyl_token_deployment.clone(),
        oyl_token_build::get_bytes(),
    );
    let _ = assert_id_points_to_alkane_id(
        deployment_ids.amm_pool_1_deployment.clone(),
        deployment_ids.pool_beacon_proxy.clone(),
    );
    let _ = assert_id_points_to_alkane_id(
        deployment_ids.amm_pool_2_deployment.clone(),
        deployment_ids.pool_beacon_proxy.clone(),
    );
    Ok(())
}

pub fn init_pool_liquidity_txs(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    previous_output: OutPoint,
    deployment_ids: &AmmTestDeploymentIds,
) -> Result<(Block, u128)> {
    let block_height = 840_000;
    let mut test_block = create_block_with_coinbase_tx(block_height);
    let next_sequence_pointer = sequence_pointer(&mut AtomicPointer::default());
    let pool_sequence = next_sequence_pointer.get_value::<u128>();
    test_block
        .txdata
        .push(create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_proxy,
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

    index_block(&test_block, block_height)?;

    return Ok((test_block, pool_sequence));
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
        sheet.get_cached(&deployment_ids.amm_pool_2_deployment.into()),
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

pub fn amm_pool_init_setup() -> Result<(Block, AmmTestDeploymentIds)> {
    let mut deployment_ids = create_deployment_ids();

    let test_block = init_factories(&deployment_ids)?;
    println!("Init factories complete");
    let previous_outpoint = OutPoint {
        txid: test_block.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    let init_factory_proxy = init_factory_proxy(previous_outpoint, &mut deployment_ids)?;
    Ok((init_factory_proxy, deployment_ids))
}

pub fn test_amm_pool_init_fixture(
    amount1: u128,
    amount2: u128,
) -> Result<(Block, BalanceSheet<IndexPointer>, AmmTestDeploymentIds)> {
    let (init_factory_proxy, mut deployment_ids) = amm_pool_init_setup()?;

    println!("Init amm factory proxy complete");

    let previous_outpoint = OutPoint {
        txid: init_factory_proxy.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    let (init_pool_1, pool_sequence_1) = init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        previous_outpoint,
        &deployment_ids,
    )?;
    println!("Init pool 1 complete");

    deployment_ids.amm_pool_1_deployment = AlkaneId {
        block: 2,
        tx: pool_sequence_1,
    };

    let previous_outpoint = OutPoint {
        txid: init_pool_1.txdata.last().unwrap().compute_txid(),
        vout: 0,
    };
    let (init_pool_2, pool_sequence_2) = init_pool_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.owned_token_3_deployment,
        previous_outpoint,
        &deployment_ids,
    )?;
    println!("Init pool 2 complete");

    deployment_ids.amm_pool_2_deployment = AlkaneId {
        block: 2,
        tx: pool_sequence_2,
    };

    assert_contracts_correct_ids(&deployment_ids)?;
    check_init_liquidity_balance(amount1, amount2, &init_pool_2, &deployment_ids)?;
    let init_runtime_balance =
        check_and_get_init_liquidity_runtime_balance(amount1, amount2, &deployment_ids)?;
    Ok((init_pool_2, init_runtime_balance, deployment_ids))
}
