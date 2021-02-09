#![cfg(feature = "runtime-benchmarks")]

use super::*;

use frame_benchmarking::{benchmarks, account};
use frame_system::RawOrigin;
use sp_std::prelude::*;
const SEED: u32 = 0;
benchmarks!{
    _ {
        let b in 1 .. 1000 => ();
    }
    link_eth {
		let b in ...;
        let caller = account("caller", 0, 0);
        let account_id: T::AccountId = account("recipient", 0, SEED);
        let index: u32 = 0;
        let addr_expected = [0_u8; 20];
        let expiring_block_number: u32 = 10000;
        let r = [0_u8; 32];
        let s = [0_u8; 32];
        let v: u8 = 0_u8;
            
    }: _ (RawOrigin::Signed(caller), account_id, index, addr_expected, 
    expiring_block_number.into(), r, s, v)
}