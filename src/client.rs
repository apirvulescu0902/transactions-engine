use std::collections::{HashMap, HashSet};

use crate::types::{DECIMAL_PRECISION, TransactionType};
use rust_decimal::Decimal;
use tracing::info;

/// Type containing all the information needed for a client account
#[derive(Debug, Default)]
pub struct Client {
    /// Client ID
    pub client: u16,
    /// Available amount
    pub available: Decimal,
    /// Amount under dispute
    pub held: Decimal,
    /// Total funds
    pub total: Decimal,
    /// Account state
    pub locked: bool,
    /// Transactions processed by the engine
    pub processed_transactions: HashMap<u32, TransactionType>,
    /// Transaction ids that are under dispute
    disputed_transactions: HashSet<u32>,
}

impl Client {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    /// Handle deposit for current client
    pub fn deposit(&mut self, amount: Decimal, tx: u32) -> Result<(), String> {
        info!(
            "Deposit - client {}, tx {}, amount {}",
            self.client, tx, amount
        );

        if self.processed_transactions.contains_key(&tx) {
            return Err("Transaction already processed".to_string());
        }

        if amount < Decimal::new(0, DECIMAL_PRECISION) {
            return Err("Negative amount".to_string());
        }

        self.available += amount;
        self.total += amount;

        Ok(())
    }

    /// Handle withdrawal for current client
    pub fn withdrawal(&mut self, amount: Decimal, tx: u32) -> Result<(), String> {
        info!(
            "Withdrawal - client {}, tx {}, amount {}",
            self.client, tx, amount
        );

        if self.processed_transactions.contains_key(&tx) {
            return Err("Transaction already processed".to_string());
        }

        if amount < Decimal::new(0, DECIMAL_PRECISION) {
            return Err("Negative amount".to_string());
        }

        if self.available < amount {
            return Err("Insufficient funds".to_string());
        }

        self.available -= amount;
        self.total -= amount;

        Ok(())
    }

    /// Handle dispute for current client and given transaction id
    pub fn dispute(&mut self, tx: u32) -> Result<(), String> {
        info!("Dispute - client {}, tx {}", self.client, tx);

        if self.disputed_transactions.contains(&tx) {
            return Err("Transaction already disputed".to_string());
        }

        let transaction = self
            .processed_transactions
            .get(&tx)
            .ok_or_else(|| "Transaction id not found in processed transactions".to_string())?;

        if let TransactionType::Deposit {
            client: _,
            tx,
            amount,
        } = transaction
        {
            self.available -= amount;
            self.held += amount;

            self.disputed_transactions.insert(*tx);
        }

        Ok(())
    }

    /// Resolve the given transaction id that is under dispute
    pub fn resolve(&mut self, tx: u32) -> Result<(), String> {
        info!("Resolve - client {}, tx {}", self.client, tx);

        let transaction = self
            .processed_transactions
            .get(&tx)
            .ok_or_else(|| "Transaction id not found in processed transactions".to_string())?;

        if !self.disputed_transactions.remove(&tx) {
            return Err("Transaction id has not been disputed".to_string())?;
        }

        if let TransactionType::Deposit {
            client: _,
            tx: _,
            amount,
        } = transaction
        {
            self.held -= amount;
            self.available += amount;
        }

        Ok(())
    }

