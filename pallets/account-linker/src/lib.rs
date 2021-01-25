#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_std::prelude::*;
use sp_io::crypto::secp256k1_ecdsa_recover_compressed;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, ensure};
use frame_system::{ensure_signed};
use btc::base58::ToBase58;
use btc::witness::WitnessProgram;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod btc;
mod util_eth;

const EXPIRING_BLOCK_NUMBER_MAX: u32 = 10 * 60 * 24 * 30; // 30 days for 6s per block
pub const MAX_ETH_LINKS: usize = 3;
pub const MAX_BTC_LINKS: usize = 3;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

enum BTCAddrType {
	Legacy,
	Segwit,
}

decl_storage! {
	trait Store for Module<T: Trait> as AccountLinkerModule {
		pub EthereumLink get(fn eth_addresses): map hasher(blake2_128_concat) T::AccountId => Vec<[u8; 20]>;
		pub BitcoinLink get(fn btc_addresses): map hasher(blake2_128_concat) T::AccountId => Vec<Vec<u8>>;
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Trait>::AccountId,
	{
		EthAddressLinked(AccountId, Vec<u8>),
		BtcAddressLinked(AccountId, Vec<u8>),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		EcdsaRecoverFailure,
		LinkRequestExpired,
		UnexpectedAddress,
		// Unexpected ethereum message length error
		UnexpectedEthMsgLength,
		InvalidBTCAddress,
		InvalidBTCAddressLength,
		InvalidExpiringBlockNumber,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where
		origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// separate sig to r, s, v because runtime only support array parameter with length <= 32
		#[weight = 1]
		pub fn link_eth(
			origin,
			account: T::AccountId,
			index: u32,
			addr_expected: [u8; 20],
			expiring_block_number: T::BlockNumber,
			r: [u8; 32],
			s: [u8; 32],
			v: u8,
		) -> dispatch::DispatchResult {

			let _ = ensure_signed(origin)?;

			let current_block_number = <frame_system::Module<T>>::block_number();
			ensure!(expiring_block_number > current_block_number, Error::<T>::LinkRequestExpired);
			ensure!((expiring_block_number - current_block_number) < T::BlockNumber::from(EXPIRING_BLOCK_NUMBER_MAX), 
				Error::<T>::InvalidExpiringBlockNumber);

			let mut bytes = b"Link Litentry: ".encode();
			let mut account_vec = account.encode();
			let mut expiring_block_number_vec = expiring_block_number.encode();

			bytes.append(&mut account_vec);
			bytes.append(&mut expiring_block_number_vec);

			let hash = util_eth::eth_data_hash(bytes).map_err(|_| Error::<T>::UnexpectedEthMsgLength)?;

			let mut msg = [0u8; 32];
			let mut sig = [0u8; 65];

			msg[..32].copy_from_slice(&hash[..32]);
			sig[..32].copy_from_slice(&r[..32]);
			sig[32..64].copy_from_slice(&s[..32]);
			sig[64] = v;

			let addr = util_eth::addr_from_sig(msg, sig)
				.map_err(|_| Error::<T>::EcdsaRecoverFailure)?;
			ensure!(addr == addr_expected, Error::<T>::UnexpectedAddress);

			let index = index as usize;
			let mut addrs = Self::eth_addresses(&account);
			// NOTE: allow linking `MAX_ETH_LINKS` eth addresses.
			if (index >= addrs.len()) && (addrs.len() != MAX_ETH_LINKS) {
				addrs.push(addr.clone());
			} else if (index >= addrs.len()) && (addrs.len() == MAX_ETH_LINKS) {
				addrs[MAX_ETH_LINKS - 1] = addr.clone();
			} else {
				addrs[index] = addr.clone();
			}

			<EthereumLink<T>>::insert(account.clone(), addrs);
			Self::deposit_event(RawEvent::EthAddressLinked(account, addr.to_vec()));

			Ok(())

		}

		/// separate sig to r, s, v because runtime only support array parameter with length <= 32
		#[weight = 1]
		pub fn link_btc(
			origin,
			account: T::AccountId,
			index: u32,
			addr_expected: Vec<u8>,
			expiring_block_number: T::BlockNumber,
			r: [u8; 32],
			s: [u8; 32],
			v: u8,
		) -> dispatch::DispatchResult {

			let _ = ensure_signed(origin)?;

			let current_block_number = <frame_system::Module<T>>::block_number();
			ensure!(expiring_block_number > current_block_number, Error::<T>::LinkRequestExpired);
			ensure!((expiring_block_number - current_block_number) < T::BlockNumber::from(EXPIRING_BLOCK_NUMBER_MAX), 
				Error::<T>::InvalidExpiringBlockNumber);

			// TODO: we may enlarge this 2
			if addr_expected.len() < 2 {
				Err(Error::<T>::InvalidBTCAddressLength)?
			}

			let addr_type = if addr_expected[0] == b'1' {
				BTCAddrType::Legacy
			} else if addr_expected[0] == b'b' && addr_expected[1] == b'c' { // TODO: a better way?
				BTCAddrType::Segwit
			} else {
				Err(Error::<T>::InvalidBTCAddress)?
			};

			let mut bytes = b"Link Litentry: ".encode();
			let mut account_vec = account.encode();
			let mut expiring_block_number_vec = expiring_block_number.encode();

			bytes.append(&mut account_vec);
			bytes.append(&mut expiring_block_number_vec);

			// TODO: seems btc uses sha256???
			let hash = sp_io::hashing::keccak_256(&bytes);

			let mut msg = [0u8; 32];
			let mut sig = [0u8; 65];

			msg[..32].copy_from_slice(&hash[..32]);
			sig[..32].copy_from_slice(&r[..32]);
			sig[32..64].copy_from_slice(&s[..32]);
			sig[64] = v;

			let pk = secp256k1_ecdsa_recover_compressed(&sig, &msg)
			.map_err(|_| Error::<T>::EcdsaRecoverFailure)?;

			let mut addr;

			match addr_type {
				BTCAddrType::Legacy => {
					addr = btc::legacy::btc_addr_from_pk(&pk).to_base58();
				},
				// Native P2WPKH is a scriptPubKey of 22 bytes. 
				// It starts with a OP_0, followed by a canonical push of the keyhash (i.e. 0x0014{20-byte keyhash})
				// keyhash is RIPEMD160(SHA256) of a compressed public key
				// https://bitcoincore.org/en/segwit_wallet_dev/
				BTCAddrType::Segwit => {
					let pk_hash = btc::legacy::hash160(&pk);
					let mut pk = [0u8; 22];
					pk[0] = 0;
					pk[1] = 20;
					pk[2..].copy_from_slice(&pk_hash);
					let wp = WitnessProgram::from_scriptpubkey(&pk.to_vec()).map_err(|_| Error::<T>::InvalidBTCAddress)?;
					addr = wp.to_address(b"bc".to_vec()).map_err(|_| Error::<T>::InvalidBTCAddress)?;
				}
			}

			ensure!(addr == addr_expected, Error::<T>::UnexpectedAddress);

			let index = index as usize;
			let mut addrs = Self::btc_addresses(&account);
			// NOTE: allow linking `MAX_BTC_LINKS` btc addresses.
			if (index >= addrs.len()) && (addrs.len() != MAX_BTC_LINKS) {
				addrs.push(addr.clone());
			} else if (index >= addrs.len()) && (addrs.len() == MAX_BTC_LINKS) {
				addrs[MAX_BTC_LINKS - 1] = addr.clone();
			} else {
				addrs[index] = addr.clone();
			}

			<BitcoinLink<T>>::insert(account.clone(), addrs);
			Self::deposit_event(RawEvent::BtcAddressLinked(account, addr));

			Ok(())

		}

	}
}
