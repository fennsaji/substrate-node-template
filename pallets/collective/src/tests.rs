// This file is part of Substrate.

// Copyright (C) 2021-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{Event as CollectiveEvent, *};
use crate as pallet_collective;
use frame_support::{
	assert_noop, assert_ok, parameter_types,
	traits::{ConstU32, ConstU64, BuildGenesisConfig},
	weights::Pays,
};
use pallet_did::types::{DIdentity, PrivateDid, AllowedRegionsVec};
use frame_system::{EventRecord, Phase, EnsureSigned};
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};


pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Event<T>},
		Collective: pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Did: pallet_did::{Pallet, Call, Storage, Config<T>, Event<T>},
		CollectiveMajority: pallet_collective::<Instance2>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		DefaultCollective: pallet_collective::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
		Democracy: mock_democracy::{Pallet, Call, Event<T>},
	}
);

mod mock_democracy {
	pub use pallet::*;
	#[frame_support::pallet]
	pub mod pallet {
		use frame_support::pallet_prelude::*;
		use frame_system::pallet_prelude::*;

		#[pallet::pallet]
		#[pallet::generate_store(pub(super) trait Store)]
		pub struct Pallet<T>(_);

		#[pallet::config]
		pub trait Config: frame_system::Config + Sized {
			type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
			type ExternalMajorityOrigin: EnsureOrigin<Self::Origin>;
		}

		#[pallet::call]
		impl<T: Config> Pallet<T> {
			#[pallet::weight(0)]
			pub fn external_propose_majority(origin: OriginFor<T>) -> DispatchResult {
				T::ExternalMajorityOrigin::ensure_origin(origin)?;
				Self::deposit_event(Event::<T>::ExternalProposed);
				Ok(())
			}
		}

		#[pallet::event]
		#[pallet::generate_deposit(pub(super) fn deposit_event)]
		pub enum Event<T: Config> {
			ExternalProposed,
		}
	}
}

pub type MaxMembers = ConstU32<100>;

parameter_types! {
	pub const MotionDuration: u64 = 3;
	pub const MaxProposals: u32 = 100;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}
impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
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
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
}
impl Config<Instance1> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = ConstU64<3>;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = ();
	type DidResolution = Did;
}
impl Config<Instance2> for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = ConstU64<3>;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = ();
	type DidResolution = Did;
}

impl pallet_did::Config for Test {
	type Event = Event;
	type ValidatorOrigin = EnsureSigned<Self::AccountId>;
	type MaxKeyChanges = ConstU32<16>;
	type OnDidUpdate = ();
	type VCResolution = ();
}

impl mock_democracy::Config for Test {
	type Event = Event;
	type ExternalMajorityOrigin = EnsureProportionAtLeast<u64, Instance1, 3, 4>;
}
impl Config for Test {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = ConstU64<3>;
	type MaxProposals = MaxProposals;
	type MaxMembers = MaxMembers;
	type DefaultVote = PrimeDefaultVote;
	type WeightInfo = ();
	type DidResolution = Did;
}

