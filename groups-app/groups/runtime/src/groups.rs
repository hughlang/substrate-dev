/// A runtime module template with necessary imports

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use parity_codec::{Encode, Decode};
use runtime_primitives::traits::{As, Hash, Zero};
use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageMap, StorageValue};
use system::ensure_signed;
// use timestamp::{OnTimestampSet, Trait};
// use timestamp::TimestampInherentData;
// use inherents::{InherentDataProviders, ProvideInherentData};
// use inherents::{RuntimeString, InherentData};

#[cfg(not(feature = "std"))]
use rstd::prelude::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

const MAX_GROUP_SIZE: u32 = 8;

/// The module's configuration trait.
pub trait Trait: system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Group<AccountId, Hash> {
    id: Hash,
	owner: AccountId,
	name: Vec<u8>,
	members: Vec<AccountId>,
	/// Limit number of users who can join the group
	max_size: u32,
	timestamp: u64,
}

// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub struct Member<AccountId, Hash> {

// }

/*
	Storage TODO
	– Invite
	– Member
	– Vote
*/

decl_storage! {
	trait Store for Module<T: Trait> as GroupsModule {
		/// Groups is a mapping of group_id hash to the Group itself
		Groups get(group): map T::Hash => Group<T::AccountId, T::Hash>;
		GroupOwner get(owner_of): map T::Hash => Option<T::AccountId>;
		AllGroupsCount get(all_groups_count): u64;


		Nonce: u64;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
		// <T as timestamp::Trait>::Timestamp,
	{
		CreatedGroup(AccountId, Hash),
	}
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		pub fn get_time(origin) -> Result {
			let _sender = ensure_signed(origin)?;
			let _now = <timestamp::Module<T>>::get();
			Ok(())
		}

		fn deposit_event<T>() = default;

		/*
			Use cases TODO:
			– Create group
			– Update group
			– Remove group
			– Request invite
			– Create invite
			– Accept invite
			– Add member
			– Remove member
			– List members
			– Verify member (groupId, accountId)
			– Broadcast message?
			– Request vote
			– Submit vote
		*/

		/// Create a group owned by the current AccountId.
		/// Usage: For name, use String::into_bytes();
		fn create_group(origin, name: Vec<u8>, max_size: u32) -> Result {
			let sender = ensure_signed(origin)?;

            let nonce = <Nonce<T>>::get();
            let random_id = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

			let total_groups = Self::all_groups_count();
			let new_groups_count = total_groups.checked_add(1).ok_or("Overflow adding a new group")?;

			let group = Group {
				id: random_id,
				owner: sender.clone(),
				name: name,
				members: Vec::new(),
				max_size: max_size,
				timestamp: 0,
			};
			<Groups<T>>::insert(random_id, group);
			<GroupOwner<T>>::insert(random_id, &sender);
			<AllGroupsCount<T>>::put(new_groups_count);

			<Nonce<T>>::mutate(|n| *n += 1);
			// Self::deposit_event(RawEvent::CreatedGroup(sender, kitty_id, new_price));

			Ok(())
		}

		/// Renaming a group by converting the String name into a byte array
		/// Rule: only the owner is allowed to use this function.
		/// Usage: For name, use String::into_bytes();
		fn rename_group(origin, group_id: T::Hash, name: Vec<u8>) -> Result {
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
			let mut group = Self::group(group_id);

			let sender = ensure_signed(origin)?;
			ensure!(group.owner == sender, "You are not the owner of this group");
			group.name = name;
			<Groups<T>>::insert(group.id, group);

			Ok(())
		}

		/// Remove group
		/// Rule: only owner can remove a group
		fn remove_group(origin, group_id: T::Hash) -> Result {
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
			let group = Self::group(group_id);
			let sender = ensure_signed(origin)?;

			ensure!(group.owner == sender, "You are not the owner of this group");
			<Groups<T>>::remove(group_id);

			Ok(())
		}


	}
}

/// Private methods
impl<T: Trait> Module<T> {
	// pub fn slot_duration() -> T::Moment {
	// 	// we double the minimum block-period so each author can always propose within
	// 	// the majority of their slot.
	// 	<timestamp::Module<T>>::minimum_period().saturating_mul(2.into())
	// }

	// fn new_group(to: T::AccountId, kitty_id: T::Hash, group: Group<T::Hash, T::Balance>) -> Result {

	// }
}

// impl timestamp::Trait for Group<T::AccountId, T::Hash> {
// 	type Moment = u64;
// 	type OnTimestampSet = ();
// }

// *****************************************************************************************************
// Beware of tests
// *****************************************************************************************************

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
	}
	impl Trait for Test {
		type Event = ();
	}
	type GroupsModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			// Just a dummy test for the dummy funtion `do_something`
			// calling the `do_something` function with a value 42
			// assert_ok!(GroupsModule::do_something(Origin::signed(1), 42));
			// asserting that the stored value is equal to what we stored
			// assert_eq!(GroupsModule::something(), Some(42));
		});
	}
}
