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
use core::{convert::TryInto, fmt};
use frame_system::{
	ensure_signed, ensure_none,
	offchain::{CreateSignedTransaction, SubmitTransaction, Signer, AppCrypto, SendSignedTransaction,},
};
use frame_support::{
	debug, dispatch, decl_module, decl_storage, decl_event, decl_error,
	ensure, storage::IterableStorageMap, Parameter,
	traits::Get,
};
use sp_core::crypto::KeyTypeId;

use sp_runtime::{
	transaction_validity::{
		ValidTransaction, InvalidTransaction, TransactionValidity, TransactionSource, TransactionLongevity,
	},
	traits::{
		Zero, AtLeast32BitUnsigned, StaticLookup, Member, CheckedAdd, CheckedSub,
		MaybeSerializeDeserialize, Saturating, Bounded,
	},
};
use sp_runtime::offchain::{http, storage::StorageValueRef,};
use codec::{Codec, Encode, Decode, EncodeLike};

// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
//   with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::{Deserialize, Deserializer};

#[cfg(test)]
mod tests;
mod urls;
mod utils;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocw!");
const TOKEN_SERVER_URL: &str = "http://127.0.0.1:4000";
const DATA_SOURCE: [urls::DataSource; 3] = [urls::DataSource::ETHERSCAN, urls::DataSource::INFURA, urls::DataSource::BLOCKCHAIN];

/// Store all API tokens for offchain worker to send request to website
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct TokenInfo {
	/// API token for etherscan service
	#[serde(deserialize_with = "de_string_to_bytes")]
	etherscan: Vec<u8>,
	/// API token for infura service
	#[serde(deserialize_with = "de_string_to_bytes")]
	infura: Vec<u8>,
	/// API token for blockchain.info website
	#[serde(deserialize_with = "de_string_to_bytes")]
	blockchain: Vec<u8>,
}

/// Balances data embedded in etherscan response
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct EtherScanBalance {
	/// Ethereum account
	#[serde(deserialize_with = "de_string_to_bytes")]
	account: Vec<u8>,
	/// Eth balance
	#[serde(deserialize_with = "de_string_to_bytes")]
	balance: Vec<u8>,
}

/// Response data from etherscan
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct EtherScanResponse {
	/// Http response status
	#[serde(deserialize_with = "de_string_to_bytes")]
	status: Vec<u8>,
	/// Http response message
	#[serde(deserialize_with = "de_string_to_bytes")]
	message: Vec<u8>,
	/// Ethereum account and its balance
	result: Vec<EtherScanBalance>,
}

/// Balances data from Infura service
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct InfuraBalance {
	/// Json RPV version
	#[serde(deserialize_with = "de_string_to_bytes")]
	jsonrpc: Vec<u8>,
	/// Query ID
	id: u32,
	/// Balance data
	#[serde(deserialize_with = "de_string_to_bytes")]
	result: Vec<u8>,
}

/// Response from Infura
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct InfuraResponse {
	/// Response vector for several Ethreum account
	response: Vec<InfuraBalance>,
}

/// Deserialize string to Vec<u8>
pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

/// Implement Debug trait for print TokenInfo
impl fmt::Debug for TokenInfo {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ etherscan: {}, infura: {}, blockchain: {} }}",
			sp_std::str::from_utf8(&self.etherscan).map_err(|_| fmt::Error)?,
			sp_std::str::from_utf8(&self.infura).map_err(|_| fmt::Error)?,
			sp_std::str::from_utf8(&self.blockchain).map_err(|_| fmt::Error)?,
		)
	}
}


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
}

