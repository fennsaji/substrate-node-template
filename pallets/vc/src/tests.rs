use super::*;
use crate::mock::{VC,  *};
use sp_core::{sr25519, Pair, H256};
use metamui_primitives::types::{ TokenVC, VC as VCStruct};
use frame_support::{
	assert_noop, assert_ok, bounded_vec,
};

#[test]
fn test_genesis_works_correctly() {
	new_test_ext().execute_with(|| {
		let vc: VCStruct<H256> = VCs::<Test>::get(VC_ID_ONE).unwrap();
		let vec_of_vcids = Lookup::<Test>::get(OWNER_DID_ONE);
		let owner_did_one = RLookup::<Test>::get(VC_ID_ONE);
		let vc_id = vec_of_vcids[vec_of_vcids.len() - 1];
		let vc_property = VC::decode_vc::<TokenVC>(&vc.vc_property).unwrap();

		assert_eq!(vc.owner, OWNER_DID_ONE);
		assert_eq!(vc_id, VC_ID_ONE);
		assert_eq!(owner_did_one, OWNER_DID_ONE);
		assert_eq!(vc.issuers, vec![OWNER_DID_ONE]);
		assert_eq!(vc.is_vc_used, false);
		assert_eq!(vc.is_vc_active, false);
		assert_eq!(vc.vc_type, VCType::TokenVC);
		assert_eq!(vc.signatures, vec![SIGNATURE_ONE]);
		
		assert_eq!(vc_property.currency_code, *b"SYK\0\0\0\0\0");
		assert_eq!(vc_property.decimal, 6);
		assert_eq!(vc_property.reservable_balance, 1000000000);
		assert_eq!(vc_property.token_name, *b"Yidindj Token\0\0\0");
		
		let vc: VCStruct<H256> = VCs::<Test>::get(VC_ID_TWO).unwrap();
		let vec_of_vcids = Lookup::<Test>::get(OWNER_DID_TWO);
		let owner_did_two = RLookup::<Test>::get(VC_ID_TWO);
		let vc_id = vec_of_vcids[vec_of_vcids.len() - 1];
		let vc_property = VC::decode_vc::<SlashMintTokens>(&vc.vc_property).unwrap();
		
		assert_eq!(vc.owner, OWNER_DID_TWO);
		assert_eq!(vc_id, VC_ID_TWO);
		assert_eq!(owner_did_two, OWNER_DID_TWO);
		assert_eq!(vc.issuers, vec![OWNER_DID_TWO]);
		assert_eq!(vc.is_vc_used, false);
		assert_eq!(vc.is_vc_active, false);
		assert_eq!(vc.vc_type, VCType::TokenTransferVC);
		assert_eq!(vc.signatures, vec![SIGNATURE_TWO]);

		assert_eq!(vc_property.currency_code, *b"SGD\0\0\0\0\0");
		assert_eq!(vc_property.amount, 2000);
		assert_eq!(vc_property.vc_id, [150, 183, 92, 102, 90, 52, 61, 127, 116, 236, 71, 33, 122, 121, 67, 98, 251, 95, 189, 26, 147, 119, 146, 161, 136, 47, 97, 227, 122, 191, 92, 206])

	})
}

#[test]
fn test_store() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));
		let vc_id = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();
		let did = RLookup::<Test>::get(vc_id);
		assert_eq!(did, BOB);
		assert_eq!(Lookup::<Test>::get(did), vec![vc_id]);
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
		assert_eq!(VCHistory::<Test>::get(vc_id), Some((vc.is_vc_active, 0)));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
	})
}

#[test]
fn test_store_tokenchainauth_vc() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let tokenauth_vc = TokenchainAuthVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
			initial_issuance: 500,
		};

		let tokenauth_vc: [u8; 128] = convert_to_array::<128>(tokenauth_vc.encode());
		let vc_type = VCType::TokenchainAuthVC;
		let owner = BOB;
		let issuers: BoundedVec<Identifier, MaxIssuers> = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &tokenauth_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: tokenauth_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));
		let vc_id = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();
		let did = RLookup::<Test>::get(vc_id);
		assert_eq!(did, BOB);
		assert_eq!(Lookup::<Test>::get(did), vec![vc_id]);
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
		assert_eq!(VCHistory::<Test>::get(vc_id), Some((vc.is_vc_active, 0)));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
	})
}

#[test]
fn test_store_tokenchainauth_vc_fails_badorigin() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let tokenauth_vc = TokenchainAuthVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
    		initial_issuance: 500,
		};

		let tokenauth_vc: [u8; 128] = convert_to_array::<128>(tokenauth_vc.encode());
		let vc_type = VCType::TokenchainAuthVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &tokenauth_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: tokenauth_vc,
		};

		assert_noop!(VC::store(Origin::signed(DAVE_ACCOUNT_ID), vc.encode()), frame_support::error::BadOrigin);
	})
}

