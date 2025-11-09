use csv::Writer;
use tracing::error;

use crate::{client::Client, types::TransactionType};
use std::collections::HashMap;

/// Transactions engine that helps with processing the transactions.
pub struct TransactionsEngine {
    clients: HashMap<u16, Client>,
}

impl TransactionsEngine {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    /// Process a given transaction
    pub fn process_transaction(&mut self, transaction: TransactionType) -> Result<(), String> {
        match transaction {
            TransactionType::Deposit { client, tx, amount } => {
                let client = self.clients.entry(client).or_insert(Client::new(client));

                client.deposit(amount, tx)?;
                client.processed_transactions.insert(tx, transaction);
            }
            TransactionType::Withdrawal { client, tx, amount } => {
                let client = self.clients.entry(client).or_insert(Client::new(client));

                client.withdrawal(amount, tx)?;
                client.processed_transactions.insert(tx, transaction);
            }
            TransactionType::Dispute { client, tx } => self
                .clients
                .entry(client)
                .or_insert(Client::new(client))
                .dispute(tx)?,
            TransactionType::Resolve { client, tx } => self
                .clients
                .entry(client)
                .or_insert(Client::new(client))
                .resolve(tx)?,
            TransactionType::Chargeback { client, tx } => self
                .clients
                .entry(client)
                .or_insert(Client::new(client))
                .chargeback(tx)?,
            _ => (),
        }

        Ok(())
    }

    /// Serialize and print current state
    pub fn print_current_state(&self) {
        let mut writer = Writer::from_writer(std::io::stdout());

        if let Err(err) = writer.write_record(["client", "available", "held", "total", "locked"]) {
            error!("Could not write record: {err:?}");
        }

        for client_data in self.clients.values() {
            if let Err(err) = writer.write_record(&[
                client_data.client.to_string(),
                client_data.available.to_string(),
                client_data.held.to_string(),
                client_data.total.to_string(),
                client_data.locked.to_string(),
            ]) {
                error!("Could not write record: {err:?}");
            }
        }
    }
}
