use parity_codec::{Encode, Decode};
use support::{decl_storage, decl_module, decl_event, ensure, StorageMap, StorageValue, dispatch::Result};
use system::ensure_signed;
use runtime_primitives::traits::{As, Hash};

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

        // ACTION: Rename this to `OwnedKittiesArray`/`kitty_of_owner_by_index`
        //         Have the key be a tuple of (T::AccountId, u64)
        OwnedKittiesArray get(kitty_of_owner_by_index): map (T::AccountId, u64) => T::Hash;

        // ACTION: Add a new storage item `OwnedKittiesCount` which is a `map` from `T::AccountId` to `u64`
        // ACTION: Add a new storage item `OwnedKittiesIndex` which is a `map` from `T::Hash` to `u64`
        OwnedKittiesCount get(owned_kitty_count): map T::AccountId => u64;
        OwnedKittiesIndex get(owned_kitties_index): map T::Hash => u64;

        Nonce: u64;
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

        Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }

    fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> Result {
        // ACTION: Check if owner exists for `kitty_id`
        //         - If it does, sanity check that `from` is the `owner`
        //         - If it doesn't, return an `Err()` that no `owner` exists

        // ensure!(<Kitties<T>>::exists(kitty_id), "This cat does not exist");
        // let mut kitty = Self::kitty(kitty_id);

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