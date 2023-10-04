use super::*;
use crate::{
	mock::{Tokens, VC, Did, *},
};
use crate::types::{CurrencyCode};
use frame_support::{assert_noop, assert_ok, bounded_vec};
use metamui_primitives::types::{MaxIssuers, TokenVC, VC as VCStruct, VCType};
use sp_core::{sr25519, Pair, H256};
use sp_runtime::traits::{BlakeTwo256, Hash};
use codec::Encode;

#[test]
fn only_vc_owner_can_issue_token() {
    new_test_ext().execute_with(|| {
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code: convert_to_array::<8>("OTH".into()),
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();
        // issue token failed due to non-registered account
        assert_noop!(
            Tokens::issue_token(Origin::signed(ALICE_ACCOUNT_ID), vc_id, token_amount),
            Error::<Test>::DidNotRegisteredWithVC
        );
    });
}

#[test]
fn issue_token_works() {
    new_test_ext().execute_with(|| {
        let reservable_balance: u128 = 1000000;
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: reservable_balance,
            decimal: 6,
            currency_code,
        };
        let token_name = token_vc.token_name;
        let decimal = token_vc.decimal;
        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        // check balance has been reserved correctly'
        // Moved to Relay chain
        // assert_eq!(
        //     Balances::free_balance(BOB_ACCOUNT_ID),
        //     INITIAL_BALANCE - reservable_balance as u64
        // );
        // assert_eq!(
        //     Balances::reserved_balance(BOB_ACCOUNT_ID),
        //     reservable_balance as u64
        // );

        // check created token details
        assert_eq!(Tokens::total_issuance(currency_code), token_amount);
        assert_eq!(adjust_null_padding(&mut Tokens::token_data(currency_code).unwrap().token_name.to_vec(), 16), token_name);
        assert_eq!(adjust_null_padding(&mut Tokens::token_data(currency_code).unwrap().currency_code.to_vec(), 8), currency_code);
        assert_eq!(Tokens::token_data(currency_code).unwrap().decimal, decimal);
        assert_eq!(Tokens::token_data(currency_code).unwrap().block_number, 0);

        // check entire token supply is credited to the creator account
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount
        );

        // check if the token owner/issuer is correct
        assert_eq!(
            Tokens::token_issuer(currency_code).unwrap(),
            Did::get_did(&BOB_ACCOUNT_ID).unwrap()
        );

        // checking slash token vc works after being used
        assert_noop!(
            Tokens::issue_token(Origin::signed(BOB_ACCOUNT_ID), vc_id, token_amount),
            Error::<Test>::VCAlreadyUsed
        );
    });
}

#[test]
fn test_transfer_token_works() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());

        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let transfer_amount: u128 = 1_000_000;
        assert_ok!(Tokens::transfer(
            Origin::signed(BOB_ACCOUNT_ID),
            DAVE,
            currency_code,
            transfer_amount,
        ));

        // check balance transfer worked correctly
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount - transfer_amount
        );
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &DAVE_ACCOUNT_ID),
            transfer_amount
        );
        assert_eq!(Tokens::total_issuance(currency_code), token_amount);

        // cannot transfer more than balance
        assert_noop!(
            Tokens::transfer(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                TREASURY_RESERVE_AMOUNT.into()
            ),
            Error::<Test>::BalanceTooLow
        );
    });
}

#[test]
fn test_get_currency_id() {
    new_test_ext().execute_with(|| {
        // derive currency id first time
        assert_eq!(Tokens::generate_ccy_id().ok().unwrap(), 1);
        // Set currency id
        Tokens::set_currency_id(1);
        // derive currency id second time
        assert_eq!(Tokens::generate_ccy_id().ok().unwrap(), 2);
    });
}

#[test]
fn test_slash_token() {
    new_test_ext().execute_with(|| {
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers: BoundedVec<Identifier, MaxIssuers> = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner: BOB,
            issuers: bounded_vec![BOB],
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let slash_amount: u128 = 1_000_000;
        let slash_vc = SlashMintTokens {
            vc_id,
            currency_code,
            amount: slash_amount,
        };

        let slash_vc: [u8; 128] = convert_to_array::<128>(slash_vc.encode());
        let vc_type = VCType::SlashTokens;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = DAVE;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &slash_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: slash_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(DAVE_ACCOUNT_ID),
            vc_struct.encode()
        ));
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::slash_token(Origin::signed(DAVE_ACCOUNT_ID), vc_id));

        // checking correctness of free balance after slash
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount - slash_amount
        );

        // checking slash token vc works after being used
        assert_noop!(
            Tokens::slash_token(Origin::signed(DAVE_ACCOUNT_ID), vc_id),
            Error::<Test>::VCAlreadyUsed
        );
    });
}

