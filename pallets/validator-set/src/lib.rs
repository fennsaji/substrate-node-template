// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
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

//! # Membership Module
//!
//! Allows control of membership of a set of `AccountId`s, useful for managing membership of of a
//! collective. A prime member may be set

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	traits::{Contains, Get, SortedMembers},
	BoundedVec,
};
use sp_std::prelude::*;
use metamui_primitives::{Did, traits::{DidResolve, ChangeMembers, InitializeMembers}};

pub mod migrations;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		/// Required origin for adding a member (though can always be Root).
		type AddOrigin: EnsureOrigin<Self::Origin>;

		/// Required origin for removing a member (though can always be Root).
		type RemoveOrigin: EnsureOrigin<Self::Origin>;

		/// Required origin for adding and removing a member in a single action.
		type SwapOrigin: EnsureOrigin<Self::Origin>;

		/// Required origin for resetting membership.
		type ResetOrigin: EnsureOrigin<Self::Origin>;

		/// Required origin for setting or resetting the prime member.
		type PrimeOrigin: EnsureOrigin<Self::Origin>;

		/// The receiver of the signal for when the membership has been initialized. This happens
		/// pre-genesis and will usually be the same as `MembershipChanged`. If you need to do
		/// something different on initialization, then you can change this accordingly.
		type MembershipInitialized: InitializeMembers;

		/// The receiver of the signal for when the membership has changed.
		type MembershipChanged: ChangeMembers;

		/// The maximum number of members that this membership can have.
		///
		/// This is used for benchmarking. Re-run the benchmarks if this changes.
		///
		/// This is enforced in the code; the membership size can not exceed this limit.
		type MaxMembers: Get<u32>;

		/// Resolve Did from Account Id
		type DidResolution: DidResolve<Self::AccountId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	/// The current membership, stored as an ordered Vec.
	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type Members<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BoundedVec<Did, T::MaxMembers>, ValueQuery>;

	/// The current prime member, if one exists.
	#[pallet::storage]
	#[pallet::getter(fn prime)]
	pub type Prime<T: Config<I>, I: 'static = ()> = StorageValue<_, Did, OptionQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		pub members: BoundedVec<Did, T::MaxMembers>,
		pub phantom: PhantomData<I>,
	}

	#[cfg(feature = "std")]
	impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
		fn default() -> Self {
			Self { members: Default::default(), phantom: Default::default() }
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

			let mut members = self.members.clone();
			members.sort();
			T::MembershipInitialized::initialize_members(&members);
			<Members<T, I>>::put(members);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// The given member was added; see the transaction for who.
		MemberAdded,
		/// The given member was removed; see the transaction for who.
		MemberRemoved,
		/// Two members were swapped; see the transaction for who.
		MembersSwapped,
		/// The membership was reset; see the transaction for who the new set is.
		MembersReset,
		/// One of the members' keys changed.
		KeyChanged,
		/// Phantom member, never used.
		Dummy { _phantom_data: PhantomData<(Did, <T as Config<I>>::Event)> },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Already a member.
		AlreadyMember,
		/// Not a member.
		NotMember,
		/// Too many members.
		TooManyMembers,
		/// Did Does Not Exist
		DIDDoesNotExist,
		/// Did is not public
		DIDNotPublic,
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Add a member `who` to the set.
		///
		/// May only be called from `T::AddOrigin`.
		#[pallet::weight(50_000_000)]
		pub fn add_member(origin: OriginFor<T>, who: Did) -> DispatchResult {
			T::AddOrigin::ensure_origin(origin)?;

			ensure!(T::DidResolution::is_did_public(&who), Error::<T, I>::DIDDoesNotExist);

			let mut members = <Members<T, I>>::get();
			let location = members.binary_search(&who).err().ok_or(Error::<T, I>::AlreadyMember)?;
			members
				.try_insert(location, who.clone())
				.map_err(|_| Error::<T, I>::TooManyMembers)?;

			<Members<T, I>>::put(&members);

			T::MembershipChanged::change_members_sorted(&members[..]);

			Self::deposit_event(Event::MemberAdded);
			Ok(())
		}

		/// Remove a member `who` from the set.
		///
		/// May only be called from `T::RemoveOrigin`.
		#[pallet::weight(50_000_000)]
		pub fn remove_member(origin: OriginFor<T>, who: Did) -> DispatchResult {
			T::RemoveOrigin::ensure_origin(origin)?;

			let mut members = <Members<T, I>>::get();
			let location = members.binary_search(&who).ok().ok_or(Error::<T, I>::NotMember)?;
			members.remove(location);

			<Members<T, I>>::put(&members);

			T::MembershipChanged::change_members_sorted(&members[..]);
			Self::rejig_prime(&members);

			Self::deposit_event(Event::MemberRemoved);
			Ok(())
		}

		/// Swap out one member `remove` for another `add`.
		///
		/// May only be called from `T::SwapOrigin`.
		///
		/// Prime membership is *not* passed from `remove` to `add`, if extant.
		#[pallet::weight(50_000_000)]
		pub fn swap_member(
			origin: OriginFor<T>,
			remove: Did,
			add: Did,
		) -> DispatchResult {
			T::SwapOrigin::ensure_origin(origin)?;
			ensure!(T::DidResolution::is_did_public(&add), Error::<T, I>::DIDDoesNotExist);

			if remove == add {
				return Ok(())
			}

			let mut members = <Members<T, I>>::get();
			let location = members.binary_search(&remove).ok().ok_or(Error::<T, I>::NotMember)?;
			let _ = members.binary_search(&add).err().ok_or(Error::<T, I>::AlreadyMember)?;
			members[location] = add.clone();
			members.sort();

			<Members<T, I>>::put(&members);

			T::MembershipChanged::change_members_sorted(&members[..]);
			Self::rejig_prime(&members);

			Self::deposit_event(Event::MembersSwapped);
			Ok(())
		}

		/// Change the membership to a new set, disregarding the existing membership. Be nice and
		/// pass `members` pre-sorted.
		///
		/// May only be called from `T::ResetOrigin`.
		#[pallet::weight(50_000_000)]
		pub fn reset_members(origin: OriginFor<T>, members: Vec<Did>) -> DispatchResult {
			T::ResetOrigin::ensure_origin(origin)?;

			let mut members: BoundedVec<Did, T::MaxMembers> =
				BoundedVec::try_from(members).map_err(|_| Error::<T, I>::TooManyMembers)?;
			members.sort();
			<Members<T, I>>::mutate(|m| {
				T::MembershipChanged::set_members_sorted(&members[..]);
				Self::rejig_prime(&members);
				*m = members;
			});

			Self::deposit_event(Event::MembersReset);
			Ok(())
		}

		/// Swap out the sending member for some other key `new`.
		///
		/// May only be called from `Signed` origin of a current member.
		///
		/// Prime membership is passed from the origin account to `new`, if extant.
		#[pallet::weight(50_000_000)]
		pub fn change_key(origin: OriginFor<T>, new: Did) -> DispatchResult {
			let remove = ensure_signed(origin)?;
			let remove = T::DidResolution::get_did(&remove).unwrap_or_default();
			ensure!(T::DidResolution::is_did_public(&new), Error::<T, I>::DIDDoesNotExist);

			if remove != new {
				let mut members = <Members<T, I>>::get();
				let location =
					members.binary_search(&remove).ok().ok_or(Error::<T, I>::NotMember)?;
				let _ = members.binary_search(&new).err().ok_or(Error::<T, I>::AlreadyMember)?;
				members[location] = new.clone();
				members.sort();

				<Members<T, I>>::put(&members);

				T::MembershipChanged::change_members_sorted(
					&members[..],
				);

				if Prime::<T, I>::get() == Some(remove) {
					Prime::<T, I>::put(&new);
					T::MembershipChanged::set_prime(Some(new));
				}
			}

			Self::deposit_event(Event::KeyChanged);
			Ok(())
		}

		/// Set the prime member. Must be a current member.
		///
		/// May only be called from `T::PrimeOrigin`.
		#[pallet::weight(50_000_000)]
		pub fn set_prime(origin: OriginFor<T>, who: Did) -> DispatchResult {
			T::PrimeOrigin::ensure_origin(origin)?;
			ensure!(T::DidResolution::is_did_public(&who), Error::<T, I>::DIDDoesNotExist);
			Self::members().binary_search(&who).ok().ok_or(Error::<T, I>::NotMember)?;
			Prime::<T, I>::put(&who);
			T::MembershipChanged::set_prime(Some(who));
			Ok(())
		}

		/// Remove the prime member if it exists.
		///
		/// May only be called from `T::PrimeOrigin`.
		#[pallet::weight(50_000_000)]
		pub fn clear_prime(origin: OriginFor<T>) -> DispatchResult {
			T::PrimeOrigin::ensure_origin(origin)?;
			Prime::<T, I>::kill();
			T::MembershipChanged::set_prime(None);
			Ok(())
		}
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	fn rejig_prime(members: &[Did]) {
		if let Some(prime) = Prime::<T, I>::get() {
			match members.binary_search(&prime) {
				Ok(_) => T::MembershipChanged::set_prime(Some(prime)),
				Err(_) => Prime::<T, I>::kill(),
			}
		}
	}
}