pub const VALIDATOR_ACCOUNT: u64 = 0;
pub const VALIDATOR_DID: [u8; 32] = *b"did:ssid:Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const VALIDATOR_PUBKEY: sr25519::Public = sr25519::Public([0; 32]);
const ALICE: metamui_primitives::Did = *b"did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const BOB: metamui_primitives::Did = *b"did:ssid:bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const DAVE: metamui_primitives::Did = *b"did:ssid:dave\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
const EVE: metamui_primitives::Did = *b"did:ssid:eve\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
pub const DAVE_ACCOUNT_ID: u64 = 13620103657161844528;
const BOB_ACCOUNT_ID: u64 = 7166219960988249998;
pub const ALICE_ACCOUNT_ID: u64 = 2077282123132384724;
const BOB_SEED: [u8; 32] = [
	57, 143, 12, 40, 249, 136, 133, 224, 70, 51, 61, 74, 65, 193, 156, 238, 76, 55, 54, 138, 152,
	50, 198, 80, 47, 108, 253, 24, 46, 42, 239, 137,
];
const DAVE_SEED: [u8; 32] = [
	134, 128, 32, 174, 6, 135, 221, 167, 213, 117, 101, 9, 58, 105, 9, 2, 17, 68, 152, 69, 167,
	225, 20, 83, 97, 40, 0, 182, 99, 48, 114, 70,
];
const ALICE_SEED: [u8; 32] = [
    229, 190, 154, 80, 146, 184, 27, 202, 100, 190, 129, 210, 18, 231, 242, 249, 235, 161, 131,
    187, 122, 144, 149, 79, 123, 118, 54, 31, 110, 219, 92, 10,
];

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance1> {
		phantom: Default::default(),
		members: vec![BOB, DAVE, ALICE],
	}
	.assimilate_storage(&mut ext)
	.unwrap();
	pallet_collective::GenesisConfig::<Test, pallet_collective::Instance2> {
		members: vec![BOB, DAVE, EVE, ALICE, VALIDATOR_DID],
		phantom: Default::default(),
	}
	.assimilate_storage(&mut ext)
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
				identifier: ALICE,
				public_key: sr25519::Pair::from_seed(&ALICE_SEED).public(),
				metadata: Default::default(),
			}),
			DIdentity::Private(PrivateDid {
				identifier: VALIDATOR_DID,
				public_key: VALIDATOR_PUBKEY,
				metadata: Default::default(),
			}),
		],
		allowed_regions,
		phantom: Default::default(),
	}
	.assimilate_storage(&mut ext)
	.unwrap();
	let mut t = sp_io::TestExternalities::new(ext);
  t.execute_with(|| System::set_block_number(1) );
  t
}

fn make_proposal(value: u64) -> Call {
	Call::System(frame_system::Call::remark_with_event { remark: value.to_be_bytes().to_vec() })
}

fn record(event: Event) -> EventRecord<Event, H256> {
	EventRecord { phase: Phase::Initialization, event, topics: vec![] }
}

#[test]
fn motions_basic_environment_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Collective::members(), vec![BOB, DAVE, ALICE]);
		assert_eq!(*Collective::proposals(), Vec::<H256>::new());
	});
}

#[test]
fn close_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));

		System::set_block_number(3);
		assert_noop!(
			Collective::close(Origin::signed(ALICE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len),
			Error::<Test, Instance1>::TooEarly
		);

		System::set_block_number(4);
		assert_ok!(Collective::close(Origin::signed(ALICE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len));

		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 3
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 2,
					no: 1
				})),
				record(Event::Collective(CollectiveEvent::Disapproved { proposal_hash: hash })),
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn proposal_weight_limit_works_on_approve() {
	new_test_ext().execute_with(|| {
		let proposal = Call::Collective(crate::Call::set_members {
			new_members: vec![BOB, DAVE, EVE],
			prime: None,
			old_count: MaxMembers::get(),
		});
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		
		// Set 1 as prime voter
		Prime::<Test, Instance1>::set(Some(BOB));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		// With 1's prime vote, this should pass
		System::set_block_number(4);
		assert_noop!(
			Collective::close(Origin::signed(4), hash, 0, proposal_weight - 100, proposal_len),
			Error::<Test, Instance1>::WrongProposalWeight
		);
		assert_ok!(Collective::close(Origin::signed(4), hash, 0, proposal_weight, proposal_len));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	})
}

#[test]
fn proposal_weight_limit_ignored_on_disapprove() {
	new_test_ext().execute_with(|| {
		let proposal = Call::Collective(crate::Call::set_members {
			new_members: vec![BOB, DAVE, EVE],
			prime: None,
			old_count: MaxMembers::get(),
		});
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));

		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		// No votes, this proposal wont pass
		System::set_block_number(4);
		assert_ok!(Collective::close(
			Origin::signed(VALIDATOR_ACCOUNT),
			hash,
			0,
			proposal_weight - 100,
			proposal_len
		));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	})
}

