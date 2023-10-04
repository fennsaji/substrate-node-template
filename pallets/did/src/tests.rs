use crate::mock::*;
use crate::types::*;
use crate::mock::Did;
use frame_support::error::BadOrigin;
use pallet_vc;
use super::*;
use frame_support::{ assert_ok, assert_noop, bounded_vec, BoundedVec, traits::ConstU32 };
use sp_core::{sr25519, Pair, H256};

//START GENESIS TESTING
#[test]
fn test_genesis_worked() {
	new_test_ext().execute_with(|| {
		let validator_pubkey: sr25519::Public = sr25519::Pair::from_seed(&VALIDATOR_SEED).public();
		assert_eq!(DIDs::<Test>::contains_key(VALIDATOR_DID.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(VALIDATOR_DID.clone()), true);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&validator_pubkey)),
			true
		);

		let (did_doc, _block_number) = Did::get_did_details(VALIDATOR_DID.clone()).unwrap();
		match did_doc {
			DIdentity::Public(public_did) => {
				assert_eq!(public_did.identifier, VALIDATOR_DID);
		    assert_eq!(public_did.public_key, validator_pubkey);
			},
			DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, VALIDATOR_DID);
		    assert_eq!(private_did.public_key, validator_pubkey);
			},
		}

		let regional_pubkey: sr25519::Public = sr25519::Pair::from_seed(&REGIONAL_SEED).public();
		assert_eq!(DIDs::<Test>::contains_key(REGIONAL_DID.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(REGIONAL_DID.clone()), true);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&regional_pubkey)),
			true
		);

		let (did_doc, block_number) = Did::get_did_details(REGIONAL_DID.clone()).unwrap();
		match did_doc {
			DIdentity::Public(public_did) => {
				assert_eq!(public_did.identifier, REGIONAL_DID);
		    assert_eq!(public_did.public_key, regional_pubkey);
			},
			DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, REGIONAL_DID);
		    assert_eq!(private_did.public_key, regional_pubkey);
			},
		}
		assert_eq!(block_number, 0);
	})
}
//END GENESIS TESTING

// START ADD_DID TESTING

// START LOCAL_VALIDATOR_ADDS_PRIVATE_DID_IN_ANOTHER_REGION TESTING
#[test]
fn test_local_validator_adds_private_did_in_another_region() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:region2:alice2\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_noop!(VcPallet::store(Origin::signed(REGIONAL_ACCOUNT), did_vc_hex), pallet_vc::Error::<Test>::InvalidDidRegion);
	})
}
// END LOCAL_VALIDATOR_ADDS_PRIVATE_DID_IN_ANOTHER_REGION TESTING

// START LOCAL_VALIDATOR_ADDS_PUBLIC_DID_IN_ANOTHER_REGION TESTING
#[test]
fn test_local_validator_adds_public_did_in_another_region() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:region2:alice2\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_noop!(VcPallet::store(Origin::signed(REGIONAL_ACCOUNT), did_vc_hex), pallet_vc::Error::<Test>::InvalidDidRegion);
	})
}
// END LOCAL_VALIDATOR_ADDS_PUBLIC_DID_IN_ANOTHER_REGION TESTING

// START ADD_INVALID_PRIVATE_DID TESTING
#[test]
fn test_add_invalid_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"d\0d:ssid:alice2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		assert_noop!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		), Error::<Test>::InvalidDid);
	})
}
// END ADD_INVALID_PRIVATE_DID TESTING

// START ADD_INVALID_PUBLIC_DID TESTING
#[test]
fn test_add_invalid_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"d\0d:ssid:alice2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		assert_noop!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		), Error::<Test>::InvalidDid);
	})
}
// END ADD_INVALID_PUBLIC_DID TESTING

// START ADD_PRIVATE_DID TESTING
#[test]
fn test_add_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));
		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			true
		);

		let (did_doc, _block_number) = Did::get_did_details(identifier.clone()).unwrap();
		match did_doc {
			types::DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, identifier);
				assert_eq!(private_did.public_key, public_key);
				let did_lookup = RLookup::<Test>::get(Did::get_accountid_from_pubkey(&public_key));
				match did_lookup {
					Some(did) => assert_eq!(did, identifier.clone()),
					None => assert!(false),
				}
			},
			_ => {}
		};
	})
}
// END ADD_PRIVATE_DID TESTING

