//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as GameModule;
use frame_benchmarking::v2::*;
use frame_support::{
	assert_ok,
	traits::{OnFinalize, OnInitialize},
};
use frame_system::RawOrigin;

fn create_setup<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	let admin: T::AccountId = account("admin", 0, 0);
	assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
	assert_ok!(GameModule::<T>::add_to_admins(RawOrigin::Root.into(), admin.clone()));
	assert_ok!(GameModule::<T>::register_user(RawOrigin::Signed(admin).into(), caller.clone()));
	caller
}

fn practise_round<T: Config>(caller: T::AccountId, game_id: u32) {
	assert_ok!(GameModule::<T>::play_game(
		RawOrigin::Signed(caller.clone()).into(),
		crate::DifficultyLevel::Practice
	));
	assert_ok!(GameModule::<T>::submit_answer(
		RawOrigin::Signed(caller.clone()).into(),
		20,
		game_id
	));
}

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn setup_game() {
		#[extrinsic_call]
		setup_game(RawOrigin::Root);
	}

	#[benchmark]
	fn register_user() {
		let caller: T::AccountId = account("caller", 0, 0);
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let admin: T::AccountId = account("admin", 0, 0);
		assert_ok!(GameModule::<T>::add_to_admins(RawOrigin::Root.into(), admin.clone()));
		#[extrinsic_call]
		register_user(RawOrigin::Signed(admin), caller.clone());

		assert!(GameModule::<T>::users(caller).is_some());
	}

	#[benchmark]
	fn give_points() {
		let caller = create_setup::<T>();
		#[extrinsic_call]
		give_points(RawOrigin::Root, caller.clone());

		assert_eq!(GameModule::<T>::users(caller).unwrap().points, 150);
	}

	#[benchmark]
	fn play_game() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		#[extrinsic_call]
		play_game(RawOrigin::Signed(caller.clone()), crate::DifficultyLevel::Player);

		assert_eq!(GameModule::<T>::game_info(1).unwrap().player, caller);
	}

	#[benchmark]
	fn submit_answer() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		#[extrinsic_call]
		submit_answer(RawOrigin::Signed(caller.clone()), 220000, 1);

		assert_eq!(GameModule::<T>::users::<AccountIdOf<T>>(caller).unwrap().nfts.xorange, 1);
	}

	#[benchmark]
	fn list_nft() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		#[extrinsic_call]
		list_nft(RawOrigin::Signed(caller.clone()), 0.into(), 0.into());

		assert_eq!(GameModule::<T>::listings(0).unwrap().owner, caller);
	}

	#[benchmark]
	fn delist_nft() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::list_nft(
			RawOrigin::Signed(caller.clone()).into(),
			0.into(),
			0.into()
		));
		#[extrinsic_call]
		delist_nft(RawOrigin::Signed(caller), 0);

		assert!(GameModule::<T>::listings(0).is_none());
	}

	#[benchmark]
	fn make_offer() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::list_nft(
			RawOrigin::Signed(caller.clone()).into(),
			0.into(),
			0.into()
		));
		let caller2: T::AccountId = account("caller2", 0, 0);
		let admin: T::AccountId = account("admin", 0, 0);
		assert_ok!(GameModule::<T>::register_user(RawOrigin::Signed(admin).into(), caller2.clone()));
		practise_round::<T>(caller2.clone(), 2);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller2.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller2.clone()).into(),
			220000,
			3
		));
		#[extrinsic_call]
		make_offer(RawOrigin::Signed(caller2.clone()), 0, 0.into(), 1.into());

		assert_eq!(GameModule::<T>::offers(0).unwrap().owner, caller2);
	}

	#[benchmark]
	fn handle_offer() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller.clone()).into(),
			220000,
			1
		));
		assert_ok!(GameModule::<T>::list_nft(
			RawOrigin::Signed(caller.clone()).into(),
			0.into(),
			0.into()
		));
		let caller2: T::AccountId = account("caller2", 0, 0);
		let admin: T::AccountId = account("admin", 0, 0);
		assert_ok!(GameModule::<T>::register_user(RawOrigin::Signed(admin).into(), caller2.clone()));
		practise_round::<T>(caller2.clone(), 2);
		assert_ok!(GameModule::<T>::play_game(
			RawOrigin::Signed(caller2.clone()).into(),
			crate::DifficultyLevel::Player
		));
		assert_ok!(GameModule::<T>::submit_answer(
			RawOrigin::Signed(caller2.clone()).into(),
			220000,
			3
		));
		assert_eq!(
			GameModule::<T>::users::<AccountIdOf<T>>(caller2.clone()).unwrap().nfts.xorange,
			1
		);
		assert_ok!(GameModule::<T>::make_offer(
			RawOrigin::Signed(caller2.clone()).into(),
			0,
			0.into(),
			1.into()
		));

		#[extrinsic_call]
		handle_offer(RawOrigin::Signed(caller), 0, crate::Offer::Accept);

		assert_eq!(GameModule::<T>::offers(0).is_none(), true);
		assert_eq!(GameModule::<T>::listings(0).is_none(), true);
	}

	#[benchmark]
	fn add_property() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let new_property = PropertyInfoData {
			id: 147229391,
			bedrooms: 2,
			bathrooms: 1,
			summary: "Superb 2 double bedroom ground floor purpose-built apartment with sole use of garden. Directly opposite Hackney Downs Park, within walking distance of Clapton, Hackney Downs & Rectory Rd Stations. Benefitting from; 2 double bedrooms, fitted kitchen/diner, modern shower/WC, separate lounge with di...".as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: "Flat".as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: "2024-04-24T16:39:27Z".as_bytes().to_vec().try_into().unwrap(),
			display_size: "".as_bytes().to_vec().try_into().unwrap(),
			display_address: "St Peters Street, Islington".as_bytes().to_vec().try_into().unwrap(),
			property_images1: "https://media.rightmove.co.uk/dir/crop/10:9-16:9/56k/55489/146480642/55489_2291824_IMG_00_0000_max_476x317.jpeg".as_bytes().to_vec().try_into().unwrap(),
			};
		#[extrinsic_call]
		add_property(RawOrigin::Root, new_property, 200000);

		assert_eq!(GameModule::<T>::test_properties().len(), 5);
	}

	#[benchmark]
	fn remove_property() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		#[extrinsic_call]
		remove_property(RawOrigin::Root, 146480642);

		assert_eq!(GameModule::<T>::test_properties().len(), 3);
	}

	#[benchmark]
	fn add_to_admins() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let new_admin: T::AccountId = account("new_admin", 1, 0);
		#[extrinsic_call]
		add_to_admins(RawOrigin::Root, new_admin);
	}

	#[benchmark]
	fn remove_from_admins() {
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		let new_admin: T::AccountId = account("new_admin", 0, 0);
		assert_ok!(GameModule::<T>::add_to_admins(RawOrigin::Root.into(), new_admin.clone()));
		#[extrinsic_call]
		remove_from_admins(RawOrigin::Root, new_admin);
	}

	#[benchmark]
	fn request_token() {
		let caller = create_setup::<T>();
		current_block::<T>(30u32.into());
		current_block::<T>(100801u32.into());
		#[extrinsic_call]
		request_token(RawOrigin::Signed(caller));
	}

	impl_benchmark_test_suite!(GameModule, crate::mock::new_test_ext(), crate::mock::Test);
}

fn current_block<T: Config>(new_block: frame_system::pallet_prelude::BlockNumberFor<T>) {
	while frame_system::Pallet::<T>::block_number() < new_block {
		if frame_system::Pallet::<T>::block_number() > 0u32.into() {
			GameModule::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
			frame_system::Pallet::<T>::on_finalize(frame_system::Pallet::<T>::block_number());
		}
		frame_system::Pallet::<T>::reset_events();
		frame_system::Pallet::<T>::set_block_number(
			frame_system::Pallet::<T>::block_number() + 1u32.into(),
		);
		frame_system::Pallet::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
		GameModule::<T>::on_initialize(frame_system::Pallet::<T>::block_number());
	}
}
