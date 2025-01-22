use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcBlockConfig;
use solana_transaction_status::{
    UiInstruction, UiParsedInstruction, UiMessage, UiTransactionEncoding, EncodedTransaction,
    TransactionDetails,
};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::clock::{Epoch, Slot};
use std::env;

const TOTAL_CAPABLE_CU: u64 = 48_000_000;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <validator_identity>", args[0]);
        std::process::exit(1);
    }

    let validator_identity_key = &args[1];

    // RPC client
    let rpc_url = "https://api.mainnet-beta.solana.com"; // Replace with your RPC URL
    let rpc_client = RpcClient::new(rpc_url.to_string());

    match calculate_validator_slot_cu(&rpc_client, validator_identity_key) {
        Ok(_) => println!("Done calculating CU for validator's slots."),
        Err(err) => eprintln!("Error calculating CU: {}", err),
    }
}

fn calculate_validator_slot_cu(
    rpc_client: &RpcClient,
    validator_identity_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the current slot and epoch
    let current_slot = rpc_client.get_slot()?;
    let epoch_info = rpc_client.get_epoch_info()?;
    let current_epoch = epoch_info.epoch;
    let first_slot_in_epoch = epoch_info.absolute_slot - epoch_info.slot_index;

    // Fetch the leader schedule for the current epoch
    let leader_schedule = rpc_client
        .get_leader_schedule(Some(first_slot_in_epoch))?
        .ok_or("Failed to fetch leader schedule")?;

    // Get slots produced by validator
    let leader_slots = leader_schedule
        .get(&validator_identity_key.to_string())
        .ok_or("Validator not found in leader schedule")?;

    println!(
        "Validator {} has {} total slots in the leader schedule.",
        validator_identity_key, leader_slots.len()
    );

    // Adjust produced slots to actual slot numbers
    let produced_slots: Vec<u64> = leader_slots
        .iter()
        .map(|&relative_slot| first_slot_in_epoch + relative_slot as u64)
        .filter(|&produced_slot| produced_slot <= current_slot)
        .collect();

    println!(
        "Validator {} has {} produced slots up to the current slot ({}).",
        validator_identity_key,
        produced_slots.len(),
        current_slot
    );

    let mut epoch_cu_percent: f64 = 0.0;
    let mut avg_vote_tx: i64 = 0;
    let mut avg_failed_tx: i64 = 0;
    let mut avg_non_vote_tx: i64 = 0;

    // Calculate CU and track transactions for each relevant slot
    for slot in &produced_slots {
        let config = RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            transaction_details: Some(TransactionDetails::Full),
            rewards: Some(false),
            max_supported_transaction_version: Some(0),
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        };

        if let Ok(block) = rpc_client.get_block_with_config(*slot, config) {
            let mut slot_cu: u64 = 0;
            let mut failed_tx_count = 0;
            let mut vote_tx_count = 0;
            let mut succeeded_tx_count = 0;

            if let Some(transactions) = block.transactions {
                for tx in transactions {
                    if let Some(meta) = tx.meta {
                        // Count vote transactions
                        if is_vote_transaction(&tx.transaction) {
                            vote_tx_count += 1;
                            avg_vote_tx += 1;
                            continue;
                        }

                        // Count failed and succeeded transactions
                        if meta.err.is_some() {
                            failed_tx_count += 1;
                            avg_failed_tx += 1;
                        } else {
                            succeeded_tx_count += 1;
                            avg_non_vote_tx += 1;
                        }

                        // Add the Compute Units used by this transaction
                        slot_cu += meta.compute_units_consumed.unwrap_or(0);
                    }
                }
            }

            // Calculate the percentage used
            let percentage_used = (slot_cu as f64 / TOTAL_CAPABLE_CU as f64) * 100.0;
            epoch_cu_percent += percentage_used;

            // Print the results for this slot
            println!(
                "Slot: {}, CU Used: {}, % of Total Capable CU: {:.2}%, Vote Transactions: {}, Succeeded Transactions: {}, Failed Transactions: {}",
                slot, slot_cu, percentage_used, vote_tx_count, succeeded_tx_count, failed_tx_count
            );
        } else {
            eprintln!("Failed to fetch block for slot {}", slot);
        }
    }
    let total_slots: f64 = produced_slots.len() as f64;
    let epoch_cu_average = epoch_cu_percent / total_slots;
    let epoch_avg_vote_tx = avg_vote_tx / (leader_slots.len() as i64);
    let epoch_avg_failed_tx = avg_failed_tx / (leader_slots.len() as i64);
    let epoch_avg_non_vote_tx = avg_non_vote_tx / (leader_slots.len() as i64);
    println!("----------------------------------------");
    println!("Epoch {} Stats", current_epoch);
    println!("Average CU per Slot: {:.2}%", epoch_cu_average);
    println!("Average Vote Transactions per Slot: {}", epoch_avg_vote_tx);
    println!("Average Succeeded Transactions per Slot: {}", epoch_avg_non_vote_tx);
    println!("Average Failed Transactions per Slot: {}", epoch_avg_failed_tx);

    Ok(())
}

fn is_vote_transaction(transaction: &EncodedTransaction) -> bool {
    match transaction {
        EncodedTransaction::Json(ui_transaction) => {
            if let message = &ui_transaction.message {
                match message {
                    UiMessage::Parsed(parsed_msg) => {
                        for instruction in &parsed_msg.instructions {
                            match instruction {
                                UiInstruction::Parsed(UiParsedInstruction::Parsed(parsed_instr)) => {
                                    // Check the program_id to see if it's a vote transaction
                                    if parsed_instr.program_id == "Vote111111111111111111111111111111111111111" {
                                        return true;
                                    }
                                }
                                _ => {} // Ignore other types of instructions
                            }
                        }
                    }
                    UiMessage::Raw(_) => {
                        // Skip raw messages
                        return false;
                    }
                }
            }
            false
        }
        _ => false, // Skip other transaction formats
    }
}
