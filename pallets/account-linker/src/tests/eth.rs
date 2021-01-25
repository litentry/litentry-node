use crate::{mock::*, Event};

use codec::Encode;
use parity_crypto::Keccak256;
use parity_crypto::publickey::{Random, Generator, Message, sign, KeyPair};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::AccountId32;

fn generate_msg(account: &AccountId32, block_number: u32) -> Message {

	let mut bytes = b"\x19Ethereum Signed Message:\n51Link Litentry: ".encode();
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
fn test_expired_block_number_eth() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let block_number: u32 = 0;

		let mut gen = Random{};
		let key_pair = gen.generate().unwrap();

		let msg = generate_msg(&account, block_number);
		let sig = generate_sig(&key_pair, &msg);
		let (r, s, v) = generate_rsv(&sig);

		assert_noop!(
			AccountLinker::link_eth(
				Origin::signed(account.clone()),
				account.clone(),
				0,
				key_pair.address().to_fixed_bytes(),
				block_number,
				r,
				s,
				v),
			AccountLinkerError::LinkRequestExpired
		);
	});
}

#[test]
fn test_invalid_expiring_block_number_eth() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([0u8; 32]);
		let block_number: u32 = crate::EXPIRING_BLOCK_NUMBER_MAX + 1;

		let mut gen = Random{};
		let key_pair = gen.generate().unwrap();

		let msg = generate_msg(&account, block_number);
		let sig = generate_sig(&key_pair, &msg);
		let (r, s, v) = generate_rsv(&sig);

		assert_noop!(
			AccountLinker::link_eth(
				Origin::signed(account.clone()),
				account.clone(),
				0,
				key_pair.address().to_fixed_bytes(),
				block_number,
				r,
				s,
				v),
			AccountLinkerError::InvalidExpiringBlockNumber
		);
	});
}

#[test]
fn test_unexpected_address_eth() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([72u8; 32]);
		let block_number: u32 = 99999;

		let mut gen = Random{};
		let key_pair = gen.generate().unwrap();

		let msg = generate_msg(&account, block_number);
		let sig = generate_sig(&key_pair, &msg);
		let (r, s, v) = generate_rsv(&sig);

		assert_noop!(
			AccountLinker::link_eth(
				Origin::signed(account.clone()),
				account.clone(),
				0,
				gen.generate().unwrap().address().to_fixed_bytes(),
				block_number,
				r,
				s,
				v),
			AccountLinkerError::UnexpectedAddress
		);
	});
}

#[test]
fn test_insert_eth_address() {
	new_test_ext().execute_with(|| {

        run_to_block(1);

		let account: AccountId32 = AccountId32::from([5u8; 32]);
		let block_number: u32 = 99999;

		let mut gen = Random{};
		let mut expected_vec = Vec::new();

		for i in 0..(MAX_ETH_LINKS) {

			let key_pair = gen.generate().unwrap();

			let msg = generate_msg(&account, block_number + i as u32);
			let sig = generate_sig(&key_pair, &msg);

			let (r, s, v) = generate_rsv(&sig);

			assert_ok!(AccountLinker::link_eth(
				Origin::signed(account.clone()),
				account.clone(),
				i as u32,
				key_pair.address().to_fixed_bytes(),
				block_number + i as u32,
				r,
				s,
				v
			));

            assert_eq!(AccountLinker::eth_addresses(&account).len(), i+1);
            assert_eq!(
                System::events()[i].event,  
                TestEvent::account_linker( Event::<Test>::EthAddressLinked(
                    account.clone(), 
                    key_pair.address().to_fixed_bytes().to_vec())
                )
            );
			expected_vec.push(key_pair.address().to_fixed_bytes());
		}
		assert_eq!(AccountLinker::eth_addresses(&account), expected_vec);
	});
}

#[test]
fn test_update_eth_address() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([40u8; 32]);
		let block_number: u32 = 99999;

		let mut gen = Random{};
		for i in 0..(MAX_ETH_LINKS) {
			let key_pair = gen.generate().unwrap();
			let msg = generate_msg(&account, block_number + i as u32);
			let sig = generate_sig(&key_pair, &msg);
			let (r, s, v) = generate_rsv(&sig);

			assert_ok!(AccountLinker::link_eth(
				Origin::signed(account.clone()),
				account.clone(),
				i as u32,
				key_pair.address().to_fixed_bytes(),
				block_number + i as u32,
				r,
				s,
				v
			));
		}

		let index: u32 = 2 as u32;
		// Retrieve previous addr
		let addr_before_update =  AccountLinker::eth_addresses(&account)[index as usize];
		// Update addr at slot `index`
		let key_pair = gen.generate().unwrap();
		let block_number = block_number + 9 as u32;
		let msg = generate_msg(&account, block_number);
		let sig = generate_sig(&key_pair, &msg);
		let (r, s, v) = generate_rsv(&sig);

		assert_ok!(AccountLinker::link_eth(
			Origin::signed(account.clone()),
			account.clone(),
			index,
			key_pair.address().to_fixed_bytes(),
			block_number,
			r,
			s,
			v
		));

		let updated_addr =  AccountLinker::eth_addresses(&account)[index as usize];
		assert_ne!(updated_addr, addr_before_update);
		assert_eq!(updated_addr, key_pair.address().to_fixed_bytes());
	});
}


#[test]
fn test_eth_address_pool_overflow() {
	new_test_ext().execute_with(|| {

		let account: AccountId32 = AccountId32::from([113u8; 32]);
		let block_number: u32 = 99999;

		let mut gen = Random{};
		let mut expected_vec = Vec::new();

		for index in 0..(MAX_ETH_LINKS*2) {
			let key_pair = gen.generate().unwrap();

			let msg = generate_msg(&account, block_number);
			let sig = generate_sig(&key_pair, &msg);
			let (r, s, v) = generate_rsv(&sig);

			assert_ok!(AccountLinker::link_eth(
				Origin::signed(account.clone()),
				account.clone(),
				index as u32,
				key_pair.address().to_fixed_bytes(),
				block_number,
				r,
				s,
				v
			));

			if index < MAX_ETH_LINKS {
				expected_vec.push(key_pair.address().to_fixed_bytes());
			} else {
				expected_vec[MAX_ETH_LINKS-1] = key_pair.address().to_fixed_bytes();
			}
		}
		assert_eq!(AccountLinker::eth_addresses(&account).len(), MAX_ETH_LINKS);
		assert_eq!(AccountLinker::eth_addresses(&account), expected_vec);
	});
}
