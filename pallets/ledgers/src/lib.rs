//! # Tokens Module
//!
//! ## Overview
//!
//! The tokens module provides fungible multi-currency functionality that
//! implements `MultiCurrency` trait.
//!
//! The tokens module provides functions for:
//!
//! - Querying and setting the balance of a given account.
//! - Getting and managing total issuance.
//! - Balance transfer between accounts.
//! - Depositing and withdrawing balance.
//! - Slashing an account balance.
//!
//! ### Implementations
//!
//! The tokens module provides implementations for following traits.
//!
//! - `MultiCurrency` - Abstraction over a fungible multi-currency system.
//! - `MultiCurrencyExtended` - Extended `MultiCurrency` with additional helper types and methods,
//!   like updating balance
//! by a given signed integer amount.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `transfer` - Transfer some balance to another account.
//! - `transfer_all` - Transfer all balance to another account.
//!
//! ### Genesis Config
//!
//! The tokens module depends on the `GenesisConfig`. Endowed accounts could be
//! configured in genesis configs.

#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	fail,
	pallet_prelude::{StorageMap, *},
	traits::{
		BalanceStatus as Status, Currency as PalletCurrency, EnsureOrigin, ExistenceRequirement,
		Get, Imbalance, ReservableCurrency as PalletReservableCurrency, SignedImbalance,
		WithdrawReasons,
	},
	weights::Weight,
	Parameter,
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use metamui_primitives::{
	traits::{DidResolve, HasVCId, SyncBalances, VCResolve},
	types::{SlashMintTokens, TokenTransferVC, TokenVC, VCType, VC},
	Balance, Did as Identifier, VCid,
};
use num::traits::{FromPrimitive, ToPrimitive};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Bounded, MaybeSerializeDeserialize, Member, Zero},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::{
	convert::{TryFrom, TryInto},
	marker, mem,
	prelude::*,
	result,
};

pub use crate::imbalances::{NegativeImbalance, PositiveImbalance};
pub type TokenBalance = u128;
use orml_traits::{
	arithmetic::{self, Signed},
	BalanceStatus, LockIdentifier, MultiCurrency, MultiCurrencyExtended, MultiReservableCurrency,
};

mod types;
pub use crate::types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod default_weight;
mod imbalances;

pub trait WeightInfo {
	fn transfer() -> Weight;
	fn transfer_all() -> Weight;
}

#[cfg(feature = "std")]
pub use serde;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The amount type, should be signed version of `Balance`
		type Amount: Signed
			+ TryInto<TokenBalance>
			+ TryFrom<TokenBalance>
			+ Parameter
			+ Member
			+ arithmetic::SimpleArithmetic
			+ Default
			+ Copy
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize;

		/// The currency ID type
		type CurrencyId: Parameter
			+ Member
			+ Copy
			+ MaybeSerializeDeserialize
			+ Ord
			+ TypeInfo
			+ MaxEncodedLen
			+ Default
			+ FromPrimitive
			+ ToPrimitive;

		/// Origin from which approvals must come.
		type RemoveOrigin: EnsureOrigin<<Self as frame_system::Config>::Origin>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;

		type DidResolution: DidResolve<Self::AccountId>;

		type VCResolution: VCResolve<Self::Hash>;

		type BalanceSync: SyncBalances;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn total_issuance)]
	/// The total issuance of a token type.
	pub type TotalIssuance<T> =
		StorageMap<_, Blake2_128Concat, CurrencyCode, TokenBalance, ValueQuery>;

	/// Any liquidity locks of a token type under an account.
	/// NOTE: Should only be accessed when setting, changing and freeing a lock.
	#[pallet::storage]
	#[pallet::getter(fn locks)]
	pub type Locks<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		CurrencyCode,
		Blake2_128Concat,
		Identifier,
		BoundedVec<BalanceLock<TokenBalance>, MaxLockLen>,
		ValueQuery,
	>;

	/// The balance of a token type under an account.
	///
	/// NOTE: If the total is ever zero, decrease account ref account.
	///
	/// NOTE: This is only used in the case that this module is used to store balances.
	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		CurrencyCode,
		Blake2_128Concat,
		Identifier,
		TokenAccountInfo<T::Index, TokenAccountData>,
		ValueQuery,
	>;

	/// map to store a friendsly name for token
	#[pallet::storage]
	#[pallet::getter(fn token_data)]
	pub(super) type TokenData<T> =
		StorageMap<_, Blake2_128Concat, CurrencyCode, TokenDetails, OptionQuery>;

	/// To get the owner of the token
	#[pallet::storage]
	#[pallet::getter(fn token_issuer)]
	pub type TokenIssuer<T> =
		StorageMap<_, Blake2_128Concat, CurrencyCode, Identifier, OptionQuery>;

	// Counter for currency
	#[pallet::storage]
	#[pallet::getter(fn currency_id)]
	pub type TokenCurrencyCounter<T: Config> = StorageValue<_, T::CurrencyId, OptionQuery>;

	/// To get the currency_code to currency_id mapping
	#[pallet::storage]
	#[pallet::getter(fn token_info)]
	pub type TokenInfo<T: Config> =
		StorageMap<_, Blake2_128Concat, CurrencyCode, T::CurrencyId, OptionQuery>;

	/// To get the reverse currency_code to currency_id mapping
	#[pallet::storage]
	#[pallet::getter(fn token_info_reverse_lookup)]
	pub type TokenInfoRLookup<T: Config> =
		StorageMap<_, Blake2_128Concat, T::CurrencyId, CurrencyCode, OptionQuery>;

	/// A storage map between Currency Code and block number
	#[pallet::storage]
	pub type RemovedTokens<T: Config> =
		StorageMap<_, Blake2_128Concat, CurrencyCode, T::BlockNumber, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Token transfer success. [CurrencyCode, from, to, amount]
		Transferred(CurrencyCode, Identifier, Identifier, TokenBalance),
		/// Token issuance successful [CurrencyCode, dest, amount]
		TokenIssued(CurrencyCode, Identifier, TokenBalance, VCid),
		/// Withdrawn from treasury reserve
		TreasuryWithdrawal(Identifier, Identifier),
		/// Token amount slashed
		TokenSlashed(CurrencyCode, Identifier, TokenBalance, VCid),
		/// Token amount is minted
		TokenMinted(CurrencyCode, Identifier, TokenBalance, VCid),
		/// Token amount is tranfered
		TransferredWithVC(CurrencyCode, Identifier, TokenBalance, VCid),
		/// Token Balance Set
		TokenBalanceSet(CurrencyCode, Identifier, TokenBalance),
		/// Token Balance Set
		TokenRemoved(CurrencyCode),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The balance is too low
		BalanceTooLow,
		/// This operation will cause balance to overflow
		BalanceOverflow,
		/// This operation will cause total issuance to overflow
		TotalIssuanceOverflow,
		/// Cannot convert Amount into Balance type
		AmountIntoBalanceFailed,
		/// Failed because liquidity restrictions due to locking
		LiquidityRestrictions,
		/// Did doesn't exist on chain
		DIDDoesNotExist,
		/// Currency Code already registered
		CurrencyCodeExists,
		/// Token Amount Overflow
		TokenAmountOverflow,
		/// Only Token owner can set other's balance or Cannot Transfer to same did
		NotAllowed,
		/// Memo length too long.
		InvalidMemoLength,
		/// Invalid VC Type
		IncorrectVC,
		/// The given VCId does not exist on chain
		VCDoesNotExist,
		/// VC is not owned by the given DID
		DidNotRegisteredWithVC,
		/// Linked VC does not exist
		LinkedVCNotFound,
		/// VC is already used, can't reused
		VCAlreadyUsed,
		/// VC status is Inactive, cant be use it
		VCIsNotActive,
		/// The currency code is invalid
		InvalidCurrencyCode,
		/// Overflow of Currency Id
		Overflow,
		/// Currency Doesnot Exist
		CurrencyNotExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Transfer some balance to another account.
		///
		/// The dispatch origin for this call must be `Signed` by the transactor.
		///
		/// # <weight>
		/// - Complexity: `O(1)`
		/// - Db reads: 4
		/// - Db writes: 2
		/// -------------------
		/// Base Weight: 84.08 µs
		/// # </weight>
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer(
			origin: OriginFor<T>,
			dest_did: Identifier,
			currency_code: CurrencyCode,
			amount: TokenBalance,
		) -> DispatchResult {
			let source_acc = ensure_signed(origin)?;

			let currency_id = Self::get_ccy_id(&currency_code)?;

			// fetch DID of account to emit event correctly
			let source_did = Self::get_did(&source_acc)?;

			// convert Destination DID to AccountId
			let dest_acc = Self::get_account_id(&dest_did)?;

			ensure!(source_acc != dest_acc, Error::<T>::NotAllowed);

			// Updating Storage

			<Self as MultiCurrency<_>>::transfer(currency_id, &source_acc, &dest_acc, amount)?;

			Self::deposit_event(Event::Transferred(currency_code, source_did, dest_did, amount));

			Ok(())
		}

		/// Transfer With Memo
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_with_memo(
			origin: OriginFor<T>,
			dest: Identifier,
			currency_code: CurrencyCode,
			amount: TokenBalance,
			memo: Memo,
		) -> DispatchResult {
			let source_acc = ensure_signed(origin)?;

			// ensure memo length is valid
			ensure!(memo.is_valid(), Error::<T>::InvalidMemoLength);

			let currency_id = Self::get_ccy_id(&currency_code)?;

			// fetch recipient AccountId from DID
			let dest_acc = Self::get_account_id(&dest)?;

			ensure!(source_acc != dest_acc, Error::<T>::NotAllowed);

			// fetch sender DID
			let source_did = Self::get_did(&source_acc)?;

			// Updating Storage

			<Self as MultiCurrency<_>>::transfer(currency_id, &source_acc, &dest_acc, amount)?;

			// Emit transfer event
			Self::deposit_event(Event::Transferred(currency_code, source_did, dest, amount));
			Ok(())
		}

		/// Transfer all remaining balance to the given account.
		///
		/// The dispatch origin for this call must be `Signed` by the transactor.
		///
		/// # <weight>
		/// - Complexity: `O(1)`
		/// - Db reads: 4
		/// - Db writes: 2
		/// -------------------
		/// Base Weight: 87.71 µs
		/// # </weight>
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_all(
			origin: OriginFor<T>,
			dest_did: Identifier,
			currency_code: CurrencyCode,
		) -> DispatchResult {
			let source_acc = ensure_signed(origin)?;

			let currency_id = Self::get_ccy_id(&currency_code)?;

			let balance =
				<Self as MultiCurrency<T::AccountId>>::free_balance(currency_id, &source_acc);

			// convert Destination DID to AccountId
			let dest_acc = Self::get_account_id(&dest_did)?;

			ensure!(source_acc != dest_acc, Error::<T>::NotAllowed);

			let source_did = Self::get_did(&source_acc)?;

			// Updating Storage

			<Self as MultiCurrency<T::AccountId>>::transfer(
				currency_id,
				&source_acc,
				&dest_acc,
				balance,
			)?;

			Self::deposit_event(Event::Transferred(currency_code, source_did, dest_did, balance));

			Ok(())
		}

		/// Create a fixed supply of tokens
		///
		/// The dispatch origin for this call must be `Signed` by either Sudo user or owner of the
		/// TokenVC.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn issue_token(
			origin: OriginFor<T>,
			vc_id: VCid,
			amount: TokenBalance,
		) -> DispatchResult {
			// Validating Origin (Only Root or VC Owner Allowed) and VC
			let (owner, vc_details) = match ensure_root(origin.clone()) {
				Ok(_) => {
					// Check if the VCId exists on chain
					let vc_details = Self::get_vc(vc_id)?;

					Self::validate_vc_constraints(&vc_details, VCType::TokenVC)?;

					let owner = Self::get_account_id(&vc_details.owner)?;

					(owner, vc_details)
				},
				Err(_) => {
					let caller = ensure_signed(origin)?;
					let vc_details = Self::get_vc(vc_id)?;

					Self::validate_vc(&caller, &vc_details, VCType::TokenVC)?;

					(caller, vc_details)
				},
			};

			let currency_id = Self::generate_ccy_id()?;
			let token_vc: TokenVC = T::VCResolution::decode_vc::<TokenVC>(&vc_details.vc_property)?;

			// Checking for duplicate currency_code
			ensure!(
				!TokenInfo::<T>::contains_key(token_vc.currency_code),
				Error::<T>::CurrencyCodeExists,
			);

			let dest_did = Self::get_did(&owner)?;

			// Updating Storage

			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			let reservable_balance: Balance =
				token_vc.reservable_balance.try_into().unwrap_or_default();
			let _ = T::BalanceSync::on_reserve_balance(vc_details.owner, reservable_balance)?;

			// set total issuance to amount
			TotalIssuance::<T>::mutate(token_vc.currency_code, |issued| {
				*issued = issued.checked_add(amount).unwrap_or_else(|| *issued)
			});

			// allocate total issuance to the destination account - the token central bank
			Self::set_free_balance(token_vc.currency_code, &owner, amount);

			// set decimal, nonce, currency code and token_name of the destination account
			Self::store_token(
				vc_details.owner,
				currency_id,
				token_vc.clone(),
				Self::array_to_bounded_vec::<16, 16>(token_vc.token_name),
			);

			// store the token issuer/owner for lookup
			TokenIssuer::<T>::insert(token_vc.currency_code, dest_did);

			Self::set_currency_id(currency_id);

			Self::deposit_event(Event::TokenIssued(
				token_vc.currency_code,
				dest_did,
				amount,
				vc_id,
			));

			Ok(())
		}

		/// Slash the balance from the issuer account
		///
		/// The dispatch origin for this call must be `Signed` by a issuer account.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn slash_token(origin: OriginFor<T>, vc_id: VCid) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc(&sender, &vc_details, VCType::SlashTokens)?;

			let slash_vc: SlashMintTokens =
				T::VCResolution::decode_vc::<SlashMintTokens>(&vc_details.vc_property)?;
			let amount: TokenBalance = slash_vc.amount.try_into().unwrap_or_default();

			let currency_id = Self::get_ccy_id(&slash_vc.currency_code)?;
			let issuer = Self::get_token_issuer(&slash_vc.currency_code)?;

			let token_vc = Self::get_linked_vc::<SlashMintTokens>(&vc_details)?;
			let token_vc_owner = Self::get_account_id(&token_vc.owner)?;

			ensure!(
				<Self as MultiCurrency<T::AccountId>>::can_slash(
					currency_id,
					&token_vc_owner,
					amount
				),
				Error::<T>::BalanceTooLow,
			);

			// Updating Storage

			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			<Self as MultiCurrency<T::AccountId>>::slash(currency_id, &token_vc_owner, amount);

			Self::deposit_event(Event::TokenSlashed(slash_vc.currency_code, issuer, amount, vc_id));

			Ok(())
		}

		/// Add amount to the issuer account
		///
		/// The dispatch origin for this call must be `Signed` by a issuer account.
		/// Sender must be part of vc
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint_token(origin: OriginFor<T>, vc_id: VCid) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc(&sender, &vc_details, VCType::MintTokens)?;

			let mint_vc: SlashMintTokens =
				T::VCResolution::decode_vc::<SlashMintTokens>(&vc_details.vc_property)?;

			let amount: TokenBalance = mint_vc.amount.try_into().unwrap_or_default();

			let currency_id = Self::get_ccy_id(&mint_vc.currency_code)?;
			let issuer = Self::get_token_issuer(&mint_vc.currency_code)?;

			let token_vc = Self::get_linked_vc::<SlashMintTokens>(&vc_details)?;
			let vc_owner = Self::get_account_id(&token_vc.owner)?;

			// Updating Storage

			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			<Self as MultiCurrency<T::AccountId>>::deposit(currency_id, &vc_owner, amount)?;

			Self::deposit_event(Event::TokenMinted(mint_vc.currency_code, issuer, amount, vc_id));

			Ok(())
		}

		/// Transfer amount from token owner Did to given account's Did
		///
		/// The dispatch origin for this call must be `Signed` by a issuer account.
		/// Sender must be part of vc
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_token(origin: OriginFor<T>, vc_id: VCid, to: Identifier) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let to_acc = Self::get_account_id(&to)?;

			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc(&sender, &vc_details, VCType::TokenTransferVC)?;

			let transfer_vc: TokenTransferVC =
				T::VCResolution::decode_vc::<TokenTransferVC>(&vc_details.vc_property)?;

			let currency_id = Self::get_ccy_id(&transfer_vc.currency_code)?;
			let amount: TokenBalance = transfer_vc.amount.try_into().unwrap_or_default();

			let token_vc = Self::get_linked_vc::<TokenTransferVC>(&vc_details)?;
			let vc_owner = Self::get_account_id(&token_vc.owner)?;

			// Updating Storage

			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			<Self as MultiCurrency<T::AccountId>>::transfer(
				currency_id,
				&vc_owner,
				&to_acc,
				amount,
			)?;

			Self::deposit_event(Event::TransferredWithVC(
				transfer_vc.currency_code,
				to,
				amount,
				vc_id,
			));

			Ok(())
		}

		/// Set Balance of given did of given currency
		/// Balance will be transfered from/to owner's did to keep total issuance same
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_balance(
			origin: OriginFor<T>,
			dest: Identifier,
			currency_code: CurrencyCode,
			amount: TokenBalance,
		) -> DispatchResult {
			let token_owner = match ensure_root(origin.clone()) {
				Ok(_) => Self::get_token_issuer(&currency_code)?,
				Err(_) => {
					let sender = ensure_signed(origin)?;
					Self::ensure_token_owner(&sender, currency_code)?
				},
			};

			ensure!(token_owner != dest, Error::<T>::NotAllowed);

			// Updating Storage

			Self::set_token_balance(currency_code, token_owner, dest, amount)?;

			Self::deposit_event(Event::TokenBalanceSet(currency_code, dest, amount));
			Ok(())
		}

		/// Remove the token from the system
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn remove_token(
			origin: OriginFor<T>,
			currency_code: CurrencyCode,
			vc_id: VCid,
			did: Option<Identifier>,
		) -> DispatchResult {
			T::RemoveOrigin::ensure_origin(origin)?;

			let vc_details = Self::get_vc(vc_id)?;
			ensure!(vc_details.vc_type == VCType::TokenVC, Error::<T>::IncorrectVC);
			ensure!(vc_details.is_vc_used == true, Error::<T>::VCAlreadyUsed);

			let token_vc: TokenVC = T::VCResolution::decode_vc::<TokenVC>(&vc_details.vc_property)?;

			ensure!(token_vc.currency_code == currency_code, Error::<T>::InvalidCurrencyCode);

			// Updating Storage

			Self::on_token_remove(
				currency_code,
				token_vc.reservable_balance,
				vc_details.owner,
				did,
			)?;

			Self::deposit_event(Event::TokenRemoved(currency_code));

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Validation Functions

	// Validate VC Is of correct type and Caller/Origin is the owner
	fn validate_vc(
		caller_acccount_id: &T::AccountId,
		vc_details: &VC<T::Hash>,
		vc_type: VCType,
	) -> DispatchResult {
		let caller_did = Self::get_did(&caller_acccount_id)?;

		Self::validate_vc_constraints(vc_details, vc_type)?;

		// ensure sender has associated vc
		ensure!(caller_did.eq(&vc_details.owner), Error::<T>::DidNotRegisteredWithVC);

		Ok(())
	}

	/// Validate VC active and not used
	fn validate_vc_constraints(vc_details: &VC<T::Hash>, vc_type: VCType) -> DispatchResult {
		// ensure vc is active
		ensure!(vc_details.is_vc_active, Error::<T>::VCIsNotActive);

		// ensure vc_type
		ensure!(vc_details.vc_type.eq(&vc_type), Error::<T>::IncorrectVC);

		// ensure VC is unused
		ensure!(!vc_details.is_vc_used, Error::<T>::VCAlreadyUsed);

		Ok(())
	}

	/// Ensure the given sender is owner of the given currency
	fn ensure_token_owner(
		sender: &T::AccountId,
		currency_code: CurrencyCode,
	) -> Result<Identifier, DispatchError> {
		let sender_did = Self::get_did(&sender)?;
		let token_owner = Self::get_token_issuer(&currency_code)?;
		ensure!(sender_did == token_owner, Error::<T>::NotAllowed);

		Ok(token_owner)
	}

	/// Getter Functions

	/// Set free balance of `who` to a new value.
	///
	/// Note this will not maintain total issuance.
	fn get_ccy_id(ccy_code: &CurrencyCode) -> Result<T::CurrencyId, DispatchError> {
		Ok(TokenInfo::<T>::get(ccy_code).ok_or_else(|| Error::<T>::CurrencyNotExist)?)
	}

	/// Set free balance of `who` to a new value.
	///
	/// Note this will not maintain total issuance.
	fn get_ccy_code(ccy_id: &T::CurrencyId) -> Result<CurrencyCode, DispatchError> {
		Ok(TokenInfoRLookup::<T>::get(ccy_id).ok_or_else(|| Error::<T>::CurrencyNotExist)?)
	}

	fn get_did(account: &T::AccountId) -> Result<Identifier, DispatchError> {
		Ok(T::DidResolution::get_did(&account).ok_or_else(|| Error::<T>::DIDDoesNotExist)?)
	}

	/// Get Account Id from DID and throw error if it doesnot Exist
	fn get_account_id(identifier: &Identifier) -> Result<T::AccountId, DispatchError> {
		Ok(T::DidResolution::get_account_id(identifier)
			.ok_or_else(|| Error::<T>::DIDDoesNotExist)?)
	}

	fn get_token_issuer(ccy_code: &CurrencyCode) -> Result<Identifier, DispatchError> {
		Ok(Self::token_issuer(ccy_code).ok_or_else(|| Error::<T>::CurrencyNotExist)?)
	}

	/// This will return unique currency_id.
	fn generate_ccy_id() -> Result<T::CurrencyId, DispatchError> {
		let currency_id: T::CurrencyId = if let Some(id) = TokenCurrencyCounter::<T>::get() {
			let mut id_u64 = id.to_u64().unwrap_or(0);
			id_u64 = id_u64.checked_add(1).ok_or(Error::<T>::Overflow)?;
			T::CurrencyId::from_u64(id_u64).unwrap_or_default()
		} else {
			T::CurrencyId::from_u64(1_u64).unwrap_or_default()
		};
		Ok(currency_id)
	}

	/// Get vc struct
	fn get_vc(vc_id: VCid) -> Result<VC<T::Hash>, DispatchError> {
		Ok(T::VCResolution::get_vc(&vc_id).ok_or_else(|| Error::<T>::VCDoesNotExist)?)
	}

	/// Gets updated token balance of owner
	/// Validate Whether balance can be set
	/// Also checks if overflow or underflow occurs
	fn get_updated_owner_balance(
		currency_code: CurrencyCode,
		token_owner: Identifier,
		dest: Identifier,
		amount: TokenBalance,
	) -> Result<TokenBalance, DispatchError> {
		let owner_balance = Self::accounts(currency_code, token_owner).data.free;
		let dest_balance = Self::accounts(currency_code, dest).data.free;
		if amount > dest_balance {
			let difference =
				amount.checked_sub(dest_balance).ok_or(Error::<T>::TokenAmountOverflow)?;
			ensure!(difference <= owner_balance, Error::<T>::TokenAmountOverflow);
			let updated_owner_balance =
				owner_balance.checked_sub(difference).ok_or(Error::<T>::TokenAmountOverflow)?;
			Ok(updated_owner_balance)
		} else {
			let difference =
				dest_balance.checked_sub(amount).ok_or(Error::<T>::TokenAmountOverflow)?;
			let updated_owner_balance =
				owner_balance.checked_add(difference).ok_or(Error::<T>::TokenAmountOverflow)?;
			Ok(updated_owner_balance)
		}
	}

	/// Get Linked VC
	fn get_linked_vc<G: codec::Decode + HasVCId>(
		vc_details: &VC<T::Hash>,
	) -> Result<VC<T::Hash>, DispatchError> {
		let vc_property: G = T::VCResolution::decode_vc::<G>(&vc_details.vc_property)?;

		let vc = if let Some(vc_details) = T::VCResolution::get_vc(&vc_property.vc_id()) {
			vc_details
		} else {
			fail!(Error::<T>::LinkedVCNotFound);
		};

		Ok(vc)
	}

	/// Storage Functions

	/// Set free balance of `who` to a new value.
	///
	/// Note this will not maintain total issuance.
	fn set_free_balance(currency_code: CurrencyCode, who: &T::AccountId, balance: TokenBalance) {
		let did = T::DidResolution::get_did(who).unwrap();
		<Accounts<T>>::mutate(currency_code, did, |account_data| account_data.data.free = balance);
	}

	/// This function will set constant fields
	fn store_token(
		_identifier: Identifier,
		ccy_id: T::CurrencyId,
		token_vc: TokenVC,
		mut token_name: TokenName,
	) {
		let mut currency_code: CurrencyCodeArray =
			Self::array_to_bounded_vec::<8, 8>(token_vc.currency_code);
		let current_block_no: BlockNumber =
			frame_system::Pallet::<T>::block_number().try_into().unwrap_or_default();
		currency_code.retain(|val| *val != 0);
		token_name.retain(|val| *val != 0);
		TokenData::<T>::insert(
			token_vc.currency_code,
			TokenDetails {
				token_name,
				currency_code,
				decimal: token_vc.decimal,
				block_number: current_block_no,
			},
		);

		// TODO
		// Accounts::<T>::mutate(token_vc.currency_code, identifier, |account_data| {
		// 	account_data.nonce = did::Module::<T>::get_nonce_from_did(identifier);
		// });
		Self::set_token_info(ccy_id, token_vc.currency_code);
	}

	/// This function will set the token related informations
	fn set_token_info(ccy_id: T::CurrencyId, ccy_code: CurrencyCode) {
		TokenInfo::<T>::insert(ccy_code, ccy_id);
		TokenInfoRLookup::<T>::insert(ccy_id, ccy_code);
	}

	/// This will set currency_id.
	fn set_currency_id(currency_id: T::CurrencyId) {
		TokenCurrencyCounter::<T>::put(currency_id);
	}

	/// Set reserved balance of `who` to a new value, meanwhile enforce
	/// existential rule.
	///
	/// Note this will not maintain total issuance, and the caller is expected
	/// to do it.
	fn set_reserved_balance(
		currency_code: CurrencyCode,
		who: &T::AccountId,
		balance: TokenBalance,
	) {
		let did = T::DidResolution::get_did(who).unwrap_or_default();
		<Accounts<T>>::mutate(currency_code, did, |account_data| {
			account_data.data.reserved = balance
		});
	}

	// Update the account entry for `who` under `currency_id`, given the locks.
	// fn update_locks(currency_id: T::CurrencyId, who: &T::AccountId, locks:
	// &[BalanceLock<T::Balance>]) { 	// update account data
	// 	<Accounts<T>>::mutate(who, currency_id, |account_data| {
	// 		account_data.frozen = Zero::zero();
	// 		for lock in locks.iter() {
	// 			account_data.frozen = account_data.frozen.max(lock.amount);
	// 		}
	// 	});

	// 	// update locks
	// 	let existed = <Locks<T>>::contains_key(who, currency_id);
	// 	if locks.is_empty() {
	// 		<Locks<T>>::remove(who, currency_id);
	// 		if existed {
	// 			// decrease account ref count when destruct lock
	// 			frame_system::Module::<T>::dec_ref(who);
	// 		}
	// 	} else {
	// 		<Locks<T>>::insert(who, currency_id, locks);
	// 		if !existed {
	// 			// increase account ref count when initialize lock
	// 			frame_system::Module::<T>::inc_ref(who);
	// 		}
	// 	}
	// }

	fn on_token_remove(
		currency_code: CurrencyCode,
		reservable_balance: TokenBalance,
		owner: Identifier,
		transfer_did: Option<Identifier>,
	) -> DispatchResult {
		let reservable_balance: Balance = reservable_balance.try_into().unwrap_or_default();

		if let Some(did) = transfer_did {
			T::BalanceSync::on_slash_reserved(owner, reservable_balance, did)?;
		} else {
			T::BalanceSync::on_unreserve_balance(owner, reservable_balance)?;
		}
		TotalIssuance::<T>::remove(currency_code);
		TokenData::<T>::remove(currency_code);
		TokenIssuer::<T>::remove(currency_code);

		let block_number = frame_system::Pallet::<T>::block_number();
		RemovedTokens::<T>::insert(currency_code, block_number);

		let currency_id = Self::get_ccy_id(&currency_code)?;
		TokenInfoRLookup::<T>::remove(currency_id);

		// TODO: Implement maybe cursor by using hooks
		let _ = Accounts::<T>::clear_prefix(currency_code, 50, None);

		Ok(())
	}

	/// Set token balance to given did
	/// Balance will be transfered from/to owner's did to keep total issuance same
	fn set_token_balance(
		currency_code: CurrencyCode,
		token_owner: Identifier,
		dest: Identifier,
		amount: TokenBalance,
	) -> DispatchResult {
		let updated_owner_balance =
			Self::get_updated_owner_balance(currency_code, token_owner, dest, amount)?;
		let dest_acc = Self::get_account_id(&dest)?;
		let owner_acc = Self::get_account_id(&token_owner)?;

		Self::set_free_balance(currency_code, &dest_acc, amount);
		Self::set_free_balance(currency_code, &owner_acc, updated_owner_balance);

		Ok(())
	}

	/// Helper Functions

	/// Conver to Array to Bounded Vec
	pub fn array_to_bounded_vec<const N: u32, const M: usize>(
		array: [u8; M],
	) -> BoundedVec<u8, ConstU32<N>> {
		let mut bounded_vec_array: BoundedVec<u8, ConstU32<N>> = Default::default();
		for item in array {
			bounded_vec_array.try_push(item).ok();
		}
		bounded_vec_array
	}
}

impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
	type CurrencyId = T::CurrencyId;
	type Balance = TokenBalance;

	fn minimum_balance(_: Self::CurrencyId) -> Self::Balance {
		Default::default()
	}

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		TotalIssuance::<T>::get(currency_code)
	}

	fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		let did = T::DidResolution::get_did(who).unwrap();
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		Self::accounts(currency_code, did).data.total()
	}

	fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		let did = T::DidResolution::get_did(who).unwrap();
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		Self::accounts(currency_code, did).data.free
	}

	// Ensure that an account can withdraw from their free balance given any
	// existing withdrawal restrictions like locks and vesting balance.
	// Is a no-op if amount to be withdrawn is zero.
	fn ensure_can_withdraw(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}
		let did = Self::get_did(who)?;
		let new_balance = Self::free_balance(currency_id, who)
			.checked_sub(amount)
			.ok_or(Error::<T>::BalanceTooLow)?;
		let currency_code = Self::get_ccy_code(&currency_id)?;
		ensure!(
			new_balance >= Self::accounts(currency_code, did).data.frozen(),
			Error::<T>::LiquidityRestrictions
		);
		Ok(())
	}

	/// Transfer some free balance from `from` to `to`.
	/// Is a no-op if value to be transferred is zero or the `from` is the same
	/// as `to`.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &T::AccountId,
		to: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() || from == to {
			return Ok(())
		}
		Self::ensure_can_withdraw(currency_id, from, amount)?;

		let from_balance = Self::free_balance(currency_id, from);
		let to_balance = Self::free_balance(currency_id, to)
			.checked_add(amount)
			.ok_or(Error::<T>::BalanceOverflow)?;
		// Cannot underflow because ensure_can_withdraw check
		let currency_code = Self::get_ccy_code(&currency_id)?;
		Self::set_free_balance(currency_code, from, from_balance - amount);
		Self::set_free_balance(currency_code, to, to_balance);

		Ok(())
	}

	/// Deposit some `amount` into the free balance of account `who`.
	///
	/// Is a no-op if the `amount` to be deposited is zero.
	fn deposit(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}

		let currency_code = Self::get_ccy_code(&currency_id)?;
		let new_total = Self::total_issuance(currency_code)
			.checked_add(amount)
			.ok_or(Error::<T>::TotalIssuanceOverflow)?;
		TotalIssuance::<T>::insert(currency_code, new_total);
		Self::set_free_balance(
			currency_code,
			who,
			Self::free_balance(currency_id, who).saturating_add(amount),
		);

		Ok(())
	}

	fn withdraw(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> DispatchResult {
		if amount.is_zero() {
			return Ok(())
		}
		Self::ensure_can_withdraw(currency_id, who, amount)?;

		let currency_code = Self::get_ccy_code(&currency_id)?;
		// Cannot underflow because ensure_can_withdraw check
		TotalIssuance::<T>::mutate(currency_code, |v| *v -= amount);
		Self::set_free_balance(currency_code, who, Self::free_balance(currency_id, who) - amount);

		Ok(())
	}

	// Check if `value` amount of free balance can be slashed from `who`.
	fn can_slash(currency_id: Self::CurrencyId, who: &T::AccountId, value: Self::Balance) -> bool {
		if value.is_zero() {
			return true
		}
		Self::free_balance(currency_id, who) >= value
	}

	/// Is a no-op if `value` to be slashed is zero.
	///
	/// NOTE: `slash()` prefers free balance, but assumes that reserve balance
	/// can be drawn from in extreme circumstances. `can_slash()` should be used
	/// prior to `slash()` to avoid having to draw from reserved funds, however
	/// we err on the side of punishment if things are inconsistent
	/// or `can_slash` wasn't used appropriately.
	fn slash(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		amount: Self::Balance,
	) -> Self::Balance {
		if amount.is_zero() {
			return amount
		}
		let did = T::DidResolution::get_did(who).unwrap_or_default();
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		let account = Self::accounts(currency_code, did);
		let free_slashed_amount = account.data.free.min(amount);
		// Cannot underflow becuase free_slashed_amount can never be greater than amount
		let mut remaining_slash = amount - free_slashed_amount;

		// slash free balance
		if !free_slashed_amount.is_zero() {
			// Cannot underflow becuase free_slashed_amount can never be greater than
			// account.free
			Self::set_free_balance(currency_code, who, account.data.free - free_slashed_amount);
		}

		// slash reserved balance
		if !remaining_slash.is_zero() {
			let reserved_slashed_amount = account.data.reserved.min(remaining_slash);
			// Cannot underflow due to above line
			remaining_slash -= reserved_slashed_amount;
			Self::set_reserved_balance(
				currency_code,
				who,
				account.data.reserved - reserved_slashed_amount,
			);
		}

		// Cannot underflow because the slashed value cannot be greater than total
		// issuance
		TotalIssuance::<T>::mutate(currency_code, |v| *v -= amount - remaining_slash);
		remaining_slash
	}
}

impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
	type Amount = T::Amount;

	fn update_balance(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		by_amount: Self::Amount,
	) -> DispatchResult {
		if by_amount.is_zero() {
			return Ok(())
		}

		// Ensure this doesn't overflow. There isn't any traits that exposes
		// `saturating_abs` so we need to do it manually.
		let by_amount_abs = if by_amount == Self::Amount::min_value() {
			Self::Amount::max_value()
		} else {
			by_amount.abs()
		};

		let by_balance = TryInto::<Self::Balance>::try_into(by_amount_abs)
			.map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
		if by_amount.is_positive() {
			Self::deposit(currency_id, who, by_balance)
		} else {
			Self::withdraw(currency_id, who, by_balance).map(|_| ())
		}
	}
}

// impl<T: Config> MultiLockableCurrency<T::AccountId> for Module<T> {
// 	type Moment = T::BlockNumber;

// Set a lock on the balance of `who` under `currency_id`.
// Is a no-op if lock amount is zero.
// fn set_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId, amount:
// Self::Balance) { 	if amount.is_zero() {
// 		return;
// 	}
// 	let mut new_lock = Some(BalanceLock { id: lock_id, amount });
// 	let mut locks = Self::locks(who, currency_id)
// 		.into_iter()
// 		.filter_map(|lock| {
// 			if lock.id == lock_id {
// 				new_lock.take()
// 			} else {
// 				Some(lock)
// 			}
// 		})
// 		.collect::<Vec<_>>();
// 	if let Some(lock) = new_lock {
// 		locks.push(lock)
// 	}
// 	Self::update_locks(currency_id, who, &locks[..]);
// }

// Extend a lock on the balance of `who` under `currency_id`.
// Is a no-op if lock amount is zero
// fn extend_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId,
// amount: Self::Balance) { 	if amount.is_zero() {
// 		return;
// 	}
// 	let mut new_lock = Some(BalanceLock { id: lock_id, amount });
// 	let mut locks = Self::locks(who, currency_id)
// 		.into_iter()
// 		.filter_map(|lock| {
// 			if lock.id == lock_id {
// 				new_lock.take().map(|nl| BalanceLock {
// 					id: lock.id,
// 					amount: lock.amount.max(nl.amount),
// 				})
// 			} else {
// 				Some(lock)
// 			}
// 		})
// 		.collect::<Vec<_>>();
// 	if let Some(lock) = new_lock {
// 		locks.push(lock)
// 	}
// 	Self::update_locks(currency_id, who, &locks[..]);
// }

// fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &T::AccountId) {
// 	let mut locks = Self::locks(who, currency_id);
// 	locks.retain(|lock| lock.id != lock_id);
// 	Self::update_locks(currency_id, who, &locks[..]);
// }
//}

impl<T: Config> MultiReservableCurrency<T::AccountId> for Pallet<T> {
	/// Check if `who` can reserve `value` from their free balance.
	///
	/// Always `true` if value to be reserved is zero.
	fn can_reserve(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> bool {
		if value.is_zero() {
			return true
		}
		Self::ensure_can_withdraw(currency_id, who, value).is_ok()
	}

	/// Slash from reserved balance, returning any amount that was unable to be
	/// slashed.
	///
	/// Is a no-op if the value to be slashed is zero.
	fn slash_reserved(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::Balance {
		if value.is_zero() {
			return value
		}

		let reserved_balance = Self::reserved_balance(currency_id, who);
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		let actual = reserved_balance.min(value);
		Self::set_reserved_balance(currency_code, who, reserved_balance - actual);
		TotalIssuance::<T>::mutate(currency_code, |v| *v -= actual);
		value - actual
	}

	fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
		let did = T::DidResolution::get_did(who).unwrap();
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		Self::accounts(currency_code, did).data.reserved
	}

	/// Move `value` from the free balance from `who` to their reserved balance.
	///
	/// Is a no-op if value to be reserved is zero.
	fn reserve(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> DispatchResult {
		if value.is_zero() {
			return Ok(())
		}
		Self::ensure_can_withdraw(currency_id, who, value)?;
		let did = Self::get_did(who)?;

		let currency_code = Self::get_ccy_code(&currency_id)?;
		let account = Self::accounts(currency_code, did);
		Self::set_free_balance(currency_code, who, account.data.free - value);
		// Cannot overflow becuase total issuance is using the same balance type and
		// this doesn't increase total issuance
		Self::set_reserved_balance(currency_code, who, account.data.reserved.saturating_add(value));
		Ok(())
	}

	/// Unreserve some funds, returning any amount that was unable to be
	/// unreserved.
	///
	/// Is a no-op if the value to be unreserved is zero.
	fn unreserve(
		currency_id: Self::CurrencyId,
		who: &T::AccountId,
		value: Self::Balance,
	) -> Self::Balance {
		if value.is_zero() {
			return value
		}
		let did = T::DidResolution::get_did(&who).unwrap();
		let currency_code = Self::get_ccy_code(&currency_id).unwrap_or_default();
		let account = Self::accounts(currency_code, did);
		let actual = account.data.reserved.min(value);
		Self::set_reserved_balance(currency_code, who, account.data.reserved - actual);
		Self::set_free_balance(currency_code, who, account.data.free.saturating_add(actual));
		value - actual
	}

	/// Move the reserved balance of one account into the balance of another,
	/// according to `status`.
	///
	/// Is a no-op if:
	/// - the value to be moved is zero; or
	/// - the `slashed` id equal to `beneficiary` and the `status` is `Reserved`.
	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError> {
		if value.is_zero() {
			return Ok(value)
		}

		if slashed == beneficiary {
			return match status {
				BalanceStatus::Free => Ok(Self::unreserve(currency_id, slashed, value)),
				BalanceStatus::Reserved =>
					Ok(value.saturating_sub(Self::reserved_balance(currency_id, slashed))),
			}
		}
		let slashed_did = Self::get_did(slashed)?;
		let ben_did = Self::get_did(beneficiary)?;

		let currency_code = Self::get_ccy_code(&currency_id)?;
		let from_account = Self::accounts(currency_code, slashed_did);
		let to_account = Self::accounts(currency_code, ben_did);
		let actual = from_account.data.reserved.min(value);
		match status {
			BalanceStatus::Free => {
				Self::set_free_balance(
					currency_code,
					beneficiary,
					to_account.data.free.saturating_add(actual),
				);
			},
			BalanceStatus::Reserved => {
				Self::set_reserved_balance(
					currency_code,
					beneficiary,
					to_account.data.reserved.saturating_add(actual),
				);
			},
		}
		Self::set_reserved_balance(currency_code, slashed, from_account.data.reserved - actual);
		Ok(value - actual)
	}
}

// fn balance_to_token_balance(input: T::Balance) -> TokenBalance {
//     TryInto::<TokenBalance>::try_into(input).unwrap_or_default()
// }

pub struct CurrencyAdapter<T, GetCurrencyId>(marker::PhantomData<(T, GetCurrencyId)>);

impl<T, GetCurrencyId> PalletCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::CurrencyId>,
{
	type Balance = TokenBalance;
	type PositiveImbalance = PositiveImbalance<T, GetCurrencyId>;
	type NegativeImbalance = NegativeImbalance<T, GetCurrencyId>;

	fn total_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::total_balance(GetCurrencyId::get(), who)
	}

	fn can_slash(who: &T::AccountId, value: Self::Balance) -> bool {
		Pallet::<T>::can_slash(GetCurrencyId::get(), who, value)
	}

	fn total_issuance() -> Self::Balance {
		let currency_id = GetCurrencyId::get();
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id).unwrap_or_default();
		Pallet::<T>::total_issuance(currency_code)
	}

	fn minimum_balance() -> Self::Balance {
		Zero::zero()
	}

	fn burn(mut amount: Self::Balance) -> Self::PositiveImbalance {
		if amount.is_zero() {
			return PositiveImbalance::zero()
		}
		let currency_id = GetCurrencyId::get();
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id).unwrap_or_default();
		TotalIssuance::<T>::mutate(currency_code, |issued| {
			*issued = issued.checked_sub(amount).unwrap_or_else(|| {
				amount = *issued;
				Zero::zero()
			});
		});
		PositiveImbalance::new(amount)
	}

	fn issue(mut amount: Self::Balance) -> Self::NegativeImbalance {
		if amount.is_zero() {
			return NegativeImbalance::zero()
		}
		let currency_id = GetCurrencyId::get();
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id).unwrap_or_default();
		TotalIssuance::<T>::mutate(currency_code, |issued| {
			*issued = issued.checked_add(amount).unwrap_or_else(|| {
				amount = Self::Balance::max_value() - *issued;
				Self::Balance::max_value()
			})
		});
		NegativeImbalance::new(amount)
	}

	fn free_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::free_balance(GetCurrencyId::get(), who)
	}

	fn ensure_can_withdraw(
		who: &T::AccountId,
		amount: Self::Balance,
		_reasons: WithdrawReasons,
		_new_balance: Self::Balance,
	) -> DispatchResult {
		Pallet::<T>::ensure_can_withdraw(GetCurrencyId::get(), who, amount)
	}

	fn transfer(
		source: &T::AccountId,
		dest: &T::AccountId,
		value: Self::Balance,
		_existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		<Pallet<T> as MultiCurrency<T::AccountId>>::transfer(
			GetCurrencyId::get(),
			&source,
			&dest,
			value,
		)
	}

	fn slash(who: &T::AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		if value.is_zero() {
			return (Self::NegativeImbalance::zero(), value)
		}

		let currency_id = GetCurrencyId::get();
		let did = T::DidResolution::get_did(who).unwrap();
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id).unwrap_or_default();
		let account = Pallet::<T>::accounts(currency_code, did);
		let free_slashed_amount = account.data.free.min(value);
		let mut remaining_slash = value - free_slashed_amount;

		// slash free balance
		if !free_slashed_amount.is_zero() {
			Pallet::<T>::set_free_balance(
				currency_code,
				who,
				account.data.free - free_slashed_amount,
			);
		}

		// slash reserved balance
		if !remaining_slash.is_zero() {
			let reserved_slashed_amount = account.data.reserved.min(remaining_slash);
			remaining_slash -= reserved_slashed_amount;
			Pallet::<T>::set_reserved_balance(
				currency_code,
				who,
				account.data.reserved - reserved_slashed_amount,
			);
			(
				Self::NegativeImbalance::new(
					free_slashed_amount.saturating_add(reserved_slashed_amount),
				),
				remaining_slash,
			)
		} else {
			(Self::NegativeImbalance::new(value), remaining_slash)
		}
	}

	fn deposit_into_existing(
		who: &T::AccountId,
		value: Self::Balance,
	) -> result::Result<Self::PositiveImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::PositiveImbalance::zero())
		}
		let currency_id = GetCurrencyId::get();
		let new_total = Pallet::<T>::free_balance(currency_id, who)
			.checked_add(value)
			.ok_or(Error::<T>::TotalIssuanceOverflow)?;
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id)?;
		Pallet::<T>::set_free_balance(currency_code, who, new_total);

		Ok(Self::PositiveImbalance::new(value))
	}

	fn deposit_creating(who: &T::AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		Self::deposit_into_existing(who, value).unwrap_or_else(|_| Self::PositiveImbalance::zero())
	}

	fn withdraw(
		who: &T::AccountId,
		value: Self::Balance,
		_reasons: WithdrawReasons,
		_liveness: ExistenceRequirement,
	) -> result::Result<Self::NegativeImbalance, DispatchError> {
		if value.is_zero() {
			return Ok(Self::NegativeImbalance::zero())
		}
		let currency_id = GetCurrencyId::get();
		Pallet::<T>::ensure_can_withdraw(currency_id, who, value)?;
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id)?;
		Pallet::<T>::set_free_balance(
			currency_code,
			who,
			Pallet::<T>::free_balance(currency_id, who) - value,
		);

		Ok(Self::NegativeImbalance::new(value))
	}

	fn make_free_balance_be(
		who: &T::AccountId,
		value: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		let did = T::DidResolution::get_did(who).unwrap_or_default();
		let currency_id = GetCurrencyId::get();
		let currency_code = Pallet::<T>::get_ccy_code(&currency_id).unwrap_or_default();

		<Accounts<T>>::mutate(
			currency_code,
			did,
			|account| -> Result<SignedImbalance<Self::Balance, Self::PositiveImbalance>, ()> {
				let imbalance = if account.data.free <= value {
					SignedImbalance::Positive(PositiveImbalance::new(value - account.data.free))
				} else {
					SignedImbalance::Negative(NegativeImbalance::new(account.data.free - value))
				};
				account.data.free = value;
				Ok(imbalance)
			},
		)
		.unwrap_or_else(|_| SignedImbalance::Positive(Self::PositiveImbalance::zero()))
	}
}