#[test]
fn close_with_prime_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::set_members(
			Origin::root(),
			vec![BOB, DAVE, EVE],
			Some(VALIDATOR_DID),
			MaxMembers::get()
		));

		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));

		System::set_block_number(4);
		assert_ok!(Collective::close(Origin::signed(VALIDATOR_ACCOUNT), hash, 0, proposal_weight, proposal_len));

		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 3
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 2,
					no: 1
				})),
				record(Event::Collective(CollectiveEvent::Disapproved { proposal_hash: hash }))
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn close_with_voting_prime_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::set_members(
			Origin::root(),
			vec![BOB, DAVE, EVE],
			Some(BOB),
			MaxMembers::get()
		));

		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));

		System::set_block_number(4);
		assert_ok!(Collective::close(Origin::signed(VALIDATOR_ACCOUNT), hash, 0, proposal_weight, proposal_len));

		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 3
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 3,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Approved { proposal_hash: hash })),
				record(Event::Collective(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Err(DispatchError::BadOrigin)
				}))
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn close_with_no_prime_but_majority_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(CollectiveMajority::set_members(
			Origin::root(),
			vec![BOB, DAVE, EVE, ALICE, VALIDATOR_DID],
			Some(VALIDATOR_DID),
			MaxMembers::get()
		));
		
		assert_ok!(CollectiveMajority::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			5,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(CollectiveMajority::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(CollectiveMajority::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(CollectiveMajority::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		assert_ok!(CollectiveMajority::vote(Origin::signed(ALICE_ACCOUNT_ID), hash, 0, true));

		System::set_block_number(4);
		assert_ok!(CollectiveMajority::close(
			Origin::signed(VALIDATOR_ACCOUNT),
			hash,
			0,
			proposal_weight,
			proposal_len
		));

		assert_eq!(
			System::events(),
			vec![
				record(Event::CollectiveMajority(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 5
				})),
				record(Event::CollectiveMajority(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::CollectiveMajority(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::CollectiveMajority(CollectiveEvent::Voted {
					account: ALICE,
					proposal_hash: hash,
					voted: true,
					yes: 3,
					no: 0
				})),
				record(Event::CollectiveMajority(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 5,
					no: 0
				})),
				record(Event::CollectiveMajority(CollectiveEvent::Approved {
					proposal_hash: hash
				})),
				record(Event::CollectiveMajority(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Err(DispatchError::BadOrigin)
				}))
			]
		);
		assert_eq!(CollectiveMajority::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn removal_of_old_voters_votes_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		let end = 4;
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 3, ayes: vec![BOB, DAVE], nays: vec![], end })
		);
		Collective::change_members_sorted(&[ALICE], &[BOB], &[DAVE, EVE, ALICE]);
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 3, ayes: vec![DAVE], nays: vec![], end })
		);
		
		let proposal = make_proposal(69);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(DAVE_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 1, true));
		assert_ok!(Collective::vote(Origin::signed(ALICE_ACCOUNT_ID), hash, 1, false));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 1, threshold: 2, ayes: vec![DAVE], nays: vec![ALICE], end })
		);
		Collective::change_members_sorted(&[], &[ALICE], &[DAVE, EVE]);
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 1, threshold: 2, ayes: vec![DAVE], nays: vec![], end })
		);
	});
}

#[test]
fn removal_of_old_voters_votes_works_with_set_members() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		let end = 4;
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 3, ayes: vec![BOB, DAVE], nays: vec![], end })
		);
		assert_ok!(Collective::set_members(Origin::root(), vec![DAVE, EVE, ALICE], None, MaxMembers::get()));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 3, ayes: vec![DAVE], nays: vec![], end })
		);

		let proposal = make_proposal(69);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(DAVE_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 1, true));
		assert_ok!(Collective::vote(Origin::signed(ALICE_ACCOUNT_ID), hash, 1, false));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 1, threshold: 2, ayes: vec![DAVE], nays: vec![ALICE], end })
		);
		assert_ok!(Collective::set_members(Origin::root(), vec![DAVE, EVE], None, MaxMembers::get()));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 1, threshold: 2, ayes: vec![DAVE], nays: vec![], end })
		);
	});
}

