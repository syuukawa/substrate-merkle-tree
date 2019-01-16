#[cfg(feature = "std")]
extern crate serde;

extern crate serde_derive;

#[cfg(feature = "std")]
extern crate parity_codec as codec;
extern crate sr_io as runtime_io;
extern crate sr_primitives as runtime_primitives;
extern crate sr_std as rstd;
#[macro_use]
extern crate srml_support as runtime_support;
extern crate substrate_primitives as primitives;
extern crate srml_system as system;

#[macro_use]
extern crate parity_codec_derive;

pub mod client;
pub mod merkle_tree;
pub use crate::merkle_tree::{Event, Module, RawEvent, Trait};
pub use crate::client::MerkleTreeClient;

#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use substrate_primitives::{H256, Blake2Hasher};

	use runtime_primitives::{
		testing::{Digest, DigestItem, Header},
		traits::{BlakeTwo256, Hash},
		BuildStorage,
	};
	use system::{EventRecord, Phase};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	impl_outer_event! {
		pub enum Event for Test {
			merkle_tree<T>,
		}
	}

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;

	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Header = Header;
		type Event = Event;
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = Event;
	}

	type System = system::Module<Test>;
	type MerkleTree = Module<Test>;

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	fn get_event_values() -> Vec<H256> {
		let mut event_values: Vec<H256> = vec![];
		for event_record in System::events().iter() {
			let hash: H256 = match event_record.event {
				Event::merkle_tree(RawEvent::Insert(e)) => e,
				_ => panic!("hello there")
			};
			event_values.push(hash);
		}
		event_values
	}

	#[test]
	fn should_be_able_to_get_correct_initial_values() {
		with_externalities(&mut new_test_ext(), || {
			let root_hash = MerkleTree::root_hash();
			let n_nodes = MerkleTree::n_nodes();
			let edge_nodes = MerkleTree::edge_nodes();
			assert_eq!(root_hash, None);
			assert_eq!(n_nodes, 0u128);
			assert_eq!(edge_nodes, vec![]);
		});
	}

	#[test]
	fn should_be_able_to_correctly_insert_values() {
		with_externalities(&mut new_test_ext(), || {
			// a
		    let a = "a".to_string().as_bytes().to_vec();
		    let a_hash = BlakeTwo256::hash_of(&a);

		    let mut result = MerkleTree::insert(a);
			assert_eq!(result, Ok(()));

		    let mut root_hash = MerkleTree::root_hash();
		    let mut edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(a_hash));
		    assert_eq!(edge_nodes, vec![Some(a_hash)]);

		    // b
		    let b = "b".to_string().as_bytes().to_vec();
		    let b_hash = BlakeTwo256::hash_of(&b);

		    let ab_hash = BlakeTwo256::hash_of(&[a_hash, b_hash]);

		    result = MerkleTree::insert(b);
			assert_eq!(result, Ok(()));

		    root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(ab_hash));
		    assert_eq!(edge_nodes, vec![None, Some(ab_hash)]);

		    // c
		    let c = "c".to_string().as_bytes().to_vec();
		    let c_hash = BlakeTwo256::hash_of(&c);

		    let c1_hash = BlakeTwo256::hash_of(&c_hash);

		    let abc1_hash = BlakeTwo256::hash_of(&[ab_hash, c1_hash]);

		    result = MerkleTree::insert(c);
			assert_eq!(result, Ok(()));

		    root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abc1_hash));
		    assert_eq!(edge_nodes, vec![Some(c_hash), Some(ab_hash)]);

		    // d
		    let d = "d".to_string().as_bytes().to_vec();
		    let d_hash = BlakeTwo256::hash_of(&d);

		    let cd_hash = BlakeTwo256::hash_of(&[c_hash, d_hash]);
		    let abcd_hash = BlakeTwo256::hash_of(&[ab_hash, cd_hash]);

		    result = MerkleTree::insert(d);
			assert_eq!(result, Ok(()));

			root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abcd_hash));
		    assert_eq!(edge_nodes, vec![None, None, Some(abcd_hash)]);

		    // e
		    let e = "e".to_string().as_bytes().to_vec();
		    let e_hash = BlakeTwo256::hash_of(&e);

		    let e1_hash = BlakeTwo256::hash_of(&e_hash);
		    let e2_hash = BlakeTwo256::hash_of(&e1_hash);
		    let abcde2_hash = BlakeTwo256::hash_of(&[abcd_hash, e2_hash]);

		    result = MerkleTree::insert(e);
			assert_eq!(result, Ok(()));

			root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abcde2_hash));
		    assert_eq!(edge_nodes, vec![Some(e_hash), None, Some(abcd_hash)]);

		    // f
		    let f = "f".to_string().as_bytes().to_vec();
		    let f_hash = BlakeTwo256::hash_of(&f);

		    let ef_hash = BlakeTwo256::hash_of(&[e_hash, f_hash]);
		    let ef1_hash = BlakeTwo256::hash_of(&ef_hash);
		    let abcdef1_hash = BlakeTwo256::hash_of(&[abcd_hash, ef1_hash]);

		    result = MerkleTree::insert(f);
			assert_eq!(result, Ok(()));

			root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abcdef1_hash));
		    assert_eq!(edge_nodes, vec![None, Some(ef_hash), Some(abcd_hash)]);

		    // g
		    let g = "g".to_string().as_bytes().to_vec();
		    let g_hash = BlakeTwo256::hash_of(&g);

		    let g1_hash = BlakeTwo256::hash_of(&g_hash);
		    let efg1_hash = BlakeTwo256::hash_of(&[ef_hash, g1_hash]);
		    let abcdefg1_hash = BlakeTwo256::hash_of(&[abcd_hash, efg1_hash]);

		    result = MerkleTree::insert(g);
			assert_eq!(result, Ok(()));

			root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abcdefg1_hash));
		    assert_eq!(edge_nodes, vec![Some(g_hash), Some(ef_hash), Some(abcd_hash)]);

		    // h
		    let h = "h".to_string().as_bytes().to_vec();
		    let h_hash = BlakeTwo256::hash_of(&h);

		    let gh_hash = BlakeTwo256::hash_of(&[g_hash, h_hash]);
		    let efgh_hash = BlakeTwo256::hash_of(&[ef_hash, gh_hash]);
		    let abcdefgh_hash = BlakeTwo256::hash_of(&[abcd_hash, efgh_hash]);

		    result = MerkleTree::insert(h);
			assert_eq!(result, Ok(()));

			root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abcdefgh_hash));
		    assert_eq!(edge_nodes, vec![None, None, None, Some(abcdefgh_hash)]);

		    // i
		    let i = "i".to_string().as_bytes().to_vec();
		    let i_hash = BlakeTwo256::hash_of(&i);

		    let i1_hash = BlakeTwo256::hash_of(&i_hash);
		    let i2_hash = BlakeTwo256::hash_of(&i1_hash);
		    let i3_hash = BlakeTwo256::hash_of(&i2_hash);
		    let abcdefghi3_hash = BlakeTwo256::hash_of(&[abcdefgh_hash, i3_hash]);

		    result = MerkleTree::insert(i);
			assert_eq!(result, Ok(()));

			root_hash = MerkleTree::root_hash();
		    edge_nodes = MerkleTree::edge_nodes();

		    assert_eq!(root_hash, Some(abcdefghi3_hash));
		    assert_eq!(edge_nodes, vec![Some(i_hash), None, None, Some(abcdefgh_hash)]);
		});
	}

	#[test]
	fn should_be_able_to_get_emitted_event() {
		with_externalities(&mut new_test_ext(), || {
			let a = "a".to_string().as_bytes().to_vec();
		    let a_hash = BlakeTwo256::hash_of(&a);

			let result = MerkleTree::insert(a);
			assert_eq!(result, Ok(()));
			assert_eq!(
				System::events(),
				vec![
					EventRecord {
						phase: Phase::ApplyExtrinsic(0),
						event: RawEvent::Insert(a_hash).into()
					}
				]
			);
		});
	}

	#[test]
	fn should_be_able_to_sync_with_client() {
		with_externalities(&mut new_test_ext(), || {
			let a = "a".to_string().as_bytes().to_vec();
			let b = "b".to_string().as_bytes().to_vec();
			let c = "c".to_string().as_bytes().to_vec();

			let mut result = MerkleTree::insert(a);
			assert_eq!(result, Ok(()));
			result = MerkleTree::insert(b);
			assert_eq!(result, Ok(()));
			result = MerkleTree::insert(c);
			assert_eq!(result, Ok(()));

			let root_hash = MerkleTree::root_hash();

			let mut client_tree = MerkleTreeClient::new();
			let event_values = get_event_values();
			client_tree.build_tree_from_events(event_values);

			let client_root_hash = client_tree.root_hash;
			assert_eq!(client_root_hash, root_hash);
		});
	}

	#[test]
	fn should_be_able_to_create_proof() {
		with_externalities(&mut new_test_ext(), || {
			let a = "a".to_string().as_bytes().to_vec();
			let b = "b".to_string().as_bytes().to_vec();
			let c = "c".to_string().as_bytes().to_vec();

			let mut result = MerkleTree::insert(a.clone());
			assert_eq!(result, Ok(()));
			result = MerkleTree::insert(b.clone());
			assert_eq!(result, Ok(()));
			result = MerkleTree::insert(c.clone());
			assert_eq!(result, Ok(()));

			let root_hash = MerkleTree::root_hash();

			let mut client_tree = MerkleTreeClient::new();
			let event_values = get_event_values();
			client_tree.build_tree_from_events(event_values);

			let node_number = client_tree.get_node_number(a.clone());
			let is_even = node_number % 2 == 0;
			let proof = client_tree.get_proof_for(a.clone(), root_hash.unwrap());
			let res = MerkleTree::verify_proof(proof.unwrap(), a.clone(), is_even, root_hash.unwrap());
			assert_eq!(res, Ok(()));
		});
	}

	#[test]
	fn should_be_able_to_create_proof_for_previous_states() {
		with_externalities(&mut new_test_ext(), || {
			let a = "a".to_string().as_bytes().to_vec();
			let b = "b".to_string().as_bytes().to_vec();
			let c = "c".to_string().as_bytes().to_vec();
			let d = "d".to_string().as_bytes().to_vec();

			let mut result = MerkleTree::insert(a.clone());
			assert_eq!(result, Ok(()));
			result = MerkleTree::insert(b.clone());
			assert_eq!(result, Ok(()));

			let root_hash_after_b = MerkleTree::root_hash();

			result = MerkleTree::insert(c.clone());
			assert_eq!(result, Ok(()));

			let root_hash_after_c = MerkleTree::root_hash();

			result = MerkleTree::insert(d.clone());
			assert_eq!(result, Ok(()));

			let root_hash_after_d = MerkleTree::root_hash();

			let mut client_tree = MerkleTreeClient::new();
			let event_values = get_event_values();
			client_tree.build_tree_from_events(event_values);

			let node_number = client_tree.get_node_number(a.clone());
			let is_even = node_number % 2 == 0;
			let mut proof = client_tree.get_proof_for(a.clone(), root_hash_after_b.unwrap());
			let mut res = MerkleTree::verify_proof(proof.unwrap(), a.clone(), is_even, root_hash_after_b.unwrap());
			assert_eq!(res, Ok(()));

			proof = client_tree.get_proof_for(a.clone(), root_hash_after_c.unwrap());
			res = MerkleTree::verify_proof(proof.unwrap(), a.clone(), is_even, root_hash_after_c.unwrap());
			assert_eq!(res, Ok(()));

			proof = client_tree.get_proof_for(a.clone(), root_hash_after_d.unwrap());
			res = MerkleTree::verify_proof(proof.unwrap(), a.clone(), is_even, root_hash_after_d.unwrap());
			assert_eq!(res, Ok(()));

			// For b also

			let node_number = client_tree.get_node_number(b.clone());
			let is_even = node_number % 2 == 0;
			proof = client_tree.get_proof_for(b.clone(), root_hash_after_b.unwrap());
			res = MerkleTree::verify_proof(proof.unwrap(), b.clone(), is_even, root_hash_after_b.unwrap());
			assert_eq!(res, Ok(()));

			proof = client_tree.get_proof_for(b.clone(), root_hash_after_c.unwrap());
			res = MerkleTree::verify_proof(proof.unwrap(), b.clone(), is_even, root_hash_after_c.unwrap());
			assert_eq!(res, Ok(()));

			proof = client_tree.get_proof_for(b.clone(), root_hash_after_d.unwrap());
			res = MerkleTree::verify_proof(proof.unwrap(), b.clone(), is_even, root_hash_after_d.unwrap());
			assert_eq!(res, Ok(()));
		});
	}
}
