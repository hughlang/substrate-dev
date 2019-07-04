/// The Groups module is designed for the most common use cases for managing and verifying *small* groups of users.
/// By itself, it only provides a on-chain storage of group membership for a set of AccountIds. Arguably, this does
/// not need to be stored on-chain since it is application-specific logic. However, in conjunction with other Substrate
/// modules, the ability to verify group membership before execution of other app/storage logic is useful to provide
/// auditable proof that group membership rules are not violated. Examples: multiplayer games, multiparty voting
///
/// Notes:
/// * The choice to include a "name" field for Group may not be advisable because of blockchain bloat.
///   Mainly, it was done to test out Vec<u8> storage of string data and conversion back to string.
/// * Still, the name field could also be used for foreign key reference to an external data store. However, the
///   current implementation does not check for uniqueness of the name field.

use parity_codec::{Encode, Decode};
use runtime_primitives::traits::{As, Hash};
use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageMap, StorageValue};
use system::ensure_signed;
// use inherents::{RuntimeString};
// use rstd::convert::TryInto;

#[cfg(not(feature = "std"))]
use rstd::prelude::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use core::str;
#[cfg(feature = "std")]
use std::str;

// TODO: Make these Configure values in genesis
pub const MAX_GROUP_SIZE: u32 = 8;
pub const MAX_GROUPS_PER_OWNER: u64 = 5;
pub const MAX_NAME_SIZE: usize = 40;

