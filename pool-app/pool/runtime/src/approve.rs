/// Approve is an experimental module for managing pooled funds

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
pub struct Decision<A, H> {
	/// Hash unique random id
    id: H,
	/// Reference to the Group
	group_id: H,
	/// Vec of AccountIds
	approvers: Vec<A>,
	/// Maximum number of members in group. Note that there is no min size of group since that is
	/// likely a business rule that can be handled in the dapp or external systems.
	/// Example: number of players required to start a game.
	record: H,
}

decl_storage! {

	// The Approve storage needs to follow model similar to SubstrateKitties example. In order to fetched
	// owned groups later, additional arrays and maps make it possible to find the number of groups owned by an
	// AccountId and lookup the Hash of a group based on the index values.
	trait Store for Module<T: Trait> as Approve {


		Nonce: u64;
	}
}


/*
Approve events TODO:
–

*/
decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
	{
		ApprovalReceived(Hash, AccountId, u32),
    }
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;


		/*
		Functions TODO:
		– register topic
		– record choice (approve, deny)
		–

		*/
		fn register_topic(origin, group_id: T::Hash, max_size: u32) -> Result {
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
		pub enum Origin for ApproveTest {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`ApproveTest`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct ApproveTest;
	impl system::Trait for ApproveTest {
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
	impl timestamp::Trait for ApproveTest {
		type Moment = u64;
		type OnTimestampSet = ();
	}
	impl Trait for ApproveTest {
		type Event = ();
	}
	type Approve = Module<ApproveTest>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	// TODO: _genesis_phantom_data: Default::default() can be removed later if using latest substrate fixes
	// Error: missing field `_genesis_phantom_data` in initializer of `groups::GenesisConfig<groups::tests::ApproveTest>`
	// See also: https://github.com/paritytech/substrate/pull/2913 and Issue #2219
	fn build_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		let t = system::GenesisConfig::<ApproveTest>::default().build_storage().unwrap().0;
		// t.extend(
		// 	GenesisConfig::<ApproveTest> {
		// 		max_group_size: 12,
		// 		max_groups_per_owner: 5,
		// 		max_name_size: 40,
		// 		_genesis_phantom_data: Default::default(),
		// 	}.build_storage().unwrap().0);
		t.into()
	}
}
