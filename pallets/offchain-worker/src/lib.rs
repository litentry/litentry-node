#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::{prelude::*};
use frame_system::{
	ensure_signed, ensure_none,
	offchain::{CreateSignedTransaction, SubmitTransaction},
};
use frame_support::{
	debug, dispatch, decl_module, decl_storage, decl_event, decl_error,
	ensure, storage::IterableStorageMap,
};
use sp_core::crypto::KeyTypeId;
use lite_json::{self, json::JsonValue};

use sp_runtime::{
	transaction_validity::{
		ValidTransaction, InvalidTransaction, TransactionValidity, TransactionSource, TransactionLongevity,
	},
};
use sp_runtime::offchain::http;
use codec::Encode;

#[cfg(test)]
mod tests;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ocw!");

mod urls {
	pub enum BlockChainType {
		ETH,
		BTC,
	}

	pub struct HttpGet<'a> {

		pub blockchain: BlockChainType,

		// URL affix
		pub prefix: &'a str,
		pub delimiter: &'a str,
		pub postfix: &'a str,
		pub api_token: &'a str,

	}

	pub struct HttpPost<'a> {

		pub blockchain: BlockChainType,

		// URL affix
		pub url_main: &'a str,
		pub api_token: &'a str,

		// Body affix
		pub prefix: &'a str,
		pub delimiter: &'a str,
		pub postfix: &'a str,
	}


	pub enum HttpRequest<'a> {
		GET(HttpGet<'a>),
		POST(HttpPost<'a>),
	}

	pub const ETHERSCAN_REQUEST: HttpGet = HttpGet {
		// https://api.etherscan.io/api?module=account&action=balancemulti&address=0x742d35Cc6634C0532925a3b844Bc454e4438f44e,0x742d35Cc6634C0532925a3b844Bc454e4438f44e&tag=latest&apikey=RF71W4Z2RDA7XQD6EN19NGB66C2QD9UPHB
		// The link is ETHER_SCAN_PREFIX + 1st Ethereum account + ETHER_SCAN_DELIMITER + 2nd Ethereum account + ... + ETHER_SCAN_POSTFIX + ETHER_SCAN_TOKEN

		blockchain: BlockChainType::ETH,
		prefix: "https://api-ropsten.etherscan.io/api?module=account&action=balancemulti&address=0x",
		delimiter: ",0x",
		postfix: "&tag=latest&apikey=", 
		api_token: "RF71W4Z2RDA7XQD6EN19NGB66C2QD9UPHB",
		//sample_acc: "742d35Cc6634C0532925a3b844Bc454e4438f44e",
		//sample_acc_add: "",
	};

	pub const BLOCKCHAIN_INFO_REQUEST: HttpGet = HttpGet {
		// https://blockchain.info/balance?active=1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa%7C15EW3AMRm2yP6LEF5YKKLYwvphy3DmMqN6
		// The link is composed of BLOCKCHAIN_INFO_PREFIX + 1st Bitcoin account + BLOCKCHAIN_INFO_DELIMITER + 2nd Bitcoin account + ... + BLOCKCHAIN_INFO_POSTFIX

		blockchain: BlockChainType::BTC,
		prefix: "https://blockchain.info/balance?active=",
		// The "%7C" is encoded of | delimiter in URL
		delimiter: "%7C",
		postfix: "",
		api_token: "",
		//sample_acc: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
		//sample_acc_add: "1XPTgDRhN8RFnzniWCddobD9iKZatrvH4",
	};

	pub const INFURA_REQUEST: HttpPost = HttpPost {
		// https://mainnet.infura.io/v3/aa0a6af5f94549928307febe80612a2a
		// Head: "Content-Type: application/json"
		// Body: 

		blockchain: BlockChainType::ETH,
		url_main: "https://mainnet.infura.io/v3/",
		api_token: "aa0a6af5f94549928307febe80612a2a",

		prefix: r#"{"jsonrpc":"2.0","method":"eth_accounts","params":["0x"#,
		delimiter: "",
		postfix: r#"","latest"],"id":1}"#,
		//sample_acc: "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
		//sample_acc_add: "1XPTgDRhN8RFnzniWCddobD9iKZatrvH4",
	};
}


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

pub trait Trait: frame_system::Trait + account_linker::Trait + CreateSignedTransaction<Call<Self>> {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Call: From<Call<Self>>;
}

decl_storage! {
	trait Store for Module<T: Trait> as OffchainWorkerModule {
		/// Record how many claims from Litentry user
		TotalClaims get(fn total_claims): u64;
		/// Record the accounts send claims in latest block
		ClaimAccountSet get(fn query_account_set): map hasher(blake2_128_concat) T::AccountId => ();
		/// Record account's btc and ethereum balance
		AccountBalance get(fn account_balance): map hasher(blake2_128_concat) T::AccountId => (u128, u128);
	}
}

