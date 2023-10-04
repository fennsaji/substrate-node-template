use super::*;
use types::*;
use sp_std::prelude::*;
use codec::{Decode, Encode};
use frame_support::sp_runtime::{DispatchError, DispatchResult};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_core::hexdisplay::HexDisplay;
// DID

/// Trait to resolve Did
pub trait DidResolve<AccountId> {
	/// return if an accountId is mapped to a DID
	fn did_exists(x: MultiAddress<AccountId>) -> bool;
	/// convert accountId to DID
	fn get_did(k: &AccountId) -> Option<Did>;
	/// convert DID to accountId
	fn get_account_id(k: &Did) -> Option<AccountId>;
	/// get public_key from accountId
	fn get_public_key(k: &Did) -> Option<PublicKey>;
	/// Check if did is public
	fn is_did_public(did: &Did) -> bool;
	/// Get allowed regions
	fn get_regions() -> AllowedRegionsVec;
}

impl<AccountId> DidResolve<AccountId> for () {
	/// return if an accountId is mapped to a DID
	fn did_exists(_: MultiAddress<AccountId>) -> bool {
		false
	}
	/// convert accountId to DID
	fn get_did(_: &AccountId) -> Option<Did> {
		None
	}
	/// convert DID to accountId
	fn get_account_id(_: &Did) -> Option<AccountId> {
		None
	}
	/// get public_key from accountId
	fn get_public_key(_: &Did) -> Option<PublicKey> {
		None
	}
	/// Check if did is public
	fn is_did_public(_did: &Did) -> bool {
		false
	}
	/// Get allowed regions
	fn get_regions() -> AllowedRegionsVec {
		Default::default()
	}
}

/// Use this struct for the account lookup
/// This struct can have the value of either rawbytes or accountid
/// This is necessary to compile all other pallets that depend on the accountID field
/// Once all pallets have been ported to the custom DID format we can remove the dependency
/// on this struct and lookup trait in general
#[derive(Encode, Decode, PartialEq, Eq, Clone, TypeInfo, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum MultiAddress<AccountId> {
	/// type for regular pubkey accountid
	Id(AccountId),
	/// type for lookup to the did identifier - referencing the did type from the did module
	Did(Did),
}

#[cfg(feature = "std")]
impl<AccountId> std::fmt::Display for MultiAddress<AccountId>
where
	AccountId: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			MultiAddress::Did(inner) => write!(f, "{}", HexDisplay::from(inner)),
			MultiAddress::Id(_inner) => write!(f, "{}", self),
		}
	}
}

// Create a MultiAddress object from an accountid passed
impl<AccountId> From<AccountId> for MultiAddress<AccountId> {
	fn from(x: AccountId) -> Self {
		MultiAddress::Id(x)
	}
}

// The default option to select when creating a Multiaddress
// The current default is set to accountid, but once we migrate all pallets
// to use did signing, we can move default to did
impl<AccountId: Default> Default for MultiAddress<AccountId> {
	fn default() -> Self {
		MultiAddress::Id(Default::default())
	}
}
// VC
/// Trait to get VC details
pub trait VCResolve<Hash> {
	/// Get VC from VC Id
	fn get_vc(vc_id: &VCid) -> Option<VC<Hash>>;
	/// Get if VC is used
	fn is_vc_used(vc_id: &VCid) -> bool;
	/// Set VC used
	fn set_is_vc_used(vc_id: &VCid, is_vc_used: bool) -> Result<(), DispatchError>;
	/// Decode VC
	fn decode_vc<E: Decode>(vc_bytes: &[u8]) -> Result<E, DispatchError>;
}

