use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature, OpaqueExtrinsic,
};
use sp_std::prelude::*;

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use scale_info::TypeInfo;
use frame_support::{traits::ConstU32, BoundedVec};
use frame_support::pallet_prelude::MaxEncodedLen;
use sp_core::sr25519::Signature as SRSignature;
use sp_core::sr25519;


/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// A timestamp: milliseconds since the unix epoch.
/// `u64` is enough to represent a duration of half a billion years, when the
/// time scale is milliseconds.
pub type Time = u64;

/// Digest item type.
pub type DigestItem = generic::DigestItem;
/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// DID
pub type Did = [u8; 32];
/// VC Id
pub type VCid = [u8; 32];
/// VC Hex
pub type VCHex = Vec<u8>;
/// VC Property type
pub type VCProperty = [u8; 128];
/// 2nd part of did
pub type DidRegion = [u8; 20];
/// Max Allowed Regions
pub type MaxAllowedRegions = ConstU32<20>;
///Allowed Regions Type
pub type AllowedRegionsVec = BoundedVec<DidRegion, MaxAllowedRegions>;

/// Pallet Name Type
pub type PalletName = [u8; 32];
/// Function Name Type
pub type FunctionName = [u8; 32];
/// Public Key Type
pub type PublicKey = sr25519::Public;
/// Maximum Size of Metadata
pub type MaxMetadata = ConstU32<32>;
/// Maximum Length of Registration Number
pub type MaxRegNumLen = ConstU32<32>;
/// Maximum Length of Company Name
pub type MaxCompNameLen = ConstU32<32>;
///  Maximum issuers allowed for VC
pub type MaxIssuers = ConstU32<20>;
/// Metadata Type
pub type Metadata = BoundedVec<u8, MaxMetadata>;
/// Registration Number Type
pub type RegistrationNumber = BoundedVec<u8, MaxRegNumLen>;
/// Company Name Type
pub type CompanyName = BoundedVec<u8, MaxCompNameLen>;
/// Currency Code
pub type CurrencyCode = [u8; 8];

/// Type of VCs
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VCType {
  /// VC to create a Token
  TokenVC,
  /// VC to slash token
  SlashTokens,
  /// VC to mint token
  MintTokens,
  /// VC to transfer token
  TokenTransferVC,
  /// VC for generic purpose
  GenericVC,
  /// VC to create public did
  PublicDidVC,
  /// VC to create private did
  PrivateDidVC,
  /// Reset Public Key of a did
  ResetPubKeyVC,
  /// Authorise a new parachain
  TokenchainAuthVC,
  /// VC to publish token
  IssueTokenVC,
}

/// Struct for VC
#[derive(Clone, PartialEq, Eq, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VC<Hash> {
  /// Hash of the data in VC
  pub hash: Hash,
  /// Owner of VC
  pub owner: Did,
  /// Issuers of VC
  pub issuers: BoundedVec<Did, MaxIssuers>,
  /// Signatures of Issuers on hash
  pub signatures: BoundedVec<SRSignature, MaxIssuers>,
  /// If VC is used or not
  pub is_vc_used: bool,
  /// If VC is active or not
  pub is_vc_active: bool,
  /// Type of VC
  pub vc_type: VCType,
  /// VC payload
  pub vc_property: VCProperty,
}

/// SlashMintTokens Type VC
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlashMintTokens {
  /// VCid field
  pub vc_id: VCid,
  /// Currency Code
  pub currency_code: CurrencyCode,
  /// Amount field
  pub amount: u128,
}

/// TokenTransfer Type VC
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenTransferVC {
  /// VCid field
  pub vc_id: VCid,
  /// Currency Code
  pub currency_code: CurrencyCode,
  /// Amount field
  pub amount: u128,
}

/// PublicDidVC Type VC
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PublicDidVC {
  /// Public Key
	pub public_key: PublicKey,
  /// Registration Number
	pub registration_number: RegistrationNumber,
  /// Name of Company
	pub company_name: CompanyName,
  /// Did
  pub did: Did,
}

/// PrivateDidVC Type VC
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrivateDidVC {
  /// Public Key
	pub public_key: PublicKey,
  /// Did
  pub did: Did,
}

/// VC used to create Tokens
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenVC {
  /// Token Name
  pub token_name: [u8; 16],
  /// Reservable Balance
  pub reservable_balance: u128,
  /// Decimal
  pub decimal: u8,
  /// Currency Code
  pub currency_code: CurrencyCode,
}

/// VC used to create Tokens
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IssueTokenVC {
  /// Currency Code
  pub currency_code: CurrencyCode,
  /// Initital Issuance
  pub initial_issuance: u128,
}

/// VC used to create Tokens
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TokenchainAuthVC {
  /// Token Name
  pub token_name: [u8; 16],
  /// Reservable Balance
  pub reservable_balance: u128,
  /// Decimal
  pub decimal: u8,
  /// Currency Code
  pub currency_code: CurrencyCode,
  /// Total Issuance
  pub initial_issuance: u128
}

/// Did Type 
#[derive(Decode, Copy, Encode, TypeInfo, Clone, PartialEq, Eq, Debug, MaxEncodedLen)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DidType {
  /// Public Did
  Public,
  /// Private Did
  Private,
}

/// ResetPubKeyVC Type VC
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResetPubKeyVC {
  /// VCid field
  pub vc_id: Option<VCid>,
  /// Did
  pub did: Did,
  /// New Account Id
  pub new_public_key: PublicKey,
  /// Old Account Id
  pub old_public_key: Option<PublicKey>,
}
