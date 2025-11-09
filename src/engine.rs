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

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::types::TransactionType;

    use super::TransactionsEngine;

    #[test]
    fn test_process_transaction() {
        let mut engine = TransactionsEngine::new();
        let client_id = 1;

        let deposit_tx = TransactionType::Deposit {
            client: client_id,
            tx: 1,
            amount: Decimal::new(3, 0),
        };
        engine
            .process_transaction(deposit_tx)
            .expect("Could not process deposit.");

        let withdrawal_tx = TransactionType::Withdrawal {
            client: client_id,
            tx: 2,
            amount: Decimal::new(2, 0),
        };
        engine
            .process_transaction(withdrawal_tx)
            .expect("Could not process withdrawal.");

        let client = engine.clients.get(&client_id).unwrap();
        assert_eq!(client.available, Decimal::new(1, 0));
        assert_eq!(client.total, Decimal::new(1, 0));
        assert_eq!(client.held, Decimal::new(0, 0));
        assert!(!client.locked);
        assert!(client.processed_transactions.contains_key(&1));

        let deposit_tx = TransactionType::Deposit {
            client: client_id,
            tx: 3,
            amount: Decimal::new(3, 0),
        };
        engine
            .process_transaction(deposit_tx.clone())
            .expect("Could not process deposit.");

        // check that processing the same transaction twice fails
        assert!(engine.process_transaction(deposit_tx).is_err());

        let resolve_tx = TransactionType::Resolve {
            client: client_id,
            tx: 3,
        };

        // check that resolving an undisputed transaction fails
        assert!(engine.process_transaction(resolve_tx.clone()).is_err());

        let dispute_tx = TransactionType::Dispute {
            client: client_id,
            tx: 3,
        };
        engine
            .process_transaction(dispute_tx)
            .expect("Could not dispute transaction.");

        let client = engine.clients.get(&client_id).unwrap();
        assert_eq!(client.available, Decimal::new(1, 0));
        assert_eq!(client.total, Decimal::new(4, 0));
        assert_eq!(client.held, Decimal::new(3, 0));
        assert!(!client.locked);

        engine
            .process_transaction(resolve_tx)
            .expect("Could not resolve transaction");

        let client = engine.clients.get(&client_id).unwrap();
        assert_eq!(client.available, Decimal::new(4, 0));
        assert_eq!(client.total, Decimal::new(4, 0));
        assert_eq!(client.held, Decimal::new(0, 0));
        assert!(!client.locked);

        let deposit_tx = TransactionType::Deposit {
            client: client_id,
            tx: 4,
            amount: Decimal::new(1, 0),
        };
        engine
            .process_transaction(deposit_tx)
            .expect("Could not process deposit.");

        let dispute_tx = TransactionType::Dispute {
            client: client_id,
            tx: 4,
        };
        engine
            .process_transaction(dispute_tx)
            .expect("Could not dispute transaction.");

        let chargeback_tx = TransactionType::Chargeback {
            client: client_id,
            tx: 4,
        };
        engine
            .process_transaction(chargeback_tx)
            .expect("Could not chargeback transaction.");

        let client = engine.clients.get(&client_id).unwrap();
        assert_eq!(client.available, Decimal::new(4, 0));
        assert_eq!(client.total, Decimal::new(4, 0));
        assert_eq!(client.held, Decimal::new(0, 0));
        assert!(client.locked);
    }
}