impl<T: Config<I>, I: 'static> Contains<Did> for Pallet<T, I> {
	fn contains(t: &Did) -> bool {
		Self::members().binary_search(t).is_ok()
	}
}

impl<T: Config<I>, I: 'static> SortedMembers<Did> for Pallet<T, I> {
	fn sorted_members() -> Vec<Did> {
		Self::members().to_vec()
	}

	fn count() -> usize {
		Members::<T, I>::decode_len().unwrap_or(0)
	}
}

#[cfg(feature = "runtime-benchmarks")]
mod benchmark {
	use super::{Pallet as Membership, *};
	use frame_benchmarking::{account, benchmarks_instance_pallet, whitelist};
	use frame_support::{assert_ok, traits::EnsureOrigin};
	use frame_system::RawOrigin;

	const SEED: u32 = 0;

	fn set_members<T: Config<I>, I: 'static>(members: Vec<Did>, prime: Option<usize>) {
		let reset_origin = T::ResetOrigin::successful_origin();
		let prime_origin = T::PrimeOrigin::successful_origin();

		assert_ok!(<Membership<T, I>>::reset_members(reset_origin, members.clone()));
		if let Some(prime) = prime.map(|i| members[i].clone()) {
			let prime_lookup = T::Lookup::unlookup(prime);
			assert_ok!(<Membership<T, I>>::set_prime(prime_origin, prime_lookup));
		} else {
			assert_ok!(<Membership<T, I>>::clear_prime(prime_origin));
		}
	}

	benchmarks_instance_pallet! {
		add_member {
			let m in 1 .. (T::MaxMembers::get() - 1);

			let members = (0..m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			set_members::<T, I>(members, None);
			let new_member = account::<Did>("add", m, SEED);
			let new_member_lookup = T::Lookup::unlookup(new_member.clone());
		}: {
			assert_ok!(<Membership<T, I>>::add_member(T::AddOrigin::successful_origin(), new_member_lookup));
		}
		verify {
			assert!(<Members<T, I>>::get().contains(&new_member));
			#[cfg(test)] crate::tests::clean();
		}

		// the case of no prime or the prime being removed is surely cheaper than the case of
		// reporting a new prime via `MembershipChanged`.
		remove_member {
			let m in 2 .. T::MaxMembers::get();

			let members = (0..m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			set_members::<T, I>(members.clone(), Some(members.len() - 1));

			let to_remove = members.first().cloned().unwrap();
			let to_remove_lookup = T::Lookup::unlookup(to_remove.clone());
		}: {
			assert_ok!(<Membership<T, I>>::remove_member(T::RemoveOrigin::successful_origin(), to_remove_lookup));
		} verify {
			assert!(!<Members<T, I>>::get().contains(&to_remove));
			// prime is rejigged
			assert!(<Prime<T, I>>::get().is_some() && T::MembershipChanged::get_prime().is_some());
			#[cfg(test)] crate::tests::clean();
		}

		// we remove a non-prime to make sure it needs to be set again.
		swap_member {
			let m in 2 .. T::MaxMembers::get();

			let members = (0..m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			set_members::<T, I>(members.clone(), Some(members.len() - 1));
			let add = account::<Did>("member", m, SEED);
			let add_lookup = T::Lookup::unlookup(add.clone());
			let remove = members.first().cloned().unwrap();
			let remove_lookup = T::Lookup::unlookup(remove.clone());
		}: {
			assert_ok!(<Membership<T, I>>::swap_member(
				T::SwapOrigin::successful_origin(),
				remove_lookup,
				add_lookup,
			));
		} verify {
			assert!(!<Members<T, I>>::get().contains(&remove));
			assert!(<Members<T, I>>::get().contains(&add));
			// prime is rejigged
			assert!(<Prime<T, I>>::get().is_some() && T::MembershipChanged::get_prime().is_some());
			#[cfg(test)] crate::tests::clean();
		}

		// er keep the prime common between incoming and outgoing to make sure it is rejigged.
		reset_member {
			let m in 1 .. T::MaxMembers::get();

			let members = (1..m+1).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			set_members::<T, I>(members.clone(), Some(members.len() - 1));
			let mut new_members = (m..2*m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
		}: {
			assert_ok!(<Membership<T, I>>::reset_members(T::ResetOrigin::successful_origin(), new_members.clone()));
		} verify {
			new_members.sort();
			assert_eq!(<Members<T, I>>::get(), new_members);
			// prime is rejigged
			assert!(<Prime<T, I>>::get().is_some() && T::MembershipChanged::get_prime().is_some());
			#[cfg(test)] crate::tests::clean();
		}

		change_key {
			let m in 1 .. T::MaxMembers::get();

			// worse case would be to change the prime
			let members = (0..m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			let prime = members.last().cloned().unwrap();
			set_members::<T, I>(members.clone(), Some(members.len() - 1));

			let add = account::<Did>("member", m, SEED);
			let add_lookup = T::Lookup::unlookup(add.clone());
			whitelist!(prime);
		}: {
			assert_ok!(<Membership<T, I>>::change_key(RawOrigin::Signed(prime.clone()).into(), add_lookup));
		} verify {
			assert!(!<Members<T, I>>::get().contains(&prime));
			assert!(<Members<T, I>>::get().contains(&add));
			// prime is rejigged
			assert_eq!(<Prime<T, I>>::get().unwrap(), add);
			#[cfg(test)] crate::tests::clean();
		}

		set_prime {
			let m in 1 .. T::MaxMembers::get();
			let members = (0..m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			let prime = members.last().cloned().unwrap();
			let prime_lookup = T::Lookup::unlookup(prime.clone());
			set_members::<T, I>(members, None);
		}: {
			assert_ok!(<Membership<T, I>>::set_prime(T::PrimeOrigin::successful_origin(), prime_lookup));
		} verify {
			assert!(<Prime<T, I>>::get().is_some());
			assert!(<T::MembershipChanged>::get_prime().is_some());
			#[cfg(test)] crate::tests::clean();
		}

		clear_prime {
			let m in 1 .. T::MaxMembers::get();
			let members = (0..m).map(|i| account("member", i, SEED)).collect::<Vec<Did>>();
			let prime = members.last().cloned().unwrap();
			set_members::<T, I>(members, None);
		}: {
			assert_ok!(<Membership<T, I>>::clear_prime(T::PrimeOrigin::successful_origin()));
		} verify {
			assert!(<Prime<T, I>>::get().is_none());
			assert!(<T::MembershipChanged>::get_prime().is_none());
			#[cfg(test)] crate::tests::clean();
		}

		impl_benchmark_test_suite!(Membership, crate::tests::new_bench_ext(), crate::tests::Test);
	}
}
