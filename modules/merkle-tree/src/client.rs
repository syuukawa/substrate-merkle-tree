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
    // Current root hash of the tree
    pub root_hash: Option<H256>,
    // Number of leaf nodes
    pub n_nodes: u128,
    // zero based index of a node
    pub node_indices: HashMap<H256, u128>,
    // Edge nodes neccessary for the next insert
    pub edge_nodes: EdgeNodes,
    // Every node in tree and its connections
    pub tree: Tree,
}

impl MerkleTreeClient {
    pub fn new() -> MerkleTreeClient {
        MerkleTreeClient {
            root_hash: None,
            n_nodes: 0,
            node_indices: HashMap::new(),
            edge_nodes: vec![],
            tree: HashMap::new(),
        }
    }

    // Builds the whole tree with events emitted by the module
    pub fn build_tree_from_events(self: &mut Self, events: Vec<H256>) {
        for event_record in events.iter() {
            self.insert(*event_record);
        }
    }

    // Get proof that specified `value` is inside the tree state with root of `root_hash`
    pub fn get_proof_for(self: &Self, value: Vec<u8>, root_hash: H256) -> Result<Proof, &'static str> {
        let value_hash = BlakeTwo256::hash_of(&value);
        let proof: Vec<Option<H256>> = vec![];
        // If `value_hash` is equal to `root_hash`, that means that tree only has one node, in which case empty proof is returned
        if value_hash == root_hash {
            return Ok(proof);
        }
        // Load the saved tree at state `root_hash`
        let tree_result = self.load_snapshot(&root_hash);
        match tree_result {
            Ok(tree) => self.find_node(&tree, proof, value_hash, root_hash).ok_or("Proof not found!"),
            Err(_e) => Err("Node not found in specified tree state!")
        }
    }

    // Get zero based index of a specified node
    pub fn get_node_index(self: &Self, value: Vec<u8>) -> u128 {
        let value_hash = BlakeTwo256::hash_of(&value);
        *self.node_indices.get(&value_hash).unwrap()
    }

    // Recursively find all the nodes needed to make the root hash `node_hash`
    // TODO: Change the `node_hash` to `root_hash`
    fn find_node(self: &Self, tree: &Tree, mut proof: Vec<Option<H256>>, hash: H256, node_hash: H256) -> Option<Proof> {
        match tree.get(&hash) {
            Some(node) => match node.sibling {
                Some(sibling) => {
                    // Node has a sibling, push it to proof array
                    proof.push(Some(sibling));

                    // Proof is complete
                    if node.parent == node_hash {
                        return Some(proof);
                    }

                    // Going up the tree one level
                    self.find_node(&tree, proof, node.parent, node_hash)
                },
                None => {
                    // Node doesnt have a sibling, push None to proof array
                    proof.push(None);

                    // Going up the tree one level
                    self.find_node(&tree, proof, node.parent, node_hash)
                },
            },
            None => None,
        }
    }

    // Insert a hash into the tree
    fn insert(self: &mut Self, value_hash: H256) {
        let mut pair_hash = value_hash;
        let mut new_edge = value_hash;
        // Get the level where the next edge node is
        let next_edge_addition_level = self.count_bit_set_from_right(self.n_nodes);

        // Loop throught the edge nodes and save all connections between nodes
        for i in 0..self.edge_nodes.len() {
            let edge_node = self.edge_nodes[i];
            pair_hash = match edge_node {
                Some(hash) => {
                    // If edge node on `i` level is not None, make a hash of pair [hash of the value, edge node]
                    let new_hash = BlakeTwo256::hash_of(&[hash, pair_hash]);
                    // Edge node now has a sibling and a parent
                    self.tree.insert(hash, Node {
                        parent: new_hash,
                        sibling: Some(pair_hash)
                    });
                    // New node now has a sibling (edge node) and a parent
                    self.tree.insert(pair_hash, Node {
                        parent: new_hash,
                        sibling: Some(hash)
                    });
                    new_hash
                },
                None => {
                    // No edge node on this level, just do the hash of itself
                    let new_hash = BlakeTwo256::hash_of(&pair_hash);
                    // New node doesnt have a sibling, but it has the parent
                    self.tree.insert(pair_hash, Node {
                        parent: new_hash,
                        sibling: None
                    });
                    new_hash
                }
            };
            // Save new edge node
            if (i + 1) as u8 == next_edge_addition_level {
                new_edge = pair_hash;
            }
        }

        // Update root hash
        self.root_hash = Some(pair_hash);
        // Set node index before we increment n_nodes
        self.node_indices.insert(value_hash, self.n_nodes);
        // Increment n_nodes
        self.n_nodes += 1;
        // Save new edge, and remove the invalid ones
        self.update_edges(new_edge, next_edge_addition_level as usize);

        // If tree has one level, no need to save snapshot
        if value_hash != pair_hash {
            self.save_snapshot(&pair_hash, &self.tree);
        }
    }

    // Saves the current tree state in `snapshots` folder
    // Snapshots will be used to check if some values exists inside `new_root_hash`

    // TODO: Since we are storing tree states to cold storage,
    // add functionality to continue syncing with on-chain tree from last saved checkpoint
    // Information about last saved checkpoint will be saved in file checkpoint inside `snapshots` folder
    fn save_snapshot(self: &Self, new_root_hash: &H256, tree: &Tree) {
        let name = format!("src/snapshots/{:?}", new_root_hash);
        let content = serde_json::to_string_pretty(tree);
        match content {
            Ok(data) => {
                let mut file = File::create(name).expect("Could not create file!");
                file.write_all(data.as_bytes()).expect("Failed to write to file!");
                file.sync_all().expect("Failed to sync file!");
            },
            Err(e) => panic!(e),
        }
    }

    // Loads the tree with state `root_hash`
    fn load_snapshot(self: &Self, root_hash: &H256) -> Result<Tree, &'static str> {
        let name = format!("src/snapshots/{:?}", root_hash);
        let file = File::open(name);
        let mut content = String::new();
        match file {
            Ok(mut f) => {
                let read = f.read_to_string(&mut content);
                match read {
                    Ok(_a) => {
                        let tree: Tree = serde_json::from_str(&content).unwrap();
                        Ok(tree)
                    },
                    Err(_e) => Err("Could not parse tree json!"),
                }
            },
            Err(_e) => Err("Could not read file!"),
        }
    }

    fn update_edges(self: &mut Self, new_edge_value: H256, addition_at_level: usize) {
        // If edge is on one level higher that current tree height, we push the new edge
        if addition_at_level >= self.edge_nodes.len() {
            self.edge_nodes.push(Some(new_edge_value));
        // If not just replace the value at level
        } else {
            self.edge_nodes[addition_at_level] = Some(new_edge_value);
        }

        // Remove all values below the level of new edge
        for i in 0..addition_at_level {
            self.edge_nodes[i] = None;
        }
    }

    // Calculating the successive number of 1 bits, starting from the right e.g.:
    // 0001 - 1
    // 0010 - 0
    // 0011 - 2
    // 1000 - 0
    fn count_bit_set_from_right(self: &mut Self, mut num: u128) -> u8 {
        let mut len: u8 = 0;
        while (num & 1) > 0 {
            num >>= 1;
            len += 1;
        }
        return len;
    }
}
