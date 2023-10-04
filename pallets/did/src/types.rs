use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::ConstU32};
use scale_info::TypeInfo;
pub use metamui_primitives::Did;
pub use metamui_primitives::types::*;
use cumulus_primitives_core::ParaId;


pub type MaxTypeChanges = ConstU32<20>;

#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrivateDid {
  pub identifier: Did,
  pub public_key: PublicKey,
  pub metadata: Metadata,
}

#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicDid {
  pub identifier: Did,
  pub public_key: PublicKey,
  pub metadata: Metadata,
  pub registration_number: RegistrationNumber,
  pub company_name: CompanyName,
}

#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DIdentity {
  Public(PublicDid),
  Private(PrivateDid),
}

#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DIDRegion {
  Local,
  Tokenchain(ParaId)
}

/// Trait for type that can handle changes to Dids.
pub trait DidUpdated {
  fn on_new_did(
    para_id: ParaId,
    public_key: PublicKey,
    identifier: Did,
    did_type: DidType,
  );

  fn on_did_removal(
    para_id: ParaId,
    identifier: Did,
  );

  fn on_key_updation(
    para_id: ParaId,
    identifier: Did,
    public_key: PublicKey,
  );

  fn on_did_type_change(
    para_id: ParaId,
    identifier: Did,
    did_type: DidType,
  );
}

impl DidUpdated for () {
  fn on_new_did(
    _: ParaId,
    _: PublicKey,
    _: Did,
    _: DidType,
  ) {
    ()
  }

  fn on_did_removal(
    _: ParaId,
    _: Did,
  ) {
    ()
  }

  fn on_key_updation(
    _: ParaId,
    _: Did,
    _: PublicKey,
  ) {
    ()
  }

  fn on_did_type_change(
    _: ParaId,
    _: Did,
    _: DidType,
  ) {
    ()
  }
}