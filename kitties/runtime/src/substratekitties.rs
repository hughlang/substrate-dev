use parity_codec::{Encode, Decode};
use rstd::cmp;
use runtime_primitives::traits::{As, Hash, Zero};
use support::{decl_storage, decl_module, decl_event, ensure, StorageMap, StorageValue, dispatch::Result};
use support::traits::Currency;
use system::ensure_signed;

use runtime_io::{with_storage, StorageOverlay, ChildrenStorageOverlay};

pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64,
}

// NOTE: We have added this `decl_event!` template for you
decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance
    {
        // ACTION: Add a `Created` event which includes an `AccountId` and a `Hash`
        Created(AccountId, Hash),
        PriceSet(AccountId, Hash, Balance),
        Transferred(AccountId, AccountId, Hash),
        Bought(AccountId, AccountId, Hash, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        // Declare storage and getter functions here

        //         - `Kitties` which maps a `T::Hash` to a `Kitty<T::Hash, T::Balance>`
        //         - `KittyOwner` which maps a `T::Hash` to an `Option<T::AccountId>`
        Kitties get(kitty): map T::Hash => Kitty<T::Hash, T::Balance>;
        KittyOwner get(owner_of): map T::Hash => Option<T::AccountId>;

        // ACTION: Create new storage items to globally track all kitties:
        //         - `AllKittiesArray` which is a `map` from `u64` to `T::Hash`, add a getter function for this
        //         - `AllKittiesCount` which is a `u64`, add a getter function for this
        //         - `AllKittiesIndex` which is a `map` from `T::Hash` to `u64`
        AllKittiesArray get(kitty_id): map u64 => T::Hash;
        AllKittiesCount get(num_of_kitties): u64;
        AllKittiesIndex get(index_of): map T::Hash => u64;

        OwnedKittiesArray get(kitty_of_owner_by_index): map (T::AccountId, u64) => T::Hash;

        // ACTION: Add a new storage item `OwnedKittiesCount` which is a `map` from `T::AccountId` to `u64`
        // ACTION: Add a new storage item `OwnedKittiesIndex` which is a `map` from `T::Hash` to `u64`
        OwnedKittiesCount get(owned_kitty_count): map T::AccountId => u64;
        OwnedKittiesIndex get(owned_kitties_index): map T::Hash => u64;

        Nonce: u64;
    }

    add_extra_genesis {
        config(kitties): Vec<(T::AccountId, T::Hash, T::Balance)>;

        build(|storage: &mut StorageOverlay, _: &mut ChildrenStorageOverlay, config: &GenesisConfig<T>| {
            with_storage(storage, || {
                for &(ref acct, hash, balance) in &config.kitties {

                    let k = Kitty {
                                id: hash,
                                dna: hash,
                                price: balance,
                                gen: 0
                            };

                    let _ = <Module<T>>::mint(acct.clone(), hash, k);
                }
            });
        });
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Declare public functions here
        fn deposit_event<T>() = default;

        fn create_kitty(origin) -> Result {
            let sender = ensure_signed(origin)?;

            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            };
            Self::mint(sender, random_hash, new_kitty)?;

            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }
        fn set_price(origin, kitty_id: T::Hash, new_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // ACTION: Check that the kitty with `kitty_id` exists
            ensure!(<Kitties<T>>::exists(kitty_id), "This cat does not exist");

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this cat");

            let mut kitty = Self::kitty(kitty_id);

            // ACTION: Set the new price for the kitty
            kitty.price = new_price;

            // ACTION: Update the kitty in storage
            <Kitties<T>>::insert(kitty_id, kitty);

            // ACTION: Deposit a `PriceSet` event with relevant data
            //         - owner
            //         - kitty id
            //         - the new price
            Self::deposit_event(RawEvent::PriceSet(sender, kitty_id, new_price));

            Ok(())
        }

        fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty");

            Self::transfer_from(sender, to, kitty_id)?;

            Ok(())
        }

        fn buy_kitty(origin, kitty_id: T::Hash, max_price: T::Balance) -> Result {
            let sender = ensure_signed(origin)?;

            // ACTION: Check the kitty `exists()`
            ensure!(<Kitties<T>>::exists(kitty_id), "This cat does not exist");

            // ACTION: Get the `owner` of the kitty if it exists, otherwise return an `Err()`
            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            // ACTION: Check that the `sender` is not the `owner`
            ensure!(owner != sender, "Cat already owned");

            let mut kitty = Self::kitty(kitty_id);
            let price = kitty.price;
            // ACTION: Get the `kitty_price` and check that it is not zero
            //   HINT:  `runtime_primitives::traits::Zero` allows you to call `kitty_price.is_zero()` which returns a bool
            ensure!(!price.is_zero(), "The cat you want to buy is not for sale");
            ensure!(price <= max_price, "The cat you want to buy costs more than your max price");

            // ACTION: Check `kitty_price` is less than or equal to max_price
            ensure!(price <= max_price, "Kitty price is above the max price submitted");

            // ACTION: Use the `Balances` module's `Currency` trait and `transfer()` function to safely transfer funds
            <balances::Module<T> as Currency<_>>::transfer(&sender, &owner, price)?;

            // ACTION: Transfer the kitty using `tranfer_from()` including a proof of why it cannot fail
            Self::transfer_from(owner.clone(), sender.clone(), kitty_id)
                .expect("`owner` is shown to own the kitty; \
                `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
                `all_kitty_count` shares the same type as `owned_kitty_count` \
                and minting ensure there won't ever be more than `max()` kitties, \
                which means transfer cannot cause an overflow; \
                qed");

            // ACTION: Reset kitty price back to zero, and update the storage
            kitty.price = <T::Balance as As<u64>>::sa(0);
            <Kitties<T>>::insert(kitty_id, kitty);
            // ACTION: Create an event for the cat being bought with relevant details
            //         - new owner
            //         - old owner
            //         - the kitty id
            //         - the price sold for
            Self::deposit_event(RawEvent::Bought(sender, owner, kitty_id, price));

            Ok(())
        }

        fn breed_kitty(origin, kitty_id_1: T::Hash, kitty_id_2: T::Hash) -> Result {
            let sender = ensure_signed(origin)?;

            // ACTION: Check both kitty 1 and kitty 2 "exists"
            ensure!(<Kitties<T>>::exists(kitty_id_1), "Kitty 1 does not exist");
            ensure!(<Kitties<T>>::exists(kitty_id_2), "Kitty 2 does not exist");

            // ACTION: Generate a `random_hash` using the <Nonce<T>>
            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            let kitty_1 = Self::kitty(kitty_id_1);
            let kitty_2 = Self::kitty(kitty_id_2);

            // NOTE: Our gene splicing algorithm, feel free to make it your own
            let mut final_dna = kitty_1.dna;
            for (i, (dna_2_element, r)) in kitty_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
                if r % 2 == 0 {
                    final_dna.as_mut()[i] = *dna_2_element;
                }
            }

            // ACTION: Create a `new_kitty` using:
            //         - `random_hash` as `id`
            //         - `final_dna` as `dna`
            //         - 0 as `price`
            //         - the max of the parent's `gen` + 1
            //   HINT: `rstd::cmp::max(1, 5) + 1` is `6`

            let new_kitty = Kitty {
                id: random_hash,
                dna: final_dna,
                price: <T::Balance as As<u64>>::sa(0),
                gen: rstd::cmp::max(kitty_1.gen, kitty_2.gen) + 1,
            };

            // ACTION: `mint()` your new kitty
            Self::mint(sender, random_hash, new_kitty)?;

            // ACTION: Update the <Nonce<T>>
            <Nonce<T>>::mutate(|n| *n += 1);

            Ok(())
        }
    }
}