// START ADD_PUBLIC_DID TESTING
#[test]
fn test_add_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);
		
		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			true
		);

		let (did_doc, _block_number) = Did::get_did_details(identifier.clone()).unwrap();
		match did_doc {
			types::DIdentity::Public(public_did) => {
				assert_eq!(public_did.identifier, identifier);
				assert_eq!(public_did.public_key, public_key);
				let did_lookup = RLookup::<Test>::get(Did::get_accountid_from_pubkey(&public_key));
				match did_lookup {
					Some(did) => assert_eq!(did, identifier.clone()),
					None => assert!(false),
				}
			},
			_ =>{}
		}
	})
}
// END ADD_PUBLIC_DID TESTING

// START ADD_EXISTING_PRIVATE_DID TESTING
#[test]
fn test_add_existing_private_did() {
	new_test_ext().execute_with(|| {
		// Adding the DID initialized at the time of genesis, so this test should fail
		let identifier = *b"did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_noop!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex), pallet_vc::Error::<Test>::DIDAlreadyExists);
	})
}
// END ADD_EXISTING_PRIVATE_DID TESTING

// START ADD_EXISTING_PUBLIC_DID TESTING
#[test]
fn test_add_existing_public_did() {
	new_test_ext().execute_with(|| {
		// Adding the DID initialized at the time of genesis, so this test should fail
		let identifier = VALIDATOR_DID;
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_noop!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex), pallet_vc::Error::<Test>::DIDAlreadyExists);
	})
}
// END ADD_EXISTING_PUBLIC_DID TESTING
//END ADD_DID TESTING

//START ADD_EXISTING_PUBLIC_KEY_FOR_PRIVATE_DID TESTING
#[test]
fn test_add_existing_pubkey_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		// To generate the same public key as the one used in genesis so it will throw error
		let public_key = sr25519::Pair::from_seed(&VALIDATOR_SEED).public();

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);
		
		assert_noop!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex), pallet_vc::Error::<Test>::PublicKeyRegistered);
	})
}
//END ADD_EXISTING_PUBLIC_KEY_FOR_PRIVATE_DID TESTING

//START ADD_EXISTING_PUBLIC_KEY_FOR_PUBLIC_DID TESTING
#[test]
fn test_add_existing_pubkey_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Pair::from_seed(&VALIDATOR_SEED).public();

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_noop!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex), pallet_vc::Error::<Test>::PublicKeyRegistered);
	})
}
//END ADD_EXISTING_PUBLIC_KEY_FOR_PUBLIC_DID TESTING

//START NON_EXISTING_DID_REMOVE TESTING
#[test]
fn test_remove_non_existing_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:DoesNotExist\0\0\0\0\0\0\0\0\0\0\0";

		assert_noop!((Did::remove(Origin::root(), identifier.clone(), None)), Error::<Test>::DIDDoesNotExist);
	})
}
//END NON_EXISTING_DID_REMOVE TESTING

//START NON_VALIDATOR_REMOVES_DID TESTING
#[test]
fn test_non_validator_removes_did() {
	new_test_ext().execute_with(|| {
		let identifier = VALIDATOR_DID;
		assert_noop!((Did::remove(Origin::signed(1), identifier.clone(), None)), BadOrigin);
	})
}
//END NON_VALIDATOR_REMOVES_DID TESTING

//START REMOVE_PRIVATE_DID TESTING
#[test]
fn test_remove_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert_ok!(Did::remove(Origin::root(), identifier.clone(), None));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), false);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), false);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			false
		);
	})
}
//END REMOVE_PRIVATE_DID TESTING

//START REMOVE_PUBLIC_DID TESTING
#[test]
fn test_remove_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public(identifier);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert_ok!(Did::remove(Origin::root(), identifier.clone(), None));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), false);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), false);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			false
		);
	})
}
//END REMOVE_PUBLIC_DID TESTING