decl_event!(
	pub enum Event<T> where	AccountId = <T as frame_system::Trait>::AccountId, 
					BlockNumber = <T as frame_system::Trait>::BlockNumber, {
		/// Event for account and its ethereum balance
		BalanceGot(AccountId, BlockNumber, u128, u128),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Error number parsing.
		InvalidNumber
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
		pub fn asset_claim(origin,) -> dispatch::DispatchResult {
			let account = ensure_signed(origin)?;

			ensure!(!ClaimAccountSet::<T>::contains_key(&account), Error::<T>::InvalidNumber);

			<ClaimAccountSet<T>>::insert(&account, ());
			Ok(())
		}

		#[weight = 10_000]
		pub fn clear_claim(origin, block: T::BlockNumber)-> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			ensure_none(origin)?;
			// Remove all claimed accounts
			<ClaimAccountSet::<T>>::remove_all();

			Ok(())
		}

		#[weight = 10_000]
		pub fn record_balance(
			origin,
			account: T::AccountId,
			block: T::BlockNumber,
			btc_balance: u128,
			eth_balance: u128,
		) -> dispatch::DispatchResult {
			// Ensuring this is an unsigned tx
			ensure_none(origin)?;
			// Record the total claims processed
			TotalClaims::put(Self::total_claims() + 1);
			// Set balance 
			<AccountBalance<T>>::insert(account.clone(), (btc_balance, eth_balance));
			// Spit out an event and Add to storage
			Self::deposit_event(RawEvent::BalanceGot(account, block, btc_balance, eth_balance));

			Ok(())
		}

		// Trigger by offchain framework in each block
		fn offchain_worker(block: T::BlockNumber) {
			// Get the all accounts who ask for asset claims
			let accounts: Vec<T::AccountId> = <ClaimAccountSet::<T>>::iter().map(|(k, _)| k).collect();
			// Remove all claimed accounts
			// TODO seems it doesn't work here to update ClaimAccountSet
			// <ClaimAccountSet::<T>>::remove_all();

			// Try to remove claims via tx
			if accounts.len() > 0 {
				let call = Call::clear_claim(block);
				let _ = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into())
				.map_err(|_| {
					debug::error!("Failed in offchain_unsigned_tx");
					<Error<T>>::InvalidNumber
				});
			}

			match Self::update(accounts, block) {
				Ok(()) => debug::info!("Offchain Worker end successfully."),
				Err(err) => debug::info!("Offchain Worker end with err {:?}.", err),
			}
		}
	}
}

