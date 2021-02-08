#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{ensure, decl_module, decl_storage, decl_event, decl_error, dispatch, traits::{Get, Currency, Randomness, ExistenceRequirement::KeepAlive}};
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

		OwnedKittiesArray get(fn kitty_of_owner): map hasher(blake2_128_concat) (T::AccountId,u64) => T::Hash;
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
		Balance =  <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance, 

    {
        Created(AccountId, KittyId),
		PriceSet(AccountId, KittyId, Balance),
		Transferred(AccountId,AccountId,KittyId),
		Bought(AccountId,AccountId,KittyId,Balance),	
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

			let new_kitty = Kitty {
			 	id: random_hash,
			 	dna: random_hash,
			 	price: balance,
			 	gen: 0,
				number: n,
			}; 

			Self::mint(sender, random_hash, new_kitty)?;

            KittyCount::mutate(|n| *n += 1);
			Nonce::mutate(|n| *n += 1);

            Ok(())
        }


		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn set_price(origin, kitty_id: T::Hash, new_price: BalanceOf<T>) -> dispatch::DispatchResult {
           let sender = ensure_signed(origin)?;
           // ACTION: Check that the kitty with `kitty_id` exists
		   ensure!(<Kitties<T>>::contains_key(kitty_id), "Kitty doesn't exist");

		   let owner = Self::owner_of_kitty(kitty_id).ok_or("No owner for this kitty")?;
 
			ensure!(owner == sender, "You do not own this cat");

            let mut kitty = Self::kitty(kitty_id);
            
			kitty.price = new_price;

			<Kitties<T>>::insert(kitty_id, kitty);

			Self::deposit_event(RawEvent::PriceSet(owner, kitty_id, new_price));

            Ok(())
        }

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn transfer(origin, to: T::AccountId, kitty_id: T::Hash) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of_kitty(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty");

            Self::transfer_from(sender, to, kitty_id)?;

            Ok(())
        }

		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		fn buy_kitty(origin, kitty_id: T::Hash, max_price: BalanceOf<T>) -> dispatch::DispatchResult {
            let sender = ensure_signed(origin)?;

            // ACTION: Check the kitty `exists()`
			ensure!(<Kitties<T>>::contains_key(kitty_id), "Kitty doesn't exist");

            // ACTION: Get the `owner` of the kitty if it exists, otherwise return an `Err()`
			let owner = Self::owner_of_kitty(kitty_id).ok_or("No owner for this kitty")?;
 
			ensure!(owner != sender, "You already own this cat");

            let mut kitty = Self::kitty(kitty_id);

            // ACTION: Get the `kitty_price` and check that it is not zero
			let kitty_price = kitty.price;
            //ensure!(!kitty_price.is_zero(), "The cat you want to buy is not for sale");
            ensure!(kitty_price <= max_price, "The cat you want to buy costs more than your max price");


            <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::transfer(&sender, &owner, kitty_price, KeepAlive)?;

            Self::transfer_from(owner.clone(), sender.clone(), kitty_id)
                .expect("`owner` is shown to own the kitty; \
                `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
                `all_kitty_count` shares the same type as `owned_kitty_count` \
                and minting ensure there won't ever be more than `max()` kitties, \
                which means transfer cannot cause an overflow; \
                qed");

            kitty.price = T::BasePrice::get();
            <Kitties<T>>::insert(kitty_id, kitty);

            Self::deposit_event(RawEvent::Bought(sender, owner, kitty_id, kitty_price));

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
        <KittyOwner<T>>::insert(&kitty_id, &to);

        <AllKittiesArray<T>>::insert(all_kitties_count, &kitty_id);
        <AllKittiesCount>::put(&new_all_kitties_count);
        <AllKittiesIndex<T>>::insert(&kitty_id, all_kitties_count);

        <OwnedKittiesArray<T>>::insert((&to, &owned_kitty_count), &kitty_id);
        <OwnedKittiesCount<T>>::insert(&to, &new_owned_kitty_count);
        <OwnedKittiesIndex<T>>::insert(&kitty_id, owned_kitty_count);

        Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }

	fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: T::Hash) -> dispatch::DispatchResult {
        // ACTION: Check if owner exists for `kitty_id`
        //         - If it does, sanity check that `from` is the `owner`
        //         - If it doesn't, return an `Err()` that no `owner` exists

		
		ensure!(<Kitties<T>>::contains_key(kitty_id), "Kitty doesn't exist");

		let owner = Self::owner_of_kitty(kitty_id).ok_or("No owner for this kitty")?;
		ensure!(owner == from, "You do not own this cat");


        let owned_kitty_count_from = Self::owned_kitty_count(&from);
        let owned_kitty_count_to = Self::owned_kitty_count(&to);

        // ACTION: Used `checked_add()` to increment the `owned_kitty_count_to` by one into `new_owned_kitty_count_to`
        // ACTION: Used `checked_sub()` to decrement the `owned_kitty_count_from` by one into `new_owned_kitty_count_from`
        //         - Return an `Err()` if overflow or underflow

		let new_owned_kitty_count_from = owned_kitty_count_from.checked_sub(1).ok_or("overflow subtract")?;
		let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1).ok_or("overflow add")?;

        // NOTE: This is the "swap and pop" algorithm we have added for you
        //       We use our storage items to help simplify the removal of elements from the OwnedKittiesArray
        //       We switch the last element of OwnedKittiesArray with the element we want to remove

		let kitty_index = <OwnedKittiesIndex<T>>::get(kitty_id);
        if kitty_index != new_owned_kitty_count_from {
            let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
            <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
            <OwnedKittiesIndex<T>>::insert(last_kitty_id, kitty_index);
        }

		<KittyOwner<T>>::insert(&kitty_id, &to);
        <OwnedKittiesIndex<T>>::insert(kitty_id, owned_kitty_count_to);

        <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

        <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);
        
        Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));
        
        Ok(())
    }
}