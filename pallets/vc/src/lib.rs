#![cfg_attr(not(feature = "std"), no_std)]

/// The VC pallet issues list of VCs that empowers any user to perfom permitted operations.
use frame_support::{
	codec::{Decode, Encode},
	ensure, fail,
	pallet_prelude::DispatchResult,
	traits::{ConstU32, EnsureOrigin},
	BoundedVec,
};


use frame_system::{self, ensure_signed};
use metamui_primitives::{
	traits::{DidResolve, IsMember, IsValidator, MultiAddress, HasPublicKey, HasDid},
	types::{
		MaxIssuers, PrivateDidVC, PublicDidVC, ResetPubKeyVC, SlashMintTokens, TokenTransferVC,
		TokenVC, TokenchainAuthVC, VCType, VC, DidRegion, IssueTokenVC,
	},
	Did as Identifier, VCHex, VCid,
};
use sp_core::sr25519;
use sp_runtime::{
	traits::{BlakeTwo256, Hash, Verify},
	DispatchError,
};
use sp_std::prelude::*;
use sr25519::Signature;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod impls;

pub mod types;
pub use crate::types::*;
use serde_big_array::big_array;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Origin from which approvals must come.
		type ApproveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// Ensure Caller Is Council Member
		type IsCouncilMember: IsMember;

		/// Ensure Caller Is Validator
		type IsValidator: IsValidator;

		/// Resolve Did from account Id
		type DidResolution: DidResolve<Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Given VC is validated
		VCValidated { vcid: VCid },
		/// Updated VC status flag
		VCStatusUpdated { vcid: VCid, vcstatus: IsVCActive },
		/// Signature Added By Issuer
		SignatureAdded { vcid: VCid },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unable to decode the VC
		InvalidVC,
		/// VC properties verification failed
		VCPropertiesNotVerified,
		/// The given VCId does not exist on chain
		VCDoesNotExist,
		/// The operation is permitted only for issuer & validator
		NotAValidatorNorIssuer,
		/// Linked VC does not exist
		LinkedVCNotFound,
		/// The given type of VC should be signed by the owner of respective TokenVC
		VCNotSignedByTokenVCOwner,
		/// VC Already Exist
		VCAlreadyExists,
		/// Either signature is invalid or signer is not a valid issuer
		InvalidSignature,
		/// The issuer has already approved the VC
		DuplicateSignature,
		/// Duplicate Issuers found on VC
		DuplicateIssuers,
		/// Invalid currency code
		InvalidCurrencyCode,
		/// The caller is not a council member
		NotACouncilMember,
		/// The caller is not a validator
		InvalidDidRegion,
		/// Did doesn't exist on chain
		DidDoesNotExist,
		/// Did doesn't exist on chain
		DIDAlreadyExists,
		/// Public key in the DidVC is already used
		PublicKeyRegistered,
		/// Issuer should be one of the Validator
		IssuerNotValidator,
		/// Issuer not a council member
		IssuerNotACouncilMember,
		/// Wrong Did VC provided
		WrongDidVC,
		/// VC Not signed by issuer of Original VC
		VCNotSignedByOriginalIssuer,
		/// Not Validator
		NotAValidator,
		/// Overflow Vec
		Overflow,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Storages Used By VC Pallet

	/// The Map for storing VC information
	#[pallet::storage]
	pub(super) type VCs<T: Config> =
		StorageMap<_, Blake2_128Concat, VCid, VC<T::Hash>, OptionQuery>;

	/// Map to enable lookup from VC Owner to VCids
	#[pallet::storage]
	pub(super) type Lookup<T: Config> =
		StorageMap<_, Blake2_128Concat, Identifier, VCIdList, ValueQuery>;

	/// Map to enable reverse lookup from VCid to VC Owner
	#[pallet::storage]
	pub(super) type RLookup<T: Config> =
		StorageMap<_, Blake2_128Concat, VCid, Identifier, ValueQuery>;

	/// The Map for storing VC Status and Block number when VC was stored
	#[pallet::storage]
	pub(super) type VCHistory<T: Config> =
		StorageMap<_, Blake2_128Concat, VCid, (IsVCActive, T::BlockNumber), OptionQuery>;

	/// Map for VC Id and Approvers(Issuers) list
	#[pallet::storage]
	pub(super) type VCApproverList<T: Config> =
		StorageMap<_, Blake2_128Concat, VCid, VCIdList, ValueQuery>;

	/// Map for Issuers to VC Id
	#[pallet::storage]
	pub(super) type VCIdLookup<T: Config> =
		StorageMap<_, Blake2_128Concat, Identifier, VCIdList, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Genesis Configs
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_vcs: Vec<InitialVCs>,
		pub phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { initial_vcs: Default::default(), phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_vcs(&self.initial_vcs);
		}
	}

	/// Defining extrinsics of the pallet
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stores VC to the storage after validating the input
		#[pallet::weight(10_000)]
		pub fn store(origin: OriginFor<T>, vc_hex: VCHex) -> DispatchResult {
			// Extracting vc from encoded vc byte array
			let vc: VC<T::Hash> = Self::decode_vc(&vc_hex)?;

			Self::validate_vc(&vc)?;

			// Validate Origin based on VC Type
			match vc.vc_type {
				VCType::TokenVC | VCType::TokenchainAuthVC | VCType::IssueTokenVC => {
					// Check if the origin of the call is approved orgin or not
					<T as Config>::ApproveOrigin::ensure_origin(origin)?;

					// Check currency code is valid
					Self::validate_currency_code(&vc)?;
				},

				VCType::SlashTokens |
				VCType::MintTokens |
				VCType::TokenTransferVC |
				VCType::PrivateDidVC |
				VCType::PublicDidVC |
				VCType::GenericVC => {
					let sender = ensure_signed(origin)?;

					// Check currency code is valid
					Self::validate_currency_code(&vc)?;

					// Validating caller of above VC types
					Self::validate_by_vc_type(&vc, &sender)?;
				},

				VCType::ResetPubKeyVC => {
					match <T as Config>::ApproveOrigin::ensure_origin(origin.clone()) {
						// If Origin is ApproveOrigin, then the VC should be issued by collective
						// members. It will be used for rotating Validator or Sudo's DID
						Ok(_) => Self::validate_reset_auth_pubkey_vc(&vc)?,

						// If Origin is not ApproveOrigin, then VC issuer should be a validator
						// And Did should not be of a validator
						Err(_) => {
							let sender = ensure_signed(origin)?;
							Self::validate_by_vc_type(&vc, &sender)?;
						},
					}
				},
			}

			// Generating vc_id from vc to emit in the event
			let vc_id: VCid = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();

			// Storing VC
			Self::store_vc(vc.owner, vc, vc_id)?;

			Self::deposit_event(Event::VCValidated { vcid: vc_id });

			Ok(())
		}

		/// Update signature of vc_hash to update status as Active or Inactive
		///
		/// This function will set vc status as Active only if all issuers's signatures are verified
		#[pallet::weight(10_000)]
		pub fn add_signature(origin: OriginFor<T>, vc_id: VCid, sign: Signature) -> DispatchResult {
			// Ensure caller is signed account
			let senders_acccount_id = ensure_signed(origin)?;

			let vc = Self::get_vc(vc_id)?;

			// Validate Caller is Issuer of the VC
			Self::validate_caller(&senders_acccount_id, &vc)?;

			Self::update_signature(vc_id, vc, sign)?;

			Self::deposit_event(Event::SignatureAdded { vcid: vc_id });

			Ok(())
		}

		/// Update status of vc_hash wheather it is active or inactive
		#[pallet::weight(10_000)]
		pub fn update_status(
			origin: OriginFor<T>,
			vc_id: VCid,
			vc_status: IsVCActive,
		) -> DispatchResult {
			// Ensure caller is signed account
			let senders_acccount_id = ensure_signed(origin)?;

			let vc = Self::get_vc(vc_id)?;

			Self::validate_caller(&senders_acccount_id, &vc)?;

			Self::update_vc_status(vc_id, vc_status)?;

			Self::deposit_event(Event::VCStatusUpdated { vcid: vc_id, vcstatus: vc_status });

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Genesis Functions
	
	/// Called during genesis to initialise VC store
	fn initialize_vcs(initial_vcs: &Vec<InitialVCs>) {
		for initial_vc in initial_vcs.iter() {
			let vc_id = &initial_vc.vc_id;
			let vc_hex = &initial_vc.vc_hex;

			let vc = Self::decode_vc::<VC<T::Hash>>(&vc_hex).unwrap();

			let mut vcids = Lookup::<T>::get(vc.owner);
			// TODO: handle Error case
			vcids.try_push(*vc_id).expect("Overflow VC");

			Lookup::<T>::insert(vc.owner, vcids);
			RLookup::<T>::insert(vc_id, vc.owner);

			VCs::<T>::insert(vc_id, vc);
		}
	}


	// Getter Functions

	/// Get VC from Storage using VC Id
	fn get_vc(vc_id: VCid) -> Result<VC<T::Hash>, DispatchError> {
		Ok(VCs::<T>::get(vc_id).ok_or_else(|| Error::<T>::VCDoesNotExist)?)
	}

	/// Get VC History from Storage using VC Id
	fn get_vc_history(vc_id: VCid) -> Result<(IsVCActive, T::BlockNumber), DispatchError> {
		Ok(VCHistory::<T>::get(vc_id).ok_or_else(|| Error::<T>::VCDoesNotExist)?)
	}

	/// Check if VC is signed by all issuers
	pub fn check_vc_status(vc: &VC<T::Hash>) -> Result<IsVCActive, DispatchError> {
		// Ensure the VC has all issuers' signature
		if vc.issuers.len() != vc.signatures.len() {
			return Ok(false)
		} else {
			let mut verified_count: usize = 0;
			for issuer in vc.issuers.iter() {
				ensure!(
					<T as pallet::Config>::DidResolution::did_exists(MultiAddress::Did(*issuer)),
					Error::<T>::DidDoesNotExist,
				);
				let public_key =
					<T as pallet::Config>::DidResolution::get_public_key(issuer).unwrap();

				for signature in vc.signatures.iter() {
					if signature.verify(vc.hash.as_ref(), &public_key) {
						verified_count += 1;
					}
				}
			}
			if verified_count != vc.signatures.len() {
				return Ok(false)
			}
		}

		Ok(true)
	}


	// Validate Params Functions

	/// Check if the caller is issuer or a validator
	fn validate_caller(
		caller_acccount_id: &T::AccountId,
		vc: &VC<T::Hash>,
	) -> Result<(), DispatchError> {
		// Check if sender's did exists on chain
		let senders_did = <T as pallet::Config>::DidResolution::get_did(&caller_acccount_id);
		ensure!(senders_did.is_some(), Error::<T>::DidDoesNotExist);
		let senders_did = senders_did.unwrap();

		ensure!(
			vc.issuers.contains(&senders_did) || <T as pallet::Config>::IsValidator::is_validator(&senders_did),
			Error::<T>::NotAValidatorNorIssuer
		);

		Ok(())
	}

	/// Validates VC
	/// If hash is correct, No duplicate Issuers
	fn validate_vc(vc: &VC<T::Hash>) -> Result<(), DispatchError> {
		// Issuer’s Did validity will be checked in the validate_approvers()
		// Check if owner’s did is registered or not
		// In Case of DID VC, the did won't exist
		if vc.vc_type != VCType::PublicDidVC && vc.vc_type != VCType::PrivateDidVC {
			ensure!(
				<T as pallet::Config>::DidResolution::did_exists(MultiAddress::Did(vc.owner)),
				Error::<T>::DidDoesNotExist,
			);
		}

		// checking for duplicate issuers
		let mut issuers = vc.issuers.clone();
		let org_issuer_count = issuers.len();
		issuers.sort();
		Self::deduplicate_issuers(&mut issuers)?;
		ensure!(org_issuer_count == issuers.len(), Error::<T>::DuplicateIssuers);

		// Verifying hash of VC is correct
		// In case of generic VC- vc property only contains id, not the content itself
		if vc.vc_type != VCType::GenericVC {
			let hash = T::Hashing::hash_of(&(&vc.vc_type, &vc.vc_property, &vc.owner, &vc.issuers));
			// ensure the valid hash
			ensure!(vc.hash.eq(&hash), Error::<T>::VCPropertiesNotVerified);
		}

		Ok(())
	}

	/// Validates VC By VC Type
	/// Validates the VC property according to the VC Type
	fn validate_by_vc_type(vc: &VC<T::Hash>, sender: &T::AccountId) -> Result<(), DispatchError> {
		// Check If Sender's Did Exists
		let sender_did = <T as pallet::Config>::DidResolution::get_did(&sender);
		ensure!(sender_did.is_some(), Error::<T>::DidDoesNotExist);
		let sender_did = &sender_did.unwrap();

		// Validate VC
		match vc.vc_type {
			// Check if the one of the issuer of the VC is a Owner of TokenVC
			VCType::SlashTokens | VCType::MintTokens => {
				let slash_or_mint: SlashMintTokens =
					Self::decode_vc::<SlashMintTokens>(&vc.vc_property)?;
				let token_vc = VCs::<T>::get(&slash_or_mint.vc_id).ok_or_else(|| Error::<T>::LinkedVCNotFound)?;
				ensure!(
					vc.issuers.contains(&token_vc.owner),
					Error::<T>::VCNotSignedByTokenVCOwner
				);
			},

			// Check if the one of the issuer of the VC is a Owner of TokenVC
			VCType::TokenTransferVC => {
				// derive Transfer Tokens
				let transfer_tokens: TokenTransferVC =
					Self::decode_vc::<TokenTransferVC>(&vc.vc_property)?;
				let token_vc = VCs::<T>::get(&transfer_tokens.vc_id).ok_or_else(|| Error::<T>::LinkedVCNotFound)?;
				ensure!(
					vc.issuers.contains(&token_vc.owner),
					Error::<T>::VCNotSignedByTokenVCOwner
				);
			},

			// Check if the Issuer of this VC is a valid validator of the respective region
			VCType::PrivateDidVC => {
				let vc_property = Self::decode_vc::<PrivateDidVC>(&vc.vc_property)?;
				Self::validate_did_vc(&vc, vc_property, &sender_did)?;
			},

			// Check if the Issuer of this VC is a valid validator of the respective region
			VCType::PublicDidVC => {
				let vc_property = Self::decode_vc::<PublicDidVC>(&vc.vc_property)?;
				Self::validate_did_vc(&vc, vc_property, &sender_did)?;
			},

			// Check if the Issuer of this VC is a valid validator of the respective region
			VCType::ResetPubKeyVC => {
				let vc_property = Self::decode_vc::<ResetPubKeyVC>(&vc.vc_property)?;
				Self::validate_reset_pubkey_vc(&vc, &vc_property, &sender_did)?;
			},

			VCType::GenericVC => {
				// ensure the caller is a council member account
				ensure!(
					<T as pallet::Config>::IsCouncilMember::is_member(sender_did),
					Error::<T>::NotACouncilMember
				);
			},

			_ => {},
		}

		Ok(())
	}

	/// Validate ResetPubKeyVC
	/// Checks if all issuers are council members and verifies properties
	fn validate_reset_auth_pubkey_vc(
		vc: &VC<T::Hash>,
	) -> Result<(), DispatchError> {
		let vc_property =
			Self::decode_vc::<ResetPubKeyVC>(&vc.vc_property)?;
		let is_issuer_council = vc
			.issuers
			.iter()
			.all(|issuer| <T as pallet::Config>::IsCouncilMember::is_member(&issuer));

		ensure!(is_issuer_council, Error::<T>::IssuerNotACouncilMember);

		ensure!(vc_property.old_public_key.is_some(), Error::<T>::InvalidVC);

		let public_key = T::DidResolution::get_public_key(&vc_property.did);
		ensure!(public_key.is_some(), Error::<T>::DidDoesNotExist);

		ensure!(
			*vc_property.old_public_key.as_ref().unwrap() == public_key.unwrap(),
			Error::<T>::InvalidVC,
		);

		Ok(())
	}

	fn validate_did_vc(
		vc: &VC<T::Hash>,
		vc_property: impl HasDid + HasPublicKey,
		sender_did: &Identifier,
	) -> Result<(), DispatchError>  {
		let public_key = vc_property.public_key();
		let account_id = T::AccountId::decode(&mut &public_key[..]).unwrap();
		ensure!(vc.owner.eq(&vc_property.did()), Error::<T>::InvalidVC);
		ensure!(!<T as pallet::Config>::DidResolution::did_exists(MultiAddress::Did(vc_property.did())), Error::<T>::DIDAlreadyExists);

		// ensure the caller is a council member account
		let did_region: DidRegion = <T as pallet::Config>::IsValidator::get_region(vc_property.did());
		let allowed_regions = <T as pallet::Config>::DidResolution::get_regions();
		ensure!(
			allowed_regions.contains(&did_region),
			Error::<T>::InvalidDidRegion,
		);
		ensure!(vc.issuers.contains(sender_did), Error::<T>::IssuerNotValidator);
		ensure!(
			!<T as pallet::Config>::DidResolution::did_exists(MultiAddress::Id(account_id)),
			Error::<T>::PublicKeyRegistered,
		);
		Ok(())
	}

	fn validate_reset_pubkey_vc(
		vc: &VC<T::Hash>,
		vc_property: &ResetPubKeyVC,
		sender_did: &Identifier,
	) -> Result<(), DispatchError>  {
		ensure!(
			!<T as pallet::Config>::IsValidator::is_validator(&vc_property.did),
			Error::<T>::NotAValidator,
		);
		ensure!(vc.issuers.contains(sender_did), Error::<T>::IssuerNotValidator);
		ensure!(
			T::DidResolution::did_exists(MultiAddress::Did(vc_property.did)),
			Error::<T>::DidDoesNotExist,
		);
		ensure!(vc_property.old_public_key.is_none(), Error::<T>::InvalidVC);

		// If DID VC exists for this did, then ensure original issuer of the DID is one of
		// the issuer Else valid validator from the respective region can issue VC
		if vc_property.vc_id.is_some() {
			let did_vc = VCs::<T>::get(&vc_property.vc_id.unwrap()).ok_or_else(|| Error::<T>::LinkedVCNotFound)?;
			let did_vc_property =
				Self::decode_vc::<ResetPubKeyVC>(&vc.vc_property)?;
			ensure!(did_vc_property.did == vc_property.did, Error::<T>::WrongDidVC);
			let is_original_issuer =
				did_vc.issuers.iter().any(|issuer| vc.issuers.contains(&issuer));
			ensure!(is_original_issuer, Error::<T>::VCNotSignedByOriginalIssuer);
		} else {
			let did_region =
				<T as pallet::Config>::IsValidator::get_region(vc_property.did);
			let allowed_regions = <T as pallet::Config>::DidResolution::get_regions();
			ensure!(
				allowed_regions.contains(&did_region),
				Error::<T>::InvalidDidRegion,
			);
		};
		Ok(())
	}

	// Checks Currency code is valid, not empty and all CAPS
	fn validate_currency_code(vc: &VC<T::Hash>) -> Result<(), DispatchError> {
		let mut currency_code: Vec<u8>;
		match vc.vc_type {
			VCType::TokenVC => {
				let vc_property: TokenVC = Self::decode_vc::<TokenVC>(&vc.vc_property)?;
				currency_code = vc_property.currency_code.into();
			},
			VCType::SlashTokens | VCType::MintTokens => {
				let vc_property: SlashMintTokens =
					Self::decode_vc::<SlashMintTokens>(&vc.vc_property)?;
				currency_code = vc_property.currency_code.into();
			},
			VCType::TokenTransferVC => {
				let vc_property: TokenTransferVC =
					Self::decode_vc::<TokenTransferVC>(&vc.vc_property)?;
				currency_code = vc_property.currency_code.into();
			},
			VCType::TokenchainAuthVC => {
				let vc_property: TokenchainAuthVC =
					Self::decode_vc::<TokenchainAuthVC>(&vc.vc_property)?;
				currency_code = vc_property.currency_code.into();
			},
			VCType::IssueTokenVC => {
				let vc_property: IssueTokenVC =
					Self::decode_vc::<IssueTokenVC>(&vc.vc_property)?;
				currency_code = vc_property.currency_code.into();
			},

			_ => return Ok(()),
		}
		currency_code.retain(|val| *val != 0);
		ensure!(!currency_code.contains(&0), Error::<T>::InvalidCurrencyCode);
		for &cc in currency_code.iter() {
			ensure!(cc.is_ascii_uppercase(), Error::<T>::InvalidCurrencyCode);
		}

		Ok(())
	}

	/// Checks if the given signature if from one of the issuer of VC else throws Error
	/// Called when a new signature is added to the VC
	fn validate_signature(vc: &VC<T::Hash>, sign: Signature, vc_id: VCid) -> Result<VCIdList, DispatchError> {
		let mut is_sign_valid = false;
		let mut vc_approvers = VCApproverList::<T>::get(vc_id);
		for issuer in vc.issuers.iter() {
			ensure!(
				<T as pallet::Config>::DidResolution::did_exists(MultiAddress::Did(*issuer)),
				Error::<T>::DidDoesNotExist
			);
			let public_key = <T as pallet::Config>::DidResolution::get_public_key(&issuer).unwrap();

			if sign.verify(vc.hash.as_ref(), &public_key) {
				if vc_approvers.contains(&issuer) {
					fail!(Error::<T>::DuplicateSignature);
				}
				match vc_approvers.try_push(*issuer) {
					Err(_) => fail!(Error::<T>::Overflow),
					Ok(_) => (),
				};
				is_sign_valid = true;
			}
		}
		if !is_sign_valid {
			fail!(Error::<T>::InvalidSignature);
		}
		Ok(vc_approvers)
	}

	/// Checks the issuers of the VC who have signed and adds them to approved issuers list
	/// Throws error if there is duplicate signature or if the signature is not from any of the
	/// issuer from the VC
	/// Called when VC is stored or synced
	fn validate_approvers(vc_id: VCid, vc: &VC<T::Hash>) -> Result<VCIdList, DispatchError> {
		let mut vc_approvers = VCApproverList::<T>::get(vc_id);
		let signatures = vc.signatures.clone();
		// Check approved signatures
		for i in 0..signatures.len() {
			let sign = &signatures[i];
			let mut is_sign_valid = false;
			for issuer in vc.issuers.iter() {
				ensure!(
					<T as pallet::Config>::DidResolution::did_exists(MultiAddress::Did(*issuer)),
					Error::<T>::DidDoesNotExist
				);
				let public_key =
					<T as pallet::Config>::DidResolution::get_public_key(issuer).unwrap();

				if sign.verify(vc.hash.as_ref(), &public_key) {
					if vc_approvers.contains(&issuer) {
						fail!(Error::<T>::DuplicateSignature);
					}
					is_sign_valid = true;
					match vc_approvers.try_push(*issuer) {
						Err(_) => fail!(Error::<T>::Overflow),
						Ok(_) => (),
					};
				}
			}
			if !is_sign_valid {
				fail!(Error::<T>::InvalidSignature);
			}
		}
		Ok(vc_approvers)
	}


	// Storage Related Functions

	/// Store VC after Validating
	fn store_vc(
		identifier: Identifier,
		mut vc: VC<T::Hash>,
		vc_id: VCid,
	) -> Result<(), DispatchError> {
		let current_block_no = <frame_system::Pallet<T>>::block_number();

		// Check if vc already exists
		ensure!(!RLookup::<T>::contains_key(&vc_id), Error::<T>::VCAlreadyExists);

		let vc_approvers = Self::validate_approvers(vc_id, &vc)?;

		let vc_status = Self::check_vc_status(&vc)?;

		// Setting is_vc_active
		vc.is_vc_active = vc_status;
		
		// Check to make sure it won't overflow
		ensure!(
			vc.issuers.len() < <MaxIssuers as frame_support::traits::Get<u32>>::get() as usize,
			Error::<T>::Overflow,
		);
		let mut owner_vc_ids: VCIdList;
		if Lookup::<T>::contains_key(&identifier) {
			owner_vc_ids = Lookup::<T>::get(identifier);
			ensure!(
				owner_vc_ids.len() < <MaxVecSize as frame_support::traits::Get<u32>>::get() as usize,
				Error::<T>::Overflow,
			);
		} else {
			owner_vc_ids = Default::default();
		}
		

		for did in vc.issuers.iter() {
			let mut issuer_vc_ids = VCIdLookup::<T>::get(did);
			match issuer_vc_ids.try_push(vc_id) {
				Err(_) => (),
				Ok(_) => (),
			};
			VCIdLookup::<T>::insert(did, issuer_vc_ids);
		}
		
		// Update owner vc list
		match owner_vc_ids.try_push(vc_id) {
			Err(_) => fail!(Error::<T>::Overflow),
			Ok(_) => (),
		};
		Lookup::<T>::insert(identifier, owner_vc_ids);

		// Insert VC to Storage
		VCs::<T>::insert(vc_id, vc.clone());
		VCApproverList::<T>::insert(vc_id, vc_approvers);
		RLookup::<T>::insert(vc_id, identifier);
		VCHistory::<T>::insert(vc_id, (vc_status, current_block_no));

		Ok(())
	}

	/// Sync VC from other chain, the vc should be active
	fn on_sync_vc(
		identifier: Identifier,
		vc: VC<T::Hash>,
		vc_id: VCid,
	) -> Result<(), DispatchError> {
		Self::store_vc(identifier, vc, vc_id)
	}

	/// Update VC status and VC history on storage
	fn update_vc_status(vc_id: VCid, status: IsVCActive) -> Result<(), DispatchError> {
		let mut vc = Self::get_vc(vc_id)?;

		if vc.is_vc_active == status {
			return Ok(());
		}

		vc.is_vc_active = status;

		let vc_history = Self::get_vc_history(vc_id)?;

		VCHistory::<T>::insert(vc_id, (status, vc_history.1));
		VCs::<T>::insert(vc_id, vc);

		Ok(())
	}

	/// Update signature to VC and updating Status if signatures added by all Issuers
	fn update_signature(
		vc_id: VCid,
		mut vc: VC<T::Hash>,
		sign: Signature,
	) -> Result<(), DispatchError> {
		// Validate if the siganature is from Issuer
		let vc_approvers = Self::validate_signature(&vc, sign.clone(), vc_id)?;

		// Add Signature
		match vc.signatures.try_push(sign) {
			Err(_) => fail!(Error::<T>::Overflow),
			Ok(_) => (),
		};

		// Setting is_vc_active
		let status = Self::check_vc_status(&vc)?;

		if vc.is_vc_active != status {
			vc.is_vc_active = status;
	
			let vc_history = Self::get_vc_history(vc_id)?;

			VCHistory::<T>::insert(vc_id, (status, vc_history.1));

			Self::deposit_event(Event::VCStatusUpdated { vcid: vc_id, vcstatus: status });
		}

		VCApproverList::<T>::insert(vc_id, vc_approvers);

		VCs::<T>::insert(vc_id, vc);

		Ok(())
	}

	/// Update vc's is_used flag
	pub fn update_vc_used(vc_id: VCid, is_vc_used: Option<bool>) -> Result<(), DispatchError> {
		let mut vc = Self::get_vc(vc_id)?;
		vc.is_vc_used = is_vc_used.unwrap_or(true);
		VCs::<T>::insert(vc_id, vc);
		Ok(())
	}


	// Helper Functions

	/// Remove Duplicate issuers from Vec of Issuers
	fn deduplicate_issuers(
		issuers: &mut IssuersList,
	) -> Result<(), DispatchError> {
		let mut deduped_issuers: IssuersList = Default::default();
		for issuer in issuers.iter() {
			if !deduped_issuers.contains(issuer) {
				match deduped_issuers.try_push(*issuer) {
					Err(_) => fail!(Error::<T>::Overflow),
					Ok(_) => (),
				};
			}
		}
		*issuers = deduped_issuers;
		Ok(())
	}
	
	/// Decoding VC and VC Property from encoded bytes
	pub fn decode_vc<E: codec::Decode>(mut vc_bytes: &[u8]) -> Result<E, DispatchError> {
		let vc: E = match Decode::decode(&mut vc_bytes) {
			Ok(vc) => vc,
			Err(_) => fail!(Error::<T>::InvalidVC),
		};
		Ok(vc)
	}

}
