use super::{*, pallet::*};
use metamui_primitives::{Did, types::DidRegion, traits::{IsValidator}};

impl<T: Config<I>, I: 'static> IsValidator for Pallet<T, I> {

	/// Check whether `who` is a member of the collective.
  fn is_validator(who: &Did) -> bool {
  	// Note: The dispatchables *do not* use this to check membership so make sure
  	// to update those if this is changed.
  	Self::is_member(who)
  }

  /// Check if given did has global permission level
  fn is_validator_global(did: &Did) -> bool {
    Self::check_validator_global(did)
  }

	fn get_region(did: Did) -> DidRegion {
    Self::get_region(did)
  }
}


impl<T: Config<I>, I: 'static> ChangeMembers for Pallet<T, I> {
	/// Update the members of the collective. Votes are updated and the prime is reset.
	///
	/// NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
	///       the weight estimations rely on it to estimate dispatchable weight.
	///
	/// # <weight>
	/// ## Weight
	/// - `O(MP + N)`
	///   - where `M` old-members-count (governance-bounded)
	///   - where `N` new-members-count (governance-bounded)
	///   - where `P` proposals-count
	/// - DB:
	///   - 1 storage read (codec `O(P)`) for reading the proposals
	///   - `P` storage mutations for updating the votes (codec `O(M)`)
	///   - 1 storage write (codec `O(N)`) for storing the new members
	///   - 1 storage write (codec `O(1)`) for deleting the old prime
	/// # </weight>
	fn change_members_sorted(
		new: &[Did],
	) {
		if new.len() > T::MaxMembers::get() as usize {
			log::error!(
				target: "runtime::collective",
				"New members count ({}) exceeds maximum amount of members expected ({}).",
				new.len(),
				T::MaxMembers::get(),
			);
		}
		Members::<T, I>::put(new);
	}

	fn set_prime(_prime: Option<Did>) {}

	fn get_prime() -> Option<Did> {
		None
	}
}

impl<T: Config<I>, I: 'static> InitializeMembers for Pallet<T, I> {
	fn initialize_members(members: &[Did]) {
		if !members.is_empty() {
			assert!(<Members<T, I>>::get().is_empty(), "Members are already initialized!");
			<Members<T, I>>::put(members);
		}
	}
}

pub struct EnsureMember<AccountId, I: 'static>(PhantomData<(AccountId, I)>);
impl<
		O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
		I,
		AccountId: Decode,
	> EnsureOrigin<O> for EnsureMember<AccountId, I>
{
	type Success = AccountId;
	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Member(id) => Ok(id),
			r => Err(O::from(r)),
		})
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_successful_origin() -> Result<O, ()> {
		let zero_account_id =
			AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
				.expect("infinite length input; no invalid inputs for type; qed");
		Ok(O::from(RawOrigin::Member(zero_account_id)))
	}
}
