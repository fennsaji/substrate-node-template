//! Low-level types used throughout the Substrate code.
//! Shared Types will be defined here

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]



/// All traits for metamui
pub mod traits;

/// All types(stucts and enums) for metamui
pub mod types;

pub use types::*;