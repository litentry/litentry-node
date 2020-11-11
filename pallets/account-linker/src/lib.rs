#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

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

			let hash = sp_io::hashing::keccak_256(&bytes);

			let mut msg = [0u8; 32];
			let mut sig = [0u8; 65];
	
			msg[0..32].copy_from_slice(&hash[0..32]);
			sig[0..32].copy_from_slice(&r[0..32]);
			sig[32..64].copy_from_slice(&s[0..32]);
			sig[64] = v;
	
			let pubkey = sp_io::crypto::secp256k1_ecdsa_recover(&sig, &msg)
				.map_err(|_| Error::<T>::EcdsaRecoverFailure)?;
			let address = sp_io::hashing::keccak_256(&pubkey);

			let mut addr = [0u8; 20];
			addr[0..20].copy_from_slice(&address[12..32]);
	
			<EthereumLink<T>>::insert(who, addr);

			Ok(())

		}
	}
}