#[test]
fn test_invalid_owner_vc() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let currency_code = convert_to_array::<8>("OTH".into());
		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code,
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();
		let did = RLookup::<Test>::get(vc_id);
		assert_eq!(did, BOB);
		assert_eq!(Lookup::<Test>::get(did), vec![vc_id]);
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
		assert_eq!(VCHistory::<Test>::get(vc_id), Some((vc.is_vc_active, 0)));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);

		// Test MintVC
		let vc_type = VCType::MintTokens;
		let owner = ALICE;
		let issuers = bounded_vec![BOB];
		let mint_vc = SlashMintTokens { vc_id, currency_code, amount: 1000 };
		let mint_vc: [u8; 128] = convert_to_array::<128>(mint_vc.encode());
		let hash = BlakeTwo256::hash_of(&(&vc_type, &mint_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());
		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: mint_vc,
		};
		// Since the owner Did (Dave) is not registered, this should fail
		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::DidDoesNotExist
	);
	})
}

#[test]
fn test_mint_vc_store() {
	new_test_ext().execute_with(|| {
		let currency_code = convert_to_array::<8>("OTH".into());
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code,
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id_1 = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();
		let did = RLookup::<Test>::get(vc_id_1);
		assert_eq!(did, BOB);
		assert_eq!(Lookup::<Test>::get(did), vec![vc_id_1]);
		assert_eq!(VCs::<Test>::get(vc_id_1), Some(vc.clone()));
		assert_eq!(VCHistory::<Test>::get(vc_id_1), Some((vc.is_vc_active, 0)));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id_1]);

		let vc_type = VCType::MintTokens;
		let owner = DAVE;
		let issuers = bounded_vec![BOB];
		let mint_vc = SlashMintTokens { vc_id: vc_id_1, currency_code, amount: 1000 };
		let mint_vc: [u8; 128] = convert_to_array::<128>(mint_vc.encode());
		let hash = BlakeTwo256::hash_of(&(&vc_type, &mint_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());
		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: false,
			is_vc_active: true,
			vc_type,
			vc_property: mint_vc,
		};
		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id_2 = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();
		let did = RLookup::<Test>::get(vc_id_2);
		assert_eq!(did, DAVE);
		assert_eq!(Lookup::<Test>::get(did), vec![vc_id_2]);
		assert_eq!(VCs::<Test>::get(vc_id_2), Some(vc.clone()));
		assert_eq!(VCHistory::<Test>::get(vc_id_2), Some((vc.is_vc_active, 0)));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id_1, vc_id_2]);
	})
}

#[test]
fn test_cccode_validation() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTHs".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers: BoundedVec<Identifier, MaxIssuers> = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers: issuers.clone(),
			signatures: bounded_vec![signature.clone()],
			is_vc_used: true,
			is_vc_active: true,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::InvalidCurrencyCode,
	);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>(" OT H".into()),
		};
		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());

		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));

		let vc: VCStruct<H256> = VCStruct {
			hash,
			signatures: bounded_vec![signature.clone()],
			vc_type: vc_type.clone(),
			owner,
			issuers: issuers.clone(),
			is_vc_used: true,
			vc_property: token_vc,
			is_vc_active: true,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::InvalidCurrencyCode,
	);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("1OTH".into()),
		};
		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::InvalidCurrencyCode,
	);
	})
}

#[test]
fn test_update_status() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id = Lookup::<Test>::get(&BOB)[0];
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
		// Updating status flag
		assert_ok!(VC::update_status(Origin::signed(BOB_ACCOUNT_ID), vc_id, false));
 
		assert_eq!((VCs::<Test>::get(vc_id)).unwrap().is_vc_active, false);
	})
}

#[test]
fn test_store_vc_with_different_account() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(NON_VALIDATOR_ACCOUNT), vc.encode()),
		DispatchError::BadOrigin
	);
	})
}

#[test]
fn test_store_vc_with_wrong_hash() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		// Wrong Hash
		let hash = H256::zero();
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner: BOB,
			issuers: bounded_vec![BOB],
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::VCPropertiesNotVerified
	);
	})
}

#[test]
fn test_store_vc_with_wrong_signature() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let wrong_hash = H256::zero();
		let signature = pair.sign(wrong_hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::InvalidSignature
	);
	})
}

#[test]
fn test_store_vc_less_approvers() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB, DAVE];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let wrong_hash = H256::zero();
		let signature = pair.sign(wrong_hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::InvalidSignature
	);
	})
}

