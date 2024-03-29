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


pub trait Trait: balances::Trait + timestamp::Trait {
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

        BalanceVal get(balance_val): Option<T::Balance>;
		// SubPool get(subpool): map T::Hash => Group<T::AccountId, T::Hash>;

		Nonce: u64;
	}
}


/*
The events declared here are meant to be used by an external event listener to record state information
in an external datastore.
*/

decl_event!(
    pub enum Event<T> where B = <T as balances::Trait>::Balance {
        NewBalance(B),
    }
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		pub fn add_funds(origin, increase_by: T::Balance) -> Result {
			// This is a public call, so we ensure that the origin is some signed account.
			let _sender = ensure_signed(origin)?;

			// use the `::get` on the storage item type itself
			let balance_val = <BalanceVal<T>>::get();

			// Calculate the new value.
			let new_balance = balance_val.map_or(increase_by, |val| val + increase_by);

			// Put the new value into storage.
			<BalanceVal<T>>::put(new_balance);

			// Deposit an event to let the outside world know this happened.
			Self::deposit_event(RawEvent::NewBalance(increase_by));

			// All good.
			Ok(())
		}

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
