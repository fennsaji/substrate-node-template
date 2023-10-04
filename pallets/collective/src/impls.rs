use super::pallet::*;
use metamui_primitives::{Did, traits::IsMember};

impl<T: Config<I>, I: 'static> IsMember for Pallet<T, I> {
  /// Check whether `who` is a member of the collective.
  fn is_member(who: &Did) -> bool {
  	// Note: The dispatchables *do not* use this to check membership so make sure
  	// to update those if this is changed.
  	Self::is_member(who)
  }
}