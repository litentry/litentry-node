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
}

