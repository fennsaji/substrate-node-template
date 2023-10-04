// This file is part of Substrate.

// Copyright (C) 2017-2022 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

use scale_info::TypeInfo;
use sp_runtime::{traits::Hash, RuntimeDebug};
use sp_std::{marker::PhantomData, prelude::*};
use metamui_primitives::{Did, types::DidRegion, traits::{DidResolve, MultiAddress, ChangeMembers, InitializeMembers}};

use frame_support::{
	codec::{Decode, Encode, MaxEncodedLen},
	dispatch::{ DispatchResultWithPostInfo, Dispatchable, PostDispatchInfo},
	ensure,
	traits::{
		Backing, EnsureOrigin, Get, GetBacking, StorageVersion,
	},
	weights::{GetDispatchInfo, Weight},
};

// #[cfg(test)]
// mod tests;

mod types;
use crate::types::*;

mod impls;
pub use crate::impls::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// A number of members.
///
/// This also serves as a number of voting members, and since for motions, each member may
/// vote exactly once, therefore also the number of votes for any given motion.
pub type MemberCount = u32;


/// Origin for the collective module.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(I))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub enum RawOrigin<AccountId, I> {
	/// It has been condoned by a single member of the collective.
	Member(AccountId),
	/// Dummy to manage the fact we have instancing.
	_Phantom(PhantomData<I>),
}

impl<AccountId, I> GetBacking for RawOrigin<AccountId, I> {
	fn get_backing(&self) -> Option<Backing> {
		match self {
			_ => None,
		}
	}
}


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The outer origin type.
		type Origin: From<RawOrigin<Self::AccountId, I>>;

		/// Call Origin
		type CallOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// The outer call dispatch type.
		type Proposal: Parameter
			+ Dispatchable<Origin = <Self as Config<I>>::Origin, PostInfo = PostDispatchInfo>
			+ From<frame_system::Call<Self>>
			+ GetDispatchInfo;

		/// The outer event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The maximum number of members supported by the pallet. Used for weight estimation.
		///
		/// NOTE:
		/// + Benchmarks will need to be re-run and weights adjusted if this changes.
		/// + This pallet assumes that dependents keep to the limit without enforcing it.
		type MaxMembers: Get<MemberCount>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// Resolve Did from account Id
		type DidResolution: DidResolve<Self::AccountId>;
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub phantom: PhantomData<(T, I)>,
		pub members: Vec<Did>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { phantom: Default::default(), members: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig<T, I> for GenesisConfig<T, I> {
		fn build(&self) {
			use sp_std::collections::btree_set::BTreeSet;
			let members_set: BTreeSet<_> = self.members.iter().collect();
			assert_eq!(
				members_set.len(),
				self.members.len(),
				"Members cannot contain duplicate accounts."
			);

			Pallet::<T, I>::initialize_members(&self.members)
		}
	}

	/// Origin for the collective pallet.
	#[pallet::origin]
	pub type Origin<T, I = ()> = RawOrigin<<T as frame_system::Config>::AccountId, I>;

	/// The current members of the collective. This is stored sorted (just by value).
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config<I>, I: 'static = ()> =
		StorageValue<_, Vec<Did>, ValueQuery>;

	/// The current members of the collective. This is stored sorted (just by value).
	#[pallet::storage]
	#[pallet::getter(fn member_permission)]
	pub type MemberPermissionLevel<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Blake2_128Concat, Did, PermissionLevel, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// A single member did some action; result will be `Ok` if it returned without error.
		MemberExecuted { proposal_hash: T::Hash, result: DispatchResult },
		/// A single member did some action; result will be `Ok` if it returned without error.
		PermissionUpdated { did: Did, level: PermissionLevel },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Account is not a member
		NotMember,
		/// The given length bound for the proposal was too low.
		WrongProposalLength,
		/// Did Doesnot Exist
		DIDDoesNotExist,
	}

	// Note that councillor operations are assigned to the operational class.
	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Set the collective's membership.
		///
		/// - `new_members`: The new member list. Be nice to the chain and provide it sorted.
		/// - `prime`: The prime member whose vote sets the default.
		/// - `old_count`: The upper bound for the previous number of members in storage. Used for
		///   weight estimation.
		///
		/// Requires root origin.
		///
		/// NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
		///       the weight estimations rely on it to estimate dispatchable weight.
		///
		/// # WARNING:
		///
		/// The `pallet-collective` can also be managed by logic outside of the pallet through the
		/// implementation of the trait [`ChangeMembers`].
		/// Any call to `set_members` must be careful that the member set doesn't get out of sync
		/// with other logic managing the member set.
		///
		/// # <weight>
		/// ## Weight
		/// - `O(MP + N)` where:
		///   - `M` old-members-count (code- and governance-bounded)
		///   - `N` new-members-count (code- and governance-bounded)
		/// - DB:
		///   - 1 storage mutation (codec `O(M)` read, `O(N)` write) for reading and writing the
		///     members
		///   - 1 storage read (codec `O(P)`) for reading the proposals
		///   - 1 storage write (codec `O(1)`) for deleting the old `prime` and setting the new one
		/// # </weight>
		#[pallet::weight((
			T::WeightInfo::set_members(
				new_members.len() as u32, // N
			),
			DispatchClass::Operational
		))]
		pub fn set_members(
			origin: OriginFor<T>,
			new_members: Vec<Did>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			if new_members.len() > T::MaxMembers::get() as usize {
				log::error!(
					target: "runtime::collective",
					"New members count ({}) exceeds maximum amount of members expected ({}).",
					new_members.len(),
					T::MaxMembers::get(),
				);
			}
			let mut new_members = new_members;
			new_members.sort();
			<Self as ChangeMembers>::set_members_sorted(&new_members);

			Ok(().into())
		}

		/// Dispatch a proposal from a member using the `Member` origin.
		///
		/// Origin must be a member of the collective.
		///
		/// # <weight>
		/// ## Weight
		/// - `O(M + P)` where `M` members-count (code-bounded) and `P` complexity of dispatching
		///   `proposal`
		/// - DB: 1 read (codec `O(M)`) + DB access of `proposal`
		/// - 1 event
		/// # </weight>
		#[pallet::weight((
			T::WeightInfo::execute(
				*length_bound, // B
				T::MaxMembers::get(), // M
			).saturating_add(proposal.get_dispatch_info().weight), // P
			DispatchClass::Operational
		))]
		pub fn execute(
			origin: OriginFor<T>,
			proposal: Box<<T as Config<I>>::Proposal>,
			#[pallet::compact] length_bound: u32,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let who_did = T::DidResolution::get_did(&who).unwrap_or_default();
			let members = Self::members();
			ensure!(members.contains(&who_did), Error::<T, I>::NotMember);
			let proposal_len = proposal.encoded_size();
			ensure!(proposal_len <= length_bound as usize, Error::<T, I>::WrongProposalLength);

			let proposal_hash = T::Hashing::hash_of(&proposal);
			let result = proposal.dispatch(RawOrigin::Member(who).into());
			Self::deposit_event(Event::MemberExecuted {
				proposal_hash,
				result: result.map(|_| ()).map_err(|e| e.error),
			});

			Ok(get_result_weight(result)
				.map(|w| {
					T::WeightInfo::execute(
						proposal_len as u32,  // B
						members.len() as u32, // M
					)
					.saturating_add(w) // P
				})
				.into())
		}


		/// Dispatch a proposal from a member using the `Member` origin.
		///
		/// Origin must be a member of the collective.
		///
		/// # <weight>
		/// ## Weight
		/// - `O(M + P)` where `M` members-count (code-bounded) and `P` complexity of dispatching
		///   `proposal`
		/// - DB: 1 read (codec `O(M)`) + DB access of `proposal`
		/// - 1 event
		/// # </weight>
		#[pallet::weight(10_000)]
		pub fn update_permission(
			origin: OriginFor<T>,
			did: Did,
			level: PermissionLevel,
		) -> DispatchResultWithPostInfo {
			// Check if origin is a from a validator
			T::CallOrigin::ensure_origin(origin)?;

			ensure!(T::DidResolution::did_exists(MultiAddress::Did(did.clone())), Error::<T, I>::DIDDoesNotExist);

			let members = Self::members();
			ensure!(members.contains(&did), Error::<T, I>::NotMember);

			MemberPermissionLevel::<T, I>::insert(did, &level);

			Self::deposit_event(Event::PermissionUpdated {did, level});

			Ok(().into())
		}
	}
}