impl<T, GetCurrencyId> PalletReservableCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
where
	T: Config,
	GetCurrencyId: Get<T::CurrencyId>,
{
	fn can_reserve(who: &T::AccountId, value: Self::Balance) -> bool {
		Pallet::<T>::can_reserve(GetCurrencyId::get(), who, value)
	}

	fn slash_reserved(
		who: &T::AccountId,
		value: Self::Balance,
	) -> (Self::NegativeImbalance, Self::Balance) {
		let actual = Pallet::<T>::slash_reserved(GetCurrencyId::get(), who, value);
		(Self::NegativeImbalance::zero(), actual)
	}

	fn reserved_balance(who: &T::AccountId) -> Self::Balance {
		Pallet::<T>::reserved_balance(GetCurrencyId::get(), who)
	}

	fn reserve(who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		Pallet::<T>::reserve(GetCurrencyId::get(), who, value)
	}

	fn unreserve(who: &T::AccountId, value: Self::Balance) -> Self::Balance {
		Pallet::<T>::unreserve(GetCurrencyId::get(), who, value)
	}

	fn repatriate_reserved(
		slashed: &T::AccountId,
		beneficiary: &T::AccountId,
		value: Self::Balance,
		status: Status,
	) -> result::Result<Self::Balance, DispatchError> {
		Pallet::<T>::repatriate_reserved(GetCurrencyId::get(), slashed, beneficiary, value, status)
	}
}

// impl<T, GetCurrencyId> PalletLockableCurrency<T::AccountId> for CurrencyAdapter<T, GetCurrencyId>
// where
// 	T: Config,
// 	GetCurrencyId: Get<T::CurrencyId>,
// {
// 	type Moment = T::BlockNumber;
// 	type MaxLocks = ();

// 	fn set_lock(id: LockIdentifier, who: &T::AccountId, amount: Self::Balance, _reasons:
// WithdrawReasons) { 		Module::<T>::set_lock(id, GetCurrencyId::get(), who, amount)
// 	}

// 	fn extend_lock(id: LockIdentifier, who: &T::AccountId, amount: Self::Balance, _reasons:
// WithdrawReasons) { 		Module::<T>::extend_lock(id, GetCurrencyId::get(), who, amount)
// 	}

// 	fn remove_lock(id: LockIdentifier, who: &T::AccountId) {
// 		Module::<T>::remove_lock(id, GetCurrencyId::get(), who)
// 	}
// }
