#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use sp_std::{prelude::*};

use frame_system::{
	ensure_signed, ensure_none,
	offchain::{CreateSignedTransaction, SubmitTransaction},
};
use frame_support::{
	debug, dispatch, decl_module, decl_storage, decl_event, decl_error,
	traits::Get,
};
use sp_core::crypto::KeyTypeId;
// use sp_runtime::{
// 	transaction_validity::{
// 		InvalidTransaction, TransactionValidity, TransactionSource,
// 	},
// };
use sp_runtime::offchain::http;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
	};
	use sp_core::sr25519::Signature as Sr25519Signature;
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: CreateSignedTransaction<Call<Self>> {
	// type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Call: From<Call<Self>>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where	AccountId = <T as frame_system::Trait>::AccountId, {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, Option<AccountId>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 10_000]
		pub fn record_price(
			origin,
			// _block: T::BlockNumber,
			price: u32
		) -> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			ensure_none(origin)?;
			// Spit out an event and Add to storage
			Self::deposit_event(RawEvent::SomethingStored(price, None));

			Ok(())
		}


		fn offchain_worker(block: T::BlockNumber) {

			debug::info!("Hello World.");
			let result = Self::fetch_github_info();
			if let Err(e) = result {
				debug::info!("Hello World.{:?} ", e);
			}
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, Some(who)));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}
	}
}

impl<T: Trait> Module<T> {
	fn fetch_github_info() -> Result<(), Error<T>> {
		let _result = Self::fetch_json(b"btc");
		
		let call = Call::record_price(100);
		SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
		.map_err(|_| {
			debug::error!("Failed in offchain_unsigned_tx");
			<Error<T>>::StorageOverflow
		})

	}

	fn fetch_json<'a>(remote_url: &'a [u8]) -> Result<(), &'static str> {
		let remote_url_str = core::str::from_utf8(remote_url)
			.map_err(|_| "Error in converting remote_url to string")?;
	
		let pending = http::Request::get(remote_url_str).send()
			.map_err(|_| "Error in sending http GET request")?;
	
		let response = pending.wait()
			.map_err(|_| "Error in waiting http response back")?;
	
		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err("Non-200 status code returned from http request");
		}
	
		let _json_result: Vec<u8> = response.body().collect::<Vec<u8>>();
	
		// print_bytes(&json_result);
	
		// let json_val: JsonValue = simple_json::parse_json(
		//   &core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?)
		//   .map_err(|_| "JSON parsing error")?;
	
		Ok(())
	}
}

// #[allow(deprecated)] // ValidateUnsigned
// impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
// 	type Call = Call<T>;

	// fn validate_unsigned(
	// 	_source: TransactionSource,
	// 	_call: &Self::Call,
	// ) -> TransactionValidity {
	// 	InvalidTransaction::Call.into()
		// Firstly let's check that we call the right function.
		// if let Call::submit_price_unsigned_with_signed_payload(
		// 	ref payload, ref signature
		// ) = call {
		// 	let signature_valid = SignedPayload::<T>::verify::<T::AuthorityId>(payload, signature.clone());
		// 	if !signature_valid {
		// 		return InvalidTransaction::BadProof.into();
		// 	}
		// 	Self::validate_transaction_parameters(&payload.block_number, &payload.price)
		// } else if let Call::submit_price_unsigned(block_number, new_price) = call {
		// 	Self::validate_transaction_parameters(block_number, new_price)
		// } else {
		// 	InvalidTransaction::Call.into()
		// }
// 	}
// }