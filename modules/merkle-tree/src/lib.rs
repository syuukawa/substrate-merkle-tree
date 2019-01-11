#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
extern crate serde;

#[cfg(feature = "std")]
extern crate serde_derive;
#[cfg(test)]
#[macro_use]
extern crate hex_literal;
#[macro_use]
extern crate parity_codec_derive;
#[macro_use]
extern crate srml_support;

extern crate parity_codec as codec;
extern crate sr_io as runtime_io;
extern crate sr_primitives as runtime_primitives;
extern crate sr_std as rstd;
extern crate srml_support as runtime_support;
extern crate substrate_primitives as primitives;

extern crate srml_system as system;

pub mod merkle_tree;

#[cfg(test)]
mod tests {
	use super::*;

	use primitives::{Blake2Hasher, H256};
	use rstd::prelude::*;
	use runtime_io::ed25519::Pair;
	use runtime_io::with_externalities;
	use runtime_support::dispatch::Result;
	use system::{EventRecord, Phase};
	use runtime_primitives::{
		testing::{Digest, DigestItem, Header},
		traits::{BlakeTwo256, Hash, OnFinalise},
		BuildStorage,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	impl_outer_dispatch! {
		pub enum Call for Test where origin: Origin { }
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
		type AccountId = H256;
		type Header = Header;
		type Event = Event;
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Claim = Vec<u8>;
		type Event = Event;
	}

	type System = system::Module<Test>;
	type MerkleTree = Module<Test>;

	fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::<Test>::default()
			.build_storage()
			.unwrap()
			.0;

		t.extend(
			merkle_tree::GenesisConfig::<Test> {
				expiration_time: 1,
				verifiers: [H256::from(9)].to_vec(),
				claims_issuers: [H256::from(1), H256::from(2), H256::from(3)].to_vec(),
			}
			.build_storage()
			.unwrap()
			.0,
		);
		t.into()
	}

	#[test]
	fn should_be_able_to_initialize_merkle_tree() {
		with_externalities(&mut new_test_ext(), || {
			System::set_block_number(1);
		});
	}
}
