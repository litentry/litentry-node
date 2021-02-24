
//! Autogenerated weights for pallet_account_linker
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-02-23, STEPS: [20, ], REPEAT: 50, LOW RANGE: [], HIGH RANGE: []
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// target/release/litentry-node
// benchmark
// --chain
// dev
// --execution=wasm
// --wasm-execution=compiled
// --pallet
// pallet_account_linker
// --extrinsic
// link_eth
// --heap-pages=4096
// --steps
// 20
// --repeat
// 50
// --output=./pallets/account-linker/src/weights.rs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_account_linker.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_account_linker::WeightInfo for WeightInfo<T> {
	fn link_eth() -> Weight {
		(379_000_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
