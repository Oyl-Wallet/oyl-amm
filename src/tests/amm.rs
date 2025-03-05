use add_liquidity::{
    check_add_liquidity_lp_balance, insert_add_liquidity_txs, insert_add_liquidity_txs_w_router,
};
use alkanes_support::cellpack::Cellpack;
use alkanes_support::trace::Trace;
use anyhow::Result;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::Witness;
use common::{get_last_outpoint_sheet, insert_single_edict_split_tx};
use init_pools::{
    calc_lp_balance_from_pool_init, init_block_with_amm_pool, insert_init_pool_liquidity_txs,
    test_amm_pool_init_fixture,
};
use num::integer::Roots;
use protorune::test_helpers::create_block_with_coinbase_tx;
use remove_liquidity::test_amm_burn_fixture;
use swap::{
    check_swap_lp_balance, insert_swap_txs, insert_swap_txs_w_router, test_simulate_amount_out,
};

use crate::tests::helper::*;
use alkane_helpers::clear;
use alkanes::indexer::index_block;
use alkanes::tests::helpers::{self as alkane_helpers, assert_token_id_has_no_deployment};
use alkanes::view;
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use std::fmt::Write;
use wasm_bindgen_test::wasm_bindgen_test;
use alkanes_support::id::AlkaneId;
use metashrew_support::utils::consume_sized_int;