    /// Performs chargeback for given transaction and locks the account
    pub fn chargeback(&mut self, tx: u32) -> Result<(), String> {
        info!("Chargeback - client {}, tx {}", self.client, tx);

        let transaction = self
            .processed_transactions
            .get(&tx)
            .ok_or_else(|| "Transaction id not found in processed transactions".to_string())?;

        if !self.disputed_transactions.remove(&tx) {
            return Err("Transaction id has not been disputed".to_string())?;
        }

        if let TransactionType::Deposit {
            client: _,
            tx: _,
            amount,
        } = transaction
        {
            self.held -= amount;
            self.total -= amount;
            self.locked = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;

    use crate::types::TransactionType;

    use super::Client;

    #[test]
    fn test_deposit() {
        let client_id = 1;
        let tx = 1;
        let amount = Decimal::new(2, 4);
        let mut client = Client::new(client_id);
        let transaction = TransactionType::Deposit {
            client: client_id,
            tx,
            amount,
        };

        client.deposit(amount, tx).expect("Deposit failed.");

        assert_eq!(client.available, amount);
        assert_eq!(client.total, amount);
        assert_eq!(client.held, Decimal::new(0, 4));

        client.processed_transactions.insert(tx, transaction);

        // try to process the same transaction again
        assert!(client.deposit(amount, tx).is_err());

        let tx2 = 2;
        let amount2 = Decimal::new(1, 4);

        client.deposit(amount2, tx2).expect("Deposit failed.");

        assert_eq!(client.available, amount + amount2);
        assert_eq!(client.total, amount + amount2);
        assert_eq!(client.held, Decimal::new(0, 4));

        // try to deposit a negative amount
        assert!(client.deposit(Decimal::new(-1, 4), 3).is_err());
    }

    #[test]
    fn test_withdrawal() {
        let client_id = 1;
        let mut client = Client::new(client_id);

        client
            .deposit(Decimal::new(4, 4), 1)
            .expect("Deposit failed.");

        client
            .withdrawal(Decimal::new(2, 4), 2)
            .expect("Withdrawal failed");

        assert_eq!(client.available, Decimal::new(2, 4));
        assert_eq!(client.total, Decimal::new(2, 4));
        assert_eq!(client.held, Decimal::new(0, 4));

        // negative amount
        assert!(client.withdrawal(Decimal::new(-1, 4), 3).is_err());

        // insufficient funds
        assert!(client.withdrawal(Decimal::new(5, 4), 4).is_err());
    }

    #[test]
    fn test_dispute() {
        let client_id = 1;
        let tx = 1;
        let amount = Decimal::new(2, 0);
        let mut client = Client::new(client_id);

        // try to dispute a transaction that does not exist
        assert!(client.dispute(tx).is_err());

        let transaction = TransactionType::Deposit {
            client: client_id,
            tx,
            amount,
        };
        client.deposit(amount, tx).expect("Deposit failed.");
        client.processed_transactions.insert(tx, transaction);

        client.dispute(tx).expect("Could not dispute transaction.");

        assert_eq!(client.available, Decimal::new(0, 0));
        assert_eq!(client.held, amount);
        assert!(client.disputed_transactions.contains(&tx));
    }

    #[test]
    fn test_resolve() {
        let client_id = 1;
        let tx = 1;
        let amount = Decimal::new(2, 0);
        let mut client = Client::new(client_id);

        let transaction = TransactionType::Deposit {
            client: client_id,
            tx,
            amount,
        };
        client.deposit(amount, tx).expect("Deposit failed.");
        client.processed_transactions.insert(tx, transaction);

        // try to resolve a transaction that is not under dispute
        assert!(client.resolve(tx).is_err());

        client.dispute(tx).expect("Could not dispute transaction.");

        client
            .resolve(tx)
            .expect("Could not resolve disputed transaction.");

        assert_eq!(client.available, amount);
        assert_eq!(client.held, Decimal::new(0, 0));
        assert!(!client.disputed_transactions.contains(&tx));
    }

    #[test]
    fn test_chargeback() {
        let client_id = 1;
        let tx = 1;
        let amount = Decimal::new(1, 0);
        let mut client = Client::new(client_id);

        let transaction = TransactionType::Deposit {
            client: client_id,
            tx,
            amount,
        };
        client.deposit(amount, tx).expect("Deposit failed.");
        client.processed_transactions.insert(tx, transaction);

        // try to chargeback a transaction that is not under dispute
        assert!(client.chargeback(tx).is_err());

        client.dispute(tx).expect("Could not dispute transaction.");

        assert_eq!(client.available, Decimal::new(0, 0));
        assert_eq!(client.held, Decimal::new(1, 0));
        assert_eq!(client.total, Decimal::new(1, 0));

        // check that withdrawal fails with funds under dispute
        assert!(client.withdrawal(Decimal::new(1, 0), 2).is_err());

        client
            .chargeback(tx)
            .expect("Could not chargeback transaction.");
        assert_eq!(client.available, Decimal::new(0, 0));
        assert_eq!(client.held, Decimal::new(0, 0));
        assert_eq!(client.total, Decimal::new(0, 0));

        assert!(client.chargeback(tx).is_err());
    }
}