//START ROTATE_KEY_FOR_PRIVATE_DID TESTING
#[test]
fn test_rotate_key_for_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([3; 32]);
		let metadata: types::Metadata = Default::default();

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let public_key2 = sr25519::Public([4; 32]);

		run_to_block(3);

		let reset_key_vc = get_reset_pub_key_vc(identifier, did_vc_id, public_key2);
		let (reset_vc_id, reset_key_vc_hex) = get_vc_id_and_hex(identifier, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));

		assert_ok!(Did::rotate_key(
			Origin::signed(VALIDATOR_ACCOUNT),
			reset_vc_id,
			None
		));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), true);

		// Ensure only a singly pubkey is mapped to a DID - inspired from toufeeq's testing
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			false
		);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key2)),
			true
		);

		let (did_doc, block_number) = Did::get_did_details(identifier.clone()).unwrap();
		match did_doc {
			types::DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, identifier);
				assert_eq!(private_did.public_key, public_key2);
				assert_eq!(private_did.metadata, metadata);
				assert_eq!(block_number, 3);
			},
			_ => {}
		}
		// check the rotated key has been added to the history of the DID
		assert_eq!(PrevKeys::<Test>::contains_key(identifier.clone()), true);
		let prev_key_list = Did::get_prev_key_details(identifier.clone()).unwrap();
		assert_eq!(prev_key_list.is_empty(), false);
		assert_eq!(prev_key_list.len(), 1);

		let (last_pub_key, block_number) = prev_key_list.first().cloned().unwrap();
		assert_eq!(last_pub_key, Did::get_accountid_from_pubkey(&public_key));
		assert_eq!(block_number, 0);
	})
}
//END ROTATE_KEY_FOR_PRIVATE_DID TESTING

//START ROTATE_KEY_FOR_PUBLIC_DID TESTING
#[test]
fn test_rotate_key_for_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([3; 32]);
		let metadata: types::Metadata = Default::default();

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let public_key2 = sr25519::Public([4; 32]);

		run_to_block(3);

		let reset_key_vc = get_reset_pub_key_vc(identifier, did_vc_id, public_key2);
		let (reset_vc_id, reset_key_vc_hex) = get_vc_id_and_hex(identifier, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));

		assert_ok!(Did::rotate_key(
			Origin::signed(VALIDATOR_ACCOUNT),
			reset_vc_id,
			None
		));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), true);

		// Ensure only a singly pubkey is mapped to a DID - inspired from toufeeq's testing
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			false
		);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key2)),
			true
		);

		let (did_doc, block_number) = Did::get_did_details(identifier.clone()).unwrap();
		match did_doc {
			types::DIdentity::Public(public_did) => {
				assert_eq!(public_did.identifier, identifier);
				assert_eq!(public_did.public_key, public_key2);
				assert_eq!(public_did.metadata, metadata);
				assert_eq!(block_number, 3);
			},
			_ => {}
		}
		// check the rotated key has been added to the history of the DID
		assert_eq!(PrevKeys::<Test>::contains_key(identifier.clone()), true);
		let prev_key_list = Did::get_prev_key_details(identifier.clone()).unwrap();
		assert_eq!(prev_key_list.is_empty(), false);
		assert_eq!(prev_key_list.len(), 1);

		let (last_pub_key, block_number) = prev_key_list.first().cloned().unwrap();
		assert_eq!(last_pub_key, Did::get_accountid_from_pubkey(&public_key));
		assert_eq!(block_number, 0);
	})
}
//END ROTATE_KEY_FOR_PUBLIC_DID TESTING

//START ROTATE_KEY_HISTORY_FOR_PRIVATE_DID TESTING
#[test]
fn test_rotate_key_history_for_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([3; 32]);
		let metadata = types::Metadata::default();

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let public_key2 = sr25519::Public([4; 32]);

		run_to_block(3);

		let reset_key_vc = get_reset_pub_key_vc(identifier, did_vc_id, public_key2);
		let (reset_vc_id, reset_key_vc_hex) = get_vc_id_and_hex(identifier, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));

		assert_ok!(Did::rotate_key(
			Origin::signed(VALIDATOR_ACCOUNT),
			reset_vc_id,
			None
		));

		run_to_block(8);

		let public_key3 = sr25519::Public([7; 32]);

		let reset_key_vc = get_reset_pub_key_vc(identifier, did_vc_id, public_key3);
		let (reset_vc_id, reset_key_vc_hex) = get_vc_id_and_hex(identifier, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));

		assert_ok!(Did::rotate_key(
			Origin::signed(VALIDATOR_ACCOUNT),
			reset_vc_id,
			None
		));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), true);

		// Ensure only a singly pubkey is mapped to a DID -  inspired from toufeeq's testing
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			false
		);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key2)),
			false
		);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key3)),
			true
		);

		let (did_doc, block_number) = Did::get_did_details(identifier.clone()).unwrap();
		match did_doc {
			types::DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, identifier);
				assert_eq!(private_did.public_key, public_key3);
				assert_eq!(private_did.metadata, metadata);
				assert_eq!(block_number, 8);
			},
			_ => {}
		}

		// check the rotated key has been added to the history of the DID
		assert_eq!(PrevKeys::<Test>::contains_key(identifier.clone()), true);
		let prev_key_list = Did::get_prev_key_details(identifier.clone()).unwrap();
		assert_eq!(prev_key_list.is_empty(), false);
		assert_eq!(prev_key_list.len(), 2);

		let (last_pub_key, block_number) = prev_key_list[0];
		assert_eq!(last_pub_key, Did::get_accountid_from_pubkey(&public_key));
		assert_eq!(block_number, 0);

		let (last_pub_key2, block_number2) = prev_key_list[1];
		assert_eq!(last_pub_key2, Did::get_accountid_from_pubkey(&public_key2));
		assert_eq!(block_number2, 3);
	})
}
//END ROTATE_KEY_HISTORY_FOR_PRIVATE_DID TESTING

//START ROTATE_KEY_HISTORY_FOR_PUBLIC_DID TESTING
#[test]
fn test_rotate_key_history_for_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([3; 32]);
		let metadata = types::Metadata::default();

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let public_key2 = sr25519::Public([4; 32]);

		run_to_block(3);

		let reset_key_vc = get_reset_pub_key_vc(identifier, did_vc_id, public_key2);
		let (reset_vc_id, reset_key_vc_hex) = get_vc_id_and_hex(identifier, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));

		assert_ok!(Did::rotate_key(
			Origin::signed(VALIDATOR_ACCOUNT),
			reset_vc_id,
			None
		));

		run_to_block(8);

		let public_key3 = sr25519::Public([7; 32]);
		let reset_key_vc = get_reset_pub_key_vc(identifier, did_vc_id, public_key3);
		let (reset_vc_id, reset_key_vc_hex) = get_vc_id_and_hex(identifier, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));


		assert_ok!(Did::rotate_key(
			Origin::signed(VALIDATOR_ACCOUNT),
			reset_vc_id,
			None
		));

		assert_eq!(DIDs::<Test>::contains_key(identifier.clone()), true);
		assert_eq!(Lookup::<Test>::contains_key(identifier.clone()), true);

		// Ensure only a singly pubkey is mapped to a DID -  inspired from toufeeq's testing
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key)),
			false
		);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key2)),
			false
		);
		assert_eq!(
			RLookup::<Test>::contains_key(Did::get_accountid_from_pubkey(&public_key3)),
			true
		);

		let (did_doc, block_number) = Did::get_did_details(identifier.clone()).unwrap();
		match did_doc {
			types::DIdentity::Public(public_did) => {
				assert_eq!(public_did.identifier, identifier);
				assert_eq!(public_did.public_key, public_key3);
				assert_eq!(public_did.metadata, metadata);
				assert_eq!(block_number, 8);
			},
			_ => {}
		}

		// check the rotated key has been added to the history of the DID
		assert_eq!(PrevKeys::<Test>::contains_key(identifier.clone()), true);
		let prev_key_list = Did::get_prev_key_details(identifier.clone()).unwrap();
		assert_eq!(prev_key_list.is_empty(), false);
		assert_eq!(prev_key_list.len(), 2);

		let (last_pub_key, block_number) = prev_key_list[0];
		assert_eq!(last_pub_key, Did::get_accountid_from_pubkey(&public_key));
		assert_eq!(block_number, 0);

		let (last_pub_key2, block_number2) = prev_key_list[1];
		assert_eq!(last_pub_key2, Did::get_accountid_from_pubkey(&public_key2));
		assert_eq!(block_number2, 3);
	})
}
//END ROTATE_KEY_HISTORY_FOR_PUBLIC_DID TESTING

//START ROTATE_KEY_FOR_NON_EXISTENT_DID TESTING
// No separate tests needed for public and private did types as principle is same
#[test]
#[should_panic]
fn test_rotate_key_for_non_existent_did() {
	new_test_ext().execute_with(|| {
		let identifier2 = *b"alice2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let did_vc_id: [u8; 32] = [0; 32];

		let public_key = sr25519::Public([7; 32]);
		let reset_key_vc = get_reset_pub_key_vc(identifier2, did_vc_id, public_key);
		let (_, reset_key_vc_hex) = get_vc_id_and_hex(identifier2, reset_key_vc, VCType::ResetPubKeyVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), reset_key_vc_hex));
	})
}
//END ROTATE_KEY_FOR_NON_EXISTENT_DID TESTING

// START CHANGE_DID_TYPE_FOR_PRIVATE_DID TESTING
#[test]
fn test_update_did_type_for_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let old_did_tuple = DIDs::<Test>::get(identifier).unwrap();
		let old_identifier: [u8; 32];
		let old_public_key: PublicKey;
		let old_metadata: Metadata;
		let old_blocknumber = old_did_tuple.1;

		let new_registration_number: RegistrationNumber = Default::default();
		let new_company_name: CompanyName = Default::default();

		match old_did_tuple.0 {
			DIdentity::Private(priv_did_struct) => {
				old_identifier = priv_did_struct.identifier;
				old_public_key = priv_did_struct.public_key;
				old_metadata = priv_did_struct.metadata;
			}

			// need this match arm to avoid 'possibly-uninitialized' errors
			DIdentity::Public(_) => {
				old_identifier = types::Did::default();
				old_public_key = sr25519::Public([100; 32]);
				old_metadata = types::Metadata::default();
			}
		};

		assert_ok!(Did::change_did_type(
			Origin::root(),
			identifier,
			DidType::Public,
			None
		));
		let current_block_no = <frame_system::Pallet<Test>>::block_number();

		let new_did_tuple = DIDs::<Test>::get(identifier).unwrap();

		match new_did_tuple.0 {
			DIdentity::Public(pub_did_struct) => {
				assert_eq!(old_identifier, pub_did_struct.identifier);
				assert_eq!(old_public_key, pub_did_struct.public_key);
				assert_eq!(old_metadata, pub_did_struct.metadata);
				assert_eq!(new_registration_number, pub_did_struct.registration_number);
				assert_eq!(new_company_name, pub_did_struct.company_name);
			}
			_ => {}
		};
		assert_eq!(old_blocknumber, new_did_tuple.1);
		let type_change_history = TypeChangeHistory::<Test>::get(identifier).unwrap();
		assert_eq!(type_change_history[0].0, DidType::Private);
		assert_eq!(current_block_no, type_change_history[0].1);
	})
}
// END CHANGE_DID_TYPE_FOR_PRIVATE_DID TESTING