#[test]
fn test_mint_token() {
    new_test_ext().execute_with(|| {
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let mint_amount: u128 = 1_000_000;
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let mint_vc = SlashMintTokens {
            vc_id,
            currency_code,
            amount: mint_amount,
        };

        let mint_vc: [u8; 128] = convert_to_array::<128>(mint_vc.encode());
        let vc_type = VCType::MintTokens;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = DAVE;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &mint_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: mint_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(DAVE_ACCOUNT_ID),
            vc_struct.encode()
        ));
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::mint_token(Origin::signed(DAVE_ACCOUNT_ID), vc_id));

        // checking correctness of free balance after mint
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount + mint_amount
        );
        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount + mint_amount
        );

        // checking mint token vc works after being used
        assert_noop!(
            Tokens::mint_token(Origin::signed(DAVE_ACCOUNT_ID), vc_id),
            Error::<Test>::VCAlreadyUsed
        );
    });
}

#[test]
fn test_transfer_token() {
    new_test_ext().execute_with(|| {
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let transfer_amount: u128 = 1_000_000;
        let transfer_vc = TokenTransferVC {
            vc_id,
            currency_code,
            amount: transfer_amount,
        };

        let transfer_vc: [u8; 128] = convert_to_array::<128>(transfer_vc.encode());
        let vc_type = VCType::TokenTransferVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = DAVE;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &transfer_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: transfer_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(DAVE_ACCOUNT_ID),
            vc_struct.encode()
        ));
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::transfer_token(
            Origin::signed(DAVE_ACCOUNT_ID),
            vc_id,
            ALICE
        ));

        // checking amount transfered
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &ALICE_ACCOUNT_ID),
            transfer_amount
        );

        // checking correctness of free balance after transfer
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount - transfer_amount
        );

        assert_eq!(Tokens::total_issuance(currency_code), token_amount);

        // checking transfer token vc works after being used
        assert_noop!(
            Tokens::transfer_token(Origin::signed(DAVE_ACCOUNT_ID), vc_id, ALICE),
            Error::<Test>::VCAlreadyUsed
        );
    });
}

#[test]
fn test_decimal_and_ccy_code() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let account_details = Accounts::<Test>::get(currency_code, BOB);
        assert_eq!(account_details.data.free, token_amount);
        assert_eq!(account_details.data.reserved, 0);
        assert_eq!(account_details.data.frozen, 0);
    });
}

#[test]
fn test_ccy_code_exists() {
    new_test_ext().execute_with(|| {
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code: convert_to_array::<8>("OTH".into()),
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        // First time tokens will be issued
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test-2".into()),
            reservable_balance: 2_000_000,
            decimal: 6,
            currency_code: convert_to_array::<8>("OTH".into()),
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = DAVE;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();
        // Second time tokens will not be issued as currency_code already registered
        assert_noop!(
            Tokens::issue_token(Origin::signed(DAVE_ACCOUNT_ID), vc_id, token_amount),
            Error::<Test>::CurrencyCodeExists,
        );
    });
}

#[test]
fn test_set_balance() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let new_amount: u128 = 1_000_000;

        assert_ok!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            )
        );

        // checking amount transfered
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &DAVE_ACCOUNT_ID),
            new_amount
        );

        // checking correctness of free balance after transfer
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount - new_amount
        );

        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}


#[test]
fn test_set_whole_balance() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));


        let new_amount = token_amount;
        assert_ok!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            )
        );

        // checking amount transfered
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &DAVE_ACCOUNT_ID),
            new_amount
        );

        // checking correctness of free balance after transfer
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount - new_amount
        );

        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}

#[test]
fn test_set_balance_greater_amount() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let new_amount = 6_000_000;
        assert_noop!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            ),
            Error::<Test>::TokenAmountOverflow
        );

        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}


#[test]
fn test_set_balance_less_than_existing() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let new_amount = 4_000_000;
        assert_ok!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            ),
        );

        let new_amount = 1_000_000;
        assert_ok!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            ),
        );

        // checking amount transfered
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &DAVE_ACCOUNT_ID),
            new_amount
        );

        // checking correctness of free balance after transfer
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount - new_amount
        );

        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}

#[test]
fn test_set_balance_zero() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        // First time tokens will be issued
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let new_amount = 4_000_000;
        assert_ok!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            ),
        );

        let new_amount = 0;
        assert_ok!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            ),
        );

        // checking amount transfered
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &DAVE_ACCOUNT_ID),
            new_amount
        );

        // checking correctness of free balance after transfer
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount
        );


        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}

#[test]
fn test_set_balance_token_owner() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let new_amount = 1_000_000;
        assert_noop!(
            Tokens::set_balance(
                Origin::signed(BOB_ACCOUNT_ID),
                BOB,
                currency_code,
                new_amount,
            ),
            Error::<Test>::NotAllowed
        );

        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}

#[test]
fn test_set_balance_not_token_owner() {
    new_test_ext().execute_with(|| {
        let currency_code = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1_000_000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&ALICE_SEED);
        let owner = BOB;
        let issuers = bounded_vec![ALICE];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let token_amount: u128 = 5_000_000;
        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        let new_amount = 1_000_000;
        assert_noop!(
            Tokens::set_balance(
                Origin::signed(DAVE_ACCOUNT_ID),
                DAVE,
                currency_code,
                new_amount,
            ),
            Error::<Test>::NotAllowed
        );

        assert_eq!(
            Tokens::total_issuance(currency_code),
            token_amount
        );
    });
}

#[test]
fn remove_token_works() {
    new_test_ext().execute_with(|| {
        let reservable_balance: u128 = 1000000;
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: reservable_balance,
            decimal: 6,
            currency_code,
        };

        let token_name = token_vc.token_name;
        let decimal = token_vc.decimal;
        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));
        // check balance has been reserved correctly
        // assert_eq!(
        //     Balances::free_balance(BOB_ACCOUNT_ID),
        //     INITIAL_BALANCE - reservable_balance as u64
        // );
        // assert_eq!(
        //     Balances::reserved_balance(BOB_ACCOUNT_ID),
        //     reservable_balance as u64
        // );

        // check created token details
        assert_eq!(Tokens::total_issuance(currency_code), token_amount);
        assert_eq!(adjust_null_padding(&mut Tokens::token_data(currency_code).unwrap().token_name.to_vec(), 16), token_name);
        assert_eq!(adjust_null_padding(&mut Tokens::token_data(currency_code).unwrap().currency_code.to_vec(), 8), currency_code);
        assert_eq!(Tokens::token_data(currency_code).unwrap().decimal, decimal);
        assert_eq!(Tokens::token_data(currency_code).unwrap().block_number, 0);

        // check entire token supply is credited to the creator account
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            token_amount
        );

        // issue token
        assert_ok!(Tokens::remove_token(
            Origin::signed(BOB_ACCOUNT_ID),
            currency_code,
            vc_id,
            None
        ));
        let block_num_on_deletion = RemovedTokens::<Test>::get(currency_code).unwrap();
        let block_number = <frame_system::Pallet<Test>>::block_number();
        assert_eq!{
            block_num_on_deletion,
            block_number
            //1
        }


        // check balance has been unreserved correctly
        // assert_eq!(
        //     Balances::free_balance(BOB_ACCOUNT_ID),
        //     INITIAL_BALANCE as u64
        // );

        // assert_eq!(
        //     Balances::reserved_balance(BOB_ACCOUNT_ID),
        //     0 as u64
        // );

        // check created token details
        assert_eq!(Tokens::total_issuance(currency_code), 0);

        assert_eq!(
            Tokens::token_data(currency_code),
            None
        );

        // check entire token supply is credited to the creator account
        assert_eq!(
            Tokens::free_balance(TEST_TOKEN_ID, &BOB_ACCOUNT_ID),
            0
        );
    });
}


#[test]
fn test_vc_already_used() {
    new_test_ext().execute_with(|| {
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));

        assert_noop!(
            Tokens::issue_token( Origin::signed(BOB_ACCOUNT_ID), vc_id, token_amount),
            Error::<Test>::VCAlreadyUsed
        );
    })
}

#[test]
fn test_recipent_did_not_registered() {
    new_test_ext().execute_with(|| {
        let currency_code: CurrencyCode = convert_to_array::<8>("OTH".into());
        let token_vc = TokenVC {
            token_name: convert_to_array::<16>("test".into()),
            reservable_balance: 1000,
            decimal: 6,
            currency_code,
        };

        let token_vc: [u8; 128] = convert_to_array::<128>(token_vc.encode());
        let vc_type = VCType::TokenVC;
        let pair: sr25519::Pair = sr25519::Pair::from_seed(&BOB_SEED);
        let owner = BOB;
        let issuers = bounded_vec![BOB];
        let hash = BlakeTwo256::hash_of(&(&vc_type, &token_vc, &owner, &issuers));
        let signature = pair.sign(hash.as_ref());

        let vc_struct: VCStruct<H256> = VCStruct {
            hash,
            signatures: bounded_vec![signature],
            vc_type,
            owner,
            issuers,
            is_vc_used: false,
            vc_property: token_vc,
            is_vc_active: false,
        };

        assert_ok!(VC::store(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_struct.encode()
        ));

        let vc_id = *BlakeTwo256::hash_of(&vc_struct).as_fixed_bytes();

        let token_amount: u128 = 5_000_000;
        // issue token
        assert_ok!(Tokens::issue_token(
            Origin::signed(BOB_ACCOUNT_ID),
            vc_id,
            token_amount
        ));
        let unregistered_did = EVE;
        let amount_to_transfer = 0;
        assert_noop!(Tokens::transfer(Origin::signed(BOB_ACCOUNT_ID), unregistered_did, currency_code, amount_to_transfer), Error::<Test>::DIDDoesNotExist);
    })
}

// TODO: Add Unit test for throwing error if currency code does not exists
