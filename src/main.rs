use csv::{ReaderBuilder, Trim};
use engine::TransactionsEngine;
use std::fs::File;
use tracing::{Level, debug, error, info};
use tracing_subscriber::EnvFilter;
use types::{TransactionRecord, TransactionType};

mod client;
mod engine;
mod types;

fn main() {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    debug!("Binary arguments: {:?}", args);

    // Check if input file was passed as an argument
    if args.len() < 2 {
        error!("Input file has not been provided");
        return;
    }

    info!("Reading input from input file: {}", args[1]);

    let file = File::open(args[1].clone()).expect("Could not open input file");

    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .has_headers(true)
        .trim(Trim::All)
        .from_reader(file);

    let mut transactions_engine = TransactionsEngine::new();

    // Process each transaction from the input file
    for line in reader.deserialize() {
        let record: TransactionRecord = match line {
            Ok(record) => record,
            Err(err) => {
                error!("Could not deserialize line: {err:?}");
                continue;
            }
        };

        debug!("Transaction record: {record:?}");

        let transaction = match TransactionType::from_transaction_record(record.clone()) {
            Ok(transaction) => transaction,
            Err(err) => {
                error!(
                    "Could not map transaction record {record:?} to a type: {err:?}. Skipping it."
                );
                continue;
            }
        };

        info!("Processing transaction {transaction:?}");

        if let Err(err) = transactions_engine.process_transaction(transaction.clone()) {
            error!("Could not process transaction {transaction:?}: {err:?}")
        }
    }

    // Write the current state
    info!("Printing the current state");
    transactions_engine.print_current_state();
}