#[test]
fn propose_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		let end = 4;
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_eq!(*Collective::proposals(), vec![hash]);
		assert_eq!(Collective::proposal_of(&hash), Some(proposal));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 3, ayes: vec![], nays: vec![], end })
		);

		assert_eq!(
			System::events(),
			vec![record(Event::Collective(CollectiveEvent::Proposed {
				account: BOB,
				proposal_index: 0,
				proposal_hash: hash,
				threshold: 3
			}))]
		);
	});
}

#[test]
fn limit_active_proposals() {
	new_test_ext().execute_with(|| {
		for i in 0..MaxProposals::get() {
			let proposal = make_proposal(i as u64);
			let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
			assert_ok!(Collective::propose(
				Origin::signed(BOB_ACCOUNT_ID),
				3,
				Box::new(proposal.clone()),
				proposal_len
			));
		}
		let proposal = make_proposal(MaxProposals::get() as u64 + 1);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		assert_noop!(
			Collective::propose(Origin::signed(BOB_ACCOUNT_ID), 3, Box::new(proposal.clone()), proposal_len),
			Error::<Test, Instance1>::TooManyProposals
		);
	})
}

#[test]
fn correct_validate_and_get_proposal() {
	new_test_ext().execute_with(|| {
		let proposal = Call::Collective(crate::Call::set_members {
			new_members: vec![BOB, DAVE, EVE],
			prime: None,
			old_count: MaxMembers::get(),
		});

		let length = proposal.encode().len() as u32;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(Origin::signed(BOB_ACCOUNT_ID), 3, Box::new(proposal.clone()), length));

		
		
		let weight = proposal.get_dispatch_info().weight;
		assert_noop!(
			Collective::validate_and_get_proposal(
				&BlakeTwo256::hash_of(&vec![3; 4]),
				length,
				weight
			),
			Error::<Test, Instance1>::ProposalMissing
		);
		assert_noop!(
			Collective::validate_and_get_proposal(&hash, length - 2, weight),
			Error::<Test, Instance1>::WrongProposalLength
		);
		assert_noop!(
			Collective::validate_and_get_proposal(&hash, length, weight - 10),
			Error::<Test, Instance1>::WrongProposalWeight
		);
		let res = Collective::validate_and_get_proposal(&hash, length, weight);
		assert_ok!(res.clone());
		let (retrieved_proposal, len) = res.unwrap();
		assert_eq!(length as usize, len);
		assert_eq!(proposal, retrieved_proposal);
	})
}

#[test]
fn motions_ignoring_non_collective_proposals_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		assert_noop!(
			Collective::propose(Origin::signed(42), 3, Box::new(proposal.clone()), proposal_len),
			Error::<Test, Instance1>::NotMember
		);
	});
}

#[test]
fn motions_ignoring_non_collective_votes_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_noop!(
			Collective::vote(Origin::signed(42), hash, 0, true),
			Error::<Test, Instance1>::NotMember,
		);
	});
}

#[test]
fn motions_ignoring_bad_index_collective_vote_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(3);
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_noop!(
			Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 1, true),
			Error::<Test, Instance1>::WrongIndex,
		);
	});
}

#[test]
fn motions_vote_after_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		let end = 4;
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		// Initially there a no votes when the motion is proposed.
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 2, ayes: vec![], nays: vec![], end })
		);
		// Cast first aye vote.
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 2, ayes: vec![BOB], nays: vec![], end })
		);
		// Try to cast a duplicate aye vote.
		assert_noop!(
			Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true),
			Error::<Test, Instance1>::DuplicateVote,
		);
		// Cast a nay vote.
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, false));
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 2, ayes: vec![], nays: vec![BOB], end })
		);
		// Try to cast a duplicate nay vote.
		assert_noop!(
			Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, false),
			Error::<Test, Instance1>::DuplicateVote,
		);

		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 2
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: false,
					yes: 0,
					no: 1
				})),
			]
		);
	});
}

