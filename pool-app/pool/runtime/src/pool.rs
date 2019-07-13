/// Pool is an experimental module for managing pooled funds

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

// #[derive(Encode, Decode, Default, Clone, PartialEq)]
// #[cfg_attr(feature = "std", derive(Debug))]
// pub struct Group<A, H> {
// 	/// Hash unique random id
//     id: H,
// 	/// Arbitrary field that can be used for human-readable name or foreign key in other system.
// 	/// The length of this field is limited by the max_name_size Config.
// 	name: Vec<u8>,
// 	/// Vec of AccountIds, where the owner is not automatically added and can just be an external actor
// 	/// The size of this list is limited by the max_group_size Config.
// 	members: Vec<A>,
// 	/// Maximum number of members in group. Note that there is no min size of group since that is
// 	/// likely a business rule that can be handled in the dapp or external systems.
// 	/// Example: number of players required to start a game.
// 	max_size: u32,
// }

decl_storage! {

	// The Pool storage needs to follow model similar to SubstrateKitties example. In order to fetched
	// owned groups later, additional arrays and maps make it possible to find the number of groups owned by an
	// AccountId and lookup the Hash of a group based on the index values.
	trait Store for Module<T: Trait> as Pool {
		// These are the config values that match the values in the testnet_genesis in chain_spec.rs
		// For unit tests, these also have to be added to the GenesisConfig
		// MaxGroupSize get(max_group_size) config(): Option<u32>;
		// MaxPoolPerOwner get(max_groups_per_owner) config(): Option<u64>;
		// MaxNameSize get(max_name_size) config(): Option<usize>;

		// // These are the primary storage vars for storing the Group struct and recording ownership of a Group
		// Pool get(group): map T::Hash => Group<T::AccountId, T::Hash>;
		// GroupOwner get(owner_of): map T::Hash => Option<T::AccountId>;

		// // This is a generic counter of all groups created in the system.
		// AllPoolCount get(all_groups_count): u64;
		// // TODO: Make this more useful by creating a lookup mapping of index to Hash?
		// // This might be useful for iterating through all known groups, but

		// // These are the mappings that provide lookups for owned groups, given AccountId or Hash
        // OwnedPoolArray get(owned_group_by_index): map (T::AccountId, u64) => T::Hash;
        // OwnedPoolCount get(owned_group_count): map T::AccountId => u64;
        // OwnedPoolIndex get(owned_groups_index): map T::Hash => u64;

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
        // <T as system::Trait>::Hash
	{
		SomethingStored(u32, AccountId),
	}
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;


	}
}

/// Custom methods – public and private
impl<T: Trait> Module<T> {

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
		pub enum Origin for PoolTest {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`PoolTest`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct PoolTest;
	impl system::Trait for PoolTest {
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
	impl timestamp::Trait for PoolTest {
		type Moment = u64;
		type OnTimestampSet = ();
	}
	impl Trait for PoolTest {
		type Event = ();
	}
	type Pool = Module<PoolTest>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	// TODO: _genesis_phantom_data: Default::default() can be removed later if using latest substrate fixes
	// Error: missing field `_genesis_phantom_data` in initializer of `groups::GenesisConfig<groups::tests::PoolTest>`
	// See also: https://github.com/paritytech/substrate/pull/2913 and Issue #2219
	fn build_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let t = system::GenesisConfig::<PoolTest>::default().build_storage().unwrap().0;
		// t.extend(
		// 	GenesisConfig::<PoolTest> {
		// 		max_group_size: 12,
		// 		max_groups_per_owner: 5,
		// 		max_name_size: 40,
		// 		_genesis_phantom_data: Default::default(),
		// 	}.build_storage().unwrap().0);
		t.into()
	}
}
