//use crate::*;
//use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
//use codec::{alloc::sync::Arc};
//use parking_lot::RwLock;
//use sp_core::{
//	offchain::{
//		testing::{self, OffchainState, PoolState},
//		OffchainExt, TransactionPoolExt,
//	},
//	sr25519::{self, Signature},
//	H256,
//};
//use sp_io::TestExternalities;
//use sp_runtime::{
//	testing::{Header, TestXt},
//	traits::{BlakeTwo256, IdentityLookup, Verify},
//};
//
//use crate as offchain_worker;
//use account_linker;
//use utils;
//use urls;
//
//type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
//type Block = frame_system::mocking::MockBlock<TestRuntime>;
//
//// Configure a mock runtime to test the pallet.
//frame_support::construct_runtime!(
//	pub enum TestRuntime where
//		Block = Block,
//		NodeBlock = Block,
//		UncheckedExtrinsic = UncheckedExtrinsic,
//	{
//		System: frame_system::{Module, Call, Storage, Event<T>},
//		OffchainWorker: offchain_worker::{Module, Call, Storage, Event<T>},
//	}
//);
//parameter_types! {
//	pub const BlockHashCount: u64 = 250;
//	pub const SS58Prefix: u8 = 42;
//}
//
//// The TestRuntime implements two pallet/frame traits: system, and simple_event
//impl frame_system::Config for TestRuntime {
//	type BaseCallFilter = ();
//	type Origin = Origin;
//	type BlockWeights = ();
//	type BlockLength = ();
//	type DbWeight = ();
//	type Index = u64;
//	type Call = Call;
//	type BlockNumber = u64;
//	type Hash = H256;
//	type Hashing = BlakeTwo256;
//	type AccountId = sr25519::Public;
//	type Lookup = IdentityLookup<Self::AccountId>;
//	type Header = Header;
//	type Event = Event;
//	type BlockHashCount = BlockHashCount;
//	type Version = ();
//	type PalletInfo = PalletInfo;
//	type AccountData = pallet_balances::AccountData<u128>;
//	type OnNewAccount = ();
//	type OnKilledAccount = ();
//	type SystemWeightInfo = ();
//	type SS58Prefix = SS58Prefix;
//}
//
//parameter_types! {
//	pub const ExistentialDeposit: u128 = 500;
//}
//
//impl pallet_balances::Config for TestRuntime {
//	type MaxLocks = ();
//	/// The type for recording an account's balance.
//	type Balance = u128;
//	/// The ubiquitous event type.
//	type Event = Event;
//	type DustRemoval = ();
//	type ExistentialDeposit = ExistentialDeposit;
//	type AccountStore = System;
//	type WeightInfo = ();
//}
//
//pub type TestExtrinsic = TestXt<offchain_worker::Call<TestRuntime>, ()>;
//
//parameter_types! {
//	pub const UnsignedPriority: u64 = 100;
//}
//
//parameter_types! {
//	pub const QueryTaskRedundancy: u32 = 3;
//	pub const QuerySessionLength: u32 = 5;
//	pub const OcwQueryReward: u128 = 1;
//}
//
//impl offchain_worker::Config for TestRuntime {
//	type AuthorityId = crypto::TestAuthId;
//	type Call = offchain_worker::Call<TestRuntime>;
//	type Event = Event;
//	type Balance = u128;
//	type QueryTaskRedundancy = QueryTaskRedundancy;
//	type QuerySessionLength = QuerySessionLength;
//	type Currency = Balances;
//	type Reward = ();
//	type OcwQueryReward = OcwQueryReward;
//}
//
//impl account_linker::Config for TestRuntime {
//	type Event = Event;
//}
//
//impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
//where
//	offchain_worker::Call<TestRuntime>: From<LocalCall>,
//{
//	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
//		call: offchain_worker::Call<TestRuntime>,
//		_public: <Signature as Verify>::Signer,
//		_account: <TestRuntime as frame_system::Config>::AccountId,
//		index: <TestRuntime as frame_system::Config>::Index,
//	) -> Option<(
//		offchain_worker::Call<TestRuntime>,
//		<TestExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
//	)> {
//		Some((call, (index, ())))
//	}
//}
//
//impl frame_system::offchain::SigningTypes for TestRuntime {
//	type Public = <Signature as Verify>::Signer;
//	type Signature = Signature;
//}
//
//impl<C> frame_system::offchain::SendTransactionTypes<C> for TestRuntime
//where
//	offchain_worker::Call<TestRuntime>: From<C>,
//{
//	type OverarchingCall = offchain_worker::Call<TestRuntime>;
//	type Extrinsic = TestExtrinsic;
//}
//
//pub type Balances = pallet_balances::Module<TestRuntime>;
//// pub type OffchainWorker = Module<TestRuntime>;
//
//// FIXME
//// Han: testing module has been removed from sp-core since 3.0.0, thus this part cannot be built any more.
////      as it's not used right now, I will comment out it temporarily
////pub struct ExternalityBuilder;
////
////impl ExternalityBuilder {
////	pub fn build() -> (
////		TestExternalities,
////		Arc<RwLock<PoolState>>,
////		Arc<RwLock<OffchainState>>,
////	) {
////		const PHRASE: &str =
////			"expire stage crawl shell boss any story swamp skull yellow bamboo copy";
////
////		let (offchain, offchain_state) = testing::TestOffchainExt::new();
////		let (pool, pool_state) = testing::TestTransactionPoolExt::new();
////		let keystore = KeyStore::new();
////		keystore
////			.write()
////			.sr25519_generate_new(KEY_TYPE, Some(&format!("{}/hunter1", PHRASE)))
////			.unwrap();
////
////		let storage = frame_system::GenesisConfig::default()
////			.build_storage::<TestRuntime>()
////			.unwrap();
////
////		let mut t = TestExternalities::from(storage);
////		t.register_extension(OffchainExt::new(offchain));
////		t.register_extension(TransactionPoolExt::new(pool));
////		t.register_extension(KeystoreExt(keystore));
////		t.execute_with(|| System::set_block_number(1));
////		(t, pool_state, offchain_state)
////	}
////}
//
//#[test]
//fn test_chars_to_u128() {
//	let correct_balance = vec!['5', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0', '0'];
//	assert_eq!(Ok(500000000000000000_u128), utils::chars_to_u128(&correct_balance));
//
//	let correct_balance = vec!['a', '2'];
//	assert_eq!(Err("Wrong u128 balance data format"), utils::chars_to_u128(&correct_balance));
//
//	let correct_balance = vec!['0', 'x', 'f', 'e'];
//	assert_eq!(Ok(254_u128), utils::chars_to_u128(&correct_balance));
//
//	// Corner case check
//	let correct_balance = vec!['0', 'x'];
//	assert_eq!(Ok(0_u128), utils::chars_to_u128(&correct_balance));
//}
//
//#[test]
////fn test_fetch_balances() {
////	let test_account = "4d88dc5D528A33E4b8bE579e9476715F60060582".as_bytes();
////	let mut test_account_byte_array = [0u8; 20];
////	test_account_byte_array.copy_from_slice(&test_account[0..20]);
////
////	let mut accounts: Vec<[u8; 20]> = Vec::new();
////	accounts.push(test_account_byte_array);
////
////	sp_io::TestExternalities::default().execute_with(|| {
////		match <Module<TestRuntime>>::fetch_balances(accounts, urls::HttpRequest::GET(urls::ETHERSCAN_REQUEST), &<Module<TestRuntime>>::parse_etherscan_balances) {
////			Ok(b) => assert_eq!(500000000000000000_u128, b),
////			Err(_) => panic!("Error occurs in test_fetch_balance!!"),
////		};
////	});
////}
//
//#[test]
//fn test_parse_etherscan_balances() {
//	let double_balances = r#"
//	{
//	"status": "1",
//	"message": "OK",
//	"result":
//		[
//			{"account":"0x742d35Cc6634C0532925a3b844Bc454e4438f44e","balance":"12"},
//			{"account":"0xBE0eB53F46cd790Cd13851d5EFf43D12404d33E8","balance":"21"}
//		]
//	}"#;
//	assert_eq!(Some(vec![12, 21]), urls::parse_etherscan_balances(double_balances));
//}
//
//#[test]
//fn test_parse_etherscan_balances_2() {
//	let double_balances = r#"
//	{
//	"status": "1",
//	"message": "OK",
//	"result":
//		[
//			{"account":"0x742d35Cc6634C0532925a3b844Bc454e4438f44e","balance":"12"},
//			{"account":"0xBE0eB53F46cd790Cd13851d5EFf43D12404d33E8","balance":"21"}
//		]
//	}"#;
//
//	let token_info: urls::EtherScanResponse = serde_json::from_str(&double_balances).unwrap();
//	assert_eq!(token_info.status, "1".as_bytes().to_vec());
//	assert_eq!(token_info.result[0].balance, "12".as_bytes().to_vec());
//}
//
//#[test]
//fn test_parse_blockchain_info_balances() {
//	let double_balances = r#"
//	{
//		"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":{"final_balance":30,"n_tx":2635,"total_received":6835384571},
//		"15EW3AMRm2yP6LEF5YKKLYwvphy3DmMqN6":{"final_balance":1220,"n_tx":4,"total_received":310925609}
//	}"#;
//	let result = urls::parse_blockchain_info_balances(double_balances);
//	assert_eq!(true, (Some(vec![30, 1220]) == result || Some(vec![1220, 30]) == result));
//
//	// Test case should fail because fraction of the first balance value is non zero
//	let double_balances = r#"
//	{
//		"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":{"final_balance":30.5,"n_tx":2635,"total_received":6835384571},
//		"15EW3AMRm2yP6LEF5YKKLYwvphy3DmMqN6":{"final_balance":1220,"n_tx":4,"total_received":310925609}
//	}"#;
//	assert_eq!(Some(vec![1220]), urls::parse_blockchain_info_balances(double_balances));
//
//	// Test case should fail because first balance value is negative
//	let double_balances = r#"
//	{
//		"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa":{"final_balance":-30,"n_tx":2635,"total_received":6835384571},
//		"15EW3AMRm2yP6LEF5YKKLYwvphy3DmMqN6":{"final_balance":1220,"n_tx":4,"total_received":310925609}
//	}"#;
//	assert_eq!(Some(vec![1220]), urls::parse_blockchain_info_balances(double_balances));
//}
//
//#[test]
//fn test_parse_infura_balances() {
//	let double_balances = r#"
//	[
//		{"jsonrpc":"2.0","id":1,"result":"0x4563918244f40000"},
//		{"jsonrpc":"2.0","id":1,"result":"0xff"}
//	]
//	"#;
//
//	assert_eq!(Some(vec![5000000000000000000, 255]), urls::parse_infura_balances(double_balances));
//}
//
//#[test]
//fn test_parse_infura_balances_2() {
//	let double_balances = r#"
//	[
//		{"jsonrpc":"2.0","id":1,"result":"0x4563918244f40000"},
//		{"jsonrpc":"2.0","id":1,"result":"0xff"}
//	]
//	"#;
//	let token_info: Vec<urls::InfuraBalance> = serde_json::from_str(double_balances).unwrap();
//	assert_eq!(token_info[0].id, 1);
//
//}

// #[test]
// fn test_offchain_unsigned_tx() {
// 	let (mut t, pool_state, _offchain_state) = ExternalityBuilder::build();

// 	t.execute_with(|| {
// 		// when
// 		let num = 32;
// 		let _acct: <TestRuntime as frame_system::Config>::AccountId = Default::default();
// 		<Module<TestRuntime>>::fetch_github_info().unwrap();
// 		// then
// 		let tx = pool_state.write().transactions.pop().unwrap();
// 		assert!(pool_state.read().transactions.is_empty());
// 		let tx = TestExtrinsic::decode(&mut &*tx).unwrap();
// 		assert_eq!(tx.signature, None);
// 		assert_eq!(tx.call, <TestRuntime as Config>::Call::record_price(num));
// 	});
// }
