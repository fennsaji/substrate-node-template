#[cfg(test)]
mod tests {
	use super::*;
	use crate as pallet_membership;

	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BadOrigin, BlakeTwo256, IdentityLookup},
	};

	use frame_support::{
		assert_noop, assert_ok, bounded_vec, ord_parameter_types, parameter_types,
		traits::{ConstU32, ConstU64, BuildGenesisConfig},
	};
	use frame_system::EnsureSignedBy;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

  pub struct DidResolution;
  impl DidResolve<AccountId> for DidResolution {
    /// return if an accountId is mapped to a DID
    fn did_exists(x: &AccountId) -> bool {
      true
    }
    /// convert accountId to DID
    fn get_did(k: &AccountId) -> Option<Did> {
      Some(*b"did:ssid:swn/0/0/0/0/0/0/0/0/0/0")
    }
  }

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Membership: pallet_membership::{Pallet, Call, Storage, Config<T>, Event<T>},
		}
	);

	parameter_types! {
		pub BlockWeights: frame_system::limits::BlockWeights =
			frame_system::limits::BlockWeights::simple_max(1024);
		pub static Members: Vec<Did> = vec![];
		pub static Prime: Option<Did> = None;
	}

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Call = Call;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = ConstU64<250>;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = ConstU32<16>;
	}
	ord_parameter_types! {
		pub const One: u64 = 1;
		pub const Two: u64 = 2;
		pub const Three: u64 = 3;
		pub const Four: u64 = 4;
		pub const Five: u64 = 5;
	}

	pub struct TestChangeMembers;
	impl ChangeMembers<Did> for TestChangeMembers {
		fn change_members_sorted(incoming: &[Did], outgoing: &[Did], new: &[Did]) {
			let mut old_plus_incoming = Members::get();
			old_plus_incoming.extend_from_slice(incoming);
			old_plus_incoming.sort();
			let mut new_plus_outgoing = new.to_vec();
			new_plus_outgoing.extend_from_slice(outgoing);
			new_plus_outgoing.sort();
			assert_eq!(old_plus_incoming, new_plus_outgoing);

			Members::set(new.to_vec());
			Prime::set(None);
		}
		fn set_prime(who: Option<Did>) {
			Prime::set(who);
		}
		fn get_prime() -> Option<Did> {
			Prime::get()
		}
	}

	impl InitializeMembers<Did> for TestChangeMembers {
		fn initialize_members(members: &[Did]) {
			MEMBERS.with(|m| *m.borrow_mut() = members.to_vec());
		}
	}

	impl Config for Test {
		type Event = Event;
		type AddOrigin = EnsureSignedBy<One, u64>;
		type RemoveOrigin = EnsureSignedBy<Two, u64>;
		type SwapOrigin = EnsureSignedBy<Three, u64>;
		type ResetOrigin = EnsureSignedBy<Four, u64>;
		type PrimeOrigin = EnsureSignedBy<Five, u64>;
		type MembershipInitialized = TestChangeMembers;
		type MembershipChanged = TestChangeMembers;
		type MaxMembers = ConstU32<10>;
		type DidResolution = DidResolution;
		type WeightInfo = ();
	}

	pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		pallet_membership::GenesisConfig::<Test> {
			members: bounded_vec![[10; 32], [20; 32], [30; 32]],
			..Default::default()
		}
		.assimilate_storage(&mut t)
		.unwrap();
		t.into()
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn new_bench_ext() -> sp_io::TestExternalities {
		frame_system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn clean() {
		Members::set(vec![]);
		Prime::set(None);
	}

	#[test]
	fn query_membership_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(Membership::members(), vec![[10; 32], [20; 32], [30; 32]]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), vec![[10; 32], [20; 32], [30; 32]]);
		});
	}

	#[test]
	fn prime_member_works() {
		new_test_ext().execute_with(|| {
			assert_noop!(Membership::set_prime(Origin::signed(4), [20; 32]), BadOrigin);
			assert_noop!(Membership::set_prime(Origin::signed(5), [15; 32]), Error::<Test, _>::NotMember);
			assert_ok!(Membership::set_prime(Origin::signed(5), [20; 32]));
			assert_eq!(Membership::prime(), Some([20; 32]));
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());

			assert_ok!(Membership::clear_prime(Origin::signed(5)));
			assert_eq!(Membership::prime(), None);
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());
		});
	}

	#[test]
	fn add_member_works() {
		new_test_ext().execute_with(|| {
			assert_noop!(Membership::add_member(Origin::signed(5), [15; 32]), BadOrigin);
			assert_noop!(
				Membership::add_member(Origin::signed(1), [10; 32]),
				Error::<Test, _>::AlreadyMember
			);
			assert_ok!(Membership::add_member(Origin::signed(1), [15; 32]));
			assert_eq!(Membership::members(), vec![[10; 32], [15; 32], [20; 32], [30; 32]]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
		});
	}

	#[test]
	fn remove_member_works() {
		new_test_ext().execute_with(|| {
			assert_noop!(Membership::remove_member(Origin::signed(5), 20), BadOrigin);
			assert_noop!(
				Membership::remove_member(Origin::signed(2), 15),
				Error::<Test, _>::NotMember
			);
			assert_ok!(Membership::set_prime(Origin::signed(5), 20));
			assert_ok!(Membership::remove_member(Origin::signed(2), 20));
			assert_eq!(Membership::members(), vec![10, 30]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
			assert_eq!(Membership::prime(), None);
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());
		});
	}

	#[test]
	fn swap_member_works() {
		new_test_ext().execute_with(|| {
			assert_noop!(Membership::swap_member(Origin::signed(5), 10, 25), BadOrigin);
			assert_noop!(
				Membership::swap_member(Origin::signed(3), 15, 25),
				Error::<Test, _>::NotMember
			);
			assert_noop!(
				Membership::swap_member(Origin::signed(3), 10, 30),
				Error::<Test, _>::AlreadyMember
			);

			assert_ok!(Membership::set_prime(Origin::signed(5), 20));
			assert_ok!(Membership::swap_member(Origin::signed(3), 20, 20));
			assert_eq!(Membership::members(), vec![10, 20, 30]);
			assert_eq!(Membership::prime(), Some(20));
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());

			assert_ok!(Membership::set_prime(Origin::signed(5), 10));
			assert_ok!(Membership::swap_member(Origin::signed(3), 10, 25));
			assert_eq!(Membership::members(), vec![20, 25, 30]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
			assert_eq!(Membership::prime(), None);
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());
		});
	}

	#[test]
	fn swap_member_works_that_does_not_change_order() {
		new_test_ext().execute_with(|| {
			assert_ok!(Membership::swap_member(Origin::signed(3), 10, 5));
			assert_eq!(Membership::members(), vec![5, 20, 30]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
		});
	}

	#[test]
	fn change_key_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(Membership::set_prime(Origin::signed(5), 10));
			assert_noop!(
				Membership::change_key(Origin::signed(3), 25),
				Error::<Test, _>::NotMember
			);
			assert_noop!(
				Membership::change_key(Origin::signed(10), 20),
				Error::<Test, _>::AlreadyMember
			);
			assert_ok!(Membership::change_key(Origin::signed(10), 40));
			assert_eq!(Membership::members(), vec![20, 30, 40]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
			assert_eq!(Membership::prime(), Some(40));
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());
		});
	}

	#[test]
	fn change_key_works_that_does_not_change_order() {
		new_test_ext().execute_with(|| {
			assert_ok!(Membership::change_key(Origin::signed(10), 5));
			assert_eq!(Membership::members(), vec![5, 20, 30]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
		});
	}

	#[test]
	fn reset_members_works() {
		new_test_ext().execute_with(|| {
			assert_ok!(Membership::set_prime(Origin::signed(5), 20));
			assert_noop!(
				Membership::reset_members(Origin::signed(1), bounded_vec![20, 40, 30]),
				BadOrigin
			);

			assert_ok!(Membership::reset_members(Origin::signed(4), vec![20, 40, 30]));
			assert_eq!(Membership::members(), vec![20, 30, 40]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
			assert_eq!(Membership::prime(), Some(20));
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());

			assert_ok!(Membership::reset_members(Origin::signed(4), vec![10, 40, 30]));
			assert_eq!(Membership::members(), vec![10, 30, 40]);
			assert_eq!(MEMBERS.with(|m| m.borrow().clone()), Membership::members().to_vec());
			assert_eq!(Membership::prime(), None);
			assert_eq!(PRIME.with(|m| *m.borrow()), Membership::prime());
		});
	}

	#[test]
	#[should_panic(expected = "Members cannot contain duplicate accounts.")]
	fn genesis_build_panics_with_duplicate_members() {
		pallet_membership::GenesisConfig::<Test> {
			members: bounded_vec![1, 2, 3, 1],
			phantom: Default::default(),
		}
		.build_storage()
		.unwrap();
	}
}
