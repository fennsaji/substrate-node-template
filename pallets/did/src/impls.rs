use super::pallet::*;
use crate::types::*;
use codec::Codec;
use metamui_primitives::traits::{DidResolve, MultiAddress};
use sp_runtime::traits::{LookupError, StaticLookup};

impl<T: Config> DidResolve<T::AccountId> for Pallet<T> {
	/// Check if Did exists
	fn did_exists(address: MultiAddress<T::AccountId>) -> bool {
		match address {
			// Return if the source is accountId
			MultiAddress::Id(id) => RLookup::<T>::contains_key(id),
			// Fetch the accountId from storage if did is passed
			MultiAddress::Did(did) => Lookup::<T>::contains_key(did),
		}
	}

	/// Get did from account id
	fn get_did(account_id: &T::AccountId) -> Option<Did> {
		RLookup::<T>::get(account_id)
	}

	/// Get accountId from did
	fn get_account_id(did: &Did) -> Option<T::AccountId> {
		Lookup::<T>::get(did)
	}

	/// Get public_key from accountId
	fn get_public_key(did: &Did) -> Option<PublicKey> {
		Self::get_pub_key(did)
	}

	/// Is DID Public or private
	fn is_did_public(did: &Did) -> bool {
		Self::check_did_public(did)
	}

	/// Get allowed regions
	fn get_regions() -> AllowedRegionsVec {
		Self::get_regions()
	}
}

/// implement the lookup trait to fetch the accountid of the
/// did from storage
impl<T: Config> StaticLookup for Pallet<T>
where
	MultiAddress<T::AccountId>: Codec,
{
	type Source = MultiAddress<T::AccountId>;
	type Target = T::AccountId;

	fn lookup(x: Self::Source) -> Result<Self::Target, LookupError> {
		match x {
			// Return if the source is accountId
			MultiAddress::Id(id) => Ok(id),
			// Fetch the accountId from storage if did is passed
			MultiAddress::Did(did) => Lookup::<T>::get(did).ok_or(LookupError),
		}
	}

	fn unlookup(x: Self::Target) -> Self::Source {
		MultiAddress::Id(x)
	}
}
