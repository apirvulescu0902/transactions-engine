use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Decimal precision for amounts
pub const DECIMAL_PRECISION: u32 = 4;

/// Transaction information read from the input file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionRecord {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<String>,
}

/// Transaction type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit {
        client: u16,
        tx: u32,
        amount: Decimal,
    },
    Withdrawal {
        client: u16,
        tx: u32,
        amount: Decimal,
    },
    Dispute {
        client: u16,
        tx: u32,
    },
    Resolve {
        client: u16,
        tx: u32,
    },
    Chargeback {
        client: u16,
        tx: u32,
    },
    Unknown,
}

impl TransactionType {
    pub fn from_transaction_record(record: TransactionRecord) -> Result<Self, String> {
        let client = record.client;
        let tx = record.tx;

        let transaction = match record.transaction_type.as_str() {
            "deposit" => Self::Deposit {
                client,
                tx,
                amount: parse_with_decimal_precision(record.amount.unwrap())?,
            },
            "withdrawal" => Self::Withdrawal {
                client,
                tx,
                amount: parse_with_decimal_precision(record.amount.unwrap())?,
            },
            "dispute" => Self::Dispute { client, tx },
            "resolve" => Self::Resolve { client, tx },
            "chargeback" => Self::Chargeback { client, tx },
            _ => Self::Unknown,
        };

        Ok(transaction)
    }
}

/// Ensure the correct precision
pub fn parse_with_decimal_precision(amount: String) -> Result<Decimal, String> {
    let decimal =
        Decimal::from_str_exact(&amount).map_err(|err| format!("Invalid decimal {err:?}"))?;
    if decimal.scale() > DECIMAL_PRECISION {
        return Err("Invalid decimal precision".to_string());
    }
    Ok(decimal)
}
