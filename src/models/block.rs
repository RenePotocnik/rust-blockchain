use super::blockchain::Blockchain;
use super::transaction::Transaction;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub proof_of_work: u64,
    pub previous_hash: String, // Hash of the previous block
    pub transactions: Vec<Transaction>,
    pub hash: String, // Hash of the current block
}

impl Block {
    pub fn new(index: u64, previous_hash: String, transactions: Vec<Transaction>) -> Self {
        // Current block to be created.
        let block = Block {
            index,
            timestamp: Utc::now().timestamp_millis() as u64,
            proof_of_work: u64::default(),
            previous_hash,
            transactions,
            hash: String::default(),
        };

        block
    }

    // Mine block hash.
    pub fn mine(&mut self, blockchain: Blockchain, mining_flag: &mut bool) {
        while *mining_flag {
            if !self.hash.starts_with(&"0".repeat(blockchain.difficulty)) {
                self.proof_of_work += 1;
                self.hash = self.generate_block_hash();
                println!("Hash: {}", self.hash);
            } else {
                break;
            }
        }
    }

    // Calculate block hash.
    pub fn generate_block_hash(&self) -> String {
        let mut block_data = self.clone();
        block_data.hash = String::default();
        // Convert block to JSON format.
        let serialized_block_data = serde_json::to_string(&block_data).unwrap();

        // println!("Serialized block data: {}", serialized_block_data);

        // Calculate and return SHA-256 hash value.
        let mut hasher = Sha256::new();
        hasher.update(serialized_block_data);

        let result = hasher.finalize();
        format!("{:x}", result)
    }

    pub fn is_mined(&self, difficulty: usize) -> bool {
        self.hash.starts_with(&"0".repeat(difficulty))
    }
}
