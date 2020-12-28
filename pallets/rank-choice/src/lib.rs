#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// https://substrate.dev/docs/en/knowledgebase/runtime/frame

use frame_support::{
		decl_module, decl_storage, 
		decl_event, decl_error, dispatch, ensure,
		traits::Get, sp_std::vec::Vec};
use frame_support::codec::{Encode, Decode};
use frame_system::ensure_signed;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;


/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: frame_system::Trait {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

pub type PollId = u64;
pub type Choices = Vec<u8>;

/// Store poll specific data
#[derive(Encode, Decode, Clone, PartialEq, PartialOrd)]
pub struct Poll<T: Trait> {
	/// The ID of the poll
	pub poll_id: PollId,
	/// The proposer who creates the poll
	pub proposer: T::AccountId,
	/// number of items in the poll
	pub num_items: u8,
	/// Frontend encoded content. A description and JSON array of items
	pub content: Vec<u8>,
	pub active: bool
}


impl <T: Trait> Poll<T> {
	pub fn new(poll_id: PollId, proposer: T::AccountId, num_items: u8, content: Vec<u8>) -> Self {
		Poll {poll_id, proposer, num_items, content, active: true}
	}
}
// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
	// A unique name is used to ensure that the pallet's storage items are isolated.
	// This name may be updated, but each pallet in the runtime must use a unique name.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as RankChoiceModule {
		pub NextPollId get(fn next_poll_id): PollId = 1;
		pub PollById get(fn poll_by_id): map hasher(twox_64_concat) PollId => Option<Poll<T>>;
		pub Votes get(fn votes): double_map hasher(twox_64_concat) PollId, hasher(blake2_128_concat) T::AccountId => Option<Choices>;
		// Learn more about declaring storage items:
		// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
		Something get(fn something): Option<u32>;
	}
}

// Pallets use events to inform users when important changes are made.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, AccountId),
		PollCreated(AccountId, PollId),
		NewVoteCast(PollId, AccountId),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// There was no poll with a given id
		NoSuchPoll,
		PollNotActive,
		AlreadyVoted
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		#[weight = 10_000 + T::DbWeight::get().writes(2)]
		pub fn new_poll(origin, num_items: u8, content: Vec<u8>) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let poll_id = Self::next_poll_id();
			let new_poll: Poll<T> = Poll::new(
				poll_id, 
				who.clone(), 
				num_items, 
				content
			);
			PollById::insert(poll_id, new_poll);
			NextPollId::mutate( |pid| { *pid += 1; });
			Self::deposit_event(RawEvent::PollCreated(who, poll_id));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().writes(2)]
		pub fn cast_vote(origin, poll_id: PollId, votes: Choices) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let poll: Poll<T> = PollById::get(poll_id).ok_or(Error::<T>::NoSuchPoll)?;
			ensure!(&poll.active, Error::<T>::PollNotActive);
			ensure!(!Votes::<T>::contains_key(poll_id, who.clone()), Error::<T>::AlreadyVoted);
			Votes::<T>::insert(poll_id, who.clone(), votes);
			Self::deposit_event(RawEvent::NewVoteCast(poll_id, who));
			Ok(())

		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = 10_000 + T::DbWeight::get().writes(1)]
		pub fn do_something(origin, something: u32) -> dispatch::DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			Something::put(something);

			// Emit an event.
			Self::deposit_event(RawEvent::SomethingStored(something, who));
			// Return a successful DispatchResult
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
		pub fn cause_error(origin) -> dispatch::DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match Something::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					Something::put(new);
					Ok(())
				},
			}
		}
	}
}
