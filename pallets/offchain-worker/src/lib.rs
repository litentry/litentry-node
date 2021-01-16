//! # Offchain Worker 
//! The pallet is responsible for get the external assets claim from the extrinsic and then query and aggregate the 
//! balance (btc and eth) according to linked external accounts in account linker pallet. Offchain worker get the data
//! from most popular websire like etherscan, infura and blockinfo. After get the balance, Offchain worker emit the event
//! with balance info and store them on chain for on-chain query.
//! 
//! ## API token
//! The offchain worker need the API token to query data from third party data provider. Currently, offchain worker get 
//! the API tokens from a local server. Then store the API tokens in offchain worder local storage.
//! 

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*};
use core::{convert::TryInto,};
use frame_system::{
	ensure_signed, ensure_none,
	offchain::{CreateSignedTransaction, SubmitTransaction, Signer, AppCrypto, SendSignedTransaction,},
};
use frame_support::{
	debug, dispatch, decl_module, decl_storage, decl_event, decl_error,
	ensure, storage::IterableStorageMap, traits::Get, weights::Weight,
};
use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	transaction_validity::{
		ValidTransaction, InvalidTransaction, TransactionValidity, TransactionSource, TransactionLongevity,
	},
};
use sp_runtime::offchain::{storage::StorageValueRef,};
use codec::{Encode, Decode};

mod urls;
mod utils;

#[cfg(test)]
mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocw!");
const TOKEN_SERVER_URL: &str = "http://127.0.0.1:4000";

pub mod crypto {
	use super::KEY_TYPE;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify, MultiSignature, MultiSigner,
	};
	use sp_core::sr25519::Signature as Sr25519Signature;
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// Response stored on chain
#[derive(Encode, Decode, Default)]
struct QueryKey<AccountId> {
	/// Response vector for several Ethreum account
	account: AccountId,
	blockchain_type: urls::BlockChainType,
}

pub trait Trait: frame_system::Trait + account_linker::Trait + CreateSignedTransaction<Call<Self>> {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Call: From<Call<Self>>;
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	type QueryTaskRedudancy: Get<u32>;
	type QuerySessionLength: Get<u32>;
}

decl_storage! {
	trait Store for Module<T: Trait> as OffchainWorkerModule {
		/// Record how many claims from Litentry user 
		TotalClaims get(fn total_claims): u64;

		/// Record the accounts send claims in latest block
		ClaimAccountSet get(fn query_account_set): map hasher(blake2_128_concat) T::AccountId => ();

		/// ClaimAccountNumber record how many accout claimed asset query in last session
		ClaimAccountNumber get(fn claim_account_number): u32;

		/// ClaimAccountIndex record the index of account claimed asset query in last session
		ClaimAccountIndex get(fn claim_account_index): map hasher(blake2_128_concat) T::AccountId => Option<u32>;

		/// Record account's btc and ethereum balance
		AccountBalance get(fn account_balance): map hasher(blake2_128_concat) T::AccountId => (Option<u128>, Option<u128>);
		
		/// Map AccountId
		LastCommitBlockNumber get(fn last_commit_block_number): map hasher(blake2_128_concat) T::AccountId => T::BlockNumber;

		/// 
		CommitAccountBalance get(fn commit_account_balance): double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) QueryKey<T::AccountId> => Option<u128>;

		/// 
		AccountIndex get(fn account_index): map hasher(blake2_128_concat) T::AccountId => Option<u32>;

		/// 
		CommittedAccountNumber get(fn committed_account_number): u32;
	}
}

