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