decl_storage! {
	trait Store for Module<T: Trait> as OffchainWorkerModule {
		/// Record how many claims from Litentry user
		TotalClaims get(fn total_claims): u64;
		/// Record the accounts send claims in latest block
		ClaimAccountSet get(fn query_account_set): map hasher(blake2_128_concat) T::AccountId => ();

		AccountBalance2 get(fn account_balance2): map hasher(blake2_128_concat) T::AccountId => (Option<u128>, Option<u128>);

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
		InvalidDataSource
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
		// fn submit_number_signed(origin, account: T::AccountId, blockChainType: urls::BlockChainType, balance: u128)-> dispatch::DispatchResult {

		fn submit_number_signed(origin, account: T::AccountId, block_number: T::BlockNumber, data_source: urls::DataSource, balance: u128)-> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			let sender = ensure_signed(origin)?;

			// Check data source
			Self::valid_data_source(data_source)?;

			// Check block number

			// Check ocw index
			let ocw_index = Self::get_ocw_index(&account);

			let mut round: u32 = 0;
			
			let mut index: u32 = 0;
			let data_source_index = urls::data_source_to_index(data_source);


			while (round < T::QueryTaskRedudancy::get() as u32) {

			}
			
			let blockchain_type = urls::data_source_to_block_chain_type(data_source);
			CommitAccountBalance::<T>::insert(&sender, &QueryKey{account, blockchain_type}, balance);

			Ok(())
		}

		// Trigger by offchain framework in each block
		fn offchain_worker(block_number: T::BlockNumber) {
			const TRANSACTION_TYPES: usize = 10;
			// let block_number_usize: usize = block_number.try_into();
			match block_number.try_into()
				.map_or(TRANSACTION_TYPES, |bn| bn % TRANSACTION_TYPES)
			{
				// The first block will trigger all offchain worker and clean the claims
				0 => Self::start(block_number),
				// The last block for aggregation and put balance queried into chain
				9 => {
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
				// Block 1 to block 8 reserved for offchain worker to query and submit result
				_ => {},
			};			
		}
	}
}

impl<T: Trait> Module<T> {
	