/// Return the weight of a dispatch call result as an `Option`.
///
/// Will return the weight regardless of what the state of the result is.
fn get_result_weight(result: DispatchResultWithPostInfo) -> Option<Weight> {
	match result {
		Ok(post_info) => post_info.actual_weight,
		Err(err) => err.post_info.actual_weight,
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Check whether `who` is a member of the collective.
	fn is_member(who: &Did) -> bool {
		// Note: The dispatchables *do not* use this to check membership so make sure
		// to update those if this is changed.
		Self::members().contains(who)
	}

	fn check_validator_global(did: &Did) -> bool {
    match Self::member_permission(did) {
			PermissionLevel::Global => true,
			_ => false,
		}
  }

	fn get_region(did: Did) -> DidRegion {
    let colon = 58;
    let index = did.iter()
      .position(|&x| x == colon)
      .unwrap_or_default();
    let did = did.split_at(index+1).1;
    let index = did.iter()
      .position(|&x| x == colon)
      .unwrap_or_default();
    let region = did.split_at(index).0;
    Self::dynamic_to_fixed_array::<20>(region)
  }

	/// Convert Dynamic array to fixed array
	fn dynamic_to_fixed_array<const N: usize>(array: &[u8]) -> [u8; N] {
		let array: Result<[u8; N], _> = array.iter()
			.chain(&[0; N])
			.copied()
			.take(N)
			.collect::<Vec<u8>>()
			.try_into();
		array.unwrap_or([0; N])
	}

}
