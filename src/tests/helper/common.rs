use alkanes::message::AlkaneMessageContext;
use alkanes::tests::helpers::{self as alkane_helpers};
use alkanes::view;
use alkanes_support::trace::{Trace, TraceEvent};
use alkanes_support::{cellpack::Cellpack, id::AlkaneId};
use anyhow::Result;
use bitcoin::address::NetworkChecked;
use bitcoin::blockdata::transaction::OutPoint;
use bitcoin::transaction::Version;
use bitcoin::{Address, Amount, Block, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness};
use hex;
use metashrew_core::index_pointer::AtomicPointer;
#[allow(unused_imports)]
use metashrew_core::{get_cache, index_pointer::IndexPointer, println, stdio::stdout};
use metashrew_support::index_pointer::KeyValuePointer;
use ordinals::{Etching, Rune, Runestone};
use protorune::protostone::Protostones;
use protorune::test_helpers::{get_address, ADDRESS1};
use protorune::{balance_sheet::load_sheet, message::MessageContext, tables::RuneTable};
use protorune_support::balance_sheet::BalanceSheet;
use protorune_support::protostone::Protostone;
use protorune_support::protostone::ProtostoneEdict;
use protorune_support::utils::consensus_encode;
use std::fmt::Write;
use std::str::FromStr;

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
    pub oyl_token_deployment: AlkaneId,
    pub oyl_auth_token_deployment: AlkaneId,
    pub amm_pool_1_deployment: AlkaneId,
    pub amm_pool_2_deployment: AlkaneId,
    pub amm_router_deployment: AlkaneId,
    pub amm_path_provider_deployment: AlkaneId,
}

pub enum CellpackOrEdict {
    Cellpack(Cellpack),
    Edict(Vec<ProtostoneEdict>),
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

fn get_sheet_for_outpoint(
    test_block: &Block,
    tx_num: usize,
    vout: u32,
) -> Result<BalanceSheet<IndexPointer>> {
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

pub fn get_sheet_for_runtime() -> BalanceSheet<IndexPointer> {
    let ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag()).RUNTIME_BALANCE;
    let sheet = load_sheet(&ptr);
    println!("runtime balances: {:?}", sheet);
    sheet
}

pub fn get_lazy_sheet_for_runtime() -> BalanceSheet<IndexPointer> {
    let ptr = RuneTable::for_protocol(AlkaneMessageContext::protocol_tag()).RUNTIME_BALANCE;
    let sheet = BalanceSheet::new_ptr_backed(ptr);
    sheet
}

pub fn get_last_outpoint_sheet(test_block: &Block) -> Result<BalanceSheet<IndexPointer>> {
    let len = test_block.txdata.len();
    get_sheet_for_outpoint(test_block, len - 1, 0)
}

pub fn create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
    witness: Witness,
    cellpacks_or_edicts: Vec<CellpackOrEdict>,
    previous_output: OutPoint,
    etch: bool,
    with_leftovers_to_separate: bool,
) -> Transaction {
    let protocol_id = 1;
    let input_script = ScriptBuf::new();
    let txin = TxIn {
        previous_output,
        script_sig: input_script,
        sequence: Sequence::MAX,
        witness,
    };
    let protostones = [
        match etch {
            true => vec![Protostone {
                burn: Some(protocol_id),
                edicts: vec![],
                pointer: Some(5),
                refund: None,
                from: None,
                protocol_tag: 13, // this value must be 13 if protoburn
                message: vec![],
            }],
            false => vec![],
        },
        cellpacks_or_edicts
            .into_iter()
            .enumerate()
            .map(|(i, cellpack_or_edict)| match cellpack_or_edict {
                CellpackOrEdict::Cellpack(cellpack) => Protostone {
                    message: cellpack.encipher(),
                    pointer: Some(0),
                    refund: Some(0),
                    edicts: vec![],
                    from: None,
                    burn: None,
                    protocol_tag: protocol_id as u128,
                },
                CellpackOrEdict::Edict(edicts) => Protostone {
                    message: vec![],
                    pointer: if with_leftovers_to_separate {
                        Some(2)
                    } else {
                        Some(0)
                    },
                    refund: if with_leftovers_to_separate {
                        Some(2)
                    } else {
                        Some(0)
                    },
                    //lazy way of mapping edicts onto next protomessage
                    edicts: edicts
                        .into_iter()
                        .map(|edict| {
                            let mut edict = edict;
                            edict.output = if etch { 5 + i as u128 } else { 4 + i as u128 };
                            if with_leftovers_to_separate {
                                edict.output += 1;
                            }
                            edict
                        })
                        .collect(),
                    from: None,
                    burn: None,
                    protocol_tag: protocol_id as u128,
                },
            })
            .collect(),
    ]
    .concat();
    let etching = if etch {
        Some(Etching {
            divisibility: Some(2),
            premine: Some(1000),
            rune: Some(Rune::from_str("TESTTESTTESTTEST").unwrap()),
            spacers: Some(0),
            symbol: Some(char::from_str("A").unwrap()),
            turbo: true,
            terms: None,
        })
    } else {
        None
    };
    let runestone: ScriptBuf = (Runestone {
        etching,
        pointer: match etch {
            true => Some(1),
            false => Some(0),
        }, // points to the OP_RETURN, so therefore targets the protoburn
        edicts: Vec::new(),
        mint: None,
        protocol: protostones.encipher().ok(),
    })
    .encipher();

    //     // op return is at output 1
    let op_return = TxOut {
        value: Amount::from_sat(0),
        script_pubkey: runestone,
    };
    let address: Address<NetworkChecked> = get_address(&ADDRESS1().as_str());

    let script_pubkey = address.script_pubkey();
    let txout = TxOut {
        value: Amount::from_sat(100_000_000),
        script_pubkey: script_pubkey.clone(),
    };
    let outputs = if with_leftovers_to_separate {
        vec![
            txout,
            op_return,
            TxOut {
                value: Amount::from_sat(546),
                script_pubkey,
            },
        ]
    } else {
        vec![txout, op_return]
    };
    Transaction {
        version: Version::ONE,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![txin],
        output: outputs,
    }
}