impl<Hash> VCResolve<Hash> for () {
	/// Get VC from VC Id
	fn get_vc(_vc_id: &VCid) -> Option<VC<Hash>> {
		None
	}
	/// Get if VC is used
	fn is_vc_used(_vc_id: &VCid) -> bool {
		true
	}
	/// Set VC used
	fn set_is_vc_used(_vc_id: &VCid, _is_vc_used: bool) -> Result<(), DispatchError> {
		Err("Not Implemented".into())
	}
	/// Decode VC
	fn decode_vc<E: Decode>(_vc_bytes: &[u8]) -> Result<E, DispatchError> {
		Err("Not Implemented".into())
	}
}

/// Trait to give back the VCid
pub trait HasVCId {
	/// Function to return the VCid
	fn vc_id(&self) -> VCid;
}

/// Implementing HasVCId for SlashMintTokens
impl HasVCId for SlashMintTokens {
	/// Function to return the VCid
	fn vc_id(&self) -> VCid {
		self.vc_id
	}
}

/// Implementing HasVCId for TokenTransferVC
impl HasVCId for TokenTransferVC {
	/// Function to return the VCid
	fn vc_id(&self) -> VCid {
		self.vc_id
	}
}

/// Trait to give back the VCid
pub trait HasDid {
	/// Function to return the VCid
	fn did(&self) -> Did;
}

impl HasDid for ResetPubKeyVC {
	/// Function to return the Did
	fn did(&self) -> VCid {
		self.did
	}
}

impl HasDid for PublicDidVC {
	/// Function to return the Did
	fn did(&self) -> VCid {
		self.did
	}
}

impl HasDid for PrivateDidVC {
	/// Function to return the Did
	fn did(&self) -> VCid {
		self.did
	}
}

/// Trait to give back the VCid
pub trait HasPublicKey {
	/// Function to return the VCid
	fn public_key(&self) -> PublicKey;
}

impl HasPublicKey for PublicDidVC {
	/// Function to return the Did
	fn public_key(&self) -> PublicKey {
		self.public_key.into()
	}
}

impl HasPublicKey for PrivateDidVC {
	/// Function to return the Did
	fn public_key(&self) -> PublicKey {
		self.public_key.into()
	}
}



/// Trait to check if a Did is a member
pub trait IsMember {
	/// Function to check membership
	fn is_member(_: &Did) -> bool;
}

impl IsMember for () {
	/// Function to check membership
	fn is_member(_: &Did) -> bool {
		false
	}
}

/// Trait for type that can handle the initialization of account IDs at genesis.
pub trait InitializeMembers {
	/// Initialize the members to the given `members`.
	fn initialize_members(members: &[Did]);
}

impl InitializeMembers for () {
	fn initialize_members(_: &[Did]) {}
}

/// Validator Set
/// Trait for type that can handle incremental changes to a set of account IDs.
pub trait ChangeMembers {
	/// A number of members `incoming` just joined the set and replaced some `outgoing` ones. The
	/// new set is given by `new`, and need not be sorted.
	///
	/// This resets any previous value of prime.
	fn change_members(mut new: Vec<Did>) {
		new.sort();
		Self::change_members_sorted(&new[..]);
	}

	/// A number of members `_incoming` just joined the set and replaced some `_outgoing` ones. The
	/// new set is thus given by `sorted_new` and **must be sorted**.
	///
	/// NOTE: This is the only function that needs to be implemented in `ChangeMembers`.
	///
	/// This resets any previous value of prime.
	fn change_members_sorted(sorted_new: &[Did]);

	/// Set the new members; they **must already be sorted**. This will compute the diff and use it
	/// to call `change_members_sorted`.
	///
	/// This resets any previous value of prime.
	fn set_members_sorted(new_members: &[Did]) {
		Self::change_members_sorted(new_members);
	}

	/// Set the prime member.
	fn set_prime(_prime: Option<Did>) {}

	/// Get the current prime.
	fn get_prime() -> Option<Did> {
		None
	}
}

impl ChangeMembers for () {
	fn change_members(_: Vec<Did>) {}
	fn change_members_sorted(_: &[Did]) {}
	fn set_members_sorted(_: &[Did]) {}
	fn set_prime(_: Option<Did>) {}
}

