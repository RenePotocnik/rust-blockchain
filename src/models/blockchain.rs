use super::block::Block;
use chrono::prelude::*;

type Blocks = Vec<Block>;

// `Blockchain` A struct that represents the blockchain.
#[derive(Debug, Clone)]
pub struct Blockchain {
    // The first block to be added to the chain.
    pub genesis_block: Block,
    // The storage for blocks.
    pub chain: Blocks,
    // Minimum amount of work required to mine a block.
    pub difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        // First block in the chain.
        let genesis_block = Block {
            index: 0,
            timestamp: Utc::now().timestamp_millis() as u64,
            proof_of_work: u64::default(),
            previous_hash: String::default(),
            data: "Genesis Block".to_string(),
            hash: String::default(),
        };

        // Create chain starting from the genesis chain.
        let mut chain = Vec::new();
        chain.push(genesis_block.clone());

        // Create a blockchain Instance.
        let blockchain = Blockchain {
            genesis_block,
            chain,
            difficulty,
        };
        blockchain
    }

    pub fn add_block(&mut self) {
        let mut new_block = Block::new(
            self.chain.len() as u64,
            self.chain[&self.chain.len() - 1].hash.clone(),
            "".to_string(),
        );

        new_block.mine(self.clone());
        self.chain.push(new_block.clone());
        println!("New block added to chain -> {:?}", new_block);
    }

    pub fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
        if block.previous_hash != previous_block.hash {
            println!("Block with id: {} has wrong previous hash", block.index);
            return false;
        } else if !block.hash.starts_with(&"0".repeat(self.difficulty)) {
            return false;
        } else if block.index != previous_block.index + 1 {
            println!(
                "Block with id: {} is not the next block after the latest: {}",
                block.index, previous_block.index
            );
        } else if block.generate_block_hash() != block.hash {
            println!("Block with id: {} has invalid hash", block.index);
        }

        true
    }

    pub fn try_to_add_a_block(&mut self, block: Block) {
        let last_block = self
            .chain
            .last()
            .expect("There should be at least one block");

        if self.is_block_valid(&block, last_block) {
            self.chain.push(block);
        } else {
            println!("Could not add block");
        }
    }

    pub fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for block_index in 0..chain.len() {
            if block_index == 0 {
                continue;
            }

            let first = chain.get(block_index - 1).expect("has to exist");
            let second = chain.get(block_index).expect("has to exist");

            if !self.is_block_valid(second, first) {
                return false;
            }
        }

        true
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if !is_local_valid && is_remote_valid {
            remote
        } else if is_local_valid && !is_remote_valid {
            local
        } else {
            panic!("Both chains are invalid");
        }
    }
}
