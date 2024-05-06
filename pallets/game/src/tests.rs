use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{traits::BadOrigin, DispatchError, ModuleError};

fn practise_round(player: AccountId, game_id: u32) {
	assert_ok!(GameModule::play_game(
		RuntimeOrigin::signed(player.clone()),
		crate::DifficultyLevel::Practice,
	));
	assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed(player), 220000, game_id));
}

#[test]
fn setup_game_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
	});
}

#[test]
fn setup_game_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(GameModule::setup_game(RuntimeOrigin::signed([0; 32].into())), BadOrigin);
	});
}

#[test]
fn play_game_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_eq!(GameModule::game_info(1).unwrap().property.id, 1);
	});
}

#[test]
fn play_game_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_noop!(
			GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Player,
			),
			Error::<Test>::UserNotRegistered
		);
	});
}

#[test]
fn play_game_fails_no_active_round() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_noop!(
			GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Practice,
			),
			Error::<Test>::NoActiveRound
		);
	});
}

#[test]
fn play_game_fails_not_enough_points() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 10, 1));
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
	});
}

#[test]
fn submit_answer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 223_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 80);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 2));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 180);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 1);
	});
}

#[test]
fn submit_answer_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_noop!(
			GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 223_000, 0),
			Error::<Test>::NoActiveGame
		);
	});
}

#[test]
fn transfer_of_nft_does_not_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				0,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn list_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
	});
}

#[test]
fn list_nft_doesnt_work() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(1).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_noop!(
			GameModule::list_nft(RuntimeOrigin::signed([1; 32].into()), 0, 0,),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn delist_nft_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::delist_nft(RuntimeOrigin::signed([0; 32].into()), 0,));
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				0,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn delist_nft_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_noop!(
			GameModule::delist_nft(RuntimeOrigin::signed([1; 32].into()), 0,),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn make_offer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [1; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
	});
}

#[test]
fn make_offer_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::give_points(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_noop!(
			GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 0,),
			Error::<Test>::ListingDoesNotExist
		);
	});
}

#[test]
fn withdraw_offer_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [1; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_ok!(GameModule::withdraw_offer(RuntimeOrigin::signed([1; 32].into()), 0));
		assert_eq!(GameModule::offers(0).is_none(), true);
	});
}

#[test]
fn withdraw_offer_fails() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [1; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_noop!(
			GameModule::withdraw_offer(RuntimeOrigin::signed([0; 32].into()), 0),
			Error::<Test>::OfferDoesNotExist
		);
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_noop!(
			GameModule::withdraw_offer(RuntimeOrigin::signed([0; 32].into()), 0),
			Error::<Test>::NoPermission
		);
	});
}

#[test]
fn handle_offer_accept_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [1; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 4));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 275);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 5));
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 3);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 495);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().nfts.xorange, 1);
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_ok!(GameModule::handle_offer(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			crate::Offer::Accept,
		));
		assert_eq!(Nfts::owner(0, 0).unwrap(), [1; 32].into());
		assert_eq!(Nfts::owner(0, 1).unwrap(), [0; 32].into());
		assert_eq!(GameModule::offers(0).is_none(), true);
		assert_eq!(GameModule::listings(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().nfts.xorange, 3);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().nfts.xorange, 1);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 495);
		assert_eq!(GameModule::users::<AccountId>([1; 32].into()).unwrap().points, 155);
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				1,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([1; 32].into()),
				0,
				0,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn handle_offer_reject_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [1; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_ok!(GameModule::make_offer(RuntimeOrigin::signed([1; 32].into()), 0, 0, 1,));
		assert_eq!(GameModule::offers(0).unwrap().owner, [1; 32].into());
		assert_ok!(GameModule::handle_offer(
			RuntimeOrigin::signed([0; 32].into()),
			0,
			crate::Offer::Reject,
		));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_eq!(GameModule::offers(0).is_none(), true);
		assert_eq!(GameModule::listings(0).is_some(), true);
		assert_noop!(
			Nfts::transfer(
				RuntimeOrigin::signed([1; 32].into()),
				0,
				1,
				sp_runtime::MultiAddress::Id([1; 32].into())
			),
			DispatchError::Module(ModuleError {
				index: 3,
				error: [12, 0, 0, 0],
				message: Some("ItemLocked")
			})
		);
	});
}

#[test]
fn handle_offer_doesnt_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [1; 32].into()));
		practise_round([0; 32].into(), 0);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([0; 32].into()), 220_000, 1));
		assert_eq!(GameModule::game_info(0).is_none(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 155);
		assert_eq!(Nfts::owner(0, 0).unwrap(), [0; 32].into());
		practise_round([1; 32].into(), 2);
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([1; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_ok!(GameModule::submit_answer(RuntimeOrigin::signed([1; 32].into()), 220_000, 3));
		assert_eq!(Nfts::owner(0, 1).unwrap(), [1; 32].into());
		assert_ok!(GameModule::list_nft(RuntimeOrigin::signed([0; 32].into()), 0, 0,));
		assert_eq!(Nfts::owner(0, 0).unwrap(), GameModule::account_id());
		assert_eq!(GameModule::listings(0).unwrap().owner, [0; 32].into());
		assert_noop!(
			GameModule::handle_offer(
				RuntimeOrigin::signed([0; 32].into()),
				0,
				crate::Offer::Reject,
			),
			Error::<Test>::OfferDoesNotExist
		);
	});
}

#[test]
fn play_multiple_rounds_works() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(GameModule::setup_game(RuntimeOrigin::root()));
		assert_eq!(GameModule::test_properties().len(), 4);
		assert_ok!(GameModule::register_user(RuntimeOrigin::root(), [0; 32].into()));
		practise_round([0; 32].into(), 0);
		for x in 1..=20 {
			assert_ok!(GameModule::play_game(
				RuntimeOrigin::signed([0; 32].into()),
				crate::DifficultyLevel::Player,
			));
			assert_ok!(GameModule::submit_answer(
				RuntimeOrigin::signed([0; 32].into()),
				217_000,
				x
			));
		}
		assert_ok!(GameModule::play_game(
			RuntimeOrigin::signed([0; 32].into()),
			crate::DifficultyLevel::Player,
		));
		assert_eq!(GameModule::game_info(21).is_some(), true);
		assert_eq!(GameModule::users::<AccountId>([0; 32].into()).unwrap().points, 555);
	});
}