#[test]
fn test_update_status_sender() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));
		
		let vc_id = Lookup::<Test>::get(&BOB)[0];
		let non_issuer = VALIDATOR_ACCOUNT;
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
		
		// Updating status flag with non issuer account
		assert_noop!(VC::update_status(Origin::signed(non_issuer), vc_id, vc.is_vc_active),
		Error::<Test>::NotAValidatorNorIssuer
	);

		// Updating status flag with non validator account
		assert_noop!(VC::update_status(Origin::signed(VALIDATOR_ACCOUNT), vc_id, vc.is_vc_active),
		Error::<Test>::NotAValidatorNorIssuer
	);
	})
}

#[test]
fn test_add_signature() {
	new_test_ext().execute_with(|| {
		let bob_pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let dave_pair: sr25519::Pair = sr25519::Pair::from_seed(&DAVE_SEED);
		let eve_pair: sr25519::Pair = sr25519::Pair::from_seed(&EVE_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB, DAVE, EVE];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let bob_sign = bob_pair.sign(hash.as_ref());
		let dave_sign = dave_pair.sign(hash.as_ref());
		let eve_sign = eve_pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![bob_sign.clone()],
			is_vc_used: true,
			is_vc_active: false,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));
		
		let vc_id = Lookup::<Test>::get(&BOB)[0];
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
		assert_eq!(VCIdLookup::<Test>::get(DAVE), vec![vc_id]);
		assert_eq!(VCIdLookup::<Test>::get(EVE), vec![vc_id]);

		// vc_status = Inactive as only one issuer signed
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
		assert_eq!(VCIdLookup::<Test>::get(DAVE), vec![vc_id]);
		assert_eq!(VCIdLookup::<Test>::get(EVE), vec![vc_id]);

		// updating DAVE's signature
		let vc: VCStruct<H256> = VCStruct {
			hash,
			signatures: bounded_vec![bob_sign.clone(), dave_sign.clone()],
			vc_type: vc_type.clone(),
			owner: BOB,
			issuers: bounded_vec![BOB, DAVE, EVE],
			is_vc_used: true,
			vc_property: token_vc,
			is_vc_active: false,
		};

		assert_ok!(VC::add_signature(Origin::signed(BOB_ACCOUNT_ID), vc_id, dave_sign.clone()));

		// vc_status = Inactive as only two issuer signed
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));

		// updating EVE's signature
		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner: BOB,
			issuers: bounded_vec![BOB, DAVE, EVE],
			signatures: bounded_vec![bob_sign, dave_sign, eve_sign.clone()],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::add_signature(Origin::signed(BOB_ACCOUNT_ID), vc_id, eve_sign));

		// vc_status = Active as only all issuer signed
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
	})
}

#[test]
fn test_add_signature_with_one_of_the_signers() {
	new_test_ext().execute_with(|| {
		let bob_pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let dave_pair: sr25519::Pair = sr25519::Pair::from_seed(&DAVE_SEED);
		let eve_pair: sr25519::Pair = sr25519::Pair::from_seed(&EVE_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB, DAVE, EVE];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let bob_sign = bob_pair.sign(hash.as_ref());
		// signed by Dave's public key
		let dave_sign = dave_pair.sign(hash.as_ref());
		// signed by Eve's public key
		let eve_sign = eve_pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![bob_sign.clone()],
			is_vc_used: true,
			is_vc_active: false,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id = Lookup::<Test>::get(&BOB)[0];
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
		assert_eq!(VCIdLookup::<Test>::get(DAVE), vec![vc_id]);
		assert_eq!(VCIdLookup::<Test>::get(EVE), vec![vc_id]);

		// vc_status = Inactive as only one issuer signed
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));

		// updating DAVE's signature
		let vc: VCStruct<H256> = VCStruct {
			hash,
			signatures: bounded_vec![bob_sign.clone(), dave_sign.clone()],
			vc_type: vc_type.clone(),
			owner: BOB,
			issuers: bounded_vec![BOB, DAVE, EVE],
			is_vc_used: true,
			vc_property: token_vc,
			is_vc_active: false,
		};

		assert_ok!(VC::add_signature(Origin::signed(DAVE_ACCOUNT_ID), vc_id, dave_sign.clone()));

		// vc_status = Inactive as only two issuer signed
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));

		// updating EVE's signature
		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner: BOB,
			issuers: bounded_vec![BOB, DAVE, EVE],
			signatures: bounded_vec![bob_sign, dave_sign, eve_sign.clone()],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::add_signature(Origin::signed(DAVE_ACCOUNT_ID), vc_id, eve_sign));

		// vc_status = Active as only all issuer signed
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
	})
}

#[test]
fn test_set_is_used_flag() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: false,
			is_vc_active: false,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id = Lookup::<Test>::get(&BOB)[0];

		// set vc is_used flag as true
		let _ = VC::update_vc_used(vc_id, Some(true));

		let vc_details = VCs::<Test>::get(vc_id).unwrap();
		assert!(vc_details.is_vc_used);
	})
}

#[test]
fn test_duplicate_issuers_signatures() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		// case when duplicate signatures are present
		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature.clone(), signature.clone()],
			is_vc_used: true,
			is_vc_active: false,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::DuplicateSignature);

		// case when duplicate issuers are present
		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB, BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: false,
			vc_type,
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::DuplicateIssuers,
	);
	})
}

#[test]
fn test_add_duplicate_issuer_signatures() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let dave_pair: sr25519::Pair = sr25519::Pair::from_seed(&DAVE_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		// case when duplicate signatures are present
		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers: BoundedVec<Identifier, MaxIssuers> = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());
		let duplicate_signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers: issuers.clone(),
			signatures: bounded_vec![signature.clone(), duplicate_signature.clone()],
			is_vc_used: true,
			is_vc_active: false,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		assert_noop!(
			VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
			Error::<Test>::DuplicateSignature,
		);

		let dave_sign = dave_pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers: issuers.clone(),
			signatures: bounded_vec![signature.clone(), dave_sign],
			is_vc_used: true,
			is_vc_active: false,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()),
		Error::<Test>::InvalidSignature
	);

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: false,
			vc_type,
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id = Lookup::<Test>::get(&BOB)[0];

		assert_noop!(VC::add_signature(Origin::signed(DAVE_ACCOUNT_ID), vc_id, duplicate_signature),
		Error::<Test>::DuplicateSignature,
	);
	})
}

#[test]
fn test_generic_vc_store() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let generic_vc = GenericVC { cid: convert_to_array::<64>("F0TAeD_UY2mK-agbzZTW".into()) };

		let generic_vc: [u8; 128] = convert_to_array::<128>(generic_vc.encode());

		let vc_type = VCType::GenericVC;
		let owner = BOB;
		let issuers = bounded_vec![BOB];
		// Hash for generic vc will be generated using
		// the data stored in vc_url of generic_vc
		let hash = BlakeTwo256::hash_of(&generic_vc);
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![signature],
			is_vc_used: true,
			is_vc_active: true,
			vc_type,
			vc_property: generic_vc,
		};

		assert_noop!(
			VC::store(Origin::signed(VALIDATOR_ACCOUNT), vc.encode()),
			Error::<Test>::NotACouncilMember,
		);

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		let vc_id = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();
		let did = RLookup::<Test>::get(vc_id);
		assert_eq!(did, BOB);
		assert_eq!(Lookup::<Test>::get(did), vec![vc_id]);
		assert_eq!(VCs::<Test>::get(vc_id), Some(vc.clone()));
		assert_eq!(VCHistory::<Test>::get(vc_id), Some((vc.is_vc_active, 0)));
		assert_eq!(VCIdLookup::<Test>::get(BOB), vec![vc_id]);
	})
}

#[test]
fn test_vc_already_exists() {
	new_test_ext().execute_with(|| {
		let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers: BoundedVec<Identifier, MaxIssuers> = bounded_vec![BOB];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let signature = pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers: issuers.clone(),
			signatures: bounded_vec![signature.clone()],
			is_vc_used: true,
			is_vc_active: false,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		assert_ok!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()));

		assert_noop!(VC::store(Origin::signed(BOB_ACCOUNT_ID), vc.encode()), Error::<Test>::VCAlreadyExists);
	})
}

#[test]
fn test_invalid_signature_for_add_signature() {
	new_test_ext().execute_with(|| {
		let bob_pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
		let dave_pair: sr25519::Pair = sr25519::Pair::from_seed(&DAVE_SEED);

		let token_vc = TokenVC {
			token_name: convert_to_array::<16>("test".into()),
			reservable_balance: 1000,
			decimal: 6,
			currency_code: convert_to_array::<8>("OTH".into()),
		};

		let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
		let vc_type = VCType::TokenVC;
		let owner = BOB;
		let issuers = bounded_vec![DAVE];
		let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
		let bob_sign = bob_pair.sign(hash.as_ref());
		let dave_sign = dave_pair.sign(hash.as_ref());

		let vc: VCStruct<H256> = VCStruct {
			hash,
			owner,
			issuers,
			signatures: bounded_vec![bob_sign.clone()],
			is_vc_used: true,
			is_vc_active: false,
			vc_type: vc_type.clone(),
			vc_property: token_vc,
		};

		let vc_id = *BlakeTwo256::hash_of(&vc).as_fixed_bytes();

		assert_ok!(VC::validate_signature(&vc, dave_sign.clone(), vc_id));
		//Error will occur If signed by someone who is not issuer, Signature will be invalid!
		assert_noop!(VC::validate_signature(&vc, bob_sign.clone(), vc_id), Error::<Test>::InvalidSignature);
	})
}
