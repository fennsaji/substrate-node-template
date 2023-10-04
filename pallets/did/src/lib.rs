#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
pub use serde;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod types;

mod impls;
pub use crate::impls::*;

#[frame_support::pallet]
pub mod pallet {
	use crate::types::*;
	use codec::Decode;
	use cumulus_primitives_core::ParaId;
	use frame_support::{pallet_prelude::{DispatchResult, *}, fail};
	use frame_system::{self, pallet_prelude::*};
	use sp_std::vec::Vec;

	use metamui_primitives::{
		traits::VCResolve,
		types::{DidType, PublicDidVC, PublicKey, AllowedRegionsVec, DidRegion, MaxAllowedRegions},
		VCid,
	};

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Validator Origin
		type ValidatorOrigin: EnsureOrigin<Self::Origin>;
		/// Maximum number of key changes by an account
		type MaxKeyChanges: Get<u32>;
		/// On Did update
		type OnDidUpdate: DidUpdated;
		/// Trait to resolve VC
		type VCResolution: VCResolve<Self::Hash>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// the map for storing did information
	#[pallet::storage]
	pub type DIDs<T: Config> =
		StorageMap<_, Blake2_128Concat, Did, (DIdentity, T::BlockNumber), OptionQuery>;

	// map to enable lookup from did to account id
	#[pallet::storage]
	pub type Lookup<T: Config> = StorageMap<_, Blake2_128Concat, Did, T::AccountId, OptionQuery>;

	// map to enable reverse lookup
	#[pallet::storage]
	pub type RLookup<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Did, OptionQuery>;

	// map to store history of key rotation
	#[pallet::storage]
	pub type PrevKeys<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Did,
		BoundedVec<(T::AccountId, T::BlockNumber), T::MaxKeyChanges>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type TypeChangeHistory<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Did,
		BoundedVec<(DidType, T::BlockNumber), MaxTypeChanges>,
		OptionQuery,
	>;

	#[pallet::storage]
	pub type AllowedRegions<T: Config> = StorageValue<_, BoundedVec<DidRegion, MaxAllowedRegions>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_dids: Vec<DIdentity>,
		pub allowed_regions: AllowedRegionsVec,
		pub phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { initial_dids: Default::default(), allowed_regions: Default::default(), phantom: Default::default() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_dids(&self.initial_dids, &self.allowed_regions);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A DID has been created
		DidCreated { did: Did },
		/// A DID has been removed
		DidRemoved { did: Did },
		/// DID key have been rotated
		DidKeyUpdated { did: Did },
		/// DID Metadata has been updated
		DidMetadataUpdated { did: Did },
		/// DID type has been updated
		DidTypeUpdated { did: Did },
		/// DID Metadata has been updated
		DidSynced { did: Did, para_id: ParaId },
		/// Region Added
		RegionAdded{ region: DidRegion },
		/// Region Removed
		RegionRemoved{ region: DidRegion },

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The given DID already exists on chain
		DIDAlreadyExists,
		/// Invalid DID, either format or length is wrong
		InvalidDid,
		/// PublicKey already linked to another DID on chain
		PublicKeyRegistered,
		/// The given DID does not exist on chain
		DIDDoesNotExist,
		/// The operation is restricted to the validator only
		NotAValidator,
		/// The given VCId does not exist on chain
		VCDoesNotExist,
		/// The entered VCId is not eligible to create Did
		InvalidVC,
		/// The did already has the requested type
		TypeAlreadySame,
		/// The did type already has been changed maximum times
		MaxTimesTypeChanged,
		/// Vec Overflow
		Overflow,
		/// Region was already added
		RegionAlreadyExists,
		/// Region not found
		RegionNotFound,
		/// Max No of Region added
		MaxAllowedRegionsExceeded,
	}