pub trait Trait: system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// #[derive(Encode, Decode, Clone, PartialEq)]
// #[cfg_attr(feature = "std", derive(Debug))]
// #[repr(u32)]
// pub enum JoinRule {
// 	Any,
// 	Request,
// }

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Group<A, H> {
	/// Hash unique random id
    id: H,
	/// Arbitrary field that can be used for human-readable name or foreign key in other system
	name: Vec<u8>,
	/// Vec of AccountIds
	members: Vec<A>,
	/// Maximum number of members in group
	max_size: u32,
	allow_any: bool,
	timestamp: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Invite<A, H> {
	/// Hash unique random id
    id: H,
	/// The Group reference
	group_id: H,
	/// AccountId of the user who can join
	user: A,
	timestamp: u64,
}


decl_storage! {

	// The Groups storage needs to follow model similar to SubstrateKitties example. In order to fetched
	// owned groups later, additional arrays and maps make it possible to find the number of groups owned by an
	// AccountId and lookup the Hash of a group based on the index values.
	//
	trait Store for Module<T: Trait> as Groups {

		Groups get(group): map T::Hash => Group<T::AccountId, T::Hash>;
		GroupOwner get(owner_of): map T::Hash => Option<T::AccountId>;

		AllGroupsCount get(all_groups_count): u64;
        OwnedGroupsArray get(owned_group_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedGroupsCount get(owned_group_count): map T::AccountId => u64;
        OwnedGroupsIndex get(owned_groups_index): map T::Hash => u64;

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

		fn deposit_event<T>() = default;

		/// TBD: Is this needed?
		fn default_group(origin) -> Result {
			// let ts = Self::get_time();
			// let name = format!("Group-{}", ts);
			Self::create_group(origin, "New Group".as_bytes().to_vec(), MAX_GROUP_SIZE)
		}

		/// Create a group owned by the current AccountId.
		/// Usage: For name, use String::into_bytes();
		fn create_group(origin, name: Vec<u8>, max_size: u32) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(name.len() <= MAX_NAME_SIZE, "Name size too long");

            let nonce = <Nonce<T>>::get();
            let random_id = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

	        ensure!(!<Groups<T>>::exists(random_id), "Group Id already exists");
	        ensure!(!<GroupOwner<T>>::exists(random_id), "GroupOwner already exists");

			let total_groups = Self::all_groups_count();
			let new_groups_count = total_groups.checked_add(1).ok_or("Overflow adding a new group")?;

			let owned_group_count = Self::owned_group_count(&sender);
			ensure!(owned_group_count < MAX_GROUPS_PER_OWNER, "Groups limit reached for this Account");
			let new_owned_group_count = owned_group_count.checked_add(1).ok_or("Overflow adding a new group")?;

			// FIXME: As conversion will be replaced by TryInto
			// https://stackoverflow.com/questions/56081117/how-do-you-convert-between-substrate-specific-types-and-rust-primitive-types
			let ts = Self::get_time();
			let group = Group {
				id: random_id,
				name: name,
				members: Vec::new(),
				max_size: max_size,
				allow_any: true,
				timestamp: ts.as_(),
			};
			<Groups<T>>::insert(random_id, group);
			<GroupOwner<T>>::insert(random_id, &sender);
			<AllGroupsCount<T>>::put(new_groups_count);

			<OwnedGroupsArray<T>>::insert((sender.clone(), owned_group_count), random_id);
			<OwnedGroupsCount<T>>::insert(&sender, new_owned_group_count);
			<OwnedGroupsIndex<T>>::insert(random_id, owned_group_count);

			<Nonce<T>>::mutate(|n| *n += 1);
			// Self::deposit_event(RawEvent::CreatedGroup(sender, group_id, new_price));

			Ok(())
		}

		/// Renaming a group by converting the String name into a byte array
		/// Rule: only the owner is allowed to use this function.
		/// Usage: For name, use String::into_bytes();
		fn rename_group(origin, group_id: T::Hash, name: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(name.len() <= MAX_NAME_SIZE, "Name size too long");

			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");

			let mut group = Self::group(group_id);
			group.name = name;
			<Groups<T>>::insert(group.id, group);
			Ok(())
		}

		/// This method updates the max_size for the specified group_id, but only
		/// for the owner of the group.
		fn update_group_size(origin, group_id: T::Hash, max_size: u32) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");
			ensure!(max_size <= MAX_GROUP_SIZE, "Group size too large");

			let mut group = Self::group(group_id);
			ensure!(group.members.len() as u32 <= max_size, "Current member count exceeds new group size");

			group.max_size = max_size;

			<Groups<T>>::insert(group.id, group);
			Ok(())
		}

		/// Remove group and update all storage with new values
		/// Rule: only owner can remove a group
		fn remove_group(origin, group_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");

			let total_groups = Self::all_groups_count();
			let new_groups_count = total_groups.checked_sub(1).ok_or("Overflow subtracting a group")?;

			let owned_group_count = Self::owned_group_count(&sender);
			let new_owned_group_count = owned_group_count.checked_sub(1).ok_or("Overflow subtracting a group")?;
			// Get the index position of the group, so it can be removed
			let group_index = <OwnedGroupsIndex<T>>::get(group_id);

			<Groups<T>>::remove(group_id);
			<GroupOwner<T>>::remove(group_id);
			<AllGroupsCount<T>>::put(new_groups_count);

			<OwnedGroupsArray<T>>::remove((sender.clone(), group_index));
			<OwnedGroupsCount<T>>::insert(&sender, new_owned_group_count);
			<OwnedGroupsIndex<T>>::remove(group_id);

			Ok(())
		}

		/*
			Rules:
			– The owner can join their own group, but is not required to be a member of that group.
			– If group.allow_any == true, any accountId can join the group up to the max_size of the group
		*/
		fn join_group(origin, group_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
			let mut group = Self::group(group_id);
			ensure!((group.members.len() as u32) < group.max_size, "Group is already full");

            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
			if owner == sender {
				ensure!(!group.members.contains(&owner), "Owner is already a member of this group");
				group.members.push(owner);
			} else {
				if group.allow_any {
					ensure!(!group.members.contains(&sender), "You are already a member of this group");
					group.members.push(sender);
				} else {

				}
			}
			Ok(())
		}

		/*
		This method allows a user to request to join a Group given its Hash group_id.

		*/
		fn request_join_group(origin, group_id: T::Hash) -> Result {


			Ok(())
		}

		fn leave_group(origin, group_id: T::Hash) -> Result {
			Ok(())
		}

		fn accept_member(origin, group_id: T::Hash, member_id: T::AccountId) -> Result {
			Ok(())
		}

		fn reject_member(origin, group_id: T::Hash, member_id: T::AccountId) -> Result {
			Ok(())
		}

		fn verify_member(origin, group_id: T::Hash, member_id: T::AccountId) -> Result {
			Ok(())
		}

		/*
		TODO
		– admin remove group
		–
		*/
	}
}

/// Private methods
impl<T: Trait> Module<T> {
	pub fn get_time() -> T::Moment {
		let now = <timestamp::Module<T>>::get();
		now
	}
}

// impl timestamp::Trait for Group<T::AccountId, T::Hash> {
// 	type Moment = u64;
// 	type OnTimestampSet = ();
// }

// *****************************************************************************************************
// Unit Tests!
// *****************************************************************************************************

#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::{with_externalities, TestExternalities};
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, assert_noop};
	use runtime_primitives::{
		BuildStorage,
		traits::{BlakeTwo256, IdentityLookup},
		testing::{Digest, DigestItem, Header}
	};

	impl_outer_origin! {
		pub enum Origin for GroupsTest {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`GroupsTest`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct GroupsTest;
	impl system::Trait for GroupsTest {
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
	impl timestamp::Trait for GroupsTest {
		type Moment = u64;
		type OnTimestampSet = ();
	}
	impl Trait for GroupsTest {
		type Event = ();
	}
	type Groups = Module<GroupsTest>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn build_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::<GroupsTest>::default().build_storage().unwrap().0;
		// t.extend(balances::GenesisConfig::<GroupsTest>::default().build_storage().unwrap().0);
		t.into()
	}

	/// Create Group test objectives:
	/// * Use the create_group method and verify ok
	/// * Verify all_groups_count == 1
	/// * Use the OwnedGroupsArray to get the Hash of the new Group
	/// * Fetch the group using the Hash. Verify group_id and name.
	/// * Rename group and with wrong AccountId should be an error
	#[test]
	fn create_group_should_work() {
		with_externalities(&mut build_ext(), || {
			let data = "First Group".as_bytes().to_vec();
			let owner = Origin::signed(10);
            assert_ok!(Groups::create_group(Origin::signed(10), data, 8));
            assert_eq!(Groups::all_groups_count(), 1);
			assert_eq!(Groups::owned_group_count(10), 1);

            let hash = Groups::owned_group_by_index((10, 0));
			let group = Groups::group(hash);
            assert_eq!(group.id, hash);

			if let Ok(name) = str::from_utf8(&group.name) {
				assert_eq!(name, "First Group");
			} else {
				assert!(false);
			}
		});
	}

	/// Rename Group test objectives:
	/// * First create_group and verify owned count is 1 and group_id Hash can be fetched
	/// * Call rename_group with the correct owner and assert ok
	/// * Load the group by hash and verify that name has changed.
	/// * And finally, call rename_group with the wrong owner and expect error
	#[test]
	fn rename_owned_group_should_work() {
		with_externalities(&mut build_ext(), || {
			let data = "Test Group".as_bytes().to_vec();
			let owner = Origin::signed(11);

            assert_ok!(Groups::create_group(Origin::signed(11), data, 8));
			assert_eq!(Groups::owned_group_count(11), 1);

            let hash = Groups::owned_group_by_index((11, 0));
			assert_ok!(Groups::rename_group(owner, hash, "Renamed Group".as_bytes().to_vec()));

			let group = Groups::group(hash);
			if let Ok(name) = str::from_utf8(&group.name) {
				assert_eq!(name, "Renamed Group");
			} else {
				runtime_io::print("Could not read group name"); // doesn't print to CLI
				assert!(false);
			}

			let data = "Invalid Group".as_bytes().to_vec();
			assert_noop!(Groups::rename_group(Origin::signed(9), hash, data), "You do not own this group");
		});
	}

	/*
		Join Group test (happy path)
		* The group.max_size will limit the number of AccountIds that can join the group
		* The owner of a group is not a member by default (and should not be a fixed requirement) and can join
		  the group IF owner AccountId does not exist in group.members
		* For non-owners: If group.allow_any == true and AccountId is not already in group.members, add it.
	*/
	#[test]
	fn join_group_should_work() {
		with_externalities(&mut build_ext(), || {
			// let data = "First Group".as_bytes().to_vec();
			// let owner = Origin::signed(10);
            // assert_ok!(Groups::create_group(Origin::signed(10), data, 8));
            // assert_eq!(Groups::all_groups_count(), 1);
			// assert_eq!(Groups::owned_group_count(10), 1);

            // let hash = Groups::owned_group_by_index((10, 0));
			// let group = Groups::group(hash);
            // assert_eq!(group.id, hash);

			// if let Ok(name) = str::from_utf8(&group.name) {
			// 	assert_eq!(name, "First Group");
			// } else {
			// 	assert!(false);
			// }
		});
	}


}
