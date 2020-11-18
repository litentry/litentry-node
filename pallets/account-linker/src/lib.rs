#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_std::prelude::*;
use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod util;

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as AccountLinker {
		EthereumLink get(fn eth_addresses): map hasher(blake2_128_concat) T::AccountId => Vec<[u8; 20]>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		SomethingStored(u32, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		EcdsaRecoverFailure,
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
		pub fn link(
			origin,
			account: T::AccountId,
			index: u32,
			block_number: T::BlockNumber,
			r: [u8; 32],
			s: [u8; 32],
			v: u8,
		) -> dispatch::DispatchResult {

			let _ = ensure_signed(origin)?;

			let current_block_number = <frame_system::Module<T>>::block_number();
			// TODO: check block number not expired

			let b0 = b"Link Litentry: ";

			let account_vec = account.encode(); // Warning: must be 32 bytes

			let block_number_vec = block_number.encode(); // Warning: must be 4 bytes
			
			// let mut message_data = format!("Link Litentry: {}, {}", b1, b2);
			// // let b2 = timestamp.to_be_bytes();
			let mut bytes = [0u8; 51]; // TODO: need to change this if b0 changes
			bytes[..15].copy_from_slice(b0);
			bytes[15..47].copy_from_slice(&account_vec);
			bytes[47..].copy_from_slice(&block_number_vec);

			let hash = sp_io::hashing::keccak_256(&bytes);

			let mut msg = [0u8; 32];
			let mut sig = [0u8; 65];
	
			msg[..32].copy_from_slice(&hash[..32]);
			sig[..32].copy_from_slice(&r[..32]);
			sig[32..64].copy_from_slice(&s[..32]);
			sig[64] = v;

			let addr = util::addr_from_sig(msg, sig)
				.map_err(|_| Error::<T>::EcdsaRecoverFailure)?;

			let index = index as usize;
			let mut addrs = Self::eth_addresses(&account);
			if (index >= addrs.len()) && (addrs.len() != 3) { // allow linking 3 eth addresses. TODO: do not use hard code
				addrs.push(addr);
			} else if (index >= addrs.len()) && (addrs.len() == 3) {
				addrs[2] = addr;
			} else {
				addrs[index] = addr;
			}

			<EthereumLink<T>>::insert(account, addrs);

			Ok(())

		}

	}
}
