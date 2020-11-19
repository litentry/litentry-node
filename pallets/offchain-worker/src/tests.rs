use crate::*;
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types};
use codec::{alloc::sync::Arc, Decode};
use parking_lot::RwLock;
use sp_core::{
	offchain::{
		testing::{self, OffchainState, PoolState},
		OffchainExt, TransactionPoolExt,
	},
	sr25519::{self, Signature},
	testing::KeyStore,
	traits::KeystoreExt,
	H256,
};
use sp_io::TestExternalities;
use sp_runtime::{
	testing::{Header, TestXt},
	traits::{BlakeTwo256, IdentityLookup, Verify},
	Perbill,
};

use crate as OffchainWorker;
use account_linker;

impl_outer_origin! {
	pub enum Origin for TestRuntime where system = frame_system {}
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		frame_system<T>,
		OffchainWorker<T>,
		account_linker<T>,
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1_000_000;
	pub const MaximumBlockLength: u32 = 10 * 1_000_000;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

// The TestRuntime implements two pallet/frame traits: system, and simple_event
impl frame_system::Trait for TestRuntime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type Call = ();
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = sr25519::Public;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

pub type TestExtrinsic = TestXt<Call<TestRuntime>, ()>;

parameter_types! {
	pub const UnsignedPriority: u64 = 100;
}

impl Trait for TestRuntime {
	// type AuthorityId = crypto::TestAuthId;
	type Call = Call<TestRuntime>;
	type Event = TestEvent;
}

impl account_linker::Trait for TestRuntime {
	type Event = TestEvent;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for TestRuntime
where
	Call<TestRuntime>: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call<TestRuntime>,
		_public: <Signature as Verify>::Signer,
		_account: <TestRuntime as frame_system::Trait>::AccountId,
		index: <TestRuntime as frame_system::Trait>::Index,
	) -> Option<(
		Call<TestRuntime>,
		<TestExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		Some((call, (index, ())))
	}
}

impl frame_system::offchain::SigningTypes for TestRuntime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for TestRuntime
where
	Call<TestRuntime>: From<C>,
{
	type OverarchingCall = Call<TestRuntime>;
	type Extrinsic = TestExtrinsic;
}

pub type System = frame_system::Module<TestRuntime>;
// pub type OffchainWorker = Module<TestRuntime>;

pub struct ExternalityBuilder;

impl ExternalityBuilder {
	pub fn build() -> (
		TestExternalities,
		Arc<RwLock<PoolState>>,
		Arc<RwLock<OffchainState>>,
	) {
		const PHRASE: &str =
			"expire stage crawl shell boss any story swamp skull yellow bamboo copy";

		let (offchain, offchain_state) = testing::TestOffchainExt::new();
		let (pool, pool_state) = testing::TestTransactionPoolExt::new();
		let keystore = KeyStore::new();
		keystore
			.write()
			.sr25519_generate_new(KEY_TYPE, Some(&format!("{}/hunter1", PHRASE)))
			.unwrap();

		let storage = frame_system::GenesisConfig::default()
			.build_storage::<TestRuntime>()
			.unwrap();

		let mut t = TestExternalities::from(storage);
		t.register_extension(OffchainExt::new(offchain));
		t.register_extension(TransactionPoolExt::new(pool));
		t.register_extension(KeystoreExt(keystore));
		t.execute_with(|| System::set_block_number(1));
		(t, pool_state, offchain_state)
	}
}

#[test]
fn test_chars_to_u64() {
	let correct_balance = vec!['1', '2'];

	assert_eq!(Ok(12), <Module<TestRuntime>>::chars_to_u64(correct_balance));

	let correct_balance = vec!['a', '2'];
	assert_eq!(Err("Wrong u64 balance data format"), <Module<TestRuntime>>::chars_to_u64(correct_balance));
}

#[test]
fn test_parse_multi_balances() {
	let double_balances = r#"
	{
	"status": "1",
	"message": "OK",
	"result": 
		[
			{"account":"0x742d35Cc6634C0532925a3b844Bc454e4438f44e","balance":"12"},
			{"account":"0xBE0eB53F46cd790Cd13851d5EFf43D12404d33E8","balance":"21"}
		]
	}"#;
	assert_eq!(Some(vec![vec!['1', '2'], vec!['2', '1']]), <Module<TestRuntime>>::parse_multi_balances(double_balances));
}

#[test]
fn test_parse_balance() {

	let balance = r#"
	{
		"status": "1",
		"message": "OK",
		"result": "12"
	}"#;
	assert_eq!(Some(vec!['1', '2']), <Module<TestRuntime>>::parse_balance(balance));
}

// #[test]
// fn test_offchain_unsigned_tx() {
// 	let (mut t, pool_state, _offchain_state) = ExternalityBuilder::build();

// 	t.execute_with(|| {
// 		// when
// 		let num = 32;
// 		let _acct: <TestRuntime as frame_system::Trait>::AccountId = Default::default();
// 		<Module<TestRuntime>>::fetch_github_info().unwrap();
// 		// then
// 		let tx = pool_state.write().transactions.pop().unwrap();
// 		assert!(pool_state.read().transactions.is_empty());
// 		let tx = TestExtrinsic::decode(&mut &*tx).unwrap();
// 		assert_eq!(tx.signature, None);
// 		assert_eq!(tx.call, <TestRuntime as Trait>::Call::record_price(num));
// 	});
// }
