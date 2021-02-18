#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod btc;
mod util_eth;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub const EXPIRING_BLOCK_NUMBER_MAX: u32 = 10 * 60 * 24 * 30; // 30 days for 6s per block
pub const MAX_ETH_LINKS: usize = 3;
pub const MAX_BTC_LINKS: usize = 3;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
	use sp_io::crypto::secp256k1_ecdsa_recover_compressed;
	use crate::btc;
	use crate::util_eth;
	use crate::btc::witness::WitnessProgram;
	use crate::btc::base58::ToBase58;
	const EXPIRING_BLOCK_NUMBER_MAX: u32 = crate::EXPIRING_BLOCK_NUMBER_MAX; // 30 days for 6s per block
	pub const MAX_ETH_LINKS: usize = crate::MAX_ETH_LINKS;
	pub const MAX_BTC_LINKS: usize = crate::MAX_BTC_LINKS;
	enum BTCAddrType {
		Legacy,
		Segwit,
	}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn eth_addresses)]
	pub(super) type EthereumLink<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<[u8; 20]>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn btc_addresses)]
	pub(super) type BitcoinLink<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Vec<Vec<u8>>, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EthAddressLinked(T::AccountId, Vec<u8>),
		BtcAddressLinked(T::AccountId, Vec<u8>),
	}
	
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		EcdsaRecoverFailure,
		LinkRequestExpired,
		UnexpectedAddress,
		// Unexpected ethereum message length error
		UnexpectedEthMsgLength,
		InvalidBTCAddress,
		InvalidBTCAddressLength,
		InvalidExpiringBlockNumber,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {

		#[pallet::weight(1)]
		pub fn link_eth(
			origin: OriginFor<T>,
			account: T::AccountId,
			index: u32,
			addr_expected: [u8; 20],
			expiring_block_number: T::BlockNumber,
			r: [u8; 32],
			s: [u8; 32],
			v: u8,
		) -> DispatchResultWithPostInfo {

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
			Self::deposit_event(Event::EthAddressLinked(account, addr.to_vec()));

			Ok(().into())

		}

		#[pallet::weight(1)]
		pub fn link_btc(
			origin: OriginFor<T>,
			account: T::AccountId,
			index: u32,
			addr_expected: Vec<u8>,
			expiring_block_number: T::BlockNumber,
			r: [u8; 32],
			s: [u8; 32],
			v: u8,
		) -> DispatchResultWithPostInfo {

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

			let addr = match addr_type {
				BTCAddrType::Legacy => {
					btc::legacy::btc_addr_from_pk(&pk).to_base58()
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
					wp.to_address(b"bc".to_vec()).map_err(|_| Error::<T>::InvalidBTCAddress)?
				}
			};

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
			Self::deposit_event(Event::BtcAddressLinked(account, addr));

			Ok(().into())
		}
		
	}
}
