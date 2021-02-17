#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, account};
use frame_system::RawOrigin;
use sp_std::prelude::*;

benchmarks!{
    _ {
        let b in 1 .. 1000 => ();
    }

    asset_claim {
        let b in ...;
        let caller = account("caller", 0, 0);
        
    }: _ (RawOrigin::Signed(caller))

    submit_balance {
        let b in ...;
        let caller = account("caller", 0, 0);
        let account_id = account("Alice", 0, 0);
        <ClaimAccountIndex<T>>::insert(&account_id, 0_u32);
        let block_number = 1_u32;
        let data_source = urls::DataSource::EthEtherScan;
        let balance = 0_u128;
        
    }: _ (RawOrigin::Signed(caller), account_id, block_number.into(), data_source.into(), balance)
}

