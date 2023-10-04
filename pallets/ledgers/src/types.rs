use super::*;

/// Currency Code type
pub type CurrencyCode = [u8; 8];
/// max length of Locks vec
pub type MaxLockLen = ConstU32<100>;
/// max length of Token name vec
pub type MaxTokenNameLen = ConstU32<16>;
/// max length of Token name vec
pub type MaxCCodeLen = ConstU32<8>;
/// Token Name
pub type TokenName = BoundedVec<u8, MaxTokenNameLen>;
/// Currency Code Bytes
pub type CurrencyCodeArray = BoundedVec<u8, MaxCCodeLen>;


/// An index to a block.
pub type BlockNumber = u32;

/// A single lock on a balance. There can be many of these on an account and
/// they "overlap", so the same balance is frozen by multiple locks.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct BalanceLock<Balance> {
    /// An identifier for this lock. Only one lock may be in existence for each
    /// identifier.
    pub id: LockIdentifier,
    /// The amount which the free balance may not drop below when this lock is
    /// in effect.
    pub amount: Balance,
}

/// Information of an account.
#[derive(Clone, Eq, PartialEq, Default, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct TokenAccountInfo<Index, TokenAccountData> {
    /// The number of transactions this account has sent.
    pub nonce: Index,
    /// The additional data that belongs to this account. Used to store the balance(s) in a lot of
    /// chains.
    pub data: TokenAccountData,
}

/// balance information for an account.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TokenAccountData {
    /// Non-reserved part of the balance. There may still be restrictions on
    /// this, but it is the total pool what may in principle be transferred,
    /// reserved.
    ///
    /// This is the only balance that matters in terms of most operations on
    /// tokens.
    pub free: TokenBalance,
    /// Balance which is reserved and may not be used at all.
    ///
    /// This can still get slashed, but gets slashed last of all.
    ///
    /// This balance is a 'reserve' balance that other subsystems use in order
    /// to set aside tokens that are still 'owned' by the account holder, but
    /// which are suspendable.
    pub reserved: TokenBalance,
    /// The amount that `free` may not drop below when withdrawing.
    pub frozen: TokenBalance,
}

impl TokenAccountData {
    /// The amount that this account's free balance may not be reduced beyond.
    pub fn frozen(&self) -> TokenBalance {
        self.frozen
    }
    /// The total balance in this account including any that is reserved and
    /// ignoring any frozen.
    pub fn total(&self) -> TokenBalance {
        self.free.saturating_add(self.reserved)
    }
}

/// currency information.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct TokenDetails {
    pub token_name: TokenName,
    pub currency_code: CurrencyCodeArray,
    pub decimal: u8,
    pub block_number: BlockNumber,
}

/// Type for leaving a note when sending a transaction.
#[derive(PartialEq, Eq, Clone, Encode, Decode, Default, Debug, TypeInfo)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Memo(Vec<u8>);

/// The max length of Memo allowed
pub const MAXIMUM_MEMO_LEN: u32 = 128;

impl From<Vec<u8>> for Memo {
    fn from(raw: Vec<u8>) -> Self {
        Self(raw)
    }
}

impl From<&[u8]> for Memo {
    fn from(raw: &[u8]) -> Self {
        Self(raw.to_vec())
    }
}

impl AsRef<[u8]> for Memo {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Memo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.0))
    }
}

#[cfg(not(feature = "std"))]
impl core::fmt::Display for Memo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Memo {
    /// Add custom checks if required.
    /// Now adding only Memo length check to avoid extreme high length memo due to fee less transaction
    pub fn is_valid(&self) -> bool {
        return !(self.0.len() > MAXIMUM_MEMO_LEN as usize);
    }
}