pub fn create_multiple_cellpack_with_witness_and_in_with_edicts(
    witness: Witness,
    cellpacks_or_edicts: Vec<CellpackOrEdict>,
    previous_output: OutPoint,
    etch: bool,
) -> Transaction {
    create_multiple_cellpack_with_witness_and_in_with_edicts_and_leftovers(
        witness,
        cellpacks_or_edicts,
        previous_output,
        etch,
        false,
    )
}

/// Asserts that the trace data from the given outpoint contains a RevertContext with the expected error message.
///
/// # Arguments
///
/// * `outpoint` - The outpoint to get trace data from
/// * `expected_error_message` - The error message to check for in the RevertContext data
///
/// # Returns
///
/// * `Result<(), anyhow::Error>` - Ok if the assertion passes, Err otherwise
///
/// # Example
///
/// ```
/// assert_revert_context(
///     &OutPoint {
///         txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
///         vout: 3,
///     },
///     "ALKANES: revert: Error: already initialized"
/// )?;
/// ```
pub fn assert_revert_context(outpoint: &OutPoint, expected_error_message: &str) -> Result<()> {
    // This is a convenience wrapper around assert_revert_context_at_index that checks the last event
    assert_revert_context_at_index(outpoint, expected_error_message, None)
}

/// Asserts that a specific trace event from the given outpoint contains a RevertContext with the expected error message.
///
/// # Arguments
///
/// * `outpoint` - The outpoint to get trace data from
/// * `expected_error_message` - The error message to check for in the RevertContext data
/// * `index` - Optional index of the trace event to check. If None, checks the last event.
///   Use negative values to count from the end (-1 = last, -2 = second to last, etc.)
///
/// # Returns
///
/// * `Result<(), anyhow::Error>` - Ok if the assertion passes, Err otherwise
///
/// # Example
///
/// ```
/// // Check the second-to-last trace event
/// assert_revert_context_at_index(
///     &OutPoint {
///         txid: test_block.txdata[test_block.txdata.len() - 1].compute_txid(),
///         vout: 3,
///     },
///     "Overflow error in expression",
///     Some(-2)
/// )?;
/// ```
pub fn assert_revert_context_at_index(
    outpoint: &OutPoint,
    expected_error_message: &str,
    index: Option<isize>,
) -> Result<()> {
    let trace_data: Trace = view::trace(outpoint)?.try_into()?;
    let trace_events = trace_data.0.lock().expect("Mutex poisoned");

    if trace_events.is_empty() {
        panic!("No trace events found");
    }

    // Determine which event to check
    let event_index = match index {
        Some(idx) if idx >= 0 => idx as usize,
        Some(idx) => {
            // Handle negative indices (counting from the end)
            let abs_idx = idx.abs() as usize;
            if abs_idx > trace_events.len() {
                panic!(
                    "Index out of bounds: requested event {} but only {} events available",
                    idx,
                    trace_events.len()
                );
            }
            trace_events.len() - abs_idx
        }
        None => trace_events.len() - 1, // Default to last event
    };

    // Get the event at the calculated index
    let event = trace_events
        .get(event_index)
        .cloned()
        .unwrap_or_else(|| panic!("Failed to get trace event at index {}", event_index));

    match event {
        TraceEvent::RevertContext(trace_response) => {
            let data = String::from_utf8_lossy(&trace_response.inner.data);
            assert!(
                data.contains(expected_error_message),
                "Expected error message '{}' not found in: '{}'",
                expected_error_message,
                data
            );
            Ok(())
        }
        _ => panic!(
            "Expected RevertContext variant at index {}, but got a different variant: {:?}",
            event_index, event
        ),
    }
}
