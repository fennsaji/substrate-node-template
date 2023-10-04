#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod impls;
pub use crate::impls::*;

use sp_std::{ prelude::*, marker::PhantomData };
use metamui_primitives::{ types::{ PalletName, FunctionName } };
pub mod types;
pub use crate::types::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
  use super::*;
  use frame_support::pallet_prelude::*;
  use frame_system::pallet_prelude::*;

  #[pallet::config]
  pub trait Config: frame_system::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    /// Sudo Origin
		type CallOrigin: EnsureOrigin<Self::Origin>;
	}
  
  #[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub whitelisted_extrinsics: Vec<InitialExtrinsics>,
    pub restricted_extrinsics: Vec<InitialExtrinsics>,
    pub phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				whitelisted_extrinsics: Default::default(),
				restricted_extrinsics: Default::default(),
        phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialise_extrinsics(&self.whitelisted_extrinsics, &self.restricted_extrinsics);
		}
	}

  #[pallet::pallet]
  #[pallet::generate_store(pub(super) trait Store)]
  pub struct Pallet<T>(_);
  
  #[pallet::storage]
  pub(super) type WhitelistedExtrinsics<T> =  StorageDoubleMap<_, Blake2_128Concat, PalletName, Blake2_128Concat, FunctionName, (), ValueQuery>;
  
  #[pallet::storage]
  pub(super) type RestrictedExtrinsics<T> =  StorageDoubleMap<_, Blake2_128Concat, PalletName, Blake2_128Concat, FunctionName, (), ValueQuery>;
  
  #[pallet::hooks]
  impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
  
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An Extrinsic has been added to Whitelist
		ExtrinsicWhitelisted { pallet_name: PalletName, function_name: FunctionName },
		/// An Extrinsic has been removed from Whitelist
		ExtrinsicRemoved { pallet_name: PalletName, function_name: FunctionName },
    /// An Extrinsic has been restricted
		ExtrinsicRestricted { pallet_name: PalletName, function_name: FunctionName },
		/// An Extrinsic has been unrestricted
		ExtrinsicUnrestricted { pallet_name: PalletName, function_name: FunctionName }
  }

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The entered extrinsic is already added
		ExtrinsicAlreadyExists,
		/// The entered extrinsic is never added
		ExtrinsicDoesNotExist,
	}

  #[pallet::call]
  impl<T: Config> Pallet<T> { 
    #[pallet::weight(1)]
    pub fn whitelist_extrinsic(origin: OriginFor<T>, pallet_name: PalletName, function_name: FunctionName) -> DispatchResultWithPostInfo {
      T::CallOrigin::ensure_origin(origin)?;
			// ensure extrinsic is not already added
			ensure!(!WhitelistedExtrinsics::<T>::contains_key(pallet_name, function_name), Error::<T>::ExtrinsicAlreadyExists);

      WhitelistedExtrinsics::<T>::insert(pallet_name, function_name, ());
			Self::deposit_event(Event::ExtrinsicWhitelisted{ pallet_name, function_name });
      Ok(().into())
    }
          
    #[pallet::weight(1)]
    pub fn remove_whitelisted_extrinsic(origin: OriginFor<T>, pallet_name: PalletName, function_name: FunctionName) -> DispatchResultWithPostInfo {
      T::CallOrigin::ensure_origin(origin)?;

			// ensure extrinsic exists in the storage
			ensure!(WhitelistedExtrinsics::<T>::contains_key(pallet_name, function_name), Error::<T>::ExtrinsicDoesNotExist);

      WhitelistedExtrinsics::<T>::remove(pallet_name, function_name);
			Self::deposit_event(Event::ExtrinsicRemoved{ pallet_name, function_name });
      Ok(().into())
		}

    #[pallet::weight(1)]
    pub fn add_restricted_extrinsic(origin: OriginFor<T>, pallet_name: PalletName, function_name: FunctionName) -> DispatchResultWithPostInfo {
      T::CallOrigin::ensure_origin(origin)?;
			// ensure extrinsic is not already added
			ensure!(!RestrictedExtrinsics::<T>::contains_key(pallet_name, function_name), Error::<T>::ExtrinsicAlreadyExists);

      RestrictedExtrinsics::<T>::insert(pallet_name, function_name, ());
			Self::deposit_event(Event::ExtrinsicRestricted{ pallet_name, function_name });
      Ok(().into())
    }
          
    #[pallet::weight(1)]
    pub fn remove_restricted_extrinsic(origin: OriginFor<T>, pallet_name: PalletName, function_name: FunctionName) -> DispatchResultWithPostInfo {
      T::CallOrigin::ensure_origin(origin)?;

			// ensure extrinsic exists on chain
			ensure!(RestrictedExtrinsics::<T>::contains_key(pallet_name, function_name), Error::<T>::ExtrinsicDoesNotExist);

      RestrictedExtrinsics::<T>::remove(pallet_name, function_name);
			Self::deposit_event(Event::ExtrinsicUnrestricted{ pallet_name, function_name });
      Ok(().into())
		}
  }

  impl<T: Config> Pallet<T> {
    fn initialise_extrinsics(
      whitelisted_extrinsics: &Vec<InitialExtrinsics>, 
      restricted_extrinsics: &Vec<InitialExtrinsics>, 
    ) {
      for whitelisted_extrinsic in whitelisted_extrinsics.iter() {
        let pallet_name = whitelisted_extrinsic.pallet_name;
        let function_name = whitelisted_extrinsic.function_name;
        <WhitelistedExtrinsics<T>>::insert(pallet_name, function_name, ());
      }

      for restricted_extrinsics in restricted_extrinsics.iter() {
        let pallet_name = restricted_extrinsics.pallet_name;
        let function_name = restricted_extrinsics.function_name;
        <RestrictedExtrinsics<T>>::insert(pallet_name, function_name, ());
      }
    }
  }
}