decl_event!(
	pub enum Event<T> where	AccountId = <T as frame_system::Trait>::AccountId,
					BlockNumber = <T as frame_system::Trait>::BlockNumber, {
		/// Event for account and its ethereum balance
		BalanceGot(AccountId, BlockNumber, Option<u128>, Option<u128>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Error number parsing.
		InvalidNumber,
		/// Account already in claim list.
		AccountAlreadyInClaimlist,
		/// No local account for offchain worker to sign extrinsic
		NoLocalAcctForSigning,
		/// Error from sign extrinsic
		OffchainSignedTxError,
		/// Invalid data source
		InvalidDataSource,
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

		const QueryTaskRedudancy: u32 = T::QueryTaskRedudancy::get();
		const QuerySessionLength: u32 = T::QuerySessionLength::get();

		// Request offchain worker to get balance of linked external account
		#[weight = 10_000]
		pub fn asset_claim(origin,) -> dispatch::DispatchResult {
			let account = ensure_signed(origin)?;

			// If the same claim not processed yet
			ensure!(!<ClaimAccountSet<T>>::contains_key(&account), Error::<T>::AccountAlreadyInClaimlist);

			<ClaimAccountSet<T>>::insert(&account, ());

			Ok(())
		}

		// Clear claimed account list
		#[weight = 10_000]
		fn clear_claim(origin, block: T::BlockNumber)-> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			ensure_none(origin)?;

			let accounts: Vec<T::AccountId> = <ClaimAccountSet::<T>>::iter().map(|(k, _)| k).collect();
 
			// Set claim account number
			ClaimAccountNumber::put(accounts.len() as u32);

			// Set account index
			let abc: Option<u32> = Some(123);
			for (index, account) in accounts.iter().enumerate() {
				<ClaimAccountIndex<T>>::insert(&account, index as u32);
			}

			// Remove all claimed accounts
			<ClaimAccountSet::<T>>::remove_all();

			Ok(())
		}

		// Record the balance on chain
		#[weight = 10_000]
		fn record_balance(
			origin,
			account: T::AccountId,
			block: T::BlockNumber,
			btc_balance: Option<u128>,
			eth_balance: Option<u128>,
		) -> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			ensure_none(origin)?;
			// Record the total claims processed
			TotalClaims::put(Self::total_claims() + 1);
			// Set balance 
			<AccountBalance<T>>::insert(account.clone(), (btc_balance, eth_balance));
			// Spit out an event and Add to storage
			// Self::deposit_event(RawEvent::BalanceGot(account, block, btc_balance, eth_balance));

			Ok(())
		}

		// Record the balance on chain
		#[weight = 10_000]
		fn submit_number_signed(origin, account: T::AccountId, block_number: T::BlockNumber, data_source: urls::DataSource, balance: u128)-> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			let sender = ensure_signed(origin)?;

			// Check data source
			Self::valid_data_source(data_source)?;

			// Check block number
			Self::valid_commit_block_number(block_number, <frame_system::Module<T>>::block_number())?;

			// Check the commit slot
			Self::valid_commit_slot(account.clone(), Self::get_ocw_index(&account), data_source)?;

			// put query result on chain
			let blockchain_type = urls::data_source_to_block_chain_type(data_source);
			CommitAccountBalance::<T>::insert(&sender, &QueryKey{account, blockchain_type}, balance);

			Ok(())
		}

		// Called at the beginning of each block
		fn on_initialize(block_number: T::BlockNumber) -> Weight {
			debug::info!("ocw on_initialize {:?}.", block_number);
			0
		}

		// Call at the end of each block
		fn on_finalize() {
			debug::info!("ocw on_finalize.");
		}

		// Trigger by offchain framework in each block
		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("ocw offchain_worker {:?}.", block_number);

			let QuerySessionLength: usize = T::QuerySessionLength::get() as usize;

			// Check session length
			if QuerySessionLength < 3 {
				debug::error!("ocw QuerySessionLength is too low as {}.", QuerySessionLength);
				return
			}

			let last_block_number = QuerySessionLength - 1;

			match block_number.try_into()
				.map_or(QuerySessionLength, |bn| bn % QuerySessionLength)
			{
				// The first block will trigger all offchain worker and clean the claims
				0 => Self::start(block_number),
				// The last block for aggregation and put balance queried into chain
				// May use the on finalized to do it.
				last_block_number => {
					// Record the account in local storage
					let account = StorageValueRef::persistent(b"offchain-worker::account");
					// account.set(&acc.id);

					match account.get::<T::AccountId>() {
						Some(Some(info)) => {
							debug::info!("Offchain Worker end successfully.");
						},
						_ => {
							debug::info!("Offchain Worker to get token from local server.");
						},
					}

				},
				// Block between 1 and last block reserved for offchain worker to query and submit result
				_ => {},
			};			
		}
	}
}

