use crate::{Error, mock::*};
use codec::Encode;
use frame_support::{assert_ok, assert_noop};
use parity_crypto::Keccak256;
use parity_crypto::publickey::{Random, Generator, Message, sign};
use sp_std::prelude::*;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {

		let account: u64 = 5;
		let block_number: u64 = 9999;

		let mut bytes = b"Link Litentry: ".encode();
		let mut account_vec = account.encode(); // Warning: must be 32 bytes
		let mut expiring_block_number_vec = block_number.encode(); // Warning: must be 4 bytes

		bytes.append(&mut account_vec);
		bytes.append(&mut expiring_block_number_vec);

		let msg = Message::from(bytes.keccak256());

		let mut gen = Random{};
		let key_pair = gen.generate().unwrap();

		let sig = sign(key_pair.secret(), &msg).unwrap().into_electrum();

		let mut r = [0u8; 32];
		let mut s = [0u8; 32];

		r[..32].copy_from_slice(&sig[..32]);
		s[..32].copy_from_slice(&sig[32..64]);
		let v = sig[64];

		assert_ok!(AccountLinker::link(
			Origin::signed(1),
			account,
			0,
			block_number,
			r,
			s,
			v
		));
		assert_eq!(AccountLinker::eth_addresses(account), vec![key_pair.address().to_fixed_bytes()]);
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
