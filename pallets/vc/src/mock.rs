use crate::{self as verified_credentials, Config};
use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{ConstU32, Everything, BuildGenesisConfig},
};
use frame_system::{EnsureSigned, EnsureSignedBy};
use pallet_did::types::{DIdentity, PrivateDid, AllowedRegionsVec};
use sp_core::{sr25519, sr25519::Signature, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use sp_std::convert::TryInto;
use super::types::InitialVCs;


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
type BlockNumber = u32;



ord_parameter_types! {
	pub const ValidAccount: u64 = BOB_ACCOUNT_ID;
}

const MILLISECS_PER_BLOCK: u64 = 5000;
const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
const HOURS: BlockNumber = MINUTES * 60;
const DAYS: BlockNumber = HOURS * 24;
// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		VC: verified_credentials::{Pallet, Call, Storage, Event<T>},
		ValidatorSet: pallet_validator_set::{Pallet, Call, Storage, Event<T>, Config<T>},
		Did: pallet_did::{Pallet, Call, Storage, Config<T>, Event<T>},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		ValidatorCommittee: pallet_validator_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
	pub const CouncilMotionDuration: BlockNumber = 5 * MINUTES;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub const MaxValidators : u32 = 20;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
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
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl Config for Test {
	type Event = Event;
	type ApproveOrigin = EnsureSignedBy<ValidAccount, u64>;
	type IsCouncilMember = Council;
	type IsValidator = ValidatorCommittee;
	type DidResolution = Did;
}

ord_parameter_types! {
	pub const One: u64 = 1;
	pub const Two: u64 = 2;
	pub const Three: u64 = 3;
	pub const Four: u64 = 4;
	pub const Five: u64 = 5;
	pub const Six: u64 = 6;
}

impl pallet_validator_set::Config for Test {
	type Event = Event;
	type AddOrigin = EnsureSignedBy<One, u64>;
	type RemoveOrigin = EnsureSignedBy<Two, u64>;
	type SwapOrigin = EnsureSignedBy<Three, u64>;
	type ResetOrigin = EnsureSignedBy<Four, u64>;
	type PrimeOrigin = EnsureSignedBy<Five, u64>;
	type MembershipInitialized = ValidatorCommittee;
	type MembershipChanged = ValidatorCommittee;
	type MaxMembers = MaxValidators;
	type DidResolution = Did;
	type WeightInfo = ();
}

impl pallet_did::Config for Test {
	type Event = Event;
	type ValidatorOrigin = EnsureSigned<Self::AccountId>;
	type MaxKeyChanges = ConstU32<16>;
	type OnDidUpdate = ();
	type VCResolution = VC;
}

pub type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type DidResolution = Did;
	type WeightInfo = ();
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

pub type ValidatorCollective = pallet_validator_collective::Instance1;
impl pallet_validator_collective::Config<ValidatorCollective> for Test {
	type Event = Event;
	type Origin = Origin;
	type Proposal = Call;
	type DidResolution = Did;
	type CallOrigin = EnsureSignedBy<Six, u64>;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = ();
}

pub const VALIDATOR_ACCOUNT: u64 = 0;
pub const VALIDATOR_DID: [u8; 32] = *b"did:ssid:Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const VALIDATOR_PUBKEY: sr25519::Public = sr25519::Public([0; 32]);
pub const NON_VALIDATOR_ACCOUNT: u64 = 2;
pub const ALICE: metamui_primitives::Did = *b"did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const BOB: metamui_primitives::Did = *b"did:ssid:bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const DAVE: metamui_primitives::Did = *b"did:ssid:dave\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const EVE: metamui_primitives::Did = *b"did:ssid:eve\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const DAVE_ACCOUNT_ID: u64 = 13620103657161844528;
pub const BOB_ACCOUNT_ID: u64 = 7166219960988249998;
pub const BOB_SEED: [u8; 32] = [
	57, 143, 12, 40, 249, 136, 133, 224, 70, 51, 61, 74, 65, 193, 156, 238, 76, 55, 54, 138, 152,
	50, 198, 80, 47, 108, 253, 24, 46, 42, 239, 137,
];
pub const DAVE_SEED: [u8; 32] = [
	134, 128, 32, 174, 6, 135, 221, 167, 213, 117, 101, 9, 58, 105, 9, 2, 17, 68, 152, 69, 167,
	225, 20, 83, 97, 40, 0, 182, 99, 48, 114, 70,
];
pub const EVE_SEED: [u8; 32] = [
	120, 106, 208, 226, 223, 69, 111, 228, 61, 209, 249, 30, 188, 162, 46, 35, 91, 193, 98, 224,
	187, 141, 83, 198, 51, 232, 200, 91, 42, 246, 139, 122,
];

pub const OWNER_DID_ONE: metamui_primitives::Did = *b"did:yidindji:owner\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const VC_ID_ONE: metamui_primitives::VCid = [206, 108, 151, 37, 75, 81, 230, 160, 51, 34, 198, 121, 30, 230, 118, 126, 112, 65, 192, 5, 120, 254, 204, 150, 143, 234, 181, 223, 254, 61, 53, 135];
pub const SIGNATURE_ONE: Signature = Signature([0, 185, 36, 124, 52, 204, 191, 241, 180, 36, 123, 164, 82, 196, 170, 174, 11, 151, 247, 185, 161, 43, 45, 0, 172, 41, 128, 219, 184, 248, 175, 59, 65, 140, 145, 148, 102, 125, 105, 172, 215, 204, 211, 41, 177, 228, 97, 27, 255, 60, 238, 88, 98, 249, 245, 105, 97, 199, 46, 208, 233, 12, 49, 133]);

pub const OWNER_DID_TWO: metamui_primitives::Did = *b"did:sgd:owner\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const VC_ID_TWO: metamui_primitives::VCid = [174,86,83,21,104,37,88,1,20,91,170,172,150,162,209,236,73,248,37,87,123,184,79,96,149,233,109,225,107,130,87,183];
pub const SIGNATURE_TWO: Signature = Signature([34, 232, 242, 42, 255, 132, 251, 62, 57, 28, 69, 22, 10, 59, 85, 141, 94, 54, 11, 42, 199, 117, 222, 186, 93, 82, 250, 171, 244, 90, 74, 6, 38, 134, 209, 64, 55, 207, 230, 74, 26, 44, 54, 199, 120, 59, 232, 23, 64, 175, 141, 151, 210, 172, 189, 133, 106, 188, 208, 155, 174, 133, 16, 136]);

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut o = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	super::GenesisConfig::<Test> { 
		initial_vcs: vec![
			InitialVCs{
				vc_id: VC_ID_ONE,
				// Owner Did Used = did:yidindji:owner (OWNER_DID_ONE)
				vc_hex: [56,217,37,135,71,49,120,122,190,83,37,217,205,40,0,52,178,149,184,192,0,99,68,200,182,59,108,5,64,254,94,17,100,105,100,58,121,105,100,105,110,100,106,105,58,111,119,110,101,114,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,100,105,100,58,121,105,100,105,110,100,106,105,58,111,119,110,101,114,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,0,185,36,124,52,204,191,241,180,36,123,164,82,196,170,174,11,151,247,185,161,43,45,0,172,41,128,219,184,248,175,59,65,140,145,148,102,125,105,172,215,204,211,41,177,228,97,27,255,60,238,88,98,249,245,105,97,199,46,208,233,12,49,133,0,0,0,89,105,100,105,110,100,106,32,84,111,107,101,110,0,0,0,0,202,154,59,0,0,0,0,0,0,0,0,0,0,0,0,6,83,89,75,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0].to_vec()
			},
			InitialVCs{
				vc_id: VC_ID_TWO,
				// Owner Did Used = did:sgd:owner (OWNER_DID_TWO)
				vc_hex: [81,101,227,103,100,177,75,217,220,78,57,1,157,174,26,105,96,189,0,50,125,154,235,208,155,243,7,104,142,201,236,166,100,105,100,58,115,103,100,58,111,119,110,101,114,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,100,105,100,58,115,103,100,58,111,119,110,101,114,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4,34,232,242,42,255,132,251,62,57,28,69,22,10,59,85,141,94,54,11,42,199,117,222,186,93,82,250,171,244,90,74,6,38,134,209,64,55,207,230,74,26,44,54,199,120,59,232,23,64,175,141,151,210,172,189,133,106,188,208,155,174,133,16,136,0,0,3,150,183,92,102,90,52,61,127,116,236,71,33,122,121,67,98,251,95,189,26,147,119,146,161,136,47,97,227,122,191,92,206,83,71,68,0,0,0,0,0,208,7,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0].to_vec()
			},
		],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut o)
	.unwrap();

	pallet_validator_set::GenesisConfig::<Test> {
		members: frame_support::bounded_vec![BOB, DAVE],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut o)
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
			DIdentity::Private(PrivateDid {
				identifier: VALIDATOR_DID,
				public_key: VALIDATOR_PUBKEY,
				metadata: Default::default(),
			}),
			DIdentity::Private(PrivateDid {
				identifier: EVE,
				public_key: sr25519::Pair::from_seed(&EVE_SEED).public(),
				metadata: Default::default(),
			}),
		],
		allowed_regions,
		phantom: Default::default(),
	}
	.assimilate_storage(&mut o)
	.unwrap();

	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance1> {
		members: vec![ALICE, BOB, DAVE],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut o)
	.unwrap();
	o.into()
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