impl<T: Trait> Module<T> {
	// Start new round of offchain worker
	fn start(block_number: T::BlockNumber) {
		let local_token = StorageValueRef::persistent(b"offchain-worker::token");
		
		match local_token.get::<urls::TokenInfo>() {
			Some(Some(token)) => {
				// Get all accounts who ask for asset claim
				let accounts: Vec<T::AccountId> = <ClaimAccountSet::<T>>::iter().map(|(k, _)| k).collect();

				if accounts.len() > 0 {
					// Try to remove claims via tx
					let call = Call::clear_claim(block_number);
					let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
					.map_err(|_| {
						debug::error!("Failed in offchain_unsigned_tx");
						<Error<T>>::InvalidNumber
					});

					// Update balance after query
					Self::query(&accounts, block_number, &token);
					debug::info!("Offchain Worker end successfully.");
				}
			},
			_ => {
				debug::info!("Offchain Worker to get token from local server.");
				// Get token from local server
				let _ = urls::get_token();
			},
		};
	}

	fn valid_commit_slot(account: T::AccountId, ocw_index: u32, data_source: urls::DataSource) -> dispatch::DispatchResult {
		// account claimed the asset query
		let account_index = Self::get_account_index(account)?;

		// ocw length
		let ocw_length = Self::get_ocw_length();
		// if no ocw works in last session, then all new ocw valid for all accounts with all data source
		if ocw_length == 0 {
			return Ok(())
		}	
		
		// ensure ocw index is valid
		ensure!(ocw_index <= ocw_length, <Error<T>>::InvalidDataSource);

		// ensure data source is valid
		ensure!(data_source != urls::DataSource::Invalid, <Error<T>>::InvalidDataSource);

		// get data source index
		let data_source_index = urls::data_source_to_index(data_source);
		
		// query task rounds
		let QueryTaskRedudancy: u32 = T::QueryTaskRedudancy::get();

		// task number per round
		let total_task_per_round = urls::TOTAL_DATA_SOURCE_NUMBER * Self::claim_account_number();

		// task index in the first round
		let task_base_index = data_source_index + account_index * urls::TOTAL_DATA_SOURCE_NUMBER;

		let mut round: u32 = 0;
		while round < QueryTaskRedudancy {
			// task index in n round
			let task_index = task_base_index + round * total_task_per_round;

			if task_index >= ocw_index {
				// if index match return Ok
				if (task_index - ocw_index) % ocw_length == 0 {
					return Ok(())
				}
			}
			round = round + 1;
		}

		// no match found, return error
		Err(<Error<T>>::InvalidDataSource.into())
	}

	fn get_account_index(account: T::AccountId) -> Result<u32, Error<T>> {
		match Self::claim_account_index(account) {
			Some(index) => Ok(index),
			None => Err(<Error<T>>::InvalidDataSource.into()),
		}
	}

	fn valid_data_source(data_source: urls::DataSource) -> dispatch::DispatchResult {
		match data_source {
			urls::DataSource::Invalid => Err(<Error<T>>::InvalidDataSource.into()),
			_ => Ok(()),
		}
	}

	fn valid_commit_block_number(commit_block_number: T::BlockNumber, current_block_number: T::BlockNumber) -> dispatch::DispatchResult {
		let zero_block: u32 = 0;
		// let a: u32 = block_number.try_into();
		let commit_block_number: u32 = commit_block_number.try_into().map_or(zero_block, |block_number| block_number as u32);
		let current_block_number: u32 = current_block_number.try_into().map_or(zero_block, |block_number| block_number as u32);

		if (commit_block_number == 0 || current_block_number == 0) {
			return Err(<Error<T>>::InvalidDataSource.into());
		}

		let sesseion_start_block = commit_block_number / T::QueryTaskRedudancy::get() * T::QueryTaskRedudancy::get();
		let sesseion_end_block = sesseion_start_block + T::QueryTaskRedudancy::get();

		if (current_block_number > sesseion_end_block || sesseion_end_block <= sesseion_start_block) {
			return Err(<Error<T>>::InvalidDataSource.into());
		}
		
		Ok(())
	}

	fn get_ocw_index(account: &T::AccountId) -> u32 {
		match Self::account_index(account) {
			Some(index_in_map) => index_in_map,
			_ => Self::get_ocw_length()
		}
	}

	fn get_ocw_length() -> u32 {
		<AccountIndex::<T>>::iter().collect::<Vec<_>>().len() as u32
	}

