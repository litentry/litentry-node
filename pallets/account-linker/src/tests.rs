use crate::{Error, mock::*};
use codec::Encode;
use frame_support::{assert_ok, assert_noop};
// use ethkey::keccak::Keccak256;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        
        let account: u64 = 5;
        let block_number: u64 = 123;

        let mut bytes = b"Link Litentry: ".encode();
        let mut account_vec = account.encode(); // Warning: must be 32 bytes
        let mut expiring_block_number_vec = block_number.encode(); // Warning: must be 4 bytes

        bytes.append(&mut account_vec);
        bytes.append(&mut expiring_block_number_vec);

        // let hash = bytes.keccak256();

		// assert_ok!(AccountLinker::link(
        //     Origin::signed(1),
        //     account,
        //     0,
        //     block_number,
            
        //     42
        // ));
		// // Read pallet storage and assert an expected result.
		// assert_eq!(AccountLinker::something(), Some(42));
	});
}

// account: T::AccountId,
// index: u32,
// block_number: T::BlockNumber,
// r: [u8; 32],
// s: [u8; 32],
// v: u8,

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			TemplateModule::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }
