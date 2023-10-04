use metamui_primitives::types::{ PalletName, FunctionName };
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitialExtrinsics {
  pub pallet_name: PalletName, 
  pub function_name: FunctionName
}