// START CHANGE_DID_TYPE_FOR_PUBLIC_DID TESTING
#[test]
fn test_update_did_type_for_public_did() {
	new_test_ext().execute_with(|| {		
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex( identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let old_did_tuple = DIDs::<Test>::get(identifier).unwrap();
		let old_identifier: [u8; 32];
		let old_public_key: PublicKey;
		let old_metadata: Metadata;
		let old_blocknumber = old_did_tuple.1;

		match old_did_tuple.0 {
			DIdentity::Public(pub_did_struct) => {
				old_identifier = pub_did_struct.identifier;
				old_public_key = pub_did_struct.public_key;
				old_metadata = pub_did_struct.metadata;
			}

			// need this match arm to avoid 'possibly-uninitialized' errors
			DIdentity::Private(_) => {
				old_identifier = types::Did::default();
				old_public_key = sr25519::Public([100; 32]);
				old_metadata = types::Metadata::default();
			}

		};

		assert_ok!(Did::change_did_type(
			Origin::root(),
			identifier,
			DidType::Private,
			None
		));

		let current_block_no = <frame_system::Pallet<Test>>::block_number();
		let new_did_tuple = DIDs::<Test>::get(identifier).unwrap();

		match new_did_tuple.0 {
			DIdentity::Private(priv_did_struct) => {
				assert_eq!(old_identifier, priv_did_struct.identifier);
				assert_eq!(old_public_key, priv_did_struct.public_key);
				assert_eq!(old_metadata, priv_did_struct.metadata);
			}
			_ => {}
		};

		assert_eq!(old_blocknumber, new_did_tuple.1);
		let type_change_history = TypeChangeHistory::<Test>::get(identifier).unwrap();
		assert_eq!(type_change_history[0].0, DidType::Public);
		assert_eq!(current_block_no, type_change_history[0].1);
	})
}
// END CHANGE_DID_TYPE_FOR_PUBLIC_DID TESTING

// START CHANGE_DID_TYPE_FOR_PRIVATE_DID_WITH_SAME_TYPE TESTING
#[test]
fn test_update_did_type_for_private_did_with_same_type() {
	new_test_ext().execute_with(|| {	
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert_noop!(
			Did::change_did_type(
				Origin::root(),
				identifier,
				DidType::Private,
				None
			),
			Error::<Test>::TypeAlreadySame
		);
	})
}
// END CHANGE_DID_TYPE_FOR_PRIVATE_DID_WITH_SAME_TYPE TESTING

// START CHANGE_DID_TYPE_FOR_PUBLIC_DID_WITH_SAME_TYPE TESTING
#[test]
fn test_update_did_type_for_public_did_with_same_type() {
	new_test_ext().execute_with(|| {		
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert_noop!(
			Did::change_did_type(
				Origin::root(),
				identifier,
				DidType::Public,
				None
			),
			Error::<Test>::TypeAlreadySame
		);
	})
}
// END CHANGE_DID_TYPE_FOR_PUBLIC_DID_WITH_SAME_TYPE TESTING

//START UPDATE_METADATA_FOR_PRIVATE_DID TESTING
#[test]
fn test_metadata_updation_for_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);
		let old_metadata = types::Metadata::default();

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		//assign new metadata to a variable
		let new_metadata: BoundedVec<u8, ConstU32<32>> = bounded_vec![0, 0, 0, 0, 0, 0, 0];

		//update the existing metadata with new metadata
		assert_ok!(Did::update_metadata(
			Origin::signed(VALIDATOR_ACCOUNT),
			identifier,
			new_metadata.clone()
		));

		//fetch did details
		let (did_doc, _block_number) = Did::get_did_details(identifier.clone()).unwrap();

		//check if the details are same as the ones we added above
		match did_doc {
			types::DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, identifier);
				assert_eq!(private_did.public_key, public_key);
				//check if the current metadata is the same as the new metadata
				assert_eq!(private_did.metadata, new_metadata);
				//check if the current metadata is not the same as the old metadata
				assert_ne!(private_did.metadata, old_metadata);
			},
			_ => {}
		}
	})
}
//END UPDATE_METADATA_FOR_PRIVATE_DID TESTING

//START UPDATE_METADATA_FOR_PUBLIC_DID TESTING
#[test]
fn test_metadata_updation_for_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);
		let old_metadata = types::Metadata::default();

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		//assign new metadata to a variable
		let new_metadata: BoundedVec<u8, ConstU32<32>> = bounded_vec![0, 0, 0, 0, 0, 0, 0];

		//update the existing metadata with new metadata
		assert_ok!(Did::update_metadata(
			Origin::signed(VALIDATOR_ACCOUNT),
			identifier,
			new_metadata.clone()
		));

		//fetch did details
		let (did_doc, _block_number) = Did::get_did_details(identifier.clone()).unwrap();

		match did_doc {
			types::DIdentity::Public(public_did) => {
				assert_eq!(public_did.identifier, identifier);
				assert_eq!(public_did.public_key, public_key);
				assert_eq!(public_did.metadata, new_metadata);
				assert_ne!(public_did.metadata, old_metadata);
			},
			_ => {}
		}
	})
}
//END UPDATE_METADATA_FOR_PUBLIC_DID TESTING

//TESTING HELPER FUNCTIONS DEFINED IN THE PALLET-IMPL

