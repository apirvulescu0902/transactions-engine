use rust_decimal::Decimal;
use tracing::info;

/// Type containing all the information needed for a client account.
#[derive(Debug, Default)]
pub struct Client {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl Client {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    pub fn deposit(&mut self, amount: Decimal) {
        info!("Deposit - client {}, amount {}", self.client, amount);
        self.available += amount;
        self.total += amount;
    }
}
