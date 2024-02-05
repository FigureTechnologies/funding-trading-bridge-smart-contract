//! Funding Trading Bridge Smart Contract
//!
//! This contract uses [Cosmwasm](https://github.com/CosmWasm/cosmwasm)'s provided architecture in
//! conjunction with [Provwasm](#https://github.com/provenance-io/provwasm) to create a wasm smart
//! contract that can be deployed to and interact with the Provenance Blockchain.
//!
//! This contract is designed to facilitate movement of funds from two different restricted [Markers](https://developer.provenance.io/docs/pb/modules/marker-module).
//! It allows one-to-one trades for the two markers, accounting for differences in artificially
//! described precisions for the coin counts in the marker denoms.

/// The entrypoint for all external commands sent to the compiled wasm.
pub mod contract;
/// All code and functions pertaining to the execute entrypoint.
pub mod execute;
/// All code and functions pertaining to the instantiate entrypoint.
pub mod instantiate;
/// All code and functions pertaining to the migrate entrypoint.
pub mod migrate;
/// All code and functions pertaining to the query entrypoint.
pub mod query;
/// All code and functions pertaining to interacting with mutable contract data storage.
pub mod store;
/// All globally-defined structs used by functions throughout the contract.
pub mod types;
/// Utility functions and traits adopted by various aspects of the contract.
pub mod util;

#[cfg(test)]
pub mod test;