#[test]
fn motions_all_first_vote_free_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		let end = 4;
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len,
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_eq!(
			Collective::voting(&hash),
			Some(Votes { index: 0, threshold: 2, ayes: vec![], nays: vec![], end })
		);

		// For the motion, acc 2's first vote, expecting Ok with Pays::No.
		let vote_rval: DispatchResultWithPostInfo =
			Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true);
		assert_eq!(vote_rval.unwrap().pays_fee, Pays::No);

		// Duplicate vote, expecting error with Pays::Yes.
		let vote_rval: DispatchResultWithPostInfo =
			Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true);
		assert_eq!(vote_rval.unwrap_err().post_info.pays_fee, Pays::Yes);

		// Modifying vote, expecting ok with Pays::Yes.
		let vote_rval: DispatchResultWithPostInfo =
			Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, false);
		assert_eq!(vote_rval.unwrap().pays_fee, Pays::Yes);

		// For the motion, acc 3's first vote, expecting Ok with Pays::No.
		let vote_rval: DispatchResultWithPostInfo =
			Collective::vote(Origin::signed(ALICE_ACCOUNT_ID), hash, 0, true);
		assert_eq!(vote_rval.unwrap().pays_fee, Pays::No);

		// acc 3 modify the vote, expecting Ok with Pays::Yes.
		let vote_rval: DispatchResultWithPostInfo =
			Collective::vote(Origin::signed(ALICE_ACCOUNT_ID), hash, 0, false);
		assert_eq!(vote_rval.unwrap().pays_fee, Pays::Yes);

		// Test close() Extrincis | Check DispatchResultWithPostInfo with Pay Info

		let proposal_weight = proposal.get_dispatch_info().weight;
		let close_rval: DispatchResultWithPostInfo =
			Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len);
		assert_eq!(close_rval.unwrap().pays_fee, Pays::No);

		// trying to close the proposal, which is already closed.
		// Expecting error "ProposalAlreadyClosed" with Pays::Yes
		let close_rval: DispatchResultWithPostInfo =
			Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len);
		assert_eq!(close_rval.unwrap_err().post_info.pays_fee, Pays::Yes);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
		
	});
}

#[test]
fn motions_reproposing_disapproved_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, false));
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
		assert_eq!(*Collective::proposals(), vec![]);
		System::set_block_number(4);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_eq!(*Collective::proposals(), vec![hash]);
	});
}

#[test]
fn motions_approval_with_enough_votes_and_lower_voting_threshold_works() {
	new_test_ext().execute_with(|| {
		let proposal = Call::Democracy(mock_democracy::Call::external_propose_majority {});
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		// The voting threshold is 2, but the required votes for `ExternalMajorityOrigin` is 3.
		// The proposal will be executed regardless of the voting threshold
		// as long as we have enough yes votes.
		//
		// Failed to execute with only 2 yes votes.
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len));
		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 2
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Approved { proposal_hash: hash })),
				record(Event::Collective(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Err(DispatchError::BadOrigin)
				})),
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);

		System::reset_events();

		System::set_block_number(4);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		// Executed with 3 yes votes.
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 1, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 1, true));
		assert_ok!(Collective::vote(Origin::signed(ALICE_ACCOUNT_ID), hash, 1, true));
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 1, proposal_weight, proposal_len));
		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 1,
					proposal_hash: hash,
					threshold: 2
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: ALICE,
					proposal_hash: hash,
					voted: true,
					yes: 3,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 3,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Approved { proposal_hash: hash })),
				record(Event::Democracy(mock_democracy::pallet::Event::<Test>::ExternalProposed)),
				record(Event::Collective(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Ok(())
				})),
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn motions_disapproval_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, false));
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len));

		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 3
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: false,
					yes: 1,
					no: 1
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 1,
					no: 1
				})),
				record(Event::Collective(CollectiveEvent::Disapproved { proposal_hash: hash })),
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn motions_approval_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len));

		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 2
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Closed {
					proposal_hash: hash,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Approved { proposal_hash: hash })),
				record(Event::Collective(CollectiveEvent::Executed {
					proposal_hash: hash,
					result: Err(DispatchError::BadOrigin)
				})),
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	});
}

