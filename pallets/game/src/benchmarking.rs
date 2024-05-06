//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as GameModule;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use frame_support::{assert_ok, traits::{OnInitialize, OnFinalize}};

fn create_setup<T: Config>() -> T::AccountId {
	let caller: T::AccountId = whitelisted_caller();
	assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
	assert_ok!(GameModule::<T>::register_user(RawOrigin::Root.into(), caller.clone()));
	caller
}

fn practise_round<T: Config>(caller: T::AccountId, game_id: u32) {
	assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Practice));
	assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller.clone()).into(), 20, game_id));
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
		let caller: T::AccountId = whitelisted_caller();
		assert_ok!(GameModule::<T>::setup_game(RawOrigin::Root.into()));
		#[extrinsic_call]
		register_user(RawOrigin::Root, caller);
	}

	#[benchmark]
	fn give_points() {
		let caller = create_setup::<T>();
		#[extrinsic_call]
		give_points(RawOrigin::Root, caller);
	}

 	#[benchmark]
	fn play_game() {
		let caller =
			create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		#[extrinsic_call]
		play_game(
			RawOrigin::Signed(caller),
			crate::DifficultyLevel::Player
		);
	} 

  	#[benchmark]
	fn submit_answer() {
		let caller =
			create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Player));
		#[extrinsic_call]
 		submit_answer(
			RawOrigin::Signed(caller),
			200000,
			1,
		); 
	} 

	#[benchmark]
	fn list_nft() {
		let caller =
			create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Player));
		assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller.clone()).into(), 220000, 1));
		#[extrinsic_call]
		list_nft(
			RawOrigin::Signed(caller),
			0.into(),
			0.into(),
		); 
	} 

	#[benchmark]
	fn delist_nft() {
		let caller =
			create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Player));
		assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller.clone()).into(), 220000, 1));
		assert_ok!(GameModule::<T>::list_nft(RawOrigin::Signed(caller.clone()).into(), 0.into(), 0.into()));
		#[extrinsic_call]
		delist_nft(
			RawOrigin::Signed(caller),
			0,
		); 
	}  

	#[benchmark]
	fn make_offer() {
		let caller =
			create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Player));
		assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller.clone()).into(), 220000, 1));
		assert_ok!(GameModule::<T>::list_nft(RawOrigin::Signed(caller.clone()).into(), 0.into(), 0.into()));
		let caller2 =
			create_setup::<T>();
		practise_round::<T>(caller2.clone(), 2);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Player));
		assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller.clone()).into(), 220000, 3));
		#[extrinsic_call]
		make_offer(
			RawOrigin::Signed(caller2),
			0,
			0.into(),
			1.into(),
		); 
	}  

	#[benchmark]
	fn handle_offer() {
		let caller =
			create_setup::<T>();
		current_block::<T>(30u32.into());
		practise_round::<T>(caller.clone(), 0);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller.clone()).into(), crate::DifficultyLevel::Player));
		assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller.clone()).into(), 220000, 1));
		assert_ok!(GameModule::<T>::list_nft(RawOrigin::Signed(caller.clone()).into(), 0.into(), 0.into()));
		let caller2 =
			create_setup::<T>();
		practise_round::<T>(caller2.clone(), 2);
		assert_ok!(GameModule::<T>::play_game(RawOrigin::Signed(caller2.clone()).into(), crate::DifficultyLevel::Player));
		assert_ok!(GameModule::<T>::submit_answer(RawOrigin::Signed(caller2.clone()).into(), 220000, 3));
		assert_eq!(GameModule::<T>::users::<AccountIdOf<T>>(caller2.clone()).unwrap().nfts.xorange, 1);
		assert_ok!(GameModule::<T>::make_offer(RawOrigin::Signed(caller2.clone()).into(), 0, 0.into(), 1.into()));

		#[extrinsic_call]
		handle_offer(
			RawOrigin::Signed(caller2),
			0,
			crate::Offer::Accept,
		); 
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