#[cfg(feature = "std")]
extern crate serde;

extern crate sr_io as runtime_io;
extern crate sr_primitives as runtime_primitives;
extern crate sr_std as rstd;
extern crate substrate_primitives as primitives;

extern crate srml_system as system;

use runtime_support::dispatch::Result;
use runtime_support::StorageValue;
use runtime_primitives::traits::{Hash};
use rstd::prelude::*;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as MerkleTree {
        // Root hash of the tree
        RootHash get(root_hash): Option<T::Hash>;
        // Number of nodes in the tree
        NNodes get(n_nodes): u128;
        // Hashes of the edge nodes needed for pairing with next insert
        EdgeNodes get(edge_nodes): Vec<Option<T::Hash>>;
    }
}

decl_event!(
    // Event fired when new addition is added. Whole tree can be derived on client from these events
    pub enum Event<T> where <T as system::Trait>::Hash {
		Insert(Hash),
	}
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;
        pub fn insert(value: Vec<u8>) -> Result {
            // Make a hash of value
            let value_hash = T::Hashing::hash_of(&value);

            // Pair node used for hashing with edge node
            let mut pair_hash = value_hash;
            let mut new_edge = value_hash;
            let n_nodes = Self::n_nodes();
            // Get the level on which will be the next new edge node
            let next_edge_addition_level = Self::count_bit_set_from_right(n_nodes);

            let edge_nodes = Self::edge_nodes();
            // Loop trought all levels of the tree
            for i in 0..edge_nodes.len() {
                let edge_node = edge_nodes[i];
                pair_hash = match edge_node {
                    // There is edge node on this level, make a hash of the pair
                    Some(hash) => T::Hashing::hash_of(&[hash, pair_hash]),
                    // There is no edge node on this level, hash itself then
                    None => T::Hashing::hash_of(&pair_hash)
                };
                if (i + 1) as u8 == next_edge_addition_level {
                    // Hash on this level is new edge
                    new_edge = pair_hash;
                }
            }

            // Update the root hash
            <RootHash<T>>::put(pair_hash);
            <NNodes<T>>::put(n_nodes + 1);
            // Update edge nodes
            Self::update_edges(edge_nodes, new_edge, next_edge_addition_level as usize);

            // Emit the event so the client can sync with the contract
            Self::deposit_event(RawEvent::Insert(value_hash));
            Ok(())
        }

        // Proove that `value` hash index of `node_index` and that it exists inside `root_hash` state
        pub fn verify_proof(proof: Vec<Option<T::Hash>>, value: Vec<u8>, node_index: u128, root_hash: T::Hash) -> Result {
            let mut value_hash = T::Hashing::hash_of(&value);
            for i in 0..proof.len() {
                let hash = proof[i];
                value_hash = match hash {
                    Some(h) => {
                        // Check if node on `i` level is left or right sibling
                        let is_right = 2u128.pow(i as u32) & node_index != 0;
                        let pair = if is_right {[h, value_hash]} else {[value_hash, h]};
                        T::Hashing::hash_of(&pair)
                    },
                    None => T::Hashing::hash_of(&value_hash),
                }
            }
            ensure!(value_hash == root_hash, "Proof not valid");
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn update_edges(mut edge_nodes: Vec<Option<T::Hash>>, new_edge_value: T::Hash, addition_at_level: usize) {
        // If edge is on one level higher that current tree height, we push the new edge
        if addition_at_level >= edge_nodes.len() {
            edge_nodes.push(Some(new_edge_value));

        // If not just replace the value at level
        } else {
            edge_nodes[addition_at_level] = Some(new_edge_value);
        }

        // Remove all values below the level of new edge
        for i in 0..addition_at_level {
            edge_nodes[i] = None;
        }

        <EdgeNodes<T>>::put(edge_nodes);
    }

    // Calculating the successive number of 1 bits, starting from the right e.g.:
    // 0001 - 1
    // 0010 - 0
    // 0011 - 2
    // 1000 - 0
    fn count_bit_set_from_right(mut num: u128) -> u8 {
        let mut len: u8 = 0;
        while (num & 1) > 0 {
            num >>= 1;
            len += 1;
        }
        return len;
    }
}
