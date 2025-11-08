use serde::{Deserialize, Serialize};

/// Transaction information read from the input file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: f64,
}