	#[pallet::call]
	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	impl<T: Config> Pallet<T> {
		/// Adds a DID on chain, where
		/// origin - the origin of the transaction
		/// vc_id - The id of the VC that is authorized to create this DID
		/// para_id - The id of the parachain if a did needs to be created there
		#[pallet::weight(10_000 + T::DbWeight::get().writes(5))]
		pub fn create_private(
			origin: OriginFor<T>,
			vc_id: VCid,
			para_id: Option<ParaId>,
		) -> DispatchResult {
			// Ensure Signed
			ensure_signed(origin)?;

			let vc = Self::get_vc(vc_id)?;

			// Verify if the VC is valid
			ensure!(Self::validate_vc(vc.clone(), VCType::PrivateDidVC), Error::<T>::InvalidVC);

			// Decode the VC for getting the metadata and public key
			let vc_property = T::VCResolution::decode_vc::<PrivateDidVC>(&vc.vc_property)?;

			// Create the did
			Self::do_store_private_did(vc_property.public_key, vc_property.did)?;

			// Set the vc to used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			// Cache DID to Parachain if Para Id is provided
			if let Some(para_id) = para_id {
				T::OnDidUpdate::on_new_did(
					para_id,
					vc_property.public_key,
					vc_property.did,
					DidType::Private,
				);
			}

			// Emit an event.
			Self::deposit_event(Event::DidCreated { did: vc_property.did });

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Adds a DID on chain, where
		/// origin - the origin of the transaction
		/// vc_id - The id of the VC that is authorized to create this DID
		/// para_id - The id of the parachain if a did needs to be created there
		#[pallet::weight(10_000 + T::DbWeight::get().writes(5))]
		pub fn create_public(
			origin: OriginFor<T>,
			vc_id: VCid,
			para_id: Option<ParaId>,
		) -> DispatchResult {
			// Ensure Signed
			ensure_signed(origin)?;

			let vc = Self::get_vc(vc_id)?;

			// Verify if the vc is valid
			ensure!(Self::validate_vc(vc.clone(), VCType::PublicDidVC), Error::<T>::InvalidVC);

			// Decode the VC for getting the registration number and company name
			let vc_property = T::VCResolution::decode_vc::<PublicDidVC>(&vc.vc_property)?;

			// Create the did
			Self::do_store_public_did(
				vc_property.public_key,
				vc_property.did,
				vc_property.registration_number.clone(),
				vc_property.company_name.clone(),
			)?;

			// Set the vc to used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			// Cache DID to Parachain if Para Id is provided
			if let Some(para_id) = para_id {
				T::OnDidUpdate::on_new_did(
					para_id,
					vc_property.public_key,
					vc_property.did,
					DidType::Public,
				);
			}

			// Emit an event.
			Self::deposit_event(Event::DidCreated { did: vc_property.did });

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// Sync did from relay chain to parachain
		/// origin - the origin of the transaction
		/// identifier - public unique identifier for the DID
		/// para_id - The id of the parachain if a did needs to be created there
		#[pallet::weight(10_000)]
		pub fn cache_did(
			origin: OriginFor<T>, 
			identifier: Did, 
			para_id: ParaId
		) -> DispatchResult {
			// Check if origin is a from a validator
			T::ValidatorOrigin::ensure_origin(origin)?;

			Self::do_cache_did(&identifier, para_id)?;

			// Emit an event.
			Self::deposit_event(Event::DidSynced { did: identifier, para_id });

			Ok(())
		}

		/// Removes a DID from chain storage, where
		/// origin - the origin of the transaction
		/// identifier - public unique identifier for the DID
		/// para_id - The id of the parachain if a did needs to be created there
		#[pallet::weight(10_000 + T::DbWeight::get().writes(4))]
		pub fn remove(
			origin: OriginFor<T>,
			identifier: Did,
			para_id: Option<ParaId>,
		) -> DispatchResult {
			// Check if origin is a from Sudo
			ensure_root(origin)?;

			Self::do_remove(&identifier)?;

			// Cache DID to Parachain if Para Id is provided
			if let Some(para_id) = para_id {
				T::OnDidUpdate::on_did_removal(para_id, identifier);
			}

			// deposit an event that the DID has been removed
			Self::deposit_event(Event::DidRemoved { did: identifier });

			Ok(())
		}

		/// Updates a DID public key on the chain
		/// origin - the origin of the transaction
		/// public_key - public key to be rotated
		/// para_id - The id of the parachain if a did needs to be created there
		#[pallet::weight(10_000 + T::DbWeight::get().writes(6))]
		pub fn rotate_key(
			origin: OriginFor<T>,
			vc_id: VCid,
			para_id: Option<ParaId>,
		) -> DispatchResult {
			ensure_signed(origin)?;

			let vc = Self::get_vc(vc_id)?;

			// Verify if the vc is valid
			ensure!(
				Self::validate_vc(vc.clone(), VCType::ResetPubKeyVC),
				Error::<T>::InvalidVC,
			);

			// Decode the VC for getting the metadata and public key
			let vc_property =
				T::VCResolution::decode_vc::<ResetPubKeyVC>(&vc.vc_property)?;

			Self::do_rotate_key(&vc_property.did, &vc_property.new_public_key)?;

			// Set the vc to used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			// Cache DID to Parachain if Para Id is provided
			if let Some(para_id) = para_id {
				T::OnDidUpdate::on_key_updation(para_id, vc_property.did, vc_property.new_public_key);
			}

			// create key updated event
			Self::deposit_event(Event::DidKeyUpdated { did: vc_property.did });

			Ok(())
		}

		/// Updates the type of a DID
		/// origin - the origin of the transaction
		/// identifier - did who's type has to be changed
		/// new_type - the new type of the did
		/// para_id - The id of the parachain if that did needs to be cacheed there
		#[pallet::weight(10_000 + T::DbWeight::get().writes(2))]
		pub fn change_did_type(
			origin: OriginFor<T>,
			identifier: Did,
			new_type: DidType,
			para_id: Option<ParaId>,
		) -> DispatchResult {
			// Check if origin is from sudo
			ensure_root(origin)?;

			Self::do_change_did_type(&identifier, new_type)?;

			// Cache DID to Parachain if Para Id is provided
			if let Some(para_id) = para_id {
				T::OnDidUpdate::on_did_type_change(para_id, identifier, new_type);
			}

			// create did type updated event
			Self::deposit_event(Event::DidTypeUpdated { did: identifier });

			Ok(())
		}

		/// Updates DID metadata on the chain
		/// origin - the origin of the transaction
		/// para_id - The id of the parachain if a did needs to be created there
		/// metadata - addional information
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn update_metadata(
			origin: OriginFor<T>,
			identifier: Did,
			metadata: Metadata,
		) -> DispatchResult {
			// Check if origin is a from a validator
			T::ValidatorOrigin::ensure_origin(origin)?;

			Self::do_update_metadata(&identifier, &metadata)?;

			// create metadata updated event
			Self::deposit_event(Event::DidMetadataUpdated { did: identifier });

			Ok(())
		}
	
		#[pallet::weight(10_000)]
		pub fn add_region(origin: OriginFor<T>, region: DidRegion) -> DispatchResult {
			ensure_root(origin)?;

			let mut allowed_regions: AllowedRegionsVec = AllowedRegions::<T>::get();
			ensure!(!allowed_regions.contains(&region), Error::<T>::RegionAlreadyExists);
			let max_regions = <MaxAllowedRegions as frame_support::traits::Get<u32>>::get();
			let regions_len = allowed_regions.len();
			ensure!( regions_len <  max_regions as usize, Error::<T>::MaxAllowedRegionsExceeded);

			allowed_regions.try_push(region.clone()).expect("Overflow Vec");
			AllowedRegions::<T>::set(allowed_regions);

			Self::deposit_event(Event::RegionAdded { region });

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn remove_region(origin: OriginFor<T>, region: DidRegion) -> DispatchResult {
			ensure_root(origin)?;

			let mut allowed_regions = AllowedRegions::<T>::get();
			ensure!(allowed_regions.contains(&region), Error::<T>::RegionNotFound);

			let index = allowed_regions.iter().position(|x| x == &region).unwrap();
			allowed_regions.remove(index);
			AllowedRegions::<T>::set(allowed_regions);

			Self::deposit_event(Event::RegionRemoved { region });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Genesis Function

		/// Initialize did during genesis
		fn initialize_dids(dids: &Vec<DIdentity>, allowed_regions_vec: &AllowedRegionsVec) {
			// This is called only in genesis, hence 0
			let block_no: T::BlockNumber = 0u32.into();
			for did in dids.iter() {

				// Did could be either public or private
				let (identifier, public_key): (Did, PublicKey) = match did {
					// Private Did
					DIdentity::Private(private_did) => {
						// Add Private DID to the storage
						DIDs::<T>::insert(
							private_did.identifier.clone(),
							(
								DIdentity::Private(PrivateDid {
									identifier: private_did.identifier.clone(),
									public_key: private_did.public_key,
									metadata: private_did.metadata.clone(),
								}),
								block_no,
							),
						);
						(private_did.identifier, private_did.public_key)
					},
					// Public Did
					DIdentity::Public(public_did) => {
						// Add Public DID to the storage
						DIDs::<T>::insert(
							public_did.identifier.clone(),
							(
								DIdentity::Public(PublicDid {
									identifier: public_did.identifier.clone(),
									public_key: public_did.public_key,
									metadata: public_did.metadata.clone(),
									registration_number: public_did.registration_number.clone(),
									company_name: public_did.company_name.clone(),
								}),
								block_no,
							),
						);
						(public_did.identifier, public_did.public_key)
					},
				};
				Lookup::<T>::insert(
					identifier.clone(),
					Self::get_accountid_from_pubkey(&public_key),
				);
				RLookup::<T>::insert(Self::get_accountid_from_pubkey(&public_key), identifier);
			}
			AllowedRegions::<T>::set(allowed_regions_vec.clone());
		}


		// Getter Functions

		/// Get VC from Storage using VC Id
		fn get_vc(vc_id: VCid) -> Result<VC<T::Hash>, DispatchError> {
			Ok(T::VCResolution::get_vc(&vc_id).ok_or_else(|| Error::<T>::VCDoesNotExist)?)
		}

		/// Get the details of the pubkey attached to the DID
		pub fn get_did_details(
			identifier: Did,
		) -> Result<(DIdentity, T::BlockNumber), DispatchError> {
			// fetch did details and last updated block
			Ok(DIDs::<T>::get(identifier).ok_or_else(|| Error::<T>::DIDDoesNotExist)?)
		}

		/// Get the details of the previous keys attached to the DID
		pub fn get_prev_key_details(
			identifier: Did,
		) -> Result<BoundedVec<(T::AccountId, T::BlockNumber), T::MaxKeyChanges>, DispatchError> {
			// Fetch Previous Keys details
			Ok(PrevKeys::<T>::get(identifier).unwrap_or_default())
		}

		/// Get the details of the previous types associated to the DID
		pub fn get_prev_type_details(
			identifier: Did,
		) -> Result<BoundedVec<(DidType, T::BlockNumber), MaxTypeChanges>, DispatchError> {
			Ok(TypeChangeHistory::<T>::get(identifier).unwrap_or_default())
		}

		/// Simple type conversion between sr25519::Public and AccountId
		/// Should not panic for any valid sr25519 - need to make more robust to check for valid
		/// publicKey
		pub fn get_accountid_from_pubkey(pk: &PublicKey) -> T::AccountId {
			//convert a publickey to an accountId
			// TODO : Need a better way to handle the option failing?
			T::AccountId::decode(&mut &pk[..]).unwrap()
		}

		/// Get public_key from accountId
		pub fn get_pub_key(identifier: &Did) -> Option<PublicKey> {
			if let Ok((did_details, _)) = Self::get_did_details(identifier.clone()) {
				match did_details {
					DIdentity::Private(private_did) => Some(private_did.public_key),
					DIdentity::Public(public_did) => Some(public_did.public_key),
				}
			} else {
				None
			}
		}
		
		/// Get allowed regions for the chain
		pub fn get_regions() -> AllowedRegionsVec {
			AllowedRegions::<T>::get()
		}


		// Validator Functions

		/// Function to validate vc when creating dids
		pub fn validate_vc(vcs_details: VC<T::Hash>, did_vc_type: VCType) -> bool {
			vcs_details.vc_type == did_vc_type &&
				vcs_details.is_vc_active &&
				!vcs_details.is_vc_used
		}

		/// Function to check if did which is going to be created is valid or not
		pub fn validate_did(identifier: Did) -> bool {
			let allowed_regions: BoundedVec<DidRegion, MaxAllowedRegions> = AllowedRegions::<T>::get();
			let expected_did_prefix: [u8; 3] = [100, 105, 100];
			let empty_array: [u8; 32] = [0; 32];
			
			// Remove trailing /0 from DID
			let clean_identifier = Self::remove_trailing_nulls(&identifier);

			// Split the identifier by :
			let did_parts: Vec<&[u8]> = clean_identifier.split(|&c| c == b':').collect();
			
			// To check if the given region is allowed to create in this chain
			let region: &[u8] = did_parts.get(1).copied().unwrap_or(&[]);
			let region = Self::dynamic_to_fixed_array::<20>(region);
			let is_region_allowed = allowed_regions.contains(&region);

			// Check if did starts with 'did'
			let did_prefix = did_parts.get(0).copied().unwrap_or(&[]);
			let did_prefix = Self::dynamic_to_fixed_array::<3>(did_prefix);
			let is_prefix_valid = did_prefix.eq(&expected_did_prefix);

			// Only lowercase, digits and some special characters are allowed
			// Only 2 : are allowed
			let allowed_special_chars: [u8; 4] = [
        b'_', b'-', b'.', b':',
    	];	
			let is_character_allowed = clean_identifier.iter().all(|&c| c.is_ascii_lowercase() || c.is_ascii_digit() || allowed_special_chars.contains(&c)) && did_parts.len() == 3;

			!clean_identifier.is_empty() &&
				clean_identifier.ne(&empty_array) &&
				is_prefix_valid &&
				is_region_allowed &&
				is_character_allowed
		}

		/// Check if DID or Public Key already exists
		pub fn can_create_did(public_key: PublicKey, identifier: Did) -> DispatchResult {
			// ensure did is valid
			ensure!(Self::validate_did(identifier.clone()), Error::<T>::InvalidDid);

			// ensure did is not already taken
			ensure!(!DIDs::<T>::contains_key(identifier.clone()), Error::<T>::DIDAlreadyExists);

			// ensure the public key is not already linked to a DID
			ensure!(
				!RLookup::<T>::contains_key(Self::get_accountid_from_pubkey(&public_key)),
				Error::<T>::PublicKeyRegistered
			);

			Ok(())
		}

		/// Check whether the DID is public or private
		pub fn check_did_public(did: &Did) -> bool {
			match DIDs::<T>::get(did) {
				Some((did_details, _)) => match did_details {
					DIdentity::Private(_) => false,
					DIdentity::Public(_) => true,
				},
				None => false,
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

		/// Remove /0 from the did
		fn remove_trailing_nulls(input: &[u8]) -> &[u8] {
			let mut last_non_null_index = None;
			for (i, &c) in input.iter().enumerate().rev() {
					if c != b'\0' {
							last_non_null_index = Some(i);
							break;
					}
			}
			if let Some(index) = last_non_null_index {
					&input[..=index]
			} else {
					&[]
			}
		}



		// Storage Functions

		/// Store Private Did
		pub fn do_store_private_did(public_key: PublicKey, identifier: Did) -> DispatchResult {
			// Validate did
			Self::can_create_did(public_key, identifier)?;

			let current_block_no = <frame_system::Pallet<T>>::block_number();

			// add DID to the storage
			DIDs::<T>::insert(
				identifier.clone(),
				(
					DIdentity::Private(PrivateDid {
						identifier: identifier.clone(),
						public_key,
						metadata: Default::default(),
					}),
					current_block_no,
				),
			);

			let account_id = Self::get_accountid_from_pubkey(&public_key);

			Lookup::<T>::insert(identifier.clone(), &account_id);
			RLookup::<T>::insert(&account_id, identifier.clone());

			// Increment providers
			if frame_system::Pallet::<T>::inc_consumers_without_limit(&account_id).is_err() {
				// This will leak a provider reference, however it only happens once (at
				// genesis) so it's really not a big deal and we assume that the user wants to
				// do this since it's the only way a non-endowed account can contain a session
				// key.
				frame_system::Pallet::<T>::inc_providers(&account_id);
			}

			Ok(())
		}

		/// Store Public Did
		pub fn do_store_public_did(
			public_key: PublicKey,
			identifier: Did,
			registration_number: RegistrationNumber,
			company_name: CompanyName,
		) -> DispatchResult {
			// Validate did
			Self::can_create_did(public_key, identifier)?;

			let current_block_no = <frame_system::Pallet<T>>::block_number();

			// add DID to the storage
			DIDs::<T>::insert(
				identifier.clone(),
				(
					DIdentity::Public(PublicDid {
						identifier: identifier.clone(),
						public_key,
						metadata: Default::default(),
						registration_number,
						company_name,
					}),
					current_block_no,
				),
			);

			let account_id = Self::get_accountid_from_pubkey(&public_key);

			Lookup::<T>::insert(identifier.clone(), &account_id);
			RLookup::<T>::insert(&account_id, identifier.clone());

			// Increment providers
			if frame_system::Pallet::<T>::inc_consumers_without_limit(&account_id).is_err() {
				// This will leak a provider reference, however it only happens once (at
				// genesis) so it's really not a big deal and we assume that the user wants to
				// do this since it's the only way a non-endowed account can contain a session
				// key.
				frame_system::Pallet::<T>::inc_providers(&account_id);
			}

			Ok(())
		}

		/// Update metadata of public and private did
		pub fn do_update_metadata(identifier: &Did, metadata: &Metadata) -> DispatchResult {
			// reject if the user does not already have DID registered
			ensure!(DIDs::<T>::contains_key(&identifier), Error::<T>::DIDDoesNotExist);

			// fetch the existing DID document
			let (did_doc, block_number) = Self::get_did_details(identifier.clone())?;

			// modify the public_key of the did doc
			match did_doc {
				DIdentity::Public(public_did) => {
					DIDs::<T>::insert(
						identifier.clone(),
						(
							DIdentity::Public(PublicDid {
								metadata: metadata.clone(),
								..public_did
							}),
							block_number,
						),
					);
				},
				DIdentity::Private(private_did) => {
					DIDs::<T>::insert(
						identifier.clone(),
						(
							DIdentity::Private(PrivateDid {
								metadata: metadata.clone(),
								..private_did
							}),
							block_number,
						),
					);
				},
			}

			Ok(())
		}

		/// Rotate key of public and private did
		pub fn do_rotate_key(identifier: &Did, public_key: &PublicKey) -> DispatchResult {
			//reject if the user does not already have DID registered
			ensure!(DIDs::<T>::contains_key(&identifier), Error::<T>::DIDDoesNotExist);

			// ensure the public key is not already linked to a DID
			ensure!(
				!RLookup::<T>::contains_key(Self::get_accountid_from_pubkey(&public_key)),
				Error::<T>::PublicKeyRegistered
			);

			// fetch the existing DID document
			let (did_doc, last_updated_block) = Self::get_did_details(identifier.clone())?;
			// Get block number
			let current_block_no = <frame_system::Pallet<T>>::block_number();

			// Update Public key of DID and get previous Public key
			let updated_did: (DIdentity, T::BlockNumber);
			let prev_public_key: PublicKey = match did_doc {
				DIdentity::Public(public_did) => {
					updated_did = (
						DIdentity::Public(PublicDid {
							public_key: public_key.clone(),
							..public_did
						}),
						current_block_no,
					);
					public_did.public_key
				},

				DIdentity::Private(private_did) => {
					updated_did = (
						DIdentity::Private(PrivateDid {
							public_key: public_key.clone(),
							..private_did
						}),
						current_block_no,
					);
					private_did.public_key
				},
			};


			// Store the previous key to history
			let mut prev_keys = Self::get_prev_key_details(identifier.clone())?;
			match prev_keys.try_push((Self::get_accountid_from_pubkey(&prev_public_key), last_updated_block)) {
				Err(_) => fail!(Error::<T>::Overflow),
				Ok(_) => (),
			};


			DIDs::<T>::insert(identifier.clone(), updated_did);

			// Remove previous lookup of pubkey to DID
			RLookup::<T>::remove(Self::get_accountid_from_pubkey(&prev_public_key));

			PrevKeys::<T>::insert(identifier.clone(), prev_keys);

			Lookup::<T>::insert(identifier.clone(), Self::get_accountid_from_pubkey(&public_key));

			RLookup::<T>::insert(Self::get_accountid_from_pubkey(&public_key), identifier.clone());

			Ok(())
		}

		/// Remove Did
		fn do_remove(identifier: &Did) -> DispatchResult {
			let (did_doc, _) = Self::get_did_details(identifier.clone())?;

			// remove DID from storage
			DIDs::<T>::remove(&identifier);

			Lookup::<T>::remove(identifier.clone());
			match did_doc {
				DIdentity::Public(public_did) => {
					RLookup::<T>::remove(Self::get_accountid_from_pubkey(&public_did.public_key));
				},
				DIdentity::Private(private_did) => {
					RLookup::<T>::remove(Self::get_accountid_from_pubkey(&private_did.public_key));
				},
			}

			Ok(())
		}

		/// Cache DID to Parachain
		fn do_cache_did(identifier: &Did, para_id: ParaId) -> DispatchResult {
			let (did_doc, _) = Self::get_did_details(identifier.clone())?;

			match did_doc {
				DIdentity::Public(public_did) => {
					T::OnDidUpdate::on_new_did(
						para_id,
						public_did.public_key,
						*identifier,
						DidType::Public,
					);
				},
				DIdentity::Private(private_did) => {
					T::OnDidUpdate::on_new_did(
						para_id,
						private_did.public_key,
						*identifier,
						DidType::Private,
					);
				},
			};

			Ok(())
		}

		/// Change DID Type i.e. Public or Private
		fn do_change_did_type(identifier: &Did, new_type: DidType) -> DispatchResult {
			// ensure did is on chain
			let did_details = DIDs::<T>::get(identifier);
			ensure!(did_details.is_some(), Error::<T>::DIDDoesNotExist);
			let old_did = did_details.unwrap();

			let updated_did: (DIdentity, T::BlockNumber);
			let old_type = match old_did.0 {
				DIdentity::Public(pub_did_struct) => {
					ensure!(!(new_type == DidType::Public), Error::<T>::TypeAlreadySame);
					updated_did = (
						DIdentity::Private(PrivateDid {
							identifier: pub_did_struct.identifier,
							public_key: pub_did_struct.public_key,
							metadata: pub_did_struct.metadata,
						}),
						old_did.1,
					);
					DidType::Public
				},
				DIdentity::Private(priv_did_struct) => {
					ensure!(!(new_type == DidType::Private), Error::<T>::TypeAlreadySame);
					updated_did = (
						DIdentity::Public(PublicDid {
							identifier: priv_did_struct.identifier,
							public_key: priv_did_struct.public_key,
							metadata: priv_did_struct.metadata,
							registration_number: Default::default(),
							company_name: Default::default(),
						}),
						old_did.1,
					);
					DidType::Private
				},
			};

			let current_block_no = <frame_system::Pallet<T>>::block_number();

			let mut prev_types = Self::get_prev_type_details(identifier.clone())?;
			match prev_types.try_push((old_type, current_block_no)) {
				Err(_) => fail!(Error::<T>::Overflow),
				Ok(_) => (),
			};

			TypeChangeHistory::<T>::insert(identifier.clone(), prev_types);

			DIDs::<T>::insert(identifier.clone(), updated_did);

			Ok(())
		}
	}
}
