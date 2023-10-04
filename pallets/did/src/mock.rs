use crate as pallet_did;
use metamui_primitives::traits::IsValidator;
use pallet_vc;
use metamui_primitives::{Did as Identifier, VCid, types::{ VCType, CompanyName, RegistrationNumber, VC, DidRegion }};
use crate::types::*;
use frame_support::{
	ord_parameter_types, bounded_vec,
	traits::{ BuildGenesisConfig, ConstU16, ConstU32, ConstU64, OnInitialize, OnFinalize },
};

use codec::Encode;
use sp_core::{ sr25519, Pair, H256 };
use frame_system as system;
use sp_runtime::{
	testing::Header,
	traits::{ BlakeTwo256, IdentityLookup, Hash },
};
use system::{EnsureSigned, EnsureSignedBy};
use sp_std::iter::*;
pub const VALIDATOR_DID: [u8; 32] = *b"did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const VALIDATOR_ACCOUNT: u64 = 2077282123132384724;
pub const VALIDATOR_SEED: [u8; 32] = [
	229, 190, 154, 80, 146, 184, 27, 202, 100, 190, 129, 210, 18, 231, 242, 249, 235, 161, 131,
  187, 122, 144, 149, 79, 123, 118, 54, 31, 110, 219, 92, 10,
	];

pub const REGIONAL_DID: [u8; 32] = *b"did:region:xyz\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const REGIONAL_ACCOUNT: u64 = 13620103657161844528;
pub const REGIONAL_SEED: [u8; 32] = [
	134, 128, 32, 174, 6, 135, 221, 167, 213, 117, 101, 9, 58, 105, 9, 2, 17, 68, 152, 69, 167,
	225, 20, 83, 97, 40, 0, 182, 99, 48, 114, 70,
];

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
		Did: pallet_did::{Pallet, Call, Storage, Event<T>, Config<T>},
		VcPallet: pallet_vc::{Pallet, Call, Storage, Event<T>, Config<T>},
	}
);

pub struct IsValidatorImplemented;
impl IsValidator for IsValidatorImplemented {

	fn is_validator(who: &[u8; 32]) -> bool {
		*who == VALIDATOR_DID
	}

	/// Check if given did has global permission level
	fn is_validator_global(_did: &[u8; 32]) -> bool {
		false
	}

	fn get_region(did: [u8; 32]) -> DidRegion {
		let colon = 58;
		let index = did.iter()
			.position(|&x| x == colon)
			.unwrap_or_default();
		let did = did.split_at(index+1).1;
		let index = did.iter()
			.position(|&x| x == colon)
			.unwrap_or_default();
		let region = did.split_at(index).0;
		dynamic_to_fixed_array::<20>(region)
	}
}

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

ord_parameter_types! {
	pub const ValidatorAccount: u64 = 1;
}

impl pallet_did::Config for Test {
	type Event = Event;
	type ValidatorOrigin = EnsureSigned<u64>;
	type MaxKeyChanges = ConstU32<16>;
	type VCResolution = VcPallet;
	type OnDidUpdate = ();
}

impl pallet_vc::Config for Test {
	type Event = Event;
	type ApproveOrigin = EnsureSignedBy<ValidatorAccount, u64>;
	type IsCouncilMember = ();
	type IsValidator = IsValidatorImplemented;
	type DidResolution = Did;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut o = frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();
	let mut allowed_regions: AllowedRegionsVec = Default::default();
	allowed_regions.try_push(*b"ssid\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0").expect("Vec Overflow");
	
	super::GenesisConfig::<Test> { 
		initial_dids: vec![
			DIdentity::Private(
				PrivateDid {
					identifier: VALIDATOR_DID,
					public_key: sr25519::Pair::from_seed(&VALIDATOR_SEED).public(),
					metadata: Default::default(),
				}
			),
			DIdentity::Private(
				PrivateDid {
					identifier: REGIONAL_DID,
					public_key: sr25519::Pair::from_seed(&REGIONAL_SEED).public(),
					metadata: Default::default(),
				}
			)
		],
		allowed_regions,
		phantom: Default::default(),
	}
		.assimilate_storage(&mut o)
		.unwrap();
	o.into()
}
	
pub fn get_public_did_vc(identifier: [u8; 32], public_key: PublicKey) -> [u8; 128]{
	let public_key = public_key;
	let did = identifier;
	let registration_number: RegistrationNumber = Default::default();
	let company_name: CompanyName = Default::default();
	let did_vc= PublicDidVC{
		public_key,
		registration_number,
		company_name,
		did
	};
	convert_to_array::<128>(did_vc.encode())
}

pub fn get_private_did_vc(identifier: [u8; 32], public_key: PublicKey) -> [u8; 128]{
	let public_key = public_key;
	let did = identifier;
	let did_vc = PrivateDidVC{
		public_key,
		did
	};
	convert_to_array::<128>(did_vc.encode())
}

pub fn get_reset_pub_key_vc(did: [u8; 32], vc_id: VCid, new_public_key: PublicKey) -> [u8; 128] {
	let reset_pub_key_vc: ResetPubKeyVC = ResetPubKeyVC {
		vc_id: Some(vc_id),
		did,
		new_public_key,
		old_public_key: None,
	};
	convert_to_array::<128>(reset_pub_key_vc.encode())
}

pub fn get_vc_id_and_hex(vc_owner: Identifier, did_vc_bytes: [u8; 128], vc_type: VCType) -> ([u8; 32], Vec<u8>) {
	let pair: sr25519::Pair = sr25519::Pair::from_seed(&VALIDATOR_SEED);
	let owner = vc_owner;
	let issuers: sp_runtime::BoundedVec<[u8; 32], ConstU32<20>> = bounded_vec![VALIDATOR_DID];
	let hash = BlakeTwo256::hash_of(&(&vc_type, &did_vc_bytes, &owner, &issuers));
	let signature = pair.sign(hash.as_ref());
	let vc_struct = VC {
		hash,
		owner,
		issuers,
		signatures: bounded_vec![signature],
		is_vc_used: false,
		is_vc_active: true,
		vc_type,
		vc_property: did_vc_bytes,
	};
	let vc_id: VCid = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();
  (vc_id, vc_struct.encode())
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

/// Convert Dynamic array to fixed array
pub fn dynamic_to_fixed_array<const N: usize>(array: &[u8]) -> [u8; N] {
	let array: Result<[u8; N], _> = array.iter()
		.chain(&[0; N])
		.copied()
		.take(N)
		.collect::<Vec<u8>>()
		.try_into();
	array.unwrap_or([0; N])
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
	}
}