/// Check permission level of a validator
pub trait IsValidator {
	/// Is Validator
	fn is_validator(who: &Did) -> bool;

	/// Check if given did has global permission level
	fn is_validator_global(did: &Did) -> bool;

	/// Get region of given validator
	/// Basically, gets sub array between two colons
	fn get_region(did: Did) -> DidRegion;
}

impl IsValidator for () {
	fn is_validator(_who: &Did) -> bool {
		false
	}

	/// Check if given did has global permission level
	fn is_validator_global(_did: &Did) -> bool {
		false
	}

	fn get_region(did: Did) -> DidRegion {
		let colon = 58;
		let index = did.iter().position(|&x| x == colon).unwrap_or_default();
		let did = did.split_at(index + 1).1;
		let index = did.iter().position(|&x| x == colon).unwrap_or_default();
		let region = did.split_at(index).0;
		dynamic_to_fixed_array::<20>(region)
	}
}


/// Trait to handle VC Updation
pub trait UpdateVC {
	/// Add VC to store
	fn add_vc(vc_hex: VCHex, vc_id: VCid) -> DispatchResult;
}

/// Impl Update VC for ()
impl UpdateVC for () {
	/// Add VC to store
	fn add_vc(_vc_hex: VCHex, _vc_id: VCid) -> DispatchResult {
		Err("Not Implemented".into())
	}
}

/// Trait to Handle Pallet Dynamics
pub trait ExtrinsicResolve {
	/// Check if an extrinsic is whitelisted
	fn is_extrinsic_whitelisted(_pallet_name: PalletName, _function_name: FunctionName) -> bool;
	
	/// Check if an extrinsic is restricted
	fn is_extrinsic_restricted(_pallet_name: PalletName, _function_name: FunctionName) -> bool;

	/// Restrict an extrinsic
	fn restrict_extrinsic(_pallet_name: PalletName, _function_name: FunctionName) -> DispatchResult;

	/// Empty the whole RestrictedExtrinsics storage
	fn remove_all_restricted(_pallet_name: PalletName) -> DispatchResult;
}

/// Impl ExtrinsicResolve for ()
impl ExtrinsicResolve for () {
	/// Check if an extrinsic is whitelisted
	fn is_extrinsic_whitelisted(_: PalletName, _: FunctionName) -> bool{
		true
	}
	
	/// Check if an extrinsic is restricted
	fn is_extrinsic_restricted(_: PalletName, _: FunctionName) -> bool {
		true
	}

	/// Restrict an extrinsic
	fn restrict_extrinsic(_: PalletName, _: FunctionName) -> DispatchResult {
		Err("Not Implemented".into())
	}

	/// Remove an extrinsic from restricted list
	fn remove_all_restricted(_: PalletName) -> DispatchResult {
		Err("Not Implemented".into())
	}
}

///
pub trait SyncBalances {
///
    fn on_reserve_balance(
        did: Did,
        reservable_balance: Balance,
    ) -> DispatchResult;
///
    fn on_unreserve_balance(
        did: Did,
        reservable_balance: Balance,
    ) -> DispatchResult;
///
    fn on_slash_reserved(
        did: Did,
        reservable_balance: Balance,
        beneficiary_did: Did,
    ) -> DispatchResult;
}

impl SyncBalances for () {
    fn on_reserve_balance(
        _did: Did,
        _reservable_balance: Balance,
    ) -> DispatchResult {
        Ok(()) 
    }

    fn on_unreserve_balance(
        _did: Did,
        _reservable_balance: Balance,
    ) -> DispatchResult {
        Ok(())
    }

    fn on_slash_reserved(
        _did: Did,
        _reservable_balance: Balance,
        _beneficiary_did: Did,
    ) -> DispatchResult {
        Ok(())
    }
}


// Helper Functions


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