	fn query(account_vec: &Vec<T::AccountId>, block_number: T::BlockNumber, info: &urls::TokenInfo) {
		let offchain_worker_account = StorageValueRef::persistent(b"offchain-worker::account");

		match offchain_worker_account.get::<T::AccountId>() {
			Some(Some(account)) => {
				let ocw_account_index = Self::get_ocw_index(&account);

				let total_task = (account_vec.len() as u32) * urls::TOTAL_DATA_SOURCE_NUMBER;
				let mut task_index = 0_u32;

				// loop for each account
				for (index, account) in account_vec.iter().enumerate() {
					// loop for each source
					for (_, source) in urls::DataSourceList.iter().enumerate() {
						if task_index % total_task == ocw_account_index {
							match source {
								urls::DataSource::EtherScan => {
									match Self::get_balance_from_etherscan(account, block_number, info) {
										Some(balance) => Self::offchain_signed_tx(account.clone(), block_number, urls::DataSource::EtherScan, balance),
										None => ()
									}
								},
								urls::DataSource::Infura => {
									match Self::get_balance_from_infura(account, block_number, info) {
										Some(balance) => Self::offchain_signed_tx(account.clone(), block_number, urls::DataSource::Infura, balance),
										None => ()
									}
								},
								urls::DataSource::BlockChain => {
									match Self::get_balance_from_blockchain_info(account, block_number, info) {
										Some(balance) => Self::offchain_signed_tx(account.clone(), block_number, urls::DataSource::BlockChain, balance),
										None => ()
									}
								},
								_ => (),
							};
						}
						task_index = task_index + 1;
					}
				}
			},
			_ => {
				urls::get_token();
			}
		}
	}

	fn get_balance_from_etherscan(account: &T::AccountId, block: T::BlockNumber, info: &urls::TokenInfo) -> Option<u128> {
		if info.etherscan.len() == 0 {
			None
		} else {
			match core::str::from_utf8(&info.etherscan) {
				Ok(token) => {
					let get = urls::HttpGet {
						blockchain: urls::BlockChainType::ETH,
						prefix: "https://api-ropsten.etherscan.io/api?module=account&action=balancemulti&address=0x",
						delimiter: ",0x",
						postfix: "&tag=latest&apikey=",
						api_token: token,
					};

					Self::fetch_balances(
						<account_linker::EthereumLink<T>>::get(account), 
						urls::HttpRequest::GET(get),
						&urls::parse_etherscan_balances).ok()
				},
				Err(_) => None,
			}
		}
	}

	fn get_balance_from_infura(account: &T::AccountId, block: T::BlockNumber, info: &urls::TokenInfo) -> Option<u128> {
		
		if info.infura.len() == 0 {
			None
		} else {
			match core::str::from_utf8(&info.infura) {
				Ok(token) => {
					let post = urls::HttpPost {
						url_main: "https://ropsten.infura.io/v3/",
						blockchain: urls::BlockChainType::ETH,
						prefix: r#"[{"jsonrpc":"2.0","method":"eth_getBalance","id":1,"params":["0x"#,
						delimiter: r#"","latest"]},{"jsonrpc":"2.0","method":"eth_getBalance","id":1,"params":["0x"#,
						postfix: r#"","latest"]}]"#,
						api_token: token,
					};
					Self::fetch_balances(
						<account_linker::EthereumLink<T>>::get(account),
						urls::HttpRequest::POST(post),
						&urls::parse_blockchain_info_balances).ok()
				},
				Err(_) => None,
			}
		}
	}

	fn get_balance_from_blockchain_info(account: &T::AccountId, block: T::BlockNumber, info: &urls::TokenInfo) -> Option<u128> {
		if info.blockchain.len() == 0 {
			None
		} else {
			match core::str::from_utf8(&info.blockchain) {
				Ok(token) => {
					let get = urls::HttpGet {
							blockchain: urls::BlockChainType::BTC,
							prefix: "https://blockchain.info/balance?active=",
							delimiter: "%7C",
							postfix: "",
							api_token: token,
					};
					Self::fetch_balances(Vec::new(), 
						urls::HttpRequest::GET(get), 
						&urls::parse_blockchain_info_balances).ok()
				},
				Err(_) => None,
			}
		}
	}

