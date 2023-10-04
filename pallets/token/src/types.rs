use frame_support::{RuntimeDebug};
use scale_info::TypeInfo;
use codec::{Decode, Encode, MaxEncodedLen};
use super::*;

/// Type used to encode the number of references an account has.
pub type RefCount = u32;

/// Tokenchain Details
pub type TokenchainInfo = (TokenName, ReservableBalance, Decimal);

/// Information of an account.
#[derive(Clone, Eq, PartialEq, Default, RuntimeDebug, TypeInfo, Encode, Decode, MaxEncodedLen)]
pub struct AccountInfo<Index, AccountData> {
  /// The number of transactions this account has sent.
	pub nonce: Index,
	/// The number of other modules that currently depend on this account's existence. The account
	/// cannot be reaped until this is zero.
	pub consumers: RefCount,
	/// The number of other modules that allow this account to exist. The account may not be reaped
	/// until this and `sufficients` are both zero.
	pub providers: RefCount,
	/// The number of modules that allow this account to exist for their own purposes only. The
	/// account may not be reaped until this and `providers` are both zero.
	pub sufficients: RefCount,
	/// The additional data that belongs to this account. Used to store the balance(s) in a lot of
	/// chains.
	pub data: AccountData,
}