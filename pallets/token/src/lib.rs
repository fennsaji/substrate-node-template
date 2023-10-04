#![cfg_attr(not(feature = "std"), no_std)]

use crate::types::*;
use frame_support::{
	pallet_prelude::*,
	sp_runtime::DispatchError,
	traits::{
		Currency as PalletCurrency, ExistenceRequirement, LockableCurrency, OnKilledAccount,
		OnNewAccount, ReservableCurrency, StoredMap,
	},
};
use frame_system::{pallet_prelude::*, split_inner};
use metamui_primitives::{
	traits::{DidResolve, ExtrinsicResolve, HasVCId, VCResolve},
	types::{SlashMintTokens, TokenTransferVC, IssueTokenVC, VCType, VC, CurrencyCode},
	Did as Identifier, VCid,
};
pub use pallet::*;
pub type TokenName = [u8; 16];
pub type ReservableBalance = u128;
pub type Decimal = u8;
use scale_info::prelude::vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	type BalanceOf<T> = <<T as Config>::Currency as PalletCurrency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Validator Origin
		type WithdrawOrigin: EnsureOrigin<Self::Origin>;
		/// The staking balance.
		type Currency: LockableCurrency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Resolve Did from account Id
		type DidResolution: DidResolve<Self::AccountId>;
		/// Resolve VC Data
		type VCResolution: VCResolve<Self::Hash>;
		/// Resolve Extrinsics
		type ExtrinsicResolution: ExtrinsicResolve;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn account)]
	pub type Account<T: Config> = StorageMap<_, Blake2_128Concat, Identifier, AccountInfo<T::Index, T::AccountData>, ValueQuery>;

	#[pallet::storage]
	pub type TokenchainCurrency<T: Config> = StorageValue<_, CurrencyCode, ValueQuery>;

	#[pallet::storage]
	pub type TokenchainDetails<T: Config> = StorageValue<_, TokenchainInfo, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub currency_code: CurrencyCode,
		pub token_details: TokenchainInfo,
		pub phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				currency_code: Default::default(),
				token_details: Default::default(),
				phantom: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_tokenchain(&self.currency_code, &self.token_details);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Withdrawn reserve from account
		ReserveWithdrawn { from: Identifier, to: Identifier },
		/// Token amount is slashed
		TokenSlashed { balance: BalanceOf<T>, vc_id: VCid },
		/// Token amount is minted
		TokenMinted { balance: BalanceOf<T>, vc_id: VCid },
		/// Token amount is tranfered
		TransferredWithVC { to: Identifier, balance: BalanceOf<T>, vc_id: VCid },
		/// Provider count increased
		ProviderIncreased { did: Identifier, account: T::AccountId },
		/// Token has been published
		TokenPublished { currency_code: CurrencyCode, initial_issuance: BalanceOf<T>, }
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Did Not Exists
		DIDDoesNotExist,
		/// Balance too low
		BalanceTooLow,
		/// VC is not owned by the given DID
		DidNotRegisteredWithVC,
		/// Linked VC does not exist
		LinkedVCNotFound,
		/// The given VCId does not exist on chain
		VCDoesNotExist,
		/// VC status is Inactive, cant be use it
		VCIsNotActive,
		/// VC is already used, can't reused
		VCAlreadyUsed,
		/// Currency code does not match with tokenchain auth vc details
		CurrencyCodeMismatch,
		/// Invalid VC Type
		IncorrectVC,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Transfer to admin from reserved amount for operational costs
		// The dispatch origin for this call must be `Signed` by a validator account.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(2))]
		pub fn withdraw_reserved(
			origin: OriginFor<T>,
			to: Identifier,
			from: Identifier,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let _ = T::WithdrawOrigin::ensure_origin(origin)?;

			let from_acc = Self::get_account_id(&from)?;
			let to_acc = Self::get_account_id(&to)?;


			// unreserve the mui balance required to issue new token
			T::Currency::unreserve(&from_acc, amount);
			// transfer amount to destination
			T::Currency::transfer(&from_acc, &to_acc, amount, ExistenceRequirement::KeepAlive)?;

			Self::deposit_event(Event::ReserveWithdrawn { from, to });

			Ok(())
		}

		/// Slash the balance from the Token Owner's account
		///
		/// The dispatch origin for this call must be `Signed` by a issuer account.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn slash_token(origin: OriginFor<T>, vc_id: VCid) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// TODO: Check if Currency Code matches with the one mentioned in VC
			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc(&sender, &vc_details, VCType::SlashTokens)?;

			let slash_vc: SlashMintTokens = T::VCResolution::decode_vc::<SlashMintTokens>(&vc_details.vc_property)?;
			let currency_code = Self::get_currency_code();
			ensure!(slash_vc.currency_code == currency_code, Error::<T>::CurrencyCodeMismatch);

			let amount: BalanceOf<T> = slash_vc.amount.try_into().ok().unwrap_or_default();

			let token_vc = Self::get_linked_vc::<SlashMintTokens>(&vc_details)?;
			let token_vc_owner = Self::get_account_id(&token_vc.owner)?;

			ensure!(T::Currency::can_slash(&token_vc_owner, amount), Error::<T>::BalanceTooLow);

			
			T::Currency::slash(&token_vc_owner, amount);
			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			Self::deposit_event(Event::TokenSlashed { balance: amount, vc_id });

			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn transfer_token(
			origin: OriginFor<T>,
			vc_id: VCid,
			to: Identifier,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc(&sender, &vc_details, VCType::TokenTransferVC)?;

			let transfer_vc: TokenTransferVC = T::VCResolution::decode_vc::<TokenTransferVC>(&vc_details.vc_property)?;
			let currency_code = Self::get_currency_code();
			ensure!(transfer_vc.currency_code == currency_code, Error::<T>::CurrencyCodeMismatch);

			let amount: BalanceOf<T> = transfer_vc.amount.try_into().ok().unwrap_or_default();
			
			let token_vc = Self::get_linked_vc::<TokenTransferVC>(&vc_details)?;
			let token_vc_owner = Self::get_account_id(&token_vc.owner)?;

			let to_acc = Self::get_account_id(&to)?;


			T::Currency::transfer(&token_vc_owner, &to_acc, amount, ExistenceRequirement::KeepAlive)?;
			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			Self::deposit_event(Event::TransferredWithVC { to, balance: amount, vc_id });

			Ok(().into())
		}

		/// Add amount to the issuer account
		///
		/// The dispatch origin for this call must be `Signed` by a issuer account.
		/// Sender must be part of vc
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn mint_token(origin: OriginFor<T>, vc_id: VCid) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc(&sender, &vc_details, VCType::MintTokens)?;

			let mint_vc: SlashMintTokens = T::VCResolution::decode_vc::<SlashMintTokens>(&vc_details.vc_property)?;
			let currency_code = Self::get_currency_code();
			ensure!(mint_vc.currency_code == currency_code, Error::<T>::CurrencyCodeMismatch);

			let amount: BalanceOf<T> = mint_vc.amount.try_into().ok().unwrap_or_default();
			
			let token_vc = Self::get_linked_vc::<SlashMintTokens>(&vc_details)?;
			let token_vc_owner = Self::get_account_id(&token_vc.owner)?;

			T::Currency::deposit_creating(&token_vc_owner, amount);

			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			Self::deposit_event(Event::TokenMinted { balance: amount, vc_id });

			Ok(().into())
		}
		
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn publish_token(origin: OriginFor<T>, vc_id: VCid) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let sender_did = Self::get_did(&sender)?;

			// Decode VC and property
			let vc_details = Self::get_vc(vc_id)?;
			Self::validate_vc_constraints(&vc_details, VCType::IssueTokenVC)?;
			let vc_property = T::VCResolution::decode_vc::<IssueTokenVC>(&vc_details.vc_property)?;
			Self::validate_token_prop(&vc_property)?;

			// Mint Initital Issuance
			let initial_issuance: BalanceOf<T> = vc_property.initial_issuance.try_into().ok().unwrap_or_default();
			let token_vc_owner = Self::get_account_id(&vc_details.owner)?;
			ensure!(sender_did == vc_details.owner, Error::<T>::IncorrectVC);

			// Update Storage
			// Enable Token Transactions
			Self::enable_token()?;
			
			T::Currency::deposit_creating(&token_vc_owner, initial_issuance);
			
			// update vc's is_used flag as used
			T::VCResolution::set_is_vc_used(&vc_id, true)?;

			Self::deposit_event(Event::TokenPublished { currency_code: vc_property.currency_code, initial_issuance });

			Ok(().into())
		}

		#[pallet::weight(10_000)]
		pub fn inc_provider(origin: OriginFor<T>, did: Identifier) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let account = Self::get_account_id(&did)?;

			if frame_system::Pallet::<T>::inc_consumers_without_limit(&account).is_err() {
				// This will leak a provider reference, however it only happens once (at
				// genesis) so it's really not a big deal and we assume that the user wants to
				// do this since it's the only way a non-endowed account can contain a session
				// key.
				frame_system::Pallet::<T>::inc_providers(&account);
			}

			Self::deposit_event(Event::ProviderIncreased { did, account });

			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Initialize tokenchain during genesis
		fn initialize_tokenchain(currency_code: &CurrencyCode, token_details: &TokenchainInfo) {
			TokenchainCurrency::<T>::set(*currency_code);
			TokenchainDetails::<T>::set(*token_details);
		}


		/// Getters
		
		/// Get Linked VC
		fn get_linked_vc<G: codec::Decode + HasVCId>(
			vc_details: &VC<T::Hash>,
		) -> Result<VC<T::Hash>, DispatchError> {
			let vc_property: G = T::VCResolution::decode_vc::<G>(&vc_details.vc_property)?;
			Ok(T::VCResolution::get_vc(&vc_property.vc_id()).ok_or_else(|| Error::<T>::LinkedVCNotFound)?)
		}

		/// Get vc struct
		fn get_vc(vc_id: VCid) -> Result<VC<T::Hash>, DispatchError> {
			Ok(T::VCResolution::get_vc(&vc_id).ok_or_else(|| Error::<T>::VCDoesNotExist)?)
		}

		fn get_did(account: &T::AccountId) -> Result<Identifier, DispatchError> {
			Ok(T::DidResolution::get_did(&account).ok_or_else(|| Error::<T>::DIDDoesNotExist)?)
		}
	
		/// Get Account Id from DID and throw error if it doesnot Exist
		fn get_account_id(identifier: &Identifier) -> Result<T::AccountId, DispatchError> {
			Ok(T::DidResolution::get_account_id(identifier)
				.ok_or_else(|| Error::<T>::DIDDoesNotExist)?)
		}

		/// Get Chain's Currency code
		fn get_currency_code() -> CurrencyCode {
			TokenchainCurrency::<T>::get()
		}


		// Validation Functions

		/// Validate vc
		fn validate_token_prop(
			token_vc_prop: &IssueTokenVC,
		) -> DispatchResult {
			let currency_code = Self::get_currency_code();
			ensure!(currency_code == token_vc_prop.currency_code, Error::<T>::CurrencyCodeMismatch);
			Ok(())
		}


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


		// Update Storage

		/// It's called after token is published to enable transaction using the token
		fn enable_token() -> DispatchResultWithPostInfo {
			let balances = *b"Balances\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
			let token = *b"Token\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
			let _balances_extrinsics_list = vec![
					*b"transfer\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"transfer_all\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"transfer_keep_alive\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"transfer_with_memo\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"burn_balance\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"force_transfer\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"force_unreserve\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"set_balance\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
			];
			let _token_extrinsics_list = vec![
					*b"withdraw_reserved\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"slash_token\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"transfer_token\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"mint_token\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
					*b"inc_provider\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
			];
			// TODO: Allow to pass extrinsic list as well
			T::ExtrinsicResolution::remove_all_restricted(balances)?;
			T::ExtrinsicResolution::remove_all_restricted(token)?;

			Ok(().into())
		}


		/// An account is being created.
		pub fn on_created_account(did: Identifier) {
			let who = T::DidResolution::get_account_id(&did);
			if who.is_some() {
				T::OnNewAccount::on_new_account(&who.unwrap());
			}
		}

		/// Do anything that needs to be done after an account has been killed.
		pub fn on_killed_account(did: Identifier) {
			let who = T::DidResolution::get_account_id(&did);
			if who.is_some() {
				T::OnKilledAccount::on_killed_account(&who.unwrap());
			}
		}

	}
}

// Implement StoredMap for a simple single-item, kill-account-on-remove system. This works fine for
// storing a single item which is required to not be empty/default for the account to exist.
// Anything more complex will need more sophisticated logic.
impl<T: Config> StoredMap<T::AccountId, T::AccountData> for Pallet<T> {
	fn get(k: &T::AccountId) -> T::AccountData {
		let did = T::DidResolution::get_did(k).unwrap_or_default();
		Account::<T>::get(did).data
	}

	fn try_mutate_exists<R, E: From<DispatchError>>(
		k: &T::AccountId,
		f: impl FnOnce(&mut Option<T::AccountData>) -> Result<R, E>,
	) -> Result<R, E> {
		let did = T::DidResolution::get_did(k).unwrap_or_default();
		Account::<T>::try_mutate_exists(did, |maybe_value| {
			let existed = maybe_value.is_some();
			let (maybe_prefix, mut maybe_data) = split_inner(maybe_value.take(), |account| {
				(
					(account.nonce, account.consumers, account.providers, account.sufficients),
					account.data,
				)
			});
			f(&mut maybe_data).map(|result| {
				*maybe_value = maybe_data.map(|data| {
					let (nonce, consumers, providers, sufficients) =
						maybe_prefix.unwrap_or_default();
					AccountInfo { nonce, consumers, providers, sufficients, data }
				});
				(existed, maybe_value.is_some(), result)
			})
		})
		.map(|(existed, exists, v)| {
			if !existed && exists {
				Self::on_created_account(did.clone());
			} else if existed && !exists {
				Self::on_killed_account(did.clone());
			}
			v
		})
	}
}
