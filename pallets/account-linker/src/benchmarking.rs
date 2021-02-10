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
        // account for Alice
        // let account_id = [212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125];
        let account_id: T::AccountId = account("Alice", 0, SEED);
        let index: u32 = 0;
        let addr_expected = [77, 136, 220, 93, 82, 138, 51, 228, 184, 190, 87, 158, 148, 118, 113, 95, 96, 6, 5, 130];
        let expiring_block_number: u32 = 10000;
        let r = [49, 132, 0, 240, 249, 189, 21, 240, 216, 132, 40, 112, 181, 16, 233, 150, 223, 252, 148, 75, 119, 17, 29, 237, 3, 164, 37, 92, 102, 232, 45, 66];
        let s = [113, 50, 231, 101, 213, 230, 187, 33, 186, 4, 109, 187, 152, 226, 139, 178, 140, 178, 190, 190, 12, 138, 206, 210, 197, 71, 172, 166, 10, 85, 72, 146];
        let v: u8 = 28_u8;
            
    }: _ (RawOrigin::Signed(caller), account_id.into(), index, addr_expected, expiring_block_number.into(), r, s, v)
}