#[test]
fn motion_with_no_votes_closes_with_disapproval() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let proposal_weight = proposal.get_dispatch_info().weight;
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			3,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		assert_eq!(
			System::events()[0],
			record(Event::Collective(CollectiveEvent::Proposed {
				account: BOB,
				proposal_index: 0,
				proposal_hash: hash,
				threshold: 3
			}))
		);

		// Closing the motion too early is not possible because it has neither
		// an approving or disapproving simple majority due to the lack of votes.
		assert_noop!(
			Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len),
			Error::<Test, Instance1>::TooEarly
		);

		// Once the motion duration passes,
		let closing_block = System::block_number() + MotionDuration::get();
		System::set_block_number(closing_block);
		// we can successfully close the motion.
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, proposal_weight, proposal_len));

		// Events show that the close ended in a disapproval.
		assert_eq!(
			System::events()[1],
			record(Event::Collective(CollectiveEvent::Closed {
				proposal_hash: hash,
				yes: 0,
				no: 3
			}))
		);
		assert_eq!(
			System::events()[2],
			record(Event::Collective(CollectiveEvent::Disapproved { proposal_hash: hash }))
		);

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	})
}

#[test]
fn close_disapprove_does_not_care_about_weight_or_len() {
	// This test confirms that if you close a proposal that would be disapproved,
	// we do not care about the proposal length or proposal weight since it will
	// not be read from storage or executed.
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));		
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		// First we make the proposal succeed
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		// It will not close with bad weight/len information
		assert_noop!(
			Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, 0, 0),
			Error::<Test, Instance1>::WrongProposalLength,
		);
		assert_noop!(
			Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, 0, proposal_len),
			Error::<Test, Instance1>::WrongProposalWeight,
		);
		// Now we make the proposal fail
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, false));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, false));
		// It can close even if the weight/len information is bad
		assert_ok!(Collective::close(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, 0, 0));

		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	})
}

#[test]
fn disapprove_proposal_works() {
	new_test_ext().execute_with(|| {
		let proposal = make_proposal(42);
		let proposal_len: u32 = proposal.using_encoded(|p| p.len() as u32);
		let block_number = frame_system::Pallet::<Test>::block_number();
		let hash = BlakeTwo256::hash_of(&(&proposal, &block_number));
		assert_ok!(Collective::propose(
			Origin::signed(BOB_ACCOUNT_ID),
			2,
			Box::new(proposal.clone()),
			proposal_len
		));
		
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Active);
		// Proposal would normally succeed
		assert_ok!(Collective::vote(Origin::signed(BOB_ACCOUNT_ID), hash, 0, true));
		assert_ok!(Collective::vote(Origin::signed(DAVE_ACCOUNT_ID), hash, 0, true));
		// But Root can disapprove and remove it anyway
		assert_ok!(Collective::disapprove_proposal(Origin::root(), hash));
		assert_eq!(
			System::events(),
			vec![
				record(Event::Collective(CollectiveEvent::Proposed {
					account: BOB,
					proposal_index: 0,
					proposal_hash: hash,
					threshold: 2
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: BOB,
					proposal_hash: hash,
					voted: true,
					yes: 1,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Voted {
					account: DAVE,
					proposal_hash: hash,
					voted: true,
					yes: 2,
					no: 0
				})),
				record(Event::Collective(CollectiveEvent::Disapproved { proposal_hash: hash })),
			]
		);
		assert_eq!(Collective::proposal_status_of(hash).unwrap().last().unwrap().0, ProposalStatus::Closed);
	})
}

#[test]
#[should_panic(expected = "Members cannot contain duplicate accounts.")]
fn genesis_build_panics_with_duplicate_members() {
	pallet_collective::GenesisConfig::<Test> {
		members: vec![BOB, DAVE, BOB],
		phantom: Default::default(),
	}
	.build_storage()
	.unwrap();
}
