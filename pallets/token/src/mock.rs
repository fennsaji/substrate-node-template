use crate as pallet_token;
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU32, Everything, BuildGenesisConfig},
};
use frame_system::{self as system, EnsureSignedBy};
use pallet_balances;
use pallet_did::{
	self,
	types::{DIdentity, PrivateDid, AllowedRegionsVec},
};
use pallet_vc;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use system::{EnsureSigned, EnsureRoot};



type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Did: pallet_did::{Pallet, Call, Storage, Config<T>, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Token: pallet_token::{Pallet, Call, Storage, Event<T>},
		VC: pallet_vc::{Pallet, Call, Storage, Event<T>},
		ExtrinsicControls: pallet_extrinsic_controls::{ Pallet, Call, Storage, Event<T> }
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub static ExistentialDeposit: u64 = 0;
}

ord_parameter_types! {
	pub const ValidAccount: u64 = BOB_ACCOUNT_ID;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
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
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_extrinsic_controls::Config for Test {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event = Event;
	/// Sudo Origin
	type CallOrigin = EnsureRoot<Self::AccountId>;
}


impl pallet_did::Config for Test {
	type Event = Event;
	type ValidatorOrigin = EnsureSigned<Self::AccountId>;
	type MaxKeyChanges = ConstU32<16>;
	type OnDidUpdate = ();
	type VCResolution = VC;
}

impl pallet_balances::Config for Test {
	type Balance = u64;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = frame_system::Pallet<Test>;
	type MaxLocks = ();
	type MaxReserves = ConstU32<2>;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
	type DidResolution = Did;
	type ApproveOrigin = frame_system::EnsureRoot<u64>;
}

impl pallet_token::Config for Test {
	type Event = Event;
	type WithdrawOrigin = EnsureSigned<Self::AccountId>;
	type Currency = Balances;
	type DidResolution = Did;
	type VCResolution = VC;
	type ExtrinsicResolution = ExtrinsicControls;
}

impl pallet_vc::Config for Test {
	type Event = Event;
	type ApproveOrigin = EnsureSignedBy<ValidAccount, u64>;
	type IsCouncilMember = ();
	type IsValidator = ();
	type DidResolution = Did;
}

pub const ALICE: metamui_primitives::Did = *b"did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const BOB: metamui_primitives::Did = *b"did:ssid:bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const DAVE: metamui_primitives::Did = *b"did:ssid:dave\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const ALICE_ACCOUNT_ID: u64 = 2077282123132384724;
pub const BOB_ACCOUNT_ID: u64 = 7166219960988249998;
pub const DAVE_ACCOUNT_ID: u64 = 13620103657161844528;
pub const INITIAL_BALANCE: u64 = 0;
pub const BOB_SEED: [u8; 32] = [
	57, 143, 12, 40, 249, 136, 133, 224, 70, 51, 61, 74, 65, 193, 156, 238, 76, 55, 54, 138, 152,
	50, 198, 80, 47, 108, 253, 24, 46, 42, 239, 137,
];
pub const DAVE_SEED: [u8; 32] = [
	134, 128, 32, 174, 6, 135, 221, 167, 213, 117, 101, 9, 58, 105, 9, 2, 17, 68, 152, 69, 167,
	225, 20, 83, 97, 40, 0, 182, 99, 48, 114, 70,
];

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(BOB_ACCOUNT_ID, INITIAL_BALANCE), (ALICE_ACCOUNT_ID, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();
	let mut allowed_regions: AllowedRegionsVec = Default::default();
	allowed_regions.try_push(*b"ssid\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").expect("Vec Overflow");

	pallet_did::GenesisConfig::<Test> {
		initial_dids: vec![
			DIdentity::Private(PrivateDid {
				identifier: BOB,
				public_key: sr25519::Pair::from_seed(&BOB_SEED).public(),
				metadata: Default::default(),
			}),
			DIdentity::Private(PrivateDid {
				identifier: DAVE,
				public_key: sr25519::Pair::from_seed(&DAVE_SEED).public(),
				metadata: Default::default(),
			}),
		],
		allowed_regions,
		phantom: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let currency_code = convert_to_array::<8>("OTH".into());
	let token_name = convert_to_array::<16>("test".into());

	pallet_token::GenesisConfig::<Test> {
		currency_code,
		token_details: (token_name, 1000, 6),
		phantom: Default::default(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	t.into()
}
pub fn convert_to_array<const N: usize>(mut v: Vec<u8>) -> [u8; N] {
	if v.len() != N {
		for _ in v.len()..N {
			v.push(0);
		}
	}
	v.try_into().unwrap_or_else(|v: Vec<u8>| {
		panic!("Expected a Vec of length {} but it was {}", N, v.len())
	})
}
