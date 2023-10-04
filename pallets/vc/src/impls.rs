use super::{*, pallet::*};
use codec::{ Decode };
use sp_runtime::{ DispatchError, DispatchResult };
use metamui_primitives::{ VCid, traits::{VCResolve, UpdateVC}, types::VC, VCHex };

impl<T: Config> VCResolve<T::Hash> for Pallet<T> {
  /// Decoding VC from encoded bytes
  fn decode_vc<E: Decode>(vc_bytes: &[u8]) -> Result<E, DispatchError> {
    Self::decode_vc::<E>(vc_bytes)
  }   

  fn get_vc(vc_id: &VCid) -> Option<VC<T::Hash>> {
    VCs::<T>::get(vc_id)
  }

  fn is_vc_used(vc_id: &VCid) -> bool {
    match VCs::<T>::get(vc_id) {
      Some(vc) => vc.is_vc_used,
      None => false
    }
  }

  fn set_is_vc_used(vc_id: &VCid, is_vc_used: bool) -> Result<(), DispatchError> {
    Self::update_vc_used(*vc_id, Some(is_vc_used))
  }
}


/// Implement update did
impl<T: Config> UpdateVC for Pallet<T> {
  fn add_vc(
      vc_hex: VCHex, 
      vc_id: VCid
  ) -> DispatchResult {

    // Extracting vc from encoded vc byte array
    let vc: VC<T::Hash> = Self::decode_vc(&vc_hex)?;

    let vc_status = Self::check_vc_status(&vc)?;

    ensure!(vc_status, Error::<T>::InvalidVC);

    // Validate did
    Self::on_sync_vc(vc.owner, vc, vc_id)?;

    Ok(())
  }
}