impl<T: Trait> Module<T> {
	// Fetch all claimed accounts
	fn update(account_vec: Vec<T::AccountId>, block: T::BlockNumber) ->  Result<(), Error<T>> {
		for (_, account) in account_vec.iter().enumerate() {
			let eth_balance = Self::fetch_balances(<account_linker::EthereumLink<T>>::get(account), urls::HttpRequest::GET(urls::ETHERSCAN_REQUEST), &Self::parse_etherscan_balances);
			let btc_balance = Self::fetch_balances(Vec::new(), urls::HttpRequest::GET(urls::BLOCKCHAIN_INFO_REQUEST), &Self::parse_blockchain_info_balances);

			match (btc_balance, eth_balance) {
				(Ok(btc), Ok(eth)) => {
					let call = Call::record_balance(account.clone(), block, btc, eth);
					let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into());
					if result.is_err() {
						debug::info!("Offchain Worker failed to submit record balance transaction");
					}
				},
				(Ok(btc), _) => {
					let call = Call::record_balance(account.clone(), block, btc, 0_u128);
					let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into());
					if result.is_err() {
						debug::info!("Offchain Worker failed to submit record balance transaction");
					}
				},
				(_, Ok(eth)) => {
					let call = Call::record_balance(account.clone(), block, 0_u128, eth);
					let result = SubmitTransaction::<T, Call<T>>::submit_unsigned_transaction(call.into());
					if result.is_err() {
						debug::info!("Offchain Worker failed to submit record balance transaction");
					}
				},
				_ => (),
			}
		}
		Ok(())
	}

	// Generic function to fetch balance for specific link type
	fn fetch_balances(wallet_accounts: Vec<[u8; 20]>, request: urls::HttpRequest, 
		parser: &dyn Fn(&str) -> Option<Vec<u128>>) -> Result<u128, Error<T>> {
		// TODO add match expression later to distinguish eth and btc
		//      generic array would be the best choice here, however seems it's still not completed in rust

		// Return if no account linked
		if wallet_accounts.len() == 0 {
			return Ok(0_u128)
		}

		let result: Vec<u8> = match request {
			urls::HttpRequest::GET(affix_set) => {
				// Compose the web request url 
				let mut link: Vec<u8> = Vec::new();

				link.extend(affix_set.prefix.as_bytes());

				for (i, each_account) in wallet_accounts.iter().enumerate() {
					// Append delimiter if there are more than one accounts in the account_vec
					if i >=1 {
						link.extend(affix_set.delimiter.as_bytes());
					};

					link.extend(Self::address_to_string(each_account));
				}
				link.extend(affix_set.postfix.as_bytes());
				link.extend(affix_set.api_token.as_bytes());

				// Get the json
				Self::fetch_json(&link[..]).map_err(|_| Error::<T>::InvalidNumber)?
			},
			// TODO finish POST
			_ => Vec::new(),

		};
		
		let response = sp_std::str::from_utf8(&result).map_err(|_| Error::<T>::InvalidNumber)?;
		let balances = parser(response);

		match balances {
			Some(data) => {
				let mut total_balance: u128 = 0;
				for balance in data {
					total_balance = total_balance + balance;
				}
				Ok(total_balance)
			},
			None => Ok(0_u128),
		}
	}

	// Fetch json result from remote URL
	fn fetch_json<'a>(remote_url: &'a [u8]) -> Result<Vec<u8>, &'static str> {
		let remote_url_str = core::str::from_utf8(remote_url)
			.map_err(|_| "Error in converting remote_url to string")?;
	
		debug::info!("Offchain Worker request url is {}.", remote_url_str);
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
		let val = lite_json::parse_json(price_str);
		let mut balance_vec: Vec<u128> = Vec::new();

		val.ok().and_then(|v| { 
				match v {
				JsonValue::Object(obj) => {
					obj.into_iter()
						.find(|(k, _)|  {
							let mut chars = "result".chars(); 
							k.iter().all(|k| Some(*k) == chars.next())
					})
					.and_then(|v|  { 
						match v.1 {
							JsonValue::Array(res_array) => {
								for element in res_array {
									match element {
										JsonValue::Object(element_vec) => {
											for pair in element_vec {			
												let mut balance_chars = "balance".chars();		
												if pair.0.iter().all(|k| Some(*k) == balance_chars.next()) {
													match pair.1 {
														JsonValue::String(balance) => {
															match Self::chars_to_u128(balance){
																Ok(b) => balance_vec.push(b),
																// TODO Proper error handling here would be necessary later
																Err(_) => return None,
															}
														},
														_ => (),
													}
												}
											};
										},
										_ => ()
									}
								}
								Some(balance_vec)
							},
							_ => None,
						}
					})
				},
				_ => None
			}
		})
	}

	// Parse balances from blockchain info response
	fn parse_blockchain_info_balances(price_str: &str) -> Option<Vec<u128>>{
		// {
		//	"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":{"final_balance":6835384571,"n_tx":2635,"total_received":6835384571},
		//  "15EW3AMRm2yP6LEF5YKKLYwvphy3DmMqN6":{"final_balance":0,"n_tx":4,"total_received":310925609}
	  // }
		let val = lite_json::parse_json(price_str);
		let mut balance_vec: Vec<u128> = Vec::new();

		val.ok().and_then(|v| match v {
			JsonValue::Object(obj) => {
				for each in obj {
					match each.1 {
						JsonValue::Object(balance_pairs) => {
							balance_vec.push(balance_pairs.into_iter().find(|(k, _)|{
								let mut matching_chars = "final_balance".chars();
								k.iter().all(|k| Some(*k) == matching_chars.next())
							})
							.and_then(|v| match v.1 {
								JsonValue::Number(balance) => {
									if balance.fraction != 0 || balance.fraction_length != 0 || balance.integer < 0 {
										// Number received with fraction or negative, abort this request
										None
									} else {
										Some( 
											if balance.exponent == 0 {
												balance.integer as u128 
											} else {
												balance.integer as u128 * 10u128.pow(balance.exponent as u32)
											})
									}
								},
								_ => None,
							})?);
						},
						_ => ()
					}
				};
				Some(balance_vec)
			},
			_ => None
		})
	}

	// u128 number string to u128
	pub fn chars_to_u128(vec: Vec<char>) -> Result<u128, &'static str> {
		let mut result: u128 = 0;
		for item in vec {
			let n = item.to_digit(10);
			match n {
				Some(i) => {
					let i_64 = i as u128; 
					result = result * 10 + i_64;
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