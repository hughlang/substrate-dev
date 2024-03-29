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
///   current implementation does not check for uniqueness of the name field, which is out of scope.

use parity_codec::{Encode, Decode};
use runtime_primitives::traits::{Hash};
use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageMap, StorageValue};
use system::ensure_signed;

// use runtime_io::{with_storage, StorageOverlay, ChildrenStorageOverlay};

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
	/// Arbitrary field that can be used for human-readable name or foreign key in other system.
	/// The length of this field is limited by the max_name_size Config.
	name: Vec<u8>,
	/// Vec of AccountIds, where the owner is not automatically added and can just be an external actor
	/// The size of this list is limited by the max_group_size Config.
	members: Vec<A>,
	/// Maximum number of members in group. Note that there is no min size of group since that is
	/// likely a business rule that can be handled in the dapp or external systems.
	/// Example: number of players required to start a game.
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
		/// CreatedGroup should provide the AccountId and group_id Hash to get recorded in another system
		CreatedGroup(Hash, AccountId, u32),

		/// This event allows event listener to update DB and UI with name change
		GroupRenamed(Hash, Vec<u8>),

		/// This event allows event listener to update DB and UI with group size change.
		/// The max_size and current_size values are also provided.
		/// This would be useful for allowing more/less users to join the group.
		GroupSizeChanged(Hash, u32, u32),

		/// Event fired when the owner removes a group.
		GroupRemoved(Hash),

		/// Event fired when a member joins a group. The max_size and current_size values are also provided.
		MemberJoinedGroup(Hash, AccountId, u32, u32),

		/// Event fired when a member leaves a group. The max_size and current_size values are also provided.
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
		fn owner_remove_group(origin, group_id: T::Hash) -> Result {
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
		The group membership functionality is barebones and is not meant to hold much application-specific logic.
		In some group-membership frameworks, there is a notion of an invite or a request to join. This may be
		a future enhancement, but it seems more likely that the state information for this should not be
		on-chain. Instead, webapps that use this module should listen for events that can be used to store
		state information in another datastore.

		Rules:
		– The owner can join their own group, but is not required to be a member of that group.
		– Otherwise, any accountId can join the group up to the max_size of the group
		*/

		/// Method for use case where user voluntarily joins a group
		fn join_group(origin, group_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");

			Self::add_member(group_id, sender)?;
			Ok(())
		}

		/// Method for use case where user voluntarily leaves a group
		fn leave_group(origin, group_id: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");

			Self::remove_member(group_id, sender)?;
			Ok(())
		}

		/// Method for use case where owner adds a group member
		fn owner_add_member(origin, group_id: T::Hash, user: T::AccountId) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");

			Self::add_member(group_id, user)?;
			Ok(())
		}

		/// Method for use case where owner removes a group member
		fn owner_remove_member(origin, group_id: T::Hash, user: T::AccountId) -> Result {
			let sender = ensure_signed(origin)?;
			ensure!(<Groups<T>>::exists(group_id), "This group does not exist");
            let owner = Self::owner_of(group_id).ok_or("No owner for this group")?;
            ensure!(owner == sender, "You do not own this group");

			Self::remove_member(group_id, user)?;
			Ok(())
		}
	}
}

/// Custom methods – public and private
impl<T: Trait> Module<T> {
	// Private method called by: join_group() and owner_add_member()
	fn add_member(group_id: T::Hash, user: T::AccountId) -> Result {
		let mut group = Self::group(group_id);
		ensure!((group.members.len() as u32) < group.max_size, "Group is already full");
		ensure!(!group.members.contains(&user), "Account is already a member of this group");
		group.members.push(user.clone());

		let max_size = group.max_size;
		let current_size = group.members.len() as u32;
		<Groups<T>>::insert(group_id, group);

		Self::deposit_event(RawEvent::MemberJoinedGroup(group_id, user, max_size, current_size));
		Ok(())
	}

	// Private method called by: leave_group() and owner_remove_member()
	fn remove_member(group_id: T::Hash, user: T::AccountId) -> Result {
		let mut group = Self::group(group_id);

		ensure!(group.members.contains(&user), "Account is not a member of this group");
		if let Some(index) = group.members.iter().position(|x| *x == user) {
			group.members.remove(index);
		}

		let max_size = group.max_size;
		let current_size = group.members.len() as u32;
		<Groups<T>>::insert(group_id, group);

		Self::deposit_event(RawEvent::MemberLeftGroup(group_id, user, max_size, current_size));
		Ok(())
	}

	/// Helper method that can be used from UI code to verify member.
	pub fn is_group_member(group_id: T::Hash, user: T::AccountId) -> bool {
		let group = Self::group(group_id);
		group.members.contains(&user)
	}

	// Unused right now. Still considering timestamps for some record-keeping
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
				max_group_size: 12,
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

	/// Update Group test objectives:
	/// * First create_group and verify owned count is 1 and group_id Hash can be fetched
	/// * Call rename_group with the correct owner and assert ok
	/// * Load the group by hash and verify that name has changed.
	/// * Update group size and verify change
	/// * Remove group and verify owned_group_count is 0
	#[test]
	fn update_owned_group_should_work() {
		with_externalities(&mut build_ext(), || {
			let data = "Test Group".as_bytes().to_vec();
			let owner = Origin::signed(11);

            assert_ok!(Groups::create_group(owner.clone(), data, 8));
			assert_eq!(Groups::owned_group_count(11), 1);

            let group_id = Groups::owned_group_by_index((11, 0));
			assert_ok!(Groups::rename_group(owner.clone(), group_id, "Renamed Group".as_bytes().to_vec()));

			let group = Groups::group(group_id);
			if let Ok(name) = str::from_utf8(&group.name) {
				assert_eq!(name, "Renamed Group");
			} else {
				runtime_io::print("Could not read group name"); // doesn't print to CLI
				assert!(false);
			}

			let data = "Invalid Group".as_bytes().to_vec();
			assert_noop!(Groups::rename_group(Origin::signed(9), group_id, data), "You do not own this group");

			// Update group max_size
			assert_ok!(Groups::update_group_size(owner.clone(), group_id, 12));
			let group = Groups::group(group_id);
			assert_eq!(group.max_size, 12);

			// Owner removes group
			assert_ok!(Groups::owner_remove_group(owner.clone(), group_id));
			assert_eq!(Groups::owned_group_count(11), 0);

		});
	}

	/*
		Join Group tests: success path
		* Comprehensive test of all ways of adding and removing members from group (voluntarily and involuntary)
	*/
	#[test]
	fn join_and_leave_group_should_work() {
		with_externalities(&mut build_ext(), || {
			// Create basic group with max_size of 4
			let data = "Group of 4".as_bytes().to_vec();
			let owner = Origin::signed(20);
            assert_ok!(Groups::create_group(owner.clone(), data, 4));

			// Lookup group_id hash and verify
            let group_id = Groups::owned_group_by_index((20, 0));
			let group = Groups::group(group_id);
            assert_eq!(group.id, group_id);

			// Add 4 members: 21-24
            assert_ok!(Groups::join_group(Origin::signed(21), group_id));
            assert_ok!(Groups::join_group(Origin::signed(22), group_id));
            assert_ok!(Groups::join_group(Origin::signed(23), group_id));
            assert_ok!(Groups::join_group(Origin::signed(24), group_id));

			// Now verify group members count and membership
			let group = Groups::group(group_id);
            assert_eq!(group.members.len(), 4);
			assert!(Groups::is_group_member(group_id, 21));
			assert!(Groups::is_group_member(group_id, 22));
			assert!(Groups::is_group_member(group_id, 23));
			assert!(Groups::is_group_member(group_id, 24));

			// 24 leaves group. Verify member count and not a member
            assert_ok!(Groups::leave_group(Origin::signed(24), group_id));
			let group = Groups::group(group_id);
            assert_eq!(group.members.len(), 3);
			assert!(!Groups::is_group_member(group_id, 24));

			// Group owner adds 25 to group.
            assert_ok!(Groups::owner_add_member(owner.clone(), group_id, 25));
			let group = Groups::group(group_id);
            assert_eq!(group.members.len(), 4);
			assert!(Groups::is_group_member(group_id, 25));

			// Group owner removes 21 from group.
            assert_ok!(Groups::owner_remove_member(owner.clone(), group_id, 21));
			let group = Groups::group(group_id);
            assert_eq!(group.members.len(), 3);
			assert!(!Groups::is_group_member(group_id, 21));

		});
	}

	/*
		Join Group tests: negative path
		* Test all error state possibilities for add/remove group members functions
	*/
	#[test]
	fn group_rules_should_err() {
		with_externalities(&mut build_ext(), || {
			// Create basic group with max_size of 4
			let data = "Strict Group of 4".as_bytes().to_vec();
			let owner = Origin::signed(20);
            assert_ok!(Groups::create_group(owner.clone(), data, 4));

			// Lookup group_id hash and verify
            let group_id = Groups::owned_group_by_index((20, 0));
			let group = Groups::group(group_id);
            assert_eq!(group.id, group_id);

			// Add 4 members: 21-24
            assert_ok!(Groups::join_group(Origin::signed(21), group_id));
            assert_ok!(Groups::join_group(Origin::signed(22), group_id));
            assert_ok!(Groups::join_group(Origin::signed(23), group_id));
            assert_ok!(Groups::join_group(Origin::signed(24), group_id));

			// Try to exceed the max_size. Even the owner can't join.
			assert_noop!(Groups::join_group(Origin::signed(20), group_id), "Group is already full");
			// Try to leave group that you don't belong to.
			assert_noop!(Groups::leave_group(Origin::signed(25), group_id), "Account is not a member of this group");
			// Try to remove user not member of group
            assert_noop!(Groups::owner_remove_member(owner.clone(), group_id, 26), "Account is not a member of this group");
			// Non-owner tries to add user
            assert_noop!(Groups::owner_add_member(Origin::signed(21), group_id, 27), "You do not own this group");

		});
	}
}
