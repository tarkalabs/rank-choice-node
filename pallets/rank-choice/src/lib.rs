#![cfg_attr(not(feature = "std"), no_std)]

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
	pub fn finalize(&mut self) {
		self.active = false;
	}
}
decl_storage! {
	trait Store for Module<T: Trait> as RankChoiceModule {
		pub NextPollId get(fn next_poll_id): PollId = 1;
		pub PollById get(fn poll_by_id): map hasher(twox_64_concat) PollId => Option<Poll<T>>;
		pub Votes get(fn votes): double_map hasher(twox_64_concat) PollId, hasher(blake2_128_concat) T::AccountId => Option<Choices>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		PollCreated(AccountId, PollId),
		PollFinalized(PollId),
		NewVoteCast(PollId, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoSuchPoll,
		PollNotActive,
		AlreadyVoted,
		NotAuthorized,
		PollAlreadyFinalized
	}
}

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
		pub fn finalize_poll(origin, poll_id: PollId) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let mut poll: Poll<T> = PollById::get(poll_id).ok_or(Error::<T>::NoSuchPoll)?;
			ensure!(poll.proposer == who, Error::<T>::NotAuthorized);
			ensure!(&poll.active, Error::<T>::PollAlreadyFinalized);
			poll.finalize();
			PollById::insert(poll_id, poll);
			Self::deposit_event(RawEvent::PollFinalized(poll_id));
			Ok(())
		}

		#[weight = 10_000 + T::DbWeight::get().reads_writes(2, 1)]
		pub fn cast_vote(origin, poll_id: PollId, votes: Choices) -> dispatch::DispatchResult {
			let who = ensure_signed(origin)?;
			let poll: Poll<T> = PollById::get(poll_id).ok_or(Error::<T>::NoSuchPoll)?;
			ensure!(&poll.active, Error::<T>::PollNotActive);
			ensure!(!Votes::<T>::contains_key(poll_id, who.clone()), Error::<T>::AlreadyVoted);
			Votes::<T>::insert(poll_id, who.clone(), votes);
			Self::deposit_event(RawEvent::NewVoteCast(poll_id, who));
			Ok(())

		}


	}
}