#[wasm_bindgen_test]
fn test_amm_pool_normal_init() -> Result<()> {
    clear();
    let (block, _ids) = test_amm_pool_init_fixture(1000000, 1000000, false)?;
    let trace_result: Trace = view::trace(
        &(OutPoint {
            txid: block.txdata[block.txdata.len() - 1].compute_txid(),
            vout: 3,
        }),
    )?
    .try_into()?;
    println!("trace: {:?}", trace_result);
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_double_init_fail() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![0],
            }],
            OutPoint {
                txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false,
        ),
    );
    index_block(&test_block, block_height)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_factory_init_one_incoming_fail() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    let input_outpoint = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_single_edict_split_tx(
        // should fail since init pool requires two alkanes, this only creates a tx with one
        1000000,
        deployment_ids.amm_pool_1_deployment.clone(),
        &mut test_block,
        input_outpoint,
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
    index_block(&test_block, block_height)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_skewed_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000 / 2, 1000000, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_zero_init() -> Result<()> {
    clear();
    test_amm_pool_init_fixture(1000000, 1, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_simulate_amount_out() -> Result<()> {
    clear();
    test_simulate_amount_out()?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_bad_init() -> Result<()> {
    clear();
    let block_height = 840_000;
    let (mut test_block, deployment_ids) = init_block_with_amm_pool(false)?;
    let input_output = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 0,
    };
    insert_init_pool_liquidity_txs(
        10000,
        1,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut test_block,
        &deployment_ids,
        input_output,
    );
    index_block(&test_block, block_height)?;
    assert_token_id_has_no_deployment(deployment_ids.amm_pool_1_deployment)?;
    let sheet = get_last_outpoint_sheet(&test_block)?;
    assert_eq!(sheet.get(&deployment_ids.amm_pool_1_deployment.into()), 0);
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_all() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp, false, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_some() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    let burn_amount = total_lp / 3;
    test_amm_burn_fixture(burn_amount, false, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_more_than_owned() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp * 2, false, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_burn_all_router() -> Result<()> {
    clear();
    let total_lp = calc_lp_balance_from_pool_init(1000000, 1000000);
    test_amm_burn_fixture(total_lp, true, false)?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
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
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        amount1,
        amount2,
        total_supply,
        &add_liquidity_block,
        deployment_ids.amm_pool_1_deployment,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_to_wrong_pool() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    insert_add_liquidity_txs(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        deployment_ids.amm_pool_2_deployment,
        &mut add_liquidity_block,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        0,
        0,
        total_supply,
        &add_liquidity_block,
        deployment_ids.amm_pool_2_deployment,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_add_more_liquidity_w_router() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let total_supply = (amount1 * amount2).sqrt();
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut add_liquidity_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    insert_add_liquidity_txs_w_router(
        amount1,
        amount2,
        deployment_ids.owned_token_1_deployment,
        deployment_ids.owned_token_2_deployment,
        &mut add_liquidity_block,
        &deployment_ids,
        input_outpoint,
    );
    index_block(&add_liquidity_block, block_height)?;

    check_add_liquidity_lp_balance(
        amount1,
        amount2,
        amount1,
        amount2,
        total_supply,
        &add_liquidity_block,
        deployment_ids.amm_pool_1_deployment,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    let amount_to_swap = 10000;
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        0,
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        &swap_block,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_large() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    let amount_to_swap = 500000;
    insert_swap_txs(
        amount_to_swap,
        deployment_ids.owned_token_1_deployment,
        0,
        &mut swap_block,
        input_outpoint,
        deployment_ids.amm_pool_1_deployment,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        &swap_block,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_w_router() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    let amount_to_swap = 10000;
    insert_swap_txs_w_router(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
        ],
        0,
        &mut swap_block,
        &deployment_ids,
        input_outpoint,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2],
        amount_to_swap,
        deployment_ids.owned_token_2_deployment,
        &swap_block,
    )?;
    Ok(())
}

#[wasm_bindgen_test]
fn test_amm_pool_swap_w_router_middle_path() -> Result<()> {
    clear();
    let (amount1, amount2) = (500000, 500000);
    let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, false)?;
    let block_height = 840_001;
    let mut swap_block = create_block_with_coinbase_tx(block_height);
    // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
    let input_outpoint = OutPoint {
        txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
        vout: 1,
    };
    let amount_to_swap = 10000;
    insert_swap_txs_w_router(
        amount_to_swap,
        vec![
            deployment_ids.owned_token_1_deployment,
            deployment_ids.owned_token_2_deployment,
            deployment_ids.owned_token_3_deployment,
        ],
        0,
        &mut swap_block,
        &deployment_ids,
        input_outpoint,
    );
    index_block(&swap_block, block_height)?;

    check_swap_lp_balance(
        vec![amount1, amount2, amount2],
        amount_to_swap,
        deployment_ids.owned_token_3_deployment,
        &swap_block,
    )?;
    Ok(())
}

// #[wasm_bindgen_test]
// fn test_amm_pool_swap_oyl() -> Result<()> {
//     clear();
//     let (amount1, amount2) = (500000, 500000);
//     let (init_block, deployment_ids) = test_amm_pool_init_fixture(amount1, amount2, true)?;
//     let block_height = 840_001;
//     let mut swap_block = create_block_with_coinbase_tx(block_height);
//     // split init tx puts 1000000 / 2 in vout 0, and the other is unspent at vout 1. The split tx is now 2 from the tail
//     let input_outpoint = OutPoint {
//         txid: init_block.txdata[init_block.txdata.len() - 2].compute_txid(),
//         vout: 1,
//     };
//     let amount_to_swap = 10000;
//     insert_swap_txs(
//         amount_to_swap,
//         deployment_ids.owned_token_1_deployment,
//         0,
//         &mut swap_block,
//         input_outpoint,
//         deployment_ids.amm_pool_1_deployment,
//     );
//     index_block(&swap_block, block_height)?;

//     check_swap_lp_balance(
//         vec![amount1, amount2],
//         amount_to_swap,
//         deployment_ids.owned_token_2_deployment,
//         &swap_block,
//     )?;
//     Ok(())
// }


#[wasm_bindgen_test]
fn test_get_all_pools() -> Result<()> {
    clear();
    let (block, deployment_ids) = test_amm_pool_init_fixture(1000000, 1000000, false)?;
    
    let block_height = 840_000;
    
    let mut test_block = protorune::test_helpers::create_block_with_coinbase_tx(block_height + 1);
    
    test_block.txdata.push(
        alkane_helpers::create_multiple_cellpack_with_witness_and_in(
            Witness::new(),
            vec![Cellpack {
                target: deployment_ids.amm_factory_deployment,
                inputs: vec![3], // Opcode 3 for get_all_pools
            }],
            OutPoint {
                txid: block.txdata[block.txdata.len() - 1].compute_txid(),
                vout: 0,
            },
            false
        )
    );
    
    index_block(&test_block, block_height + 1)?;
    
    let outpoint_3 = OutPoint {
        txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
        vout: 3,
    };
  
    let trace_data = view::trace(&outpoint_3)?;
    println!("Trace data length from vout 3: {}", trace_data.len());
    
    // The pool data starts at offset 87 in the trace data
    // This is where the actual return data from get_all_pools() begins
    const POOL_DATA_OFFSET: usize = 87;
    
    // Parse the pool count (first 16 bytes of the pool data)
    let pool_count_bytes: [u8; 16] = trace_data[POOL_DATA_OFFSET..POOL_DATA_OFFSET+16]
        .try_into()
        .map_err(|_| anyhow::anyhow!("Failed to read pool count"))?;
    let pool_count = u128::from_le_bytes(pool_count_bytes);
    println!("Pool count: {}", pool_count);
    
    // Parse each pool ID
    let mut pools = Vec::new();
    for i in 0..pool_count {
        let offset = POOL_DATA_OFFSET + 16 + (i as usize * 32); // 16 bytes for count, then 32 bytes per pool
        
        // Read block ID (16 bytes)
        let block_bytes: [u8; 16] = trace_data[offset..offset+16]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read block ID"))?;
        let block = u128::from_le_bytes(block_bytes);
        
        // Read tx ID (16 bytes)
        let tx_bytes: [u8; 16] = trace_data[offset+16..offset+32]
            .try_into()
            .map_err(|_| anyhow::anyhow!("Failed to read tx ID"))?;
        let tx = u128::from_le_bytes(tx_bytes);
        
        println!("Pool ID {}: ({}, {})", i, block, tx);
        pools.push(AlkaneId::new(block, tx));
    }
    
    // Verify we have the expected number of pools
    assert_eq!(pools.len() as u128, pool_count, "Expected {} pool IDs, but got {}", pool_count, pools.len());
    
    // Verify the pool IDs match what we expect
    if pools.len() >= 1 {
        assert_eq!(pools[0].block, 2, "First pool block ID should be 2");
        assert_eq!(pools[0].tx, 11, "First pool tx ID should be 11");
    }
    
    if pools.len() >= 2 {
        assert_eq!(pools[1].block, 2, "Second pool block ID should be 2");
        assert_eq!(pools[1].tx, 12, "Second pool tx ID should be 12");
    }
    
    Ok(())
}