	fn valid_data_source(data_source: urls::DataSource) -> dispatch::DispatchResult {
		match data_source {
			urls::DataSource::INVALID => Err(<Error<T>>::InvalidDataSource.into()),
			_ => Ok(()),
		}
	}
	// Start new round of offchain worker
	fn start(block_number: T::BlockNumber) {
		let local_token = StorageValueRef::persistent(b"offchain-worker::token");
		
		match local_token.get::<TokenInfo>() {
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
				let _ = Self::get_token();
			},
		};
	}

	fn get_ocw_index(account: &T::AccountId) -> u32 {
		match Self::account_index(account) {
			Some(index_in_map) => index_in_map,
			_ => <AccountIndex::<T>>::iter().collect::<Vec<_>>().len() as u32,
		}
	}

	fn query(account_vec: &Vec<T::AccountId>, block_number: T::BlockNumber, info: &TokenInfo) {
		let offchain_worker_account = StorageValueRef::persistent(b"offchain-worker::account");

		match offchain_worker_account.get::<T::AccountId>() {
			Some(Some(account)) => {
				let ocw_account_index = Self::get_ocw_index(&account);

				let total_task = (account_vec.len() * DATA_SOURCE.len()) as u32;
				let mut task_index = 0_u32;

				// loop for each account
				for (index, account) in account_vec.iter().enumerate() {
					// loop for each source
					for (_, source) in DATA_SOURCE.iter().enumerate() {
						if task_index % total_task == ocw_account_index {
							match source {
								urls::DataSource::ETHERSCAN => {
									match Self::get_balance_from_etherscan(account, block_number, info) {
										Some(balance) => Self::offchain_signed_tx(account.clone(), block_number, urls::DataSource::ETHERSCAN, balance),
										None => ()
									}
								},
								urls::DataSource::INFURA => {
									match Self::get_balance_from_infura(account, block_number, info) {
										Some(balance) => Self::offchain_signed_tx(account.clone(), block_number, urls::DataSource::INFURA, balance),
										None => ()
									}
								},
								urls::DataSource::BLOCKCHAIN => {
									match Self::get_balance_from_blockchain_info(account, block_number, info) {
										Some(balance) => Self::offchain_signed_tx(account.clone(), block_number, urls::DataSource::BLOCKCHAIN, balance),
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
				Self::get_token();
			}
		}
	}

	fn get_balance_from_etherscan(account: &T::AccountId, block: T::BlockNumber, info: &TokenInfo) -> Option<u128> {
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
						&Self::parse_etherscan_balances).ok()
				},
				Err(_) => None,
			}
		}
	}

	fn get_balance_from_infura(account: &T::AccountId, block: T::BlockNumber, info: &TokenInfo) -> Option<u128> {
		
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
						&Self::parse_blockchain_info_balances).ok()
				},
				Err(_) => None,
			}
		}
	}

	fn get_balance_from_blockchain_info(account: &T::AccountId, block: T::BlockNumber, info: &TokenInfo) -> Option<u128> {
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
						&Self::parse_blockchain_info_balances).ok()
				},
				Err(_) => None,
			}
		}
	}

	fn offchain_signed_tx(account: T::AccountId, block_number: T::BlockNumber, data_source: urls::DataSource, balance: u128) {
		// We retrieve a signer and check if it is valid.
		//   Since this pallet only has one key in the keystore. We use `any_account()1 to
		//   retrieve it. If there are multiple keys and we want to pinpoint it, `with_filter()` can be chained,
		//   ref: https://substrate.dev/rustdocs/v2.0.0/frame_system/offchain/struct.Signer.html
		let signer = Signer::<T, T::AuthorityId>::any_account();

		// let tmp_account: T::AccountId = Default::default();
		// signer.for_any(|account| account == tmp_account);

		// Translating the current block number to number and submit it on-chain
		let number: u64 = block_number.try_into().unwrap_or(0) as u64;

		// <Test<T>>::insert(signer, block_number);

		// `result` is in the type of `Option<(Account<T>, Result<(), ()>)>`. It is:
		//   - `None`: no account is available for sending transaction
		//   - `Some((account, Ok(())))`: transaction is successfully sent
		//   - `Some((account, Err(())))`: error occured when sending the transaction
		let result = signer.send_signed_transaction(|_acct|
			// This is the on-chain function
			Call::submit_number_signed(account.clone(), block_number, data_source, balance)
		);

		// Display error if the signed tx fails.
		if let Some((acc, res)) = result {
			if res.is_err() {
				debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
				// return Err(<Error<T>>::OffchainSignedTxError);
			} else {
				debug::error!("successful: offchain_signed_tx: tx sent: {:?} index is {:?}", acc.id, acc.index);
			}

			// Record the account in local storage
			let account = StorageValueRef::persistent(b"offchain-worker::account");
			account.set(&acc.id);

			// Transaction is sent successfully
			// return Ok(());
		} else {
			debug::error!("No local account available");
		}

		// The case of `None`: no account is available for sending
		
		// Err(<Error<T>>::NoLocalAcctForSigning)
		// Ok(())
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

					link.extend(Self::address_to_string(each_account));
				}
				link.extend(get_req.postfix.as_bytes());
				link.extend(get_req.api_token.as_bytes());

				// Fetch json response via http get
				Self::fetch_json_http_get(&link[..]).map_err(|_| Error::<T>::InvalidNumber)?
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

					body.extend(Self::address_to_string(each_account));
				}
				body.extend(post_req.postfix.as_bytes());

				// Fetch json response via http post 
				Self::fetch_json_http_post(&link[..], &body[..]).map_err(|_| Error::<T>::InvalidNumber)?
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

	// Fetch json result from remote URL with get method
	fn fetch_json_http_get<'a>(remote_url: &'a [u8]) -> Result<Vec<u8>, &'static str> {
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

		let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

		let balance =
			core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?;

		Ok(balance.as_bytes().to_vec())
	}

	// Fetch json result from remote URL with post method
	fn fetch_json_http_post<'a>(remote_url: &'a [u8], body: &'a [u8]) -> Result<Vec<u8>, &'static str> {
		let remote_url_str = core::str::from_utf8(remote_url)
			.map_err(|_| "Error in converting remote_url to string")?;
	
		debug::info!("Offchain Worker post request url is {}.", remote_url_str);
		
		let pending = http::Request::post(remote_url_str, vec![body]).send()
			.map_err(|_| "Error in sending http POST request")?;
	
		let response = pending.wait()
			.map_err(|_| "Error in waiting http response back")?;
	
		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err("Non-200 status code returned from http request");
		}
	
		let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();
		
		let balance =
			core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?;
	
		Ok(balance.as_bytes().to_vec())
	}

	// Parse the balance from etherscan response
	fn parse_etherscan_balances(price_str: &str) -> Option<Vec<u128>> {
		// {
		// "status": "1",
		// "message": "OK",
		// "result":
		//   [
		//     {"account":"0x742d35Cc6634C0532925a3b844Bc454e4438f44e","balance":"3804372455842738500000001"},
		//     {"account":"0xBE0eB53F46cd790Cd13851d5EFf43D12404d33E8","balance":"2571179226430511381996287"}
		//   ]
		// }
		debug::info!("Offchain Worker response from etherscan is {:?}", price_str);

		let token_info: EtherScanResponse = serde_json::from_str(price_str).ok()?;
		let result: Vec<u128> = token_info.result.iter().map(|item| match Self::chars_to_u128(&item.balance.iter().map(|i| *i as char).collect()) {
			Ok(balance) => balance,
			Err(_) => 0_u128,
		}).collect();
		Some(result)
	}

	// Parse balances from blockchain info response
	fn parse_blockchain_info_balances(price_str: &str) -> Option<Vec<u128>>{
		// {
		//	"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":{"final_balance":6835384571,"n_tx":2635,"total_received":6835384571},
		//  "15EW3AMRm2yP6LEF5YKKLYwvphy3DmMqN6":{"final_balance":0,"n_tx":4,"total_received":310925609}
	  	// }
		let mut balance_vec: Vec<u128> = Vec::new();

		let value: serde_json::Value = serde_json::from_str(price_str).ok()?;

		match value {
			serde_json::Value::Object(map_data) => {
				for (_, v) in map_data.iter() {
					match v["final_balance"].as_u64() {
					Some(balance) =>  balance_vec.push(balance as u128),
					None => (),    
					}
				}
			},
			_ => (),
		};

		Some(balance_vec)
	}

	// Parse the balance from infura response
	fn parse_infura_balances(price_str: &str) -> Option<Vec<u128>> {
		//[
		//  {"jsonrpc":"2.0","id":1,"result":"0x4563918244f40000"},
		//  {"jsonrpc":"2.0","id":1,"result":"0xff"}
		//]

		let token_info: Vec<InfuraBalance> = serde_json::from_str(price_str).ok()?;
		let result: Vec<u128> = token_info.iter().map(|item| match Self::chars_to_u128(&item.result.iter().map(|i| *i as char).collect()) {
			Ok(balance) => balance,
			Err(_) => 0_u128,
		}).collect();
		Some(result)
	}

	// u128 number string to u128
	pub fn chars_to_u128(vec: &Vec<char>) -> Result<u128, &'static str> {
		// Check if the number string is decimal or hexadecimal (whether starting with 0x or not) 
		let base = if vec.len() >= 2 && vec[0] == '0' && vec[1] == 'x' {
			// This is a hexadecimal number
			16
		} else {
			// This is a decimal number
			10
		};

		let mut result: u128 = 0;
		for (i, item) in vec.iter().enumerate() {
			// Skip the 0 and x digit for hex. 
			// Using skip here instead of a new vec build to avoid an unnecessary copy operation
			if base == 16 && i < 2 {
				continue;
			}

			let n = item.to_digit(base);
			match n {
				Some(i) => {
					let i_64 = i as u128; 
					result = result * base as u128 + i_64;
					if result < i_64 {
						return Err("Wrong u128 balance data format");
					}
				},
				None => return Err("Wrong u128 balance data format"),
			}
		}
		return Ok(result)
	}

	// number byte to string byte
	fn u8_to_str_byte(a: u8) -> u8{
		if a < 10 {
			return a + 48 as u8;
		}
		else {
			return a + 87 as u8;
		}
	}

	// address to string bytes
	fn address_to_string(address: &[u8; 20]) -> Vec<u8> {

		let mut vec_result: Vec<u8> = Vec::new();
		for item in address {
			let a: u8 = item & 0x0F;
			let b: u8 = item >> 4;
			vec_result.push(Self::u8_to_str_byte(b));
			vec_result.push(Self::u8_to_str_byte(a));
		}
		return vec_result;
	}

	// Get the API tokens from local server
	fn get_token<'a>() -> Result<(), &'static str> {
	
		let pending = http::Request::get(TOKEN_SERVER_URL).send()
			.map_err(|_| "Error in sending http GET request")?;

		let response = pending.wait()
			.map_err(|_| "Error in waiting http response back")?;

		if response.code != 200 {
			debug::warn!("Unexpected status code: {}", response.code);
			return Err("Non-200 status code returned from http request");
		}

		let json_result: Vec<u8> = response.body().collect::<Vec<u8>>();

		let balance =
			core::str::from_utf8(&json_result).map_err(|_| "JSON result cannot convert to string")?;

		debug::info!("Token json from local server is {:?}.", &balance);

		let _ = Self::parse_store_tokens(balance);

		Ok(())
	}

	// Parse the balance from infura response
	fn parse_store_tokens(resp_str: &str) -> Result<(), Error<T>> {
		let token_info: TokenInfo = serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::InvalidNumber)?;

		let s_info = StorageValueRef::persistent(b"offchain-worker::token");

		s_info.set(&token_info);

		debug::info!("Token info get from local server is {:?}.", &token_info);

		Ok(())
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
