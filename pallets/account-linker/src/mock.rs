use frame_support::{
	impl_outer_origin, impl_outer_event, parameter_types, weights::Weight,
	traits::{OnFinalize, OnInitialize},
};
use frame_system as system;
use crate as account_linker;
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	Perbill,
	AccountId32,
	generic,
};

pub use crate::MAX_ETH_LINKS;

pub struct PanicPalletInfo;

impl frame_support::traits::PalletInfo for PanicPalletInfo {
	fn index<P: 'static>() -> Option<usize> {
		Some(0)
	}
	fn name<P: 'static>() -> Option<&'static str> {
		Some("")
	}
}

impl_outer_origin! {
	pub enum Origin for Test {}
}

impl_outer_event! {
	pub enum TestEvent for Test {
		system<T>,
		account_linker<T>,
	}
}

// Configure a mock runtime to test the pallet.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
	pub const BlockHashCount: u32 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u32;
	type BlockNumber = u32;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId32;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = generic::Header<Self::BlockNumber, BlakeTwo256>;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PanicPalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl crate::Config for Test {
	type Event = TestEvent;
}

pub type AccountLinker = crate::Module<Test>;
pub type AccountLinkerError = crate::Error<Test>;
pub type System = system::Module<Test>;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub fn run_to_block(n: u32) {
    while System::block_number() < n {
        AccountLinker::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        AccountLinker::on_initialize(System::block_number());
    }
}
