use crate::types::DECIMAL_PRECISION;
use rust_decimal::Decimal;
use tracing::{error, info};

/// Type containing all the information needed for a client account
#[derive(Debug, Default)]
pub struct Client {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    /// Handle deposit for current client
    pub fn deposit(&mut self, amount: Decimal) -> Result<(), String> {
        info!("Deposit - client {}, amount {}", self.client, amount);

        if amount < Decimal::new(0, DECIMAL_PRECISION) {
            return Err("Negative amount".to_string());
        }

        self.available += amount;
        self.total += amount;

        Ok(())
    }

    /// Handle withdrawal for current client
    pub fn withdrawal(&mut self, amount: Decimal) -> Result<(), String> {
        info!("Withdrawal - client {}, amount {}", self.client, amount);

        if self.available < amount {
            return Err("Insufficient funds".to_string());
        }

        self.available -= amount;
        self.total -= amount;

        Ok(())
    }
}
