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
        <T as system::Trait>::Hash
    {
        // ACTION: Add a `Created` event which includes an `AccountId` and a `Hash`
        Created(AccountId, Hash),
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
        OwnedKittiesCount get(num_owned_kitties): map T::AccountId => u64;
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

            // ACTION: Generate variables `owned_kitty_count` and `new_owned_kitty_count`
            //         similar to `all_kitties_count` below
            let owned_kitty_count = Self::num_owned_kitties(&sender);
            let new_owned_kitty_count = owned_kitty_count.checked_add(1).ok_or("Overflow adding a new kitty")?;

            // ACTION: Get the current `AllKittiesCount` value and store it in `all_kitties_count`
            // ACTION: Create a `new_all_kitties_count` by doing a `checked_add()` to increment `all_kitties_count`
            //      REMINDER: Return an `Err()` if there is an overflow
            let all_kitties_count = Self::num_of_kitties();
            let new_all_kitties_count = all_kitties_count.checked_add(1).ok_or("Overflow adding a new kitty")?;

            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            ensure!(!<KittyOwner<T>>::exists(random_hash), "Kitty already exists");

            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: <T::Balance as As<u64>>::sa(0),
                gen: 0,
            };

            <Kitties<T>>::insert(random_hash, new_kitty);
            <KittyOwner<T>>::insert(random_hash, &sender);

            // ACTION: Update the storage for the global kitty tracking
            //         - `AllKittiesArray` should use the `all_kitties_count` (remember `index` is `count - 1`)
            //         - `AllKittiesCount` should use `new_all_kitties_count`
            //         - `AllKittiesIndex` should use `all_kitties_count`
            <AllKittiesArray<T>>::insert(all_kitties_count, random_hash);
            <AllKittiesCount<T>>::put(new_all_kitties_count);
            <AllKittiesIndex<T>>::insert(random_hash, all_kitties_count);

            <OwnedKittiesArray<T>>::insert((sender.clone(), owned_kitty_count), random_hash);
            <OwnedKittiesCount<T>>::insert(&sender, new_owned_kitty_count);
            <OwnedKittiesIndex<T>>::insert(random_hash, owned_kitty_count);

            <Nonce<T>>::mutate(|n| *n += 1);
            Self::deposit_event(RawEvent::Created(sender, random_hash));

            Ok(())
        }
    }
}

