use std::collections::HashSet;

use libp2p::{
    NetworkBehaviour, PeerId, Swarm,
    floodsub::{Floodsub, FloodsubEvent, Topic},
    identity,
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::{blockchain::Blockchain, models::block, models::transaction::Transaction};


pub static KEYS: Lazy<identity::Keypair> = Lazy::new(identity::Keypair::generate_ed25519);
pub static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
pub static CHAIN_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("chains"));
pub static BLOCK_TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("blocks"));

#[derive(Serialize, Deserialize, Debug)]
pub struct ChainResponse {
    pub blocks: Vec<block::Block>,
    pub receiver: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Input(String),
    Init,
}

#[derive(NetworkBehaviour)]
pub struct BlockchainBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ChainResponse>,
    #[behaviour(ignore)]
    pub init_sender: mpsc::UnboundedSender<bool>,
    #[behaviour(ignore)]
    pub blockchain: Blockchain,
    #[behaviour(ignore)]
    pub mining: bool,
}

impl BlockchainBehaviour {
    pub async fn new(
        blockchain: Blockchain,
        response_sender: mpsc::UnboundedSender<ChainResponse>,
        init_sender: mpsc::UnboundedSender<bool>,
    ) -> Self {
        let mut behaviour = Self {
            blockchain,
            floodsub: Floodsub::new(*PEER_ID),
            mdns: Mdns::new(Default::default())
                .await
                .expect("can create mdns"),
            response_sender,
            init_sender,
            mining: false,
        };

        behaviour.floodsub.subscribe(CHAIN_TOPIC.clone());
        behaviour.floodsub.subscribe(BLOCK_TOPIC.clone());

        behaviour
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for BlockchainBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

impl NetworkBehaviourEventProcess<FloodsubEvent> for BlockchainBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(msg) = event {
            if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                if resp.receiver == PEER_ID.to_string() {
                    println!("response from {}", msg.source);

                    resp.blocks.iter().for_each(|block| println!("{:?}", block));
                    self.blockchain.chain = self
                        .blockchain
                        .choose_chain(self.blockchain.chain.clone(), resp.blocks);
                }
            } else if let Ok(resp) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                println!("sending local chain to {}", msg.source);

                let peer_id = resp.from_peer_id;
                if PEER_ID.to_string() == peer_id {
                    if let Err(err) = self.response_sender.send(ChainResponse {
                        blocks: self.blockchain.chain.clone(),
                        receiver: msg.source.to_string(),
                    }) {
                        println!("error sending response via channel {}", err);
                    }
                }
            } else if let Ok(block) = serde_json::from_slice::<block::Block>(&msg.data) {
                println!("received new block from {}", msg.source);

                if block.is_mined(self.blockchain.difficulty) {
                    // Block is already mined, stop mining and try to add it to the blockchain
                    self.mining = false;
                    self.blockchain.try_to_add_a_block(block);
                } else {
                    // Block is not mined, start mining
                    self.mining = true;
                    let mut block = block.clone();
                    block.mine(self.blockchain.clone(), &mut self.mining);

                    // Broadcast the mined block
                    let json = serde_json::to_string(&block).expect("can jsonify request");
                    self.floodsub.publish(BLOCK_TOPIC.clone(), json.as_bytes());

                    self.blockchain.try_to_add_a_block(block);
                }
            }
        }
    }
}

pub fn get_list_peers(swarm: &Swarm<BlockchainBehaviour>) -> Vec<String> {
    println!("discovered peers");

    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();

    for peer in nodes {
        unique_peers.insert(peer);
    }

    unique_peers.iter().map(|peer| peer.to_string()).collect()
}

pub fn handle_print_peers(swarm: &Swarm<BlockchainBehaviour>) {
    let peers = get_list_peers(&swarm);
    peers.iter().for_each(|peer| println!("{}", peer));
}

pub fn handle_print_chain(swarm: &Swarm<BlockchainBehaviour>) {
    println!("local blockchain");

    let pretty_json = serde_json::to_string_pretty(&swarm.behaviour().blockchain.chain)
        .expect("can jsonify blocks");

    println!("{}", pretty_json);
}

pub fn handle_create_block(cmd: &str, swarm: &mut Swarm<BlockchainBehaviour>) {
    if let Some(data) = cmd.strip_prefix("create b") {
        let behaviour = swarm.behaviour_mut();

        let latest_block = behaviour
            .blockchain
            .chain
            .last()
            .expect("there is at least one block");

        let transactions: Vec<Transaction> = serde_json::from_str(data).expect("can parse transactions");

        let block = block::Block::new(
            latest_block.index + 1,
            latest_block.hash.clone(),
            transactions,
        );

        let json = serde_json::to_string(&block).expect("can jsonify request");

        println!("broadcasting new block for mining");

        behaviour
            .floodsub
            .publish(BLOCK_TOPIC.clone(), json.as_bytes());
    }
}
