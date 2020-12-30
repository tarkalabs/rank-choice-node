use crate::{Error, mock::*, RawEvent};
use frame_support::{assert_ok, assert_noop};


#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1); // this is important to generate events
		let next_poll_id = RankChoiceModule::next_poll_id();
		let content = br#"{"description":"test poll", "items": ["item1", "item2", "item3", "item4"]}"#.to_vec();
		assert_ok!(RankChoiceModule::new_poll(Origin::signed(1), 4, content)); 
		let poll = RankChoiceModule::poll_by_id(next_poll_id);
		assert!(!&poll.is_none());
		assert_eq!(poll.unwrap().num_items, 4);
		assert_eq!(last_event(), RawEvent::PollCreated(next_poll_id, 1));
	});
}

#[test]
fn it_records_votes_for_a_poll() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1); // this is important to generate events
		let poll_id = RankChoiceModule::next_poll_id();
		let content = br#"{"description":"test poll", "items": ["item1", "item2", "item3", "item4"]}"#.to_vec();
		assert_ok!(RankChoiceModule::new_poll(Origin::signed(1), 4, content)); 
		let votes = vec![4, 1, 2];
		assert_ok!(RankChoiceModule::cast_vote(Origin::signed(2), poll_id, votes.clone()));
		let recorded_votes = RankChoiceModule::votes(poll_id, 2);
		assert!(!&recorded_votes.is_none());
		assert_eq!(recorded_votes.unwrap(), votes);
		assert_eq!(last_event(), RawEvent::NewVoteCast(poll_id, 2));
	});
}

#[test]
fn it_raises_error_for_duplicate_vote() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1); // this is important to generate events
		let poll_id = RankChoiceModule::next_poll_id();
		let content = br#"{"description":"test poll", "items": ["item1", "item2", "item3", "item4"]}"#.to_vec();
		assert_ok!(RankChoiceModule::new_poll(Origin::signed(1), 4, content)); 
		let votes = vec![4, 1, 2];
		assert_ok!(RankChoiceModule::cast_vote(Origin::signed(2), poll_id, votes.clone()));
		let recorded_votes = RankChoiceModule::votes(poll_id, 2);
		assert!(!&recorded_votes.is_none());
		assert_eq!(recorded_votes.unwrap(), votes);
		assert_eq!(last_event(), RawEvent::NewVoteCast(poll_id, 2));
		System::set_block_number(2); // this is important to generate events
		let new_votes = vec![4, 1, 2];
		assert_noop!(
			RankChoiceModule::cast_vote(Origin::signed(2), poll_id, new_votes),
			Error::<Test>::AlreadyVoted
		);
	});
}
