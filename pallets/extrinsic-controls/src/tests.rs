use crate::mock::*;
use super::*;
use frame_support::{ assert_ok, assert_noop };

#[test]
fn test_genesis_worked() {
	new_test_ext().execute_with(|| {
		// check WhitelistedExtrinsics storage
		assert_eq!(WhitelistedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), true);
		let empty_tuple = WhitelistedExtrinsics::<Test>::get(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME);
		assert_eq!(empty_tuple == (), true);

		// check RestrictedExtrinsics storage
		assert_eq!(RestrictedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), true);
		let empty_tuple = RestrictedExtrinsics::<Test>::get(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME);
		assert_eq!(empty_tuple == (), true);
  })
}

#[test]
fn test_whitelist_extrinsic() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicControls::whitelist_extrinsic(
			Origin::root(),
			SECOND_PALLET_NAME,
      SECOND_FUNCTION_NAME
		));
    assert_eq!(WhitelistedExtrinsics::<Test>::contains_key(SECOND_PALLET_NAME, SECOND_FUNCTION_NAME), true);
	})
}

#[test]
fn test_whitelist_already_added_extrinsic() {
	new_test_ext().execute_with(|| {
		assert_eq!(WhitelistedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), true);
		assert_noop!(ExtrinsicControls::whitelist_extrinsic(
			Origin::root(),
			FIRST_PALLET_NAME,
      FIRST_FUNCTION_NAME
		), Error::<Test>::ExtrinsicAlreadyExists);
	})
}

#[test]
fn test_remove_extrinsic_from_whitelist() {
	new_test_ext().execute_with(|| {
    assert_eq!(WhitelistedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), true);

    assert_ok!(ExtrinsicControls::remove_whitelisted_extrinsic(
			Origin::root(),
			FIRST_PALLET_NAME,
      FIRST_FUNCTION_NAME
		));

    assert_eq!(WhitelistedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), false);
	})
}

#[test]
fn test_remove_non_existing_extrinsic_from_whitelist() {
	new_test_ext().execute_with(|| {
    assert_eq!(WhitelistedExtrinsics::<Test>::contains_key(SECOND_PALLET_NAME, SECOND_FUNCTION_NAME), false);

    assert_noop!(ExtrinsicControls::remove_whitelisted_extrinsic(
			Origin::root(),
			SECOND_PALLET_NAME,
      SECOND_FUNCTION_NAME
		), Error::<Test>::ExtrinsicDoesNotExist);
	})
}

// START RESTRICTED EXTRINSICS TEST
#[test]
fn test_restrict_extrinsic() {
	new_test_ext().execute_with(|| {
		assert_ok!(ExtrinsicControls::add_restricted_extrinsic(
			Origin::root(),
			SECOND_PALLET_NAME,
      SECOND_FUNCTION_NAME
		));
    assert_eq!(RestrictedExtrinsics::<Test>::contains_key(SECOND_PALLET_NAME, SECOND_FUNCTION_NAME), true);
	})
}

#[test]
fn test_restrict_already_restricted_extrinsic() {
	new_test_ext().execute_with(|| {
		assert_eq!(RestrictedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), true);
		assert_noop!(ExtrinsicControls::add_restricted_extrinsic(
			Origin::root(),
			FIRST_PALLET_NAME,
      FIRST_FUNCTION_NAME
		), Error::<Test>::ExtrinsicAlreadyExists);
	})
}

#[test]
fn test_remove_extrinsic_from_restricted_list() {
	new_test_ext().execute_with(|| {
    assert_eq!(RestrictedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), true);

    assert_ok!(ExtrinsicControls::remove_restricted_extrinsic(
			Origin::root(),
			FIRST_PALLET_NAME,
      FIRST_FUNCTION_NAME
		));

    assert_eq!(RestrictedExtrinsics::<Test>::contains_key(FIRST_PALLET_NAME, FIRST_FUNCTION_NAME), false);
	})
}

#[test]
fn test_remove_non_existing_extrinsic_from_restricted_list() {
	new_test_ext().execute_with(|| {
    assert_eq!(RestrictedExtrinsics::<Test>::contains_key(SECOND_PALLET_NAME, SECOND_FUNCTION_NAME), false);

    assert_noop!(ExtrinsicControls::remove_restricted_extrinsic(
			Origin::root(),
			SECOND_PALLET_NAME,
      SECOND_FUNCTION_NAME
		), Error::<Test>::ExtrinsicDoesNotExist);
	})
}
// END RESTRICTED EXTRINSICS TEST