//START VALIDATE_DID TESTING
#[test]
fn test_did_validation() {
	new_test_ext().execute_with(|| {
		// without did: prefix
		let without_did_colon = *b"alice\0\0\0\0\0\0\0\0\\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(!Did::validate_did(without_did_colon));

		// zero did
		let zero_did = [0; 32];
		assert!(!Did::validate_did(zero_did));

		// zero after did: prefix
		let zero_after_did_colon = *b"did:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(!Did::validate_did(zero_after_did_colon));

		// space followed by zeros
		let space_followed_by_zero =
			*b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(!Did::validate_did(space_followed_by_zero));

		// space followed by correct did
		let space_followed_correct_did = *b" did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(!Did::validate_did(space_followed_correct_did));

		// correct did
		let correct_did = *b"did:ssid:swn\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(correct_did));
	})
}
//END VALIDATE_DID TESTING

//START GET_DID_DETAILS_FOR_PRIVATE_DID TESTING
#[test]
fn test_get_private_did_details() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);
		let metadata = types::Metadata::default();

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let (did_doc, _block_number) = Did::get_did_details(identifier.clone()).unwrap();

		//check if the details are same as the ones we added above
		match did_doc {
			types::DIdentity::Private(private_did) => {
				assert_eq!(private_did.identifier, identifier);
				assert_eq!(private_did.public_key, public_key);
				assert_eq!(private_did.metadata, metadata);
			},
			_ => {}
		}
	})
}
//END GET_DID_DETAILS_FOR_PRIVATE_DID TESTING

//START GET_DID_DETAILS_FOR_PUBLIC_DID TESTING
#[test]
fn test_get_public_did_details() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);
		let metadata = types::Metadata::default();

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let (did_doc, _block_number) = Did::get_did_details(identifier.clone()).unwrap();

		match did_doc {
			types::DIdentity::Public(public_did) => {
				//check if the details are same as the ones we added above
				assert_eq!(public_did.identifier, identifier);
				assert_eq!(public_did.public_key, public_key);
				assert_eq!(public_did.metadata, metadata);
			},
			_ => {}
		}
	})
}
//END GET_DID_DETAILS_FOR_PUBLIC_DID TESTING

//START GET_PUBLIC_KEY_FROM_PRIVATE_DID TESTING
#[test]
fn test_get_public_key_from_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let fetched_public_key = Did::get_pub_key(&identifier).unwrap();
		assert_eq!(fetched_public_key, public_key);
	})
}
//END GET_PUBLIC_KEY_FROM_PRIVATE_DID TESTING

//START GET_PUBLIC_KEY_FROM_PUBLIC_DID TESTING
#[test]
fn test_get_public_key_from_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let fetched_public_key = Did::get_pub_key(&identifier).unwrap();
		assert_eq!(fetched_public_key, public_key);
	})
}
//END GET_PUBLIC_KEY_FROM_PUBLIC_DID TESTING

//START CHECK_DID_PUBLIC_WITH_PRIVATE_DID TESTING
#[test]
fn test_check_did_public_with_private_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));
		
		assert_ok!(Did::create_private(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert!(!Did::check_did_public(&identifier));
	})
}
//END CHECK_DID_PUBLIC_WITH_PRIVATE_DID TESTING

//START CHECK_DID_PUBLIC_WITH_PUBLIC_DID TESTING
#[test]
fn test_check_did_public_with_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		assert!(Did::check_did_public(&identifier));
	})
}
//END CHECK_DID_PUBLIC_WITH_PUBLIC_DID TESTING

//START GET_ACCOUNT_ID_FROM_PUBLIC_KEY_OF_PRIVATE_DID TESTING
#[test]
fn test_get_account_id_from_public_key_of_private_did() {
	new_test_ext().execute_with(|| {
		let account_id = Did::get_accountid_from_pubkey(&sr25519::Pair::from_seed(&VALIDATOR_SEED).public());
		assert_eq!(account_id, VALIDATOR_ACCOUNT);
	})
}
//END GET_ACCOUNT_ID_FROM_PUBLIC_KEY_OF_PRIVATE_DID TESTING

