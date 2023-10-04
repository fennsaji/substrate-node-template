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
	traits::{ConstU32, ConstU64, BuildGenesisConfig, StorageVersion},
	weights::Pays,
	Hashable,
};
use frame_system::{EventRecord, Phase};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;
pub type AccountId = u64;

pub struct DidResolution;
impl DidResolve<AccountId> for DidResolution {
	/// return if an accountId is mapped to a DID
  fn did_exists(x: &AccountId) -> bool {
		true
	}
  /// convert accountId to DID
  fn get_did(k: &AccountId) -> Option<Did> {
		Some(*b"did:ssid:swn/0/0/0/0/0/0/0/0/0/0")
	}
}

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Event<T>},
		Collective: pallet_collective::<Instance1>::{Pallet, Call, Event<T>, Origin<T>, Config<T>},
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
	type DidResolution = DidResolution;
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
	type DidResolution = DidResolution;
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
	type DidResolution = DidResolution;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut ext: sp_io::TestExternalities = GenesisConfig {
		collective: pallet_collective::GenesisConfig {
			members: vec![1, 2, 3],
			phantom: Default::default(),
		},
		collective_majority: pallet_collective::GenesisConfig {
			members: vec![1, 2, 3, 4, 5],
			phantom: Default::default(),
		},
		default_collective: Default::default(),
	}
	.build_storage()
	.unwrap()
	.into();
	ext.execute_with(|| System::set_block_number(1));
	ext
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
		assert_eq!(Collective::members(), vec![1, 2, 3]);
		assert_eq!(*Collective::proposals(), Vec::<H256>::new());
	});
}

#[test]
#[should_panic(expected = "Members cannot contain duplicate accounts.")]
fn genesis_build_panics_with_duplicate_members() {
	pallet_collective::GenesisConfig::<Test> {
		members: vec![1, 2, 3, 1],
		phantom: Default::default(),
	}
	.build_storage()
	.unwrap();
}
