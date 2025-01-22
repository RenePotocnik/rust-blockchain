use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
}

impl Transaction {
    pub fn new(sender: String, receiver: String, amount: u64) -> Self {
        Transaction { sender, receiver, amount }
    }
}