impl<T: Trait> Module<T> {
    fn mint(to: T::AccountId, kitty_id: T::Hash, new_kitty: Kitty<T::Hash, T::Balance>) -> Result {

        // ACTION: Generate variables `owned_kitty_count` and `new_owned_kitty_count`
        //         similar to `all_kitties_count` below
        let owned_kitty_count = Self::owned_kitty_count(&to);
        let new_owned_kitty_count = owned_kitty_count.checked_add(1).ok_or("Overflow adding a new kitty")?;

        // ACTION: Get the current `AllKittiesCount` value and store it in `all_kitties_count`
        // ACTION: Create a `new_all_kitties_count` by doing a `checked_add()` to increment `all_kitties_count`
        //      REMINDER: Return an `Err()` if there is an overflow
        let all_kitties_count = Self::num_of_kitties();
        let new_all_kitties_count = all_kitties_count.checked_add(1).ok_or("Overflow adding a new kitty")?;

        ensure!(!<KittyOwner<T>>::exists(kitty_id), "Kitty already exists");

        <Kitties<T>>::insert(kitty_id, new_kitty);
        <KittyOwner<T>>::insert(kitty_id, &to);

        // ACTION: Update the storage for the global kitty tracking
        //         - `AllKittiesArray` should use the `all_kitties_count` (remember `index` is `count - 1`)
        //         - `AllKittiesCount` should use `new_all_kitties_count`
        //         - `AllKittiesIndex` should use `all_kitties_count`
        <AllKittiesArray<T>>::insert(all_kitties_count, kitty_id);
        <AllKittiesCount<T>>::put(new_all_kitties_count);
        <AllKittiesIndex<T>>::insert(kitty_id, all_kitties_count);

        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count);

        // Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }

    fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> Result {
        // ACTION: Check if owner exists for `kitty_id`
        //         - If it does, sanity check that `from` is the `owner`
        //         - If it doesn't, return an `Err()` that no `owner` exists

        let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
        ensure!(owner == from, "From account is not the owner");

        let owned_kitty_count_from = Self::owned_kitty_count(&from);
        let owned_kitty_count_to = Self::owned_kitty_count(&to);

        // ACTION: Used `checked_add()` to increment the `owned_kitty_count_to` by one into `new_owned_kitty_count_to`
        // ACTION: Used `checked_sub()` to decrement the `owned_kitty_count_from` by one into `new_owned_kitty_count_from`
        //         - Return an `Err()` if overflow or underflow

        let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1).ok_or("Overflow adding a new kitty to account balance")?;
        let new_owned_kitty_count_from = owned_kitty_count_from.checked_sub(1).ok_or("Overflow subtracing a new kitty to account balance")?;

        // NOTE: This is the "swap and pop" algorithm we have added for you
        //       We use our storage items to help simplify the removal of elements from the OwnedKittiesArray
        //       We switch the last element of OwnedKittiesArray with the element we want to remove
        let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);
        if kitty_index != new_owned_kitty_count_from {
            let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
            <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
            <OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
        }
        // Now we can remove this item by removing the last element

        // ACTION: Update KittyOwner for `kitty_id`
        <KittyOwner<T>>::insert(kitty_id, &to);
        // ACTION: Update OwnedKittiesIndex for `kitty_id`
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

        // ACTION: Update OwnedKittiesArray to remove the element from `from`, and add an element to `to`
        //   HINT: The last element in OwnedKittiesArray(from) is `new_owned_kitty_count_from`
        //              The last element in OwnedKittiesArray(to) is `owned_kitty_count_to`

        <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

        // ACTION: Update the OwnedKittiesCount for `from` and `to`
        <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);
        // ACTION: Deposit a `Transferred` event with the relevant data:
        //         - from
        //         - to
        //         - kitty_id

        Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ACTION: Import test module dependencies here
    use support::{impl_outer_origin, assert_ok, assert_noop};
    use runtime_io::{with_externalities, TestExternalities};
    use primitives::{H256, Blake2Hasher};
    use runtime_primitives::{
        BuildStorage,
        traits::{BlakeTwo256, IdentityLookup},
        testing::{Digest, DigestItem, Header}
    };

    impl_outer_origin! {
        pub enum Origin for KittiesTest {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct KittiesTest;

    impl system::Trait for KittiesTest {
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

    impl balances::Trait for KittiesTest {
        // ACTION: Implement traits for balances module
        type Balance = u64;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type TransferPayment = ();
        type DustRemoval = ();
    }

    impl super::Trait for KittiesTest {
        // ACTION: Implement traits for your own module
        type Event = ();
    }

    // ACTION: Build a genesis storage key/value store
    type Kitties = super::Module<KittiesTest>;

    fn build_ext() -> TestExternalities<Blake2Hasher> {
        let mut t = system::GenesisConfig::<KittiesTest>::default().build_storage().unwrap().0;
        t.extend(balances::GenesisConfig::<KittiesTest>::default().build_storage().unwrap().0);
        t.extend(GenesisConfig::<KittiesTest> {
            kitties: vec![  (0, H256::random(), 50),
                            (1, H256::zero(), 100)],
        }.build_storage().unwrap().0);

        t.into()
    }

    #[test]
    fn create_kitty_should_work() {
        // ACTION: test that create kitty works
        with_externalities(&mut build_ext(), || {
            // create a kitty with account #10.
            assert_ok!(Kitties::create_kitty(Origin::signed(10)));

            // check that there is now 1 kitty in storage
            assert_eq!(Kitties::all_kitties_count(), 1);

            // check that account #10 owns 1 kitty
            assert_eq!(Kitties::owned_kitty_count(10), 1);

            // check that some random account #5 does not own a kitty
            assert_eq!(Kitties::owned_kitty_count(5), 0);

            // check that this kitty is specifically owned by account #10
            let hash = Kitties::kitty_by_index(0);
            assert_eq!(Kitties::owner_of(hash), Some(10));

            let other_hash = Kitties::kitty_of_owner_by_index((10, 0));
            assert_eq!(hash, other_hash);
        })
    }

    #[test]
    fn transfer_kitty_should_work() {
        // ACTION: test that transfer kitty works
        with_externalities(&mut build_ext(), || {
            // check that 10 own a kitty
            assert_ok!(Kitties::create_kitty(Origin::signed(10)));

            assert_eq!(Kitties::owned_kitty_count(10), 1);
            let hash = Kitties::kitty_of_owner_by_index((10, 0));

            // send kitty to 1.
            assert_ok!(Kitties::transfer(Origin::signed(10), 1, hash));

            // 10 now has nothing
            assert_eq!(Kitties::owned_kitty_count(10), 0);
            // but 1 does
            assert_eq!(Kitties::owned_kitty_count(1), 1);
            let new_hash = Kitties::kitty_of_owner_by_index((1, 0));
            // and it has the same hash
            assert_eq!(hash, new_hash);
        })
    }

    #[test]
    fn transfer_not_owned_kitty_should_fail() {
        // ACTION: test that transfering owned kitty correctly fails
        with_externalities(&mut build_ext(), || {
            // check that 10 own a kitty
            assert_ok!(Kitties::create_kitty(Origin::signed(10)));
            let hash = Kitties::kitty_of_owner_by_index((10, 0));

            // account 0 cannot transfer a kitty with this hash.
            assert_noop!(Kitties::transfer(Origin::signed(9), 1, hash), "You do not own this kitty");
        })
    }
}