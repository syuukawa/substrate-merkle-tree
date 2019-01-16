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
        RootHash get(root_hash): Option<T::Hash>;
        NNodes get(n_nodes): u128;
        EdgeNodes get(edge_nodes): Vec<Option<T::Hash>>;
    }
}

decl_event!(
    pub enum Event<T> where <T as system::Trait>::Hash {
		Insert(Hash),
	}
);

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event<T>() = default;
        pub fn insert(value: Vec<u8>) -> Result {
            let value_hash = T::Hashing::hash_of(&value);

            let mut pair_hash = value_hash;
            let mut new_edge = value_hash;
            let n_nodes = Self::n_nodes();
            let next_edge_addition_level = Self::count_bit_set_from_right(n_nodes);

            let edge_nodes = Self::edge_nodes();
            for i in 0..edge_nodes.len() {
                let edge_node = edge_nodes[i];
                pair_hash = match edge_node {
                    Some(hash) => T::Hashing::hash_of(&[hash, pair_hash]),
                    None => T::Hashing::hash_of(&pair_hash)
                };
                if (i + 1) as u8 == next_edge_addition_level {
                    new_edge = pair_hash;
                }
            }

            <RootHash<T>>::put(pair_hash);
            <NNodes<T>>::put(n_nodes + 1);
            Self::update_edges(edge_nodes, new_edge, next_edge_addition_level as usize);

            Self::deposit_event(RawEvent::Insert(value_hash));
            Ok(())
        }

        pub fn verify_proof(proof: Vec<Option<T::Hash>>, value: Vec<u8>, node_n: u128, root_hash: T::Hash) -> Result {
            let mut value_hash = T::Hashing::hash_of(&value);
            for i in 0..proof.len() {
                let hash = proof[i];
                value_hash = match hash {
                    Some(h) => {
                        let is_even = 2u128.pow(i as u32) & node_n != 0;
                        let pair = if is_even {[h, value_hash]} else {[value_hash, h]};
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
        if addition_at_level >= edge_nodes.len() {
            edge_nodes.push(Some(new_edge_value));
        } else {
            edge_nodes[addition_at_level] = Some(new_edge_value);
        }

        for i in 0..addition_at_level {
            edge_nodes[i] = None;
        }

        <EdgeNodes<T>>::put(edge_nodes);
    }

    fn count_bit_set_from_right(mut num: u128) -> u8 {
        let mut len: u8 = 0;
        while (num & 1) > 0 {
            num >>= 1;
            len += 1;
        }
        return len;
    }
}
