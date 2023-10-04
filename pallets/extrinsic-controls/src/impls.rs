use super::{ *, pallet::* };
use sp_runtime::{ DispatchResult };
use metamui_primitives::{ traits::{ ExtrinsicResolve } };

/// Impl AccessResolve for Pallet
impl<T: Config> ExtrinsicResolve for Pallet<T> {
	/// Check if an extrinsic is whitelisted
	fn is_extrinsic_whitelisted(pallet_name: PalletName, function_name: FunctionName) -> bool{
		WhitelistedExtrinsics::<T>::contains_key(pallet_name, function_name)
	}
	
	/// Check if an extrinsic is restricted
	fn is_extrinsic_restricted(pallet_name: PalletName, function_name: FunctionName) -> bool {
		RestrictedExtrinsics::<T>::contains_key(pallet_name, function_name)
	}

	/// Restrict an extrinsic
	fn restrict_extrinsic(pallet_name: PalletName, function_name: FunctionName) -> DispatchResult {
    if <RestrictedExtrinsics<T>>::contains_key(pallet_name, function_name) {
      Err("Extrinsic Already Added".into())
    }
    else {
      <RestrictedExtrinsics<T>>::insert(pallet_name, function_name, ());
      Ok(().into())
    }
	}

	/// Remove an extrinsic from restricted list
	fn remove_all_restricted(pallet_name: PalletName) -> DispatchResult {
    let _ = <RestrictedExtrinsics<T>>::clear_prefix(pallet_name, 10, None);
    Ok(().into())
	}
}