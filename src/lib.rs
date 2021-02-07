#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{decl_module, decl_storage, decl_event, decl_error, dispatch, traits::{Get, Currency, Randomness}};
use frame_system::{ensure_signed};
use codec::{Encode, Decode};
// use node_primitives::{Balance, Hash};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
	id: Hash,
	dna: Hash,
	price: Balance,
	gen: u64,
	number: u32,
}


type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type KittyOf<T> =
    Kitty<<T as frame_system::Trait>::Hash, BalanceOf<T>>;

/// Configure the pallet by specifying the parameters and types on which it depends.

pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Randomness: frame_support::traits::Randomness<Self::Hash>;
    type Currency: frame_support::traits::LockableCurrency<Self::AccountId>;
    type BasePrice: Get<BalanceOf<Self>>;
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as TemplateModule {
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		AllKittiesArray get(fn kitty_by_index): map hasher(blake2_128_concat) u64 => T::Hash;
        AllKittiesCount get(fn all_kitties_count): u64;
        AllKittiesIndex: map hasher(blake2_128_concat) T::Hash => u64;

		OwnedKittiesArray get(fn kitty_of_owner): map hasher(blake2_128_concat) (T::AccountId,u64) => KittyOf<T>;
		OwnedKittiesCount get(fn owned_kitty_count): map hasher(blake2_128_concat) T::AccountId => u64;
        OwnedKittiesIndex: map hasher(blake2_128_concat) T::Hash => u64;


		
		KittyCount: u32;
		Kitties get(fn kitty): map hasher(blake2_128_concat) T::Hash  => KittyOf<T>;
		KittyOwner get(fn owner_of_kitty): map hasher(blake2_128_concat) T::Hash => Option<T::AccountId>;
		Nonce: u8;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
    pub enum Event<T>
    where
        KittyId = <T as frame_system::Trait>::Hash,
        AccountId = <T as frame_system::Trait>::AccountId,
    {
        Created(AccountId, KittyId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        CreateFailure,
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		
		fn deposit_event() = default;
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn create_kitty(origin) -> dispatch::DispatchResult {
            // ACTION: "Ensure" that the transaction is signed
			let sender = ensure_signed(origin)?;

			let balance =  T::BasePrice::get();
			let nonce = Nonce::get();
			let random_hash = T::Randomness::random(&[nonce]);
			let n = KittyCount::get();
			//let _hash_of_random = frame_support::traits::LockableCurrency<Self::AccountId>;
			// ensure!(!<KittyOwner<T>>::exists(&random_hash), "Kitty already exists");
			// //let my_zero_balance = <T::Balance as As<u64>>::sa(0);
			// let balance = frame_support::traits::LockableCurrency<Self::AccountId>;
			let new_kitty = Kitty {
			 	id: random_hash,
			 	dna: random_hash,
			 	price: balance,
			 	gen: 0,
				number: n,
			}; 
            // // ACTION: "Put" the value into storage
			// <OwnedKittiesArray<T>>::insert((&sender, owned_kitty_count), &_kitty);
			// <OwnedKittiesCount<T>>::insert(&sender, new_owned_kitty_count);
			// <OwnedKittiesIndex<T>>::insert(&random_hash, owned_kitty_count);
			
			// <KittyOwner<T>>::insert(&random_hash, &sender);
            // <Kitties<T>>::insert(&random_hash, _kitty);
			
			
            // <AllKittiesArray<T>>::insert(all_kitties_count, random_hash);
            // <AllKittiesCount>::put(new_all_kitties_count);
            // <AllKittiesIndex<T>>::insert(random_hash, all_kitties_count);

			Self::mint(sender, random_hash, new_kitty)?;

            KittyCount::mutate(|n| *n += 1);
			Nonce::mutate(|n| *n += 1);

            Ok(())
        }

	}




}

impl<T: Trait> Module<T> {
    fn mint(to: T::AccountId, kitty_id: T::Hash, new_kitty: KittyOf<T>) -> dispatch::DispatchResult {
        //ensure!(!<KittyOwner<T>>::exists(kitty_id), "Kitty already exists");

		let all_kitties_count = Self::all_kitties_count();

			
        let new_all_kitties_count = all_kitties_count.checked_add(1)
                .ok_or("Overflow adding a new kitty to total supply")?;

		let owned_kitty_count = Self::owned_kitty_count(&to);
		
		let new_owned_kitty_count = owned_kitty_count.checked_add(1)
                .ok_or("Overflow adding a new kitty to total supply")?;
			


        <Kitties<T>>::insert(kitty_id, &new_kitty);
        <KittyOwner<T>>::insert(kitty_id, &to);

        <AllKittiesArray<T>>::insert(all_kitties_count, &kitty_id);
        <AllKittiesCount>::put(&new_all_kitties_count);
        <AllKittiesIndex<T>>::insert(&kitty_id, all_kitties_count);

        <OwnedKittiesArray<T>>::insert((&to, &owned_kitty_count), new_kitty);
        <OwnedKittiesCount<T>>::insert(&to, &new_owned_kitty_count);
        <OwnedKittiesIndex<T>>::insert(&kitty_id, owned_kitty_count);

        Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }
}