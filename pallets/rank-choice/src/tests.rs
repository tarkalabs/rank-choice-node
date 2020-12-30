use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let next_poll_id = RankChoiceModule::next_poll_id();
		let content = br#"{"description":"test poll", "items": ["item1", "item2", "item3", "item4"]}"#.to_vec();
		assert_ok!(RankChoiceModule::new_poll(Origin::signed(1), 4, content)); 
		let poll = RankChoiceModule::poll_by_id(next_poll_id);
		assert!(!&poll.is_none());
		assert_eq!(poll.unwrap().num_items, 4);
	});
}