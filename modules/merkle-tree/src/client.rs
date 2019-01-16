use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

extern crate serde;
extern crate serde_json;
extern crate sr_primitives;
extern crate substrate_primitives;

use sr_primitives::traits::BlakeTwo256;
use runtime_primitives::traits::Hash;
use substrate_primitives::H256;

type Proof = Vec<Option<H256>>;
type EdgeNodes = Vec<Option<H256>>;
pub type Tree = HashMap<H256, Node>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub parent: H256,
    pub sibling: Option<H256>
}

pub struct MerkleTreeClient {
    pub root_hash: Option<H256>,
    pub n_nodes: u128,
    pub node_numbers: HashMap<H256, u128>,
    pub edge_nodes: EdgeNodes,
    pub tree: Tree,
}

impl MerkleTreeClient {
    pub fn new() -> MerkleTreeClient {
        MerkleTreeClient {
            root_hash: None,
            n_nodes: 0,
            node_numbers: HashMap::new(),
            edge_nodes: vec![],
            tree: HashMap::new(),
        }
    }

    pub fn build_tree_from_events(self: &mut Self, events: Vec<H256>) {
        for event_record in events.iter() {
            self.insert(*event_record);
        }
    }

    pub fn get_proof_for(self: &Self, value: Vec<u8>, root_hash: H256) -> Result<Proof, &'static str> {
        let value_hash = BlakeTwo256::hash_of(&value);
        let proof: Vec<Option<H256>> = vec![];
        let tree = self.load_snapshot(&root_hash);
        self.find_node(&tree, proof, value_hash, root_hash).ok_or("Proof not found")
    }

    pub fn get_node_number(self: &Self, value: Vec<u8>) -> u128 {
        let value_hash = BlakeTwo256::hash_of(&value);
        *self.node_numbers.get(&value_hash).unwrap()
    }

    fn find_node(self: &Self, tree: &Tree, mut proof: Vec<Option<H256>>, hash: H256, node_hash: H256) -> Option<Proof> {
        match tree.get(&hash) {
            Some(node) => match node.sibling {
                Some(sibling) => {
                    proof.push(Some(sibling));
                    if node.parent == node_hash {
                        return Some(proof);
                    }

                    self.find_node(&tree, proof, node.parent, node_hash)
                },
                None => {
                    proof.push(None);

                    self.find_node(&tree, proof, node.parent, node_hash)
                },
            },
            None => None,
        }
    }

    fn insert(self: &mut Self, value_hash: H256) {
        let mut pair_hash = value_hash;
        let mut new_edge = value_hash;
        let next_edge_addition_level = self.count_bit_set_from_right(self.n_nodes);

        for i in 0..self.edge_nodes.len() {
            let edge_node = self.edge_nodes[i];
            pair_hash = match edge_node {
                Some(hash) => {
                    let new_hash = BlakeTwo256::hash_of(&[hash, pair_hash]);
                    self.tree.insert(hash, Node {
                        parent: new_hash,
                        sibling: Some(pair_hash)
                    });
                    self.tree.insert(pair_hash, Node {
                        parent: new_hash,
                        sibling: Some(hash)
                    });
                    new_hash
                },
                None => {
                    let new_hash = BlakeTwo256::hash_of(&pair_hash);
                    self.tree.insert(pair_hash, Node {
                        parent: new_hash,
                        sibling: None
                    });
                    new_hash
                }
            };
            if (i + 1) as u8 == next_edge_addition_level {
                new_edge = pair_hash;
            }
        }

        self.root_hash = Some(pair_hash);
        self.n_nodes += 1;
        self.node_numbers.insert(value_hash, self.n_nodes);
        self.update_edges(new_edge, next_edge_addition_level as usize);

        self.save_snapshot(&pair_hash, &self.tree)
    }

    fn save_snapshot(self: &Self, new_root_hash: &H256, tree: &Tree) {
        let name = format!("src/snapshots/{:?}", new_root_hash);
        let content = serde_json::to_string_pretty(tree);
        match content {
            Ok(data) => {
                let mut file = File::create(name).expect("Could not create file!");
                file.write_all(data.as_bytes()).expect("Failed to write to file!");
            },
            Err(e) => panic!(e),
        }
    }

    fn load_snapshot(self: &Self, root_hash: &H256) -> Tree {
        let name = format!("src/snapshots/{:?}", root_hash);
        let mut file = File::open(name).expect("Could not open file!");
        let mut content = String::new();
        file.read_to_string(&mut content).expect("Could not read file!");;

        let tree: Tree = serde_json::from_str(&content).unwrap();
        tree
    }

    fn update_edges(self: &mut Self, new_edge_value: H256, addition_at_level: usize) {
        if addition_at_level >= self.edge_nodes.len() {
            self.edge_nodes.push(Some(new_edge_value));
        } else {
            self.edge_nodes[addition_at_level] = Some(new_edge_value);
        }

        for i in 0..addition_at_level {
            self.edge_nodes[i] = None;
        }
    }

    fn count_bit_set_from_right(self: &mut Self, mut num: u128) -> u8 {
        let mut len: u8 = 0;
        while (num & 1) > 0 {
            num >>= 1;
            len += 1;
        }
        return len;
    }
}
