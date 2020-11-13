#![cfg_attr(not(feature = "std"), no_std)]

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
	trait Store for Module<T: Trait> as TemplateModule {
		/// currently only one link per account!
		EthereumLink get(fn eth_address): map hasher(blake2_128_concat) T::AccountId => [u8; 20];
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
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// separate sig to r, s, v because runtime only support array parameter with length <= 32
		#[weight = 1]
		pub fn link(
			origin, 
			block_number: u32,
			timestamp: u32,
			r: [u8; 32],
			s: [u8; 32],
			v: u8,
		) -> dispatch::DispatchResult {

			let who = ensure_signed(origin)?;

			let b1 = block_number.to_be_bytes();
			let b2 = timestamp.to_be_bytes();
			let bytes = [b1, b2].concat();

			// TODO: add check, add accountId, add prefix

			let hash = sp_io::hashing::keccak_256(&bytes);

			let mut msg = [0u8; 32];
			let mut sig = [0u8; 65];
	
			msg[0..32].copy_from_slice(&hash[0..32]);
			sig[0..32].copy_from_slice(&r[0..32]);
			sig[32..64].copy_from_slice(&s[0..32]);
			sig[64] = v;

			let addr = util::addr_from_sig(msg, sig)
				.map_err(|_| Error::<T>::EcdsaRecoverFailure)?;
	
			<EthereumLink<T>>::insert(who, addr);

			Ok(())

		}
	}
}
