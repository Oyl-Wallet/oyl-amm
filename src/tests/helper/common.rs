use alkanes::message::AlkaneMessageContext;
use alkanes::tests::helpers::{self as alkane_helpers};
use alkanes::view;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use bitcoin::address::NetworkChecked;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::{Address, Amount, Block, ScriptBuf, Sequence, TxIn, TxOut, Witness};
use hex;
#[allow(unused_imports)]
use metashrew::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use metashrew_support::index_pointer::KeyValuePointer;
use protorune::{balance_sheet::load_sheet, message::MessageContext, tables::RuneTable};
use protorune_support::balance_sheet::BalanceSheet;
use protorune_support::protostone::Protostone;
use protorune_support::protostone::ProtostoneEdict;
use protorune_support::utils::consensus_encode;
use std::fmt::Write;

pub struct AmmTestDeploymentIds {
    pub amm_pool_factory: AlkaneId,
    pub oyl_amm_pool_factory: AlkaneId,
    pub auth_token_factory: AlkaneId,
    pub amm_factory_deployment: AlkaneId,
    pub owned_token_1_deployment: AlkaneId,
    pub auth_token_1_deployment: AlkaneId,
    pub owned_token_2_deployment: AlkaneId,
    pub auth_token_2_deployment: AlkaneId,
    pub owned_token_3_deployment: AlkaneId,
    pub auth_token_3_deployment: AlkaneId,
    pub amm_pool_1_deployment: AlkaneId,
    pub amm_pool_2_deployment: AlkaneId,
    pub amm_router_deployment: AlkaneId,
}

pub fn insert_split_tx(
    test_block: &mut Block,
    input_outpoint: OutPoint,
    protostone_edicts: Vec<ProtostoneEdict>,
) {
    let address: Address<NetworkChecked> =
        protorune::test_helpers::get_address(&protorune::test_helpers::ADDRESS1().as_str());
    let script_pubkey = address.script_pubkey();
    let split = alkane_helpers::create_protostone_tx_with_inputs(
        vec![TxIn {
            previous_output: input_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        }],
        vec![
            TxOut {
                value: Amount::from_sat(546),
                script_pubkey: script_pubkey.clone(),
            },
            TxOut {
                value: Amount::from_sat(546),
                script_pubkey: script_pubkey.clone(),
            },
        ],
        Protostone {
            from: None,
            burn: None,
            protocol_tag: 1,
            message: vec![],
            pointer: Some(1),
            refund: None,
            edicts: protostone_edicts,
        },
    );
    test_block.txdata.push(split);
}

pub fn insert_single_edict_split_tx(
    amount: u128,
    target: AlkaneId,
    test_block: &mut Block,
    input_outpoint: OutPoint,
) {
    insert_split_tx(
        test_block,
        input_outpoint,
        vec![ProtostoneEdict {
            id: target.into(),
            amount: amount,
            output: 0,
        }],
    );
}

pub fn insert_two_edict_split_tx(
    amount1: u128,
    amount2: u128,
    token1_address: AlkaneId,
    token2_address: AlkaneId,
    test_block: &mut Block,
    input_outpoint: OutPoint,
) {
    insert_split_tx(
        test_block,
        input_outpoint,
        vec![
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
        ],
    );
}

fn get_sheet_for_outpoint(test_block: &Block, tx_num: usize, vout: u32) -> Result<BalanceSheet> {
    let outpoint = OutPoint {
        txid: test_block.txdata[tx_num].compute_txid(),
        vout,
    };
    let ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag())
        .OUTPOINT_TO_RUNES
        .select(&consensus_encode(&outpoint)?);
    let sheet = load_sheet(&ptr);
    println!(
        "balances at outpoint tx {} vout {}: {:?}",
        tx_num, vout, sheet
    );
    Ok(sheet)
}
fn get_trace_for_outpoint(test_block: &Block, tx_num: usize, vout: u32) -> Result<Vec<u8>> {
    let outpoint = OutPoint {
        txid: test_block.txdata[tx_num].compute_txid(),
        vout,
    };
    let trace = view::trace(&outpoint).unwrap();
    println!(
        "trace at outpoint tx {} vout {}: {:?}",
        tx_num,
        vout,
        hex::encode(&trace)
    );
    Ok(trace)
}

pub fn get_last_outpoint_sheet(test_block: &Block) -> Result<BalanceSheet> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 1, 0)
}

pub fn get_sheet_with_pool_1_init(test_block: &Block) -> Result<BalanceSheet> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 3, 0)
}

pub fn get_sheet_with_remaining_lp_after_burn(test_block: &Block) -> Result<BalanceSheet> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 2, 1)
}
pub fn get_trace_after_burn(test_block: &Block) -> Result<Vec<u8>> {
    let len = test_block.txdata.len();
    get_trace_for_outpoint(test_block, len - 2, 3)
}
