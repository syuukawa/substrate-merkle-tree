//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

extern crate futures;
#[macro_use]
extern crate error_chain;
extern crate tokio;
#[macro_use]
extern crate log;
extern crate substrate_cli;
extern crate substrate_primitives as primitives;
extern crate substrate_consensus_aura as consensus;
extern crate substrate_client as client;
#[macro_use]
extern crate substrate_network as network;
#[macro_use]
extern crate substrate_executor;
extern crate substrate_transaction_pool as transaction_pool;
extern crate substrate_basic_authorship as basic_authorship;
#[macro_use]
extern crate substrate_service;
extern crate substrate_merkle_tree_runtime;
#[macro_use]
extern crate structopt;
extern crate node_executor;
extern crate sr_primitives as runtime_primitives;

mod chain_spec;
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn run() -> cli::error::Result<()> {
	let version = VersionInfo {
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "substrate-merkle-tree",
		author: "filiplazovic",
		description: "substrate-merkle-tree",
	};
	cli::run(::std::env::args(), cli::Exit, version)
}

quick_main!(run);
