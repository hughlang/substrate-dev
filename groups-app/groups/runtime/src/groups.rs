/// A runtime module template with necessary imports

/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/master/srml/example/src/lib.rs
use parity_codec::{Encode, Decode};
use runtime_primitives::traits::{As, Hash, Zero};
use support::{decl_module, decl_storage, decl_event, ensure, dispatch::Result, StorageMap, StorageValue};
use system::ensure_signed;
// use inherents::{RuntimeString, InherentData};

#[cfg(not(feature = "std"))]
use rstd::prelude::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// The module's configuration trait.
pub trait Trait: system::Trait {

	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Group<Hash> {
    id: Hash,
	name: Vec<u8>,
	status: u32,
	max_size: u32,
	members: Vec<Hash>,
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
		Groups get(group): map T::Hash => Group<T::Hash>;
		GroupOwner get(owner_of): map T::Hash => Option<T::AccountId>;
		LiveGroupsCount get(live_groups_count): u64;

		// AllGroupsCount get(total_groups): u64;

		Nonce: u64;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
	{
		CreatedGroup(AccountId, Hash),
	}
);

decl_module! {
	/// The module declaration.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Initializing events
		// this is needed only if you are using events in your module
		fn deposit_event<T>() = default;

		/*
			Use cases TODO:
			– Create group
			– Update group
			– Remove group
			– Request invite
			– Create invite
			– Accept invite
			– List members
			– Verify member (groupId, accountId)
			– Broadcast message?
			– Request vote
			– Submit vote
		*/

		/// Create a group owned by the current AccountId
		fn create_group(origin, bytes: Vec<u8>, max_size: u32) -> Result {
			let sender = ensure_signed(origin)?;

            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);
			// let bytes = name.into_bytes();
			// let group = Group {
			// 	id: random_hash,
			// 	name: bytes,
			// 	max_size: max_size,
			// 	owner: sender,
			// };
			<Nonce<T>>::mutate(|n| *n += 1);
			Ok(())
		}

		fn rename_group(group_id: T::Hash, bytes: Vec<u8>) -> Result {
			Ok(())
		}

	}
}

/// Private methods
impl<T: Trait> Module<T> {
	// fn new_group(to: T::AccountId, kitty_id: T::Hash, group: Group<T::Hash, T::Balance>) -> Result {

	// }
}

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
