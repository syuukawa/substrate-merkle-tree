#[cfg(feature = "std")]
extern crate serde;

#[cfg(feature = "std")]
extern crate parity_codec as codec;
extern crate sr_io as runtime_io;
extern crate sr_primitives as runtime_primitives;
extern crate sr_std as rstd;
extern crate srml_support as runtime_support;
extern crate substrate_primitives as primitives;

extern crate srml_system as system;

use runtime_support::dispatch::Result;
use runtime_support::{Parameter, StorageMap, StorageValue};
use parity_codec::Encode;
use runtime_primitives::traits::Hash;
use rstd::prelude::*;

pub trait Trait: system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait> as MerkleTree {
        RootHash get(root_hash): Option<T::Hash>;
        NNodes get(n_nodes): u32;
        EdgeNodes get(edge_nodes): Vec<Option<T::Hash>>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        pub fn insert(_origin, value: &[u8]) -> Result {
            let value_hash = T::Hashing::hash_of(&value);

            let mut pair_hash = value_hash;
            let mut new_edge = value_hash;
            let n_nodes = Self::n_nodes();
            let next_edge_addition_level = 1;

            let edge_nodes = Self::edge_nodes();
            for i in 0..edge_nodes.len() {
                let edge_node = edge_nodes[i];
                pair_hash = match edge_node {
                    Some(hash) => T::Hashing::hash_of(&edge_node.concat(&pair_hash)),
                    None => T::Hashing::hash_of(&pair_hash)
                }
                if (i + 1) as u8 == next_edge_addition_level {
                    new_edge = pair_hash;
                }
            }

            <RootHash<T>>::put(Some(pair_hash));
            <NNodes<T>>::put(n_nodes + 1);
            Self::update_edges(new_edge, next_edge_addition_level as usize);
            Ok(())
        }

        fn update_edges(new_edge_value: T::Hash, addition_at_level: usize) {
            let edge_nodes = Self::edge_nodes();
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

        // fn count_bit_set_from_right(mut num: u32) -> u8 {
        //     let mut len: u8 = 0;
        //     while (num & 1) > 0 {
        //         num >>= 1;
        //         len += 1;
        //     }
        //     return len;
        // }
    }
}
