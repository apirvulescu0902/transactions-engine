use crate::{client::Client, types::TransactionType};
use std::collections::HashMap;

/// Transactions engine that helps with processing the transactions.
struct TransactionsEngine {
    clients: HashMap<u16, Client>,
}

impl TransactionsEngine {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: TransactionType) -> Result<(), String> {
        match transaction {
            TransactionType::Deposit { client, tx, amount } => self
                .clients
                .entry(client)
                .or_insert(Client::new(client))
                .deposit(amount),
            _ => (),
        }

        Ok(())
    }
}
