use crate::mock::*;

use codec::Encode;
use parity_crypto::Keccak256;
use parity_crypto::publickey::{Random, Generator, Message, sign, KeyPair};
use frame_support::dispatch::DispatchError;


fn generate_msg(account: u64, block_number: u64) -> Message {

	let mut bytes = b"Link Litentry: ".encode();
	let mut account_vec = account.encode();
	let mut expiring_block_number_vec = block_number.encode();

	bytes.append(&mut account_vec);
	bytes.append(&mut expiring_block_number_vec);

	Message::from(bytes.keccak256())
}

fn generate_sig(key_pair: &KeyPair, msg: &Message) -> [u8; 65] {
	sign(key_pair.secret(), &msg).unwrap().into_electrum()
}

fn generate_rsv(sig: &[u8; 65]) -> ([u8; 32], [u8; 32], u8) {
	let mut r = [0u8; 32];
	let mut s = [0u8; 32];

	r[..32].copy_from_slice(&sig[..32]);
	s[..32].copy_from_slice(&sig[32..64]);
	let v = sig[64];
	(r, s, v)
}

#[test]
fn test_invalid_block_number() {
	new_test_ext().execute_with(|| {

		let account: u64 = 5;
		let block_number: u64 = 0;

		let mut gen = Random{};
		let key_pair = gen.generate().unwrap();

		let msg = generate_msg(account, block_number);
		let sig = generate_sig(&key_pair, &msg);
		let (r, s, v) = generate_rsv(&sig);

		let result = AccountLinker::link(Origin::signed(1), account, 0, block_number, r, s, v);
		assert_eq!(result.is_err(), true);
		assert_eq!(result.err(), Some(DispatchError::from(AccountLinkerError::LinkRequestExpired)));
	});
}

#[test]
fn test_insert_eth_address() {
	new_test_ext().execute_with(|| {

		let account: u64 = 5;
		let block_number: u64 = 99999;


		let mut gen = Random{};
		let mut expected_vec = Vec::new();

		for i in 0..(MAX_ETH_LINKS) {

			let key_pair = gen.generate().unwrap();

			let msg = generate_msg(account, block_number + i as u64);
			let sig = generate_sig(&key_pair, &msg);

			let (r, s, v) = generate_rsv(&sig);

			let _ = AccountLinker::link(Origin::signed(1),
										account,
										i as u32,
										block_number + i as u64,
										r,
										s,
										v);
			assert_eq!(AccountLinker::eth_addresses(account).len(), i+1);
			expected_vec.push(key_pair.address().to_fixed_bytes());
		}
		assert_eq!(AccountLinker::eth_addresses(account), expected_vec);
	});
}

#[test]
fn test_update_eth_address() {
	new_test_ext().execute_with(|| {

		let account: u64 = 5;
		let block_number: u64 = 99999;

		let mut gen = Random{};
		for i in 0..(MAX_ETH_LINKS) {
			let key_pair = gen.generate().unwrap();
			let msg = generate_msg(account, block_number + i as u64);
			let sig = generate_sig(&key_pair, &msg);
			let (r, s, v) = generate_rsv(&sig);

			let _ = AccountLinker::link(Origin::signed(1),
										account,
										i as u32,
										block_number + i as u64,
										r,
										s,
										v);
		}

		let index: u32 = 2 as u32;
		// Retrieve previous addr
		let addr_before_update =  AccountLinker::eth_addresses(&account)[index as usize];
		// Update addr at slot `index`
		let key_pair = gen.generate().unwrap();
		let block_number = block_number + 9 as u64;
		let msg = generate_msg(account, block_number);
		let sig = generate_sig(&key_pair, &msg);
		let (r, s, v) = generate_rsv(&sig);

		let _ = AccountLinker::link(Origin::signed(1),
									account,
									index,
									block_number,
									r,
									s,
									v);
		let updated_addr =  AccountLinker::eth_addresses(&account)[index as usize];
		assert_ne!(updated_addr, addr_before_update);
		assert_eq!(updated_addr, key_pair.address().to_fixed_bytes());
	});
}


#[test]
fn test_eth_address_pool_overflow() {
	new_test_ext().execute_with(|| {

		let account: u64 = 5;
		let block_number: u64 = 99999;

		let mut gen = Random{};
		let mut expected_vec = Vec::new();

		for index in 0..(MAX_ETH_LINKS*2) {
			let key_pair = gen.generate().unwrap();

			let msg = generate_msg(account, block_number);
			let sig = generate_sig(&key_pair, &msg);
			let (r, s, v) = generate_rsv(&sig);

			let _ = AccountLinker::link(Origin::signed(1),
										account,
										index as u32,
										block_number,
										r,
										s,
										v);
			if index < MAX_ETH_LINKS {
				expected_vec.push(key_pair.address().to_fixed_bytes());
			} else {
				expected_vec[MAX_ETH_LINKS-1] = key_pair.address().to_fixed_bytes();
			}
		}
		assert_eq!(AccountLinker::eth_addresses(account).len(), MAX_ETH_LINKS);
		assert_eq!(AccountLinker::eth_addresses(account), expected_vec);
	});
}