//START GET_ACCOUNT_ID_FROM_PUBLIC_KEY_OF_PUBLIC_DID TESTING
#[test]
fn test_get_account_id_from_public_key_of_public_did() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:bob\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let bob_seed: [u8; 32] = [
			57, 143, 12, 40, 249, 136, 133, 224, 70, 51, 61, 74, 65, 193, 156, 238, 76, 55, 54, 138, 152,
			50, 198, 80, 47, 108, 253, 24, 46, 42, 239, 137,
		];
		let public_key = sr25519::Pair::from_seed(&bob_seed).public();
		let bob_account_id: u64 = 7166219960988249998;

		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (did_vc_id, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex));

		assert_ok!(Did::create_public(
			Origin::signed(VALIDATOR_ACCOUNT),
			did_vc_id,
			None
		));

		let account_id = Did::get_accountid_from_pubkey(&public_key);
		assert_eq!(account_id, bob_account_id);
	})
}
//END GET_ACCOUNT_ID_FROM_PUBLIC_KEY_OF_PUBLIC_DID TESTING

//START VERIFY_PRIVATE_DID_VC TESTING
#[test]
fn test_verify_private_did_vc() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);

		let did_vc_bytes = get_private_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PrivateDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex.clone()));
		assert!(Did::validate_vc(
			VcPallet::decode_vc::<VC<H256>>(&did_vc_hex).unwrap(),
			VCType::PrivateDidVC
		))
	})
}
//END VERIFY_PRIVATE_DID_VC TESTING

//START VERIFY_PUBLIC_DID_VC TESTING
#[test]
fn test_verify_public_did_vc() {
	new_test_ext().execute_with(|| {
		let identifier = *b"did:ssid:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		let public_key = sr25519::Public([5; 32]);
		let did_vc_bytes = get_public_did_vc(identifier, public_key);
		let (_, did_vc_hex) = get_vc_id_and_hex(identifier, did_vc_bytes, VCType::PublicDidVC);

		assert_ok!(VcPallet::store(Origin::signed(VALIDATOR_ACCOUNT), did_vc_hex.clone()));

		assert!(Did::validate_vc(
			VcPallet::decode_vc::<VC<H256>>(&did_vc_hex).unwrap(),
			VCType::PublicDidVC,
		))
	})
}
//END VERIFY_PUBLIC_DID_VC TESTING

#[test]
fn test_validate_did() {
	new_test_ext().execute_with(|| {
		// Normal case
		let identifier = *b"did:ssid:alicx\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == true);
		// Digits are allowed
		let identifier = *b"did:ssid:alic1\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == true);
		// Should start with did:
		let identifier = *b"std:ssid:alic:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		// Only ssid org is allowed for test
		let identifier = *b"did:nbgs:alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		// Only lower case characters are allowed
		let identifier = *b"did:ssid:Alice\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		// Did Should not contain empty spaces
		let identifier = *b"did:ssid:alic \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		let identifier = *b"did:ssid:ali\0c\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		let identifier = *b"did:ssid\0:ali\0c\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		// Only these Special characters are allowed ! @ # $ % ^ & * ( ) _ - ? { } . [ ] + = : \0
		// Only 2 : is allowed
		let identifier = *b"did:ssid:alice_\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == true);
		let identifier = *b"did:ssid:ali-ce\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == true);
		let identifier = *b"did:ssid:f.isw\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == true);
		let identifier = *b"did:ssid:alic;\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		let identifier = *b"did:ssid:alic/\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		let identifier = *b"did:ssid:alic;\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
		let identifier = *b"did:ssid:alic:\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert!(Did::validate_did(identifier) == false);
	})
}



#[test]
fn test_add_region() {
	new_test_ext().execute_with(|| {
		let region = *b"sgd\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert_ok!(Did::add_region(Origin::root(), region));
		let updated_region = AllowedRegions::<Test>::get();
		assert_eq!(updated_region.contains(&region), true);

		// Existing region cannot be added
		assert_noop!(Did::add_region(Origin::root(), region), Error::<Test>::RegionAlreadyExists);
	})
}

#[test]
fn test_remove_region() {
	new_test_ext().execute_with(|| {
		let region = *b"sgd\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
		assert_ok!(Did::add_region(Origin::root(), region));
		let updated_region = AllowedRegions::<Test>::get();
		assert_eq!(updated_region.contains(&region), true);

		assert_ok!(Did::remove_region(Origin::root(), region));
		let updated_region = AllowedRegions::<Test>::get();
		assert_eq!(updated_region.contains(&region), false);

		assert_noop!(Did::remove_region(Origin::root(), region), Error::<Test>::RegionNotFound);
	})
}

