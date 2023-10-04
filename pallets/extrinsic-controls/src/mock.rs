use crate as pallet_extrinsic_controls;
use super::*;

use frame_system as system;
use frame_support::{
	traits::{ BuildGenesisConfig, ConstU16, ConstU32, ConstU64 },
};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use system::EnsureRoot;

pub const FIRST_PALLET_NAME: [u8;32] = [0;32];
pub const FIRST_FUNCTION_NAME: [u8;32] = [1;32];
pub const SECOND_PALLET_NAME: [u8; 32] = [2; 32];
pub const SECOND_FUNCTION_NAME: [u8; 32] = [3; 32];

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{ Pallet, Call, Config, Storage, Event<T> },
    ExtrinsicControls: pallet_extrinsic_controls::{ Pallet, Call, Storage, Event<T> }
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}

impl pallet_extrinsic_controls::Config for Test {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event = Event;
	/// Sudo Origin
	type CallOrigin = EnsureRoot<Self::AccountId>;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut o = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

		super::GenesisConfig::<Test> {
			whitelisted_extrinsics: vec![InitialExtrinsics {
					pallet_name: FIRST_PALLET_NAME,
					function_name: FIRST_FUNCTION_NAME
				}
			],
			restricted_extrinsics: vec![InitialExtrinsics {
				pallet_name: FIRST_PALLET_NAME,
				function_name: FIRST_FUNCTION_NAME
			}
		],
			phantom: Default::default(),
		}
			.assimilate_storage(&mut o)
			.unwrap();
  o.into()
}