	// Sign the query result 
	fn offchain_signed_tx(account: T::AccountId, block_number: T::BlockNumber, data_source: urls::DataSource, balance: u128) {
		// Get signer from ocw
		let signer = Signer::<T, T::AuthorityId>::any_account();

		// Translating the current block number to number and submit it on-chain
		// let number: u64 = block_number.try_into().unwrap_or(0) as u64;

		let result = signer.send_signed_transaction(|_acct|
			// This is the on-chain function
			Call::submit_number_signed(account.clone(), block_number, data_source, balance)
		);

		// Display error if the signed tx fails.
		if let Some((acc, res)) = result {
			if res.is_err() {
				debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
			} else {
				debug::info!("successful: offchain_signed_tx: tx sent: {:?} index is {:?}", acc.id, acc.index);
			}

			// Record the account in local storage then we can know my index
			let account = StorageValueRef::persistent(b"offchain-worker::account");
			account.set(&acc.id);
		} else {
			debug::error!("No local account available");
		}
	}

	// Generic function to fetch balance for specific link type
	fn fetch_balances(wallet_accounts: Vec<[u8; 20]>, request: urls::HttpRequest, 
		parser: &dyn Fn(&str) -> Option<Vec<u128>>) -> Result<u128, Error<T>> {
		// Return if no account linked
		if wallet_accounts.len() == 0 {
			return Ok(0_u128)
		}

		let result: Vec<u8> = match request {
			urls::HttpRequest::GET(get_req) => {
				// Compose the get request URL 
				let mut link: Vec<u8> = Vec::new();
				link.extend(get_req.prefix.as_bytes());

				for (i, each_account) in wallet_accounts.iter().enumerate() {
					// Append delimiter if there are more than one accounts in the account_vec
					if i >=1 {
						link.extend(get_req.delimiter.as_bytes());
					};

					link.extend(utils::address_to_string(each_account));
				}
				link.extend(get_req.postfix.as_bytes());
				link.extend(get_req.api_token.as_bytes());

				// Fetch json response via http get
				urls::fetch_json_http_get(&link[..]).map_err(|_| Error::<T>::InvalidNumber)?
			},

			urls::HttpRequest::POST(post_req) => {
				// Compose the post request URL
				let mut link: Vec<u8> = Vec::new();
				link.extend(post_req.url_main.as_bytes());
				link.extend(post_req.api_token.as_bytes());

				// Batch multiple JSON-RPC calls for multiple getBalance operations within one post
				let mut body: Vec<u8> = Vec::new();
				body.extend(post_req.prefix.as_bytes());

				for (i, each_account) in wallet_accounts.iter().enumerate() {
					// Append delimiter if there are more than one accounts in the account_vec
					if i >=1 {
						body.extend(post_req.delimiter.as_bytes());
					};

					body.extend(utils::address_to_string(each_account));
				}
				body.extend(post_req.postfix.as_bytes());

				// Fetch json response via http post 
				urls::fetch_json_http_post(&link[..], &body[..]).map_err(|_| Error::<T>::InvalidNumber)?
			},
		};
		
		let response = sp_std::str::from_utf8(&result).map_err(|_| Error::<T>::InvalidNumber)?;
		let balances = parser(response);

		match balances {
			Some(data) => {
				let mut total_balance: u128 = 0;
				// Sum up the balance
				for balance in data {
					total_balance = total_balance + balance;
				}
				Ok(total_balance)
			},
			None => Ok(0_u128),
		}
	}

}

#[allow(deprecated)]
impl<T: Trait> frame_support::unsigned::ValidateUnsigned for Module<T> {
	type Call = Call<T>;

	#[allow(deprecated)]
	fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {

		match call {
		Call::record_balance(account, block, btc_balance, eth_balance) => Ok(ValidTransaction {
			priority: 0,
			requires: vec![],
			provides: vec![(account, block, btc_balance, eth_balance).encode()],
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		}),

		Call::clear_claim(block) => Ok(ValidTransaction {
			priority: 0,
			requires: vec![],
			provides: vec![(block).encode()],
			longevity: TransactionLongevity::max_value(),
			propagate: true,
		}),
		_ => InvalidTransaction::Call.into()
		}
	}
}
