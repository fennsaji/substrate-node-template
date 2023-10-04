use super::*;

/// Permission Level of validator
#[derive(Decode, Encode, TypeInfo, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PermissionLevel {
  Global,
  Local,
}

impl Default for PermissionLevel {
  fn default() -> Self {
      Self::Local
  }
}