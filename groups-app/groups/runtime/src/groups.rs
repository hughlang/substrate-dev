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
use runtime_primitives::traits::{Hash};
use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageMap, StorageValue};
use system::ensure_signed;

#[cfg(not(feature = "std"))]
use rstd::prelude::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use core::str;
#[cfg(feature = "std")]
use std::str;

pub trait Trait: system::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

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
}

decl_storage! {

	// The Groups storage needs to follow model similar to SubstrateKitties example. In order to fetched
	// owned groups later, additional arrays and maps make it possible to find the number of groups owned by an
	// AccountId and lookup the Hash of a group based on the index values.
	trait Store for Module<T: Trait> as Groups {
		// These are the config values that match the values in the testnet_genesis in chain_spec.rs
		// For unit tests, these also have to be added to the GenesisConfig
		MaxGroupSize get(max_group_size) config(): Option<u32>;
		MaxGroupsPerOwner get(max_groups_per_owner) config(): Option<u64>;
		MaxNameSize get(max_name_size) config(): Option<usize>;

		// These are the primary storage vars for storing the Group struct and recording ownership of a Group
		Groups get(group): map T::Hash => Group<T::AccountId, T::Hash>;
		GroupOwner get(owner_of): map T::Hash => Option<T::AccountId>;

		// This is a generic counter of all groups created in the system.
		AllGroupsCount get(all_groups_count): u64;
		// TODO: Make this more useful by creating a lookup mapping of index to Hash?
		// This might be useful for iterating through all known groups, but

		// These are the mappings that provide lookups for owned groups, given AccountId or Hash
        OwnedGroupsArray get(owned_group_by_index): map (T::AccountId, u64) => T::Hash;
        OwnedGroupsCount get(owned_group_count): map T::AccountId => u64;
        OwnedGroupsIndex get(owned_groups_index): map T::Hash => u64;

		Nonce: u64;
	}
}


/*
The events declared here are meant to be used by an external event listener to record state information
in an external datastore.
*/
decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
	{
		// CreatedGroup should provide the AccountId and group_id Hash to get recorded in another system
		CreatedGroup(Hash, AccountId, u32),

		GroupRenamed(Hash, Vec<u8>),

		GroupSizeChanged(Hash, u32, u32),

		GroupRemoved(Hash),

		MemberJoinedGroup(Hash, AccountId, u32, u32),

		MemberLeftGroup(Hash, AccountId, u32, u32),
	}
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		/// Create a group owned by the current AccountId.
		/// Usage: For name, use String::into_bytes();
		fn create_group(origin, name: Vec<u8>, max_size: u32) -> Result {
			let sender = ensure_signed(origin)?;

			let max_name_size = Self::max_name_size().ok_or("Config max_name_size not set")?;
			ensure!(name.len() <= max_name_size, "Name is too long");

            let nonce = <Nonce<T>>::get();
            let group_id = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

	        ensure!(!<Groups<T>>::exists(group_id), "Group Id already exists");
	        ensure!(!<GroupOwner<T>>::exists(group_id), "GroupOwner already exists");

			let total_groups = Self::all_groups_count();
			let new_groups_count = total_groups.checked_add(1).ok_or("Overflow adding a new group")?;

			let owned_group_count = Self::owned_group_count(&sender);
			let new_owned_group_count = owned_group_count.checked_add(1).ok_or("Overflow adding a new group")?;

			let max_groups_per_owner = Self::max_groups_per_owner().ok_or("Config max_groups_per_owner not set")?;
			ensure!(owned_group_count < max_groups_per_owner, "Groups limit reached for this Account");

			// FIXME: As conversion will be replaced by TryInto
			// https://stackoverflow.com/questions/56081117/how-do-you-convert-between-substrate-specific-types-and-rust-primitive-types
			// let ts = Self::get_time();
			let group = Group {
				id: group_id,
				name: name,
				members: Vec::new(),
				max_size: max_size,
			};
			<Groups<T>>::insert(group_id, group);
			<GroupOwner<T>>::insert(group_id, &sender);
			<AllGroupsCount<T>>::put(new_groups_count);

			<OwnedGroupsArray<T>>::insert((sender.clone(), owned_group_count), group_id);
			<OwnedGroupsCount<T>>::insert(&sender, new_owned_group_count);
			<OwnedGroupsIndex<T>>::insert(group_id, owned_group_count);

			<Nonce<T>>::mutate(|n| *n += 1);

			Self::deposit_event(RawEvent::CreatedGroup(group_id, sender, max_size));
			Ok(())
		}

		/// Renaming a group by providing a byte array of the string value
		/// Rule: only the owner is allowed to use this function.
		/// Usage: For name, use String::into_bytes();
		fn rename_group(origin, group_id: T::Hash, name: Vec<u8>) -> Result {
			let sender = ensure_signed(origin)?;

			let max_name_size = Self::max_name_size().ok_or("Config max_name_size not set")?;
			ensure!(name.len() <= max_name_size, "Name is too long");

			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");

			let mut group = Self::group(group_id);

			// TODO: ensure unchanged?
			group.name = name.clone();
			<Groups<T>>::insert(group.id, group);

			Self::deposit_event(RawEvent::GroupRenamed(group_id, name));
			Ok(())
		}

		/// This method updates the max_size for the specified group_id, but only
		/// for the owner of the group.
		fn update_group_size(origin, group_id: T::Hash, max_size: u32) -> Result {
			let sender = ensure_signed(origin)?;

			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");

			let max_group_size = Self::max_group_size().ok_or("Config max_group_size not set")?;
			ensure!(max_size <= max_group_size, "Group size too large");

			let mut group = Self::group(group_id);
			let current_size = group.members.len() as u32;
			ensure!(current_size <= max_size, "Current member count exceeds new group size");

			// TODO: ensure unchanged?
			group.max_size = max_size;
			<Groups<T>>::insert(group.id, group);

			Self::deposit_event(RawEvent::GroupSizeChanged(group_id, max_size, current_size));
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

			Self::deposit_event(RawEvent::GroupRemoved(group_id));
			Ok(())
		}

		/*
		The Join functionality is barebones and is not meant to hold much application-specific logic.
		In some group-membership frameworks, there is a notion of an invite or a request to join. This may be
		a future enhancement, but it seems more likely that the state information for this should not be
		on-chain. Instead, webapps that use this module should listen for events that can be used to store
		state information in another datastore.

		Also, one desired improvement is to record proof from an external oracle that verifies that the
		join_group event can include an authorization hash that represents another system's proof that the
		action is approved.

		Rules:
		– The owner can join their own group, but is not required to be a member of that group.
		– Otherwise, any accountId can join the group up to the max_size of the group
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
				ensure!(!group.members.contains(&sender), "You are already a member of this group");
				group.members.push(sender.clone());
			}

			let current_size = group.members.len() as u32;
			Self::deposit_event(RawEvent::MemberJoinedGroup(group_id, sender, group.max_size, current_size));
			Ok(())
		}

		fn leave_group(origin, group_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
			let mut group = Self::group(group_id);
			ensure!(group.members.contains(&sender), "You are not a member of this group");

			if let Some(index) = group.members.iter().position(|x| *x == sender) {
				group.members.remove(index);
			}

			let current_size = group.members.len() as u32;
			Self::deposit_event(RawEvent::MemberLeftGroup(group_id, sender, group.max_size, current_size));
			Ok(())
		}

		fn remove_group_member(origin, group_id: T::Hash, user: T::AccountId) -> Result {
			Ok(())
		}

		// This method is meant to be used for absolute verification that an AccountId is a member of
		// the specified group. Currently, group membership is dynamic
		fn verify_group_member(origin, group_id: T::Hash, user: T::AccountId) -> Result {
			Ok(())
		}
	}
}

/// Custom methods but public and private go here
impl<T: Trait> Module<T> {
	// Unused right now.
	pub fn get_time() -> T::Moment {
		let now = <timestamp::Module<T>>::get();
		now
	}
}

// *****************************************************************************************************
// Unit Tests!
// *****************************************************************************************************

#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::{with_externalities};
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
	// TODO: _genesis_phantom_data: Default::default() can be removed later if using latest substrate fixes
	// Error: missing field `_genesis_phantom_data` in initializer of `groups::GenesisConfig<groups::tests::GroupsTest>`
	// See also: https://github.com/paritytech/substrate/pull/2913 and Issue #2219
	fn build_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::<GroupsTest>::default().build_storage().unwrap().0;
		t.extend(
			GenesisConfig::<GroupsTest> {
				max_group_size: 10,
				max_groups_per_owner: 5,
				max_name_size: 40,
				_genesis_phantom_data: Default::default(),
			}.build_storage().unwrap().0);
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
