#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

pub mod properties;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as pallet_nfts::Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::Balance;

use frame_support::{
	traits::{Currency, Incrementable},
	PalletId,
};

use frame_support::sp_runtime::{
	traits::{AccountIdConversion, StaticLookup},
	Saturating,
};

use pallet_nfts::{
	CollectionConfig, CollectionSetting, CollectionSettings, ItemConfig, ItemSettings, MintSettings,
};

use frame_system::RawOrigin;

use enumflags2::BitFlags;

use frame_support::traits::Randomness;

use codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_system::{
	self as system,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, SendUnsignedTransaction,
		SignedPayload, Signer, SigningTypes, SubmitTransaction,
	},
	pallet_prelude::BlockNumberFor,
};
use lite_json::json::JsonValue;
use scale_info::prelude::string::String;
use sp_core::crypto::KeyTypeId;

use sp_runtime::{
	offchain::{
		http,
		storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
		Duration,
	},
	traits::Zero,
	transaction_validity::{InvalidTransaction, TransactionValidity, ValidTransaction},
	BoundedVec, RuntimeDebug,
};
use sp_std::vec::Vec;


/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"btc!");

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use super::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};
	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Difficulty level of game enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum DifficultyLevel {
		Practice,
		Player,
		Pro,
	}

	/// Offer enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum Offer {
		Accept,
		Reject,
	}

	/// Nft color enum.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub enum NftColor {
		Xorange,
		Xpink,
		Xblue,
		Xcyan,
		Xcoral,
		Xpurple,
		Xleafgreen,
		Xgreen,
	}

	impl NftColor {
		pub fn from_index(index: usize) -> Option<Self> {
			match index {
				0 => Some(NftColor::Xorange),
				1 => Some(NftColor::Xpink),
				2 => Some(NftColor::Xblue),
				3 => Some(NftColor::Xcyan),
				4 => Some(NftColor::Xcoral),
				5 => Some(NftColor::Xpurple),
				6 => Some(NftColor::Xleafgreen),
				7 => Some(NftColor::Xgreen),
				_ => None,
			}
		}
	}

	/// AccountId storage.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	pub struct PalletIdStorage<T: Config> {
		pallet_id: AccountIdOf<T>,
	}

	/// Game Data.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct GameData<T: Config> {
		pub difficulty: DifficultyLevel,
		pub player: AccountIdOf<T>,
		pub property: PropertyInfoData<T>,
	}

	/// Listing infos of a NFT.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct ListingInfo<CollectionId, ItemId, T: Config> {
		pub owner: AccountIdOf<T>,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
	}

	/// Offer infos of a listing.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct OfferInfo<CollectionId, ItemId, T: Config> {
		pub owner: AccountIdOf<T>,
		pub listing_id: u32,
		pub collection_id: CollectionId,
		pub item_id: ItemId,
	}

	/// Struct to store the property data for a game.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, frame_support::pallet_prelude::RuntimeDebugNoBound, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct PropertyInfoData<T: Config> {
		pub id: u32,
		pub bedrooms: u32,
		pub bathrooms: u32,
		pub summary: BoundedVec<u8, <T as Config>::StringLimit>,
		pub property_sub_type: BoundedVec<u8, <T as Config>::StringLimit>,
		pub first_visible_date: BoundedVec<u8, <T as Config>::StringLimit>,
		pub display_size: BoundedVec<u8, <T as Config>::StringLimit>,
		pub display_address: BoundedVec<u8, <T as Config>::StringLimit>,
		// pub property_images: BoundedVec<u8, <T as Config>::StringLimit>,
	}

	/// Struct for the user datas.
	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct User {
		pub points: u32,
		pub wins: u32,
		pub losses: u32,
		pub practise_rounds: u8,
		pub last_played_round: u32,
		pub nfts: CollectedColors,
	}

	impl User {
		pub fn add_nft_color(&mut self, color: NftColor) -> DispatchResult {
			self.nfts.add_nft_color(color)?;
			Ok(())
		}

		pub fn sub_nft_color(&mut self, color: NftColor) -> DispatchResult {
			self.nfts.sub_nft_color(color)?;
			Ok(())
		}

		pub fn has_four_of_all_colors(&self) -> bool {
			self.nfts.has_four_of_all_colors()
		}

		pub fn calculate_points(&mut self, color: NftColor) -> u32 {
			match color {
				NftColor::Xorange if self.nfts.xorange == 1 => 100,
				NftColor::Xorange if self.nfts.xorange == 2 => 120,
				NftColor::Xorange if self.nfts.xorange == 3 => 220,
				NftColor::Xorange if self.nfts.xorange == 4 => 340,
				NftColor::Xpink if self.nfts.xpink == 1 => 100,
				NftColor::Xpink if self.nfts.xpink == 2 => 120,
				NftColor::Xpink if self.nfts.xpink == 3 => 220,
				NftColor::Xpink if self.nfts.xpink == 4 => 340,
				NftColor::Xblue if self.nfts.xblue == 1 => 100,
				NftColor::Xblue if self.nfts.xblue == 2 => 120,
				NftColor::Xblue if self.nfts.xblue == 3 => 220,
				NftColor::Xblue if self.nfts.xblue == 4 => 340,
				NftColor::Xcyan if self.nfts.xcyan == 1 => 100,
				NftColor::Xcyan if self.nfts.xcyan == 2 => 120,
				NftColor::Xcyan if self.nfts.xcyan == 3 => 220,
				NftColor::Xcyan if self.nfts.xcyan == 4 => 340,
				NftColor::Xcoral if self.nfts.xcoral == 1 => 100,
				NftColor::Xcoral if self.nfts.xcoral == 2 => 120,
				NftColor::Xcoral if self.nfts.xcoral == 3 => 220,
				NftColor::Xcoral if self.nfts.xcoral == 4 => 340,
				NftColor::Xpurple if self.nfts.xpurple == 1 => 100,
				NftColor::Xpurple if self.nfts.xpurple == 2 => 120,
				NftColor::Xpurple if self.nfts.xpurple == 3 => 220,
				NftColor::Xpurple if self.nfts.xpurple == 4 => 340,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 1 => 100,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 2 => 120,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 3 => 220,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 4 => 340,
				NftColor::Xgreen if self.nfts.xgreen == 1 => 100,
				NftColor::Xgreen if self.nfts.xgreen == 2 => 120,
				NftColor::Xgreen if self.nfts.xgreen == 3 => 220,
				NftColor::Xgreen if self.nfts.xgreen == 4 => 340,
				_ => 0,
			}
		}

		pub fn subtracting_calculate_points(&mut self, color: NftColor) -> u32 {
			match color {
				NftColor::Xorange if self.nfts.xorange == 0 => 100,
				NftColor::Xorange if self.nfts.xorange == 1 => 120,
				NftColor::Xorange if self.nfts.xorange == 2 => 220,
				NftColor::Xorange if self.nfts.xorange == 3 => 340,
				NftColor::Xpink if self.nfts.xpink == 0 => 100,
				NftColor::Xpink if self.nfts.xpink == 1 => 120,
				NftColor::Xpink if self.nfts.xpink == 2 => 220,
				NftColor::Xpink if self.nfts.xpink == 3 => 340,
				NftColor::Xblue if self.nfts.xblue == 0 => 100,
				NftColor::Xblue if self.nfts.xblue == 1 => 120,
				NftColor::Xblue if self.nfts.xblue == 2 => 220,
				NftColor::Xblue if self.nfts.xblue == 3 => 340,
				NftColor::Xcyan if self.nfts.xcyan == 0 => 100,
				NftColor::Xcyan if self.nfts.xcyan == 1 => 120,
				NftColor::Xcyan if self.nfts.xcyan == 2 => 220,
				NftColor::Xcyan if self.nfts.xcyan == 3 => 340,
				NftColor::Xcoral if self.nfts.xcoral == 0 => 100,
				NftColor::Xcoral if self.nfts.xcoral == 1 => 120,
				NftColor::Xcoral if self.nfts.xcoral == 2 => 220,
				NftColor::Xcoral if self.nfts.xcoral == 3 => 340,
				NftColor::Xpurple if self.nfts.xpurple == 0 => 100,
				NftColor::Xpurple if self.nfts.xpurple == 1 => 120,
				NftColor::Xpurple if self.nfts.xpurple == 2 => 220,
				NftColor::Xpurple if self.nfts.xpurple == 3 => 340,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 0 => 100,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 1 => 120,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 2 => 220,
				NftColor::Xleafgreen if self.nfts.xleafgreen == 3 => 340,
				NftColor::Xgreen if self.nfts.xgreen == 0 => 100,
				NftColor::Xgreen if self.nfts.xgreen == 1 => 120,
				NftColor::Xgreen if self.nfts.xgreen == 2 => 220,
				NftColor::Xgreen if self.nfts.xgreen == 3 => 340,
				_ => 0,
			}
		}
	}

	#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
	#[derive(
		Encode, Decode, Clone, PartialEq, Eq, MaxEncodedLen, RuntimeDebug, TypeInfo, Default,
	)]
	#[scale_info(skip_type_params(T))]
	pub struct CollectedColors {
		pub xorange: u32,
		pub xpink: u32,
		pub xblue: u32,
		pub xcyan: u32,
		pub xcoral: u32,
		pub xpurple: u32,
		pub xleafgreen: u32,
		pub xgreen: u32,
	}

	impl CollectedColors {
		pub fn add_nft_color(&mut self, color: NftColor) -> DispatchResult {
			match color {
				NftColor::Xorange => {
					self.xorange = self.xorange.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xpink => {
					self.xpink = self.xpink.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xblue => {
					self.xblue = self.xblue.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xcyan => {
					self.xcyan = self.xcyan.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xcoral => {
					self.xcoral = self.xcoral.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xpurple => {
					self.xpurple = self.xpurple.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xleafgreen => {
					self.xleafgreen =
						self.xleafgreen.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
				NftColor::Xgreen => {
					self.xgreen = self.xgreen.checked_add(1).ok_or("Arithmetic overflow")?;
					Ok(())
				},
			}
		}

		pub fn sub_nft_color(&mut self, color: NftColor) -> DispatchResult {
			match color {
				NftColor::Xorange => {
					self.xorange = self.xorange.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xpink => {
					self.xpink = self.xpink.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xblue => {
					self.xblue = self.xblue.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xcyan => {
					self.xcyan = self.xcyan.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xcoral => {
					self.xcoral = self.xcoral.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xpurple => {
					self.xpurple = self.xpurple.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xleafgreen => {
					self.xleafgreen =
						self.xleafgreen.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
				NftColor::Xgreen => {
					self.xgreen = self.xgreen.checked_sub(1).ok_or("Arithmetic underflow")?;
					Ok(())
				},
			}
		}

		pub fn has_four_of_all_colors(&self) -> bool {
			self.xorange >= 4 &&
				self.xpink >= 4 && self.xblue >= 4 &&
				self.xcyan >= 4 && self.xcoral >= 4 &&
				self.xpurple >= 4 &&
				self.xleafgreen >= 4 &&
				self.xgreen >= 4
		}
	}

		// Preperty Info Data struct
		#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
		#[derive(
			Encode,
			Decode,
			Clone,
			PartialEq,
			Eq,
			MaxEncodedLen,
			frame_support::pallet_prelude::RuntimeDebugNoBound,
			TypeInfo,
		)]
		#[scale_info(skip_type_params(T))]
		pub struct PropertyDataInfo<T: Config> {
			pub id: u32,
			pub bedrooms: u32,
			pub bathrooms: u32,
			pub summary: BoundedVec<u8, <T as Config>::StringLimit>,
			pub property_sub_type: BoundedVec<u8, <T as Config>::StringLimit>,
			pub first_visible_date: BoundedVec<u8, <T as Config>::StringLimit>,
			pub display_size: BoundedVec<u8, <T as Config>::StringLimit>,
			pub display_address: BoundedVec<u8, <T as Config>::StringLimit>,
			// pub property_images: BoundedVec<u8, <T as Config>::StringLimit>,
		}
	
	
		enum TransactionType {
			Signed,
			UnsignedForAny,
			UnsignedForAll,
			Raw,
			None,
		}

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config + pallet_nfts::Config //+ pallet_babe::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		/// Origin who can create a new game.
		type GameOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		/// Collection id type from pallet nfts.
		type CollectionId: IsType<<Self as pallet_nfts::Config>::CollectionId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		/// Item id type from pallet nfts.
		type ItemId: IsType<<Self as pallet_nfts::Config>::ItemId>
			+ Parameter
			+ From<u32>
			+ Ord
			+ Copy
			+ MaxEncodedLen
			+ Encode;
		/// The maximum amount of properties.
		#[pallet::constant]
		type MaxProperty: Get<u32> + Clone + PartialEq + Eq;
		/// The marketplace's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		/// The maximum amount of games that can be played at the same time.
		#[pallet::constant]
		type MaxOngoingGames: Get<u32>;
		/// Randomness used for choosing a random property.
		type GameRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;
		/// The maximum length of data stored in string.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		/// The maximum length of leaderboard.
		#[pallet::constant]
		type LeaderboardLimit: Get<u32>;
		/// OCW configuration settings
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
		// Configuration parameters

		/// A grace period after we send transaction.
		///
		/// To avoid sending too many transactions, we only attempt to send one
		/// every `GRACE_PERIOD` blocks. We use Local Storage to coordinate
		/// sending between distinct runs of this offchain worker.
		#[pallet::constant]
		type GracePeriod: Get<BlockNumberFor<Self>>;
		/// Number of blocks of cooldown after unsigned transaction is included.
		///
		/// This ensures that we only accept unsigned transactions once, every `UnsignedInterval`
		/// blocks.
		#[pallet::constant]
		type UnsignedInterval: Get<BlockNumberFor<Self>>;
		/// A configuration for base priority of unsigned transactions.
		///
		/// This is exposed so that it can be tuned for particular runtime, when
		/// multiple pallets send unsigned transactions.
		#[pallet::constant]
		type UnsignedPriority: Get<TransactionPriority>;
		/// Maximum number of prices.
		#[pallet::constant]
		type MaxPrices: Get<u32>;
	}

	pub type CollectionId<T> = <T as Config>::CollectionId;
	pub type ItemId<T> = <T as Config>::ItemId;

	/// The id of the current round.
	#[pallet::storage]
	#[pallet::getter(fn current_round)]
	pub(super) type CurrentRound<T> = StorageValue<_, u32, ValueQuery>;

	/// Bool if there is a round currently ongoing.
	#[pallet::storage]
	#[pallet::getter(fn round_active)]
	pub(super) type RoundActive<T> = StorageValue<_, bool, ValueQuery>;

	/// A mapping of a round to the winner of the round.
	#[pallet::storage]
	#[pallet::getter(fn round_champion)]
	pub(super) type RoundChampion<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, AccountIdOf<T>, OptionQuery>;

	/// The next item id in a collection.	
	#[pallet::storage]
	#[pallet::getter(fn next_color_id)]
	pub(super) type NextColorId<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, u32, ValueQuery>;

	/// Mapping of a collection to the correlated color.
	#[pallet::storage]
	#[pallet::getter(fn collection_color)]
	pub(super) type CollectionColor<T: Config> =
		StorageMap<_, Blake2_128Concat, <T as pallet::Config>::CollectionId, NftColor, OptionQuery>;

	/// The next id of listings.
	#[pallet::storage]
	#[pallet::getter(fn next_lising_id)]
	pub(super) type NextListingId<T> = StorageValue<_, u32, ValueQuery>;

	/// The next id of offers.
	#[pallet::storage]
	#[pallet::getter(fn next_offer_id)]
	pub(super) type NextOfferId<T> = StorageValue<_, u32, ValueQuery>;

	/// The next id of game.
	#[pallet::storage]
	#[pallet::getter(fn game_id)]
	pub type GameId<T> = StorageValue<_, u32, ValueQuery>;

	/// The leaderboard of the game.
	#[pallet::storage]
	#[pallet::getter(fn leaderboard)]
	pub type Leaderboard<T> = StorageValue<_, BoundedVec<(AccountIdOf<T>, u32), <T as Config>::LeaderboardLimit>, ValueQuery>;

	/// Mapping of an account id to the user data of the account.
	#[pallet::storage]
	#[pallet::getter(fn users)]
	pub type Users<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, User, OptionQuery>;

	/// Mapping of game id to the game info.
	#[pallet::storage]
	#[pallet::getter(fn game_info)]
	pub type GameInfo<T: Config> = StorageMap<_, Blake2_128Concat, u32, GameData<T>, OptionQuery>;

	/// Mapping of listing id to the listing data.
	#[pallet::storage]
	#[pallet::getter(fn listings)]
	pub type Listings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		ListingInfo<<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId, T>,
		OptionQuery,
	>;

	/// Mapping of offer id to the offer data.
	#[pallet::storage]
	#[pallet::getter(fn offers)]
	pub type Offers<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u32,
		OfferInfo<<T as pallet::Config>::CollectionId, <T as pallet::Config>::ItemId, T>,
		OptionQuery,
	>;

	/// Stores the game keys and round types ending on a given block.
	#[pallet::storage]
	pub type GamesExpiring<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BlockNumberFor<T>,
		BoundedVec<u32, T::MaxOngoingGames>,
		ValueQuery,
	>;

	// A test property.
	#[pallet::storage]
	#[pallet::getter(fn test_property)]
	pub type TestProperties<T: Config> = StorageValue<_, PropertyInfoData<T>, OptionQuery>;

	/// Test for properties.
	#[pallet::storage]
	#[pallet::getter(fn test_prices)]
	pub type TestPrices<T: Config> = StorageMap<_, Blake2_128Concat, u32, u32, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A user has received points.
		PointsReceived { receiver: AccountIdOf<T>, amount: u32 },
		/// A game has started.
		GameStarted { player: AccountIdOf<T>, game_id: u32 },
		/// An answer has been submitted.
		AnswerSubmitted { player: AccountIdOf<T>, game_id: u32 },
		/// A nft has been listed.
		NftListed { owner: AccountIdOf<T>, collection_id: CollectionId<T>, item_id: ItemId<T> },
		/// A nft has been delisted.
		NftDelisted { owner: AccountIdOf<T>, collection_id: CollectionId<T>, item_id: ItemId<T> },
		/// An offer has been made.
		OfferMade {
			owner: AccountIdOf<T>,
			listing_id: u32,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		},
		/// An offer has been withdrawn.
		OfferWithdrawn { owner: AccountIdOf<T>, offer_id: u32 },
		/// An offer has been handled.
		OfferHandeld { offer_id: u32, offer: Offer },
		/// A new player has been registered
		NewPlayerRegistered { player: AccountIdOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// A player has not enough points to play.
		NotEnoughPoints,
		ConversionError,
		ArithmeticOverflow,
		ArithmeticUnderflow,
		MultiplyError,
		DivisionError,
		/// There are too many games active.
		TooManyGames,
		/// The caller has no permission.
		NoThePlayer,
		/// This is not an active game.
		NoActiveGame,
		NoPermission,
		/// This listing is not listed.
		ListingDoesNotExist,
		/// This offer does not exist.
		OfferDoesNotExist,
		/// There are too many test properties.
		TooManyTest,
		/// No property could be found.
		NoProperty,
		/// The user has not yet been registered.
		UserNotRegistered,
		/// The user has already made 5 practise rounds.
		TooManyPractise,
		/// The user has not yet made a practise round.
		NoPractise,
		InvalidIndex,
		/// The color for this collection is not known.
		CollectionUnknown,
		/// There is no active round at the moment.
		NoActiveRound,
		/// The player is already registered.
		PlayerAlreadyRegistered,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: frame_system::pallet_prelude::BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads_writes(1, 1);
			let ended_games = GamesExpiring::<T>::take(n);

			// Checks if there is a voting for a loan expiring in this block.
			ended_games.iter().for_each(|index| {
				weight = weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				let game_info = <GameInfo<T>>::take(index);
				if let Some(game_info) = game_info {
					let _ = Self::no_answer_result(game_info);
				}
			});
			weight
		}

		/// Offchain Worker entry point.
		///
		/// By implementing `fn offchain_worker` you declare a new offchain worker.
		/// This function will be called when the node is fully synced and a new best block is
		/// successfully imported.
		/// Note that it's not guaranteed for offchain workers to run on EVERY block, there might
		/// be cases where some blocks are skipped, or for some the worker runs twice (re-orgs),
		/// so the code should be able to handle that.
		/// You can use `Local Storage` API to coordinate runs of the worker.
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			// Note that having logs compiled to WASM may cause the size of the blob to increase
			// significantly. You can use `RuntimeDebug` custom derive to hide details of the types
			// in WASM. The `sp-api` crate also provides a feature `disable-logging` to disable
			// all logging and thus, remove any logging from the WASM.
			log::info!("Hello World from offchain workers!");

			let price = Self::fetch_property().map_err(|_| "Failed to fetch price");
			//TestProperties::<T>::put(price.unwrap());

			// Since off-chain workers are just part of the runtime code, they have direct access
			// to the storage and other included pallets.
			//
			// We can easily import `frame_system` and retrieve a block hash of the parent block.
			let parent_hash = <system::Pallet<T>>::block_hash(block_number - 1u32.into());
			log::debug!("Current block: {:?} (parent hash: {:?})", block_number, parent_hash);

			Self::fetch_property_and_send_signed();
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates the setup for a new game.
		///
		/// The origin must be the sudo.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::setup_game())]
		pub fn setup_game(origin: OriginFor<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			for x in 0..8 {
				if pallet_nfts::NextCollectionId::<T>::get().is_none() {
					pallet_nfts::NextCollectionId::<T>::set(
						<T as pallet_nfts::Config>::CollectionId::initial_value(),
					);
				};
				let collection_id = pallet_nfts::NextCollectionId::<T>::get().unwrap();
				let next_collection_id = collection_id.increment();
				pallet_nfts::NextCollectionId::<T>::set(next_collection_id);
				let collection_id: CollectionId<T> = collection_id.into();
				let pallet_id: AccountIdOf<T> =
					<T as pallet::Config>::PalletId::get().into_account_truncating();
				pallet_nfts::Pallet::<T>::do_create_collection(
					collection_id.into(),
					pallet_id.clone(),
					pallet_id.clone(),
					Self::default_collection_config(),
					T::CollectionDeposit::get(),
					pallet_nfts::Event::Created {
						creator: pallet_id.clone(),
						owner: pallet_id,
						collection: collection_id.into(),
					},
				)?;
				let color = NftColor::from_index(x).ok_or(Error::<T>::InvalidIndex)?;
				CollectionColor::<T>::insert(collection_id, color);
			}
			//Self::create_test_properties()?;
			let mut round = Self::current_round();
			round = round.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			CurrentRound::<T>::put(round);
			RoundActive::<T>::put(true);
			Ok(())
		}

		/// Registers a player and gives him initialy 50 points.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `player`: The AccountId of the user who gets registered.
		///
		/// Emits `NewPlayerRegistered` event when succesfful.
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::register_user())]
		pub fn register_user(origin: OriginFor<T>, player: AccountIdOf<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			ensure!(Self::users(player.clone()).is_none(), Error::<T>::PlayerAlreadyRegistered);
			let user = User {
				points: 50,
				wins: Default::default(),
				losses: Default::default(),
				practise_rounds: Default::default(),
				last_played_round: Default::default(),
				nfts: CollectedColors::default(),
			};
			Users::<T>::insert(player.clone(), user);
			Self::deposit_event(Event::<T>::NewPlayerRegistered { player });
			Ok(())
		}

		/// Gives points to a user.
		///
		/// The origin must be the sudo.
		///
		/// Parameters:
		/// - `receiver`: The AccountId of the user who gets points.
		///
		/// Emits `LocationCreated` event when succesfful.
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::give_points())]
		pub fn give_points(origin: OriginFor<T>, receiver: AccountIdOf<T>) -> DispatchResult {
			T::GameOrigin::ensure_origin(origin)?;
			let mut user = Self::users(receiver.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			user.points = user.points.checked_add(100).ok_or(Error::<T>::ArithmeticOverflow)?;
			Users::<T>::insert(receiver.clone(), user);
			Self::deposit_event(Event::<T>::PointsReceived { receiver, amount: 100 });
			Ok(())
		}

		/// Starts a game for the player.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `game_type`: The difficulty level of the game.
		///
		/// Emits `GameStarted` event when succesfful.
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::play_game())]
		pub fn play_game(origin: OriginFor<T>, game_type: DifficultyLevel) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::check_enough_points(signer.clone(), game_type.clone())?;
			ensure!(Self::round_active(), Error::<T>::NoActiveRound);
			let mut user = Self::users(signer.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			if Self::current_round() != user.last_played_round {
				user.nfts == Default::default();
			}
			let game_id = Self::game_id();
			if game_type == DifficultyLevel::Player {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block = current_block_number.saturating_add(8u32.into());

				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			} else if game_type == DifficultyLevel::Pro {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block = current_block_number.saturating_add(5u32.into());

				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			} else {
				let current_block_number = <frame_system::Pallet<T>>::block_number();
				let expiry_block = current_block_number.saturating_add(10u32.into());

				GamesExpiring::<T>::try_mutate(expiry_block, |keys| {
					keys.try_push(game_id).map_err(|_| Error::<T>::TooManyGames)?;
					Ok::<(), DispatchError>(())
				})?;
			}
			let property = Self::test_property().ok_or(Error::<T>::NoProperty)?;
			let game_datas = GameData { difficulty: game_type, player: signer.clone(), property };
			GameInfo::<T>::insert(game_id, game_datas);
			let next_game_id = game_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			GameId::<T>::put(next_game_id);
			Self::deposit_event(Event::<T>::GameStarted { player: signer, game_id });
			Ok(())
		}

		/// Checks the answer of the player and handles rewards accordingly.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `guess`: The answer of the player.
		/// - `game_id`: The id of the game that the player wants to answer to.
		///
		/// Emits `AnswerSubmitted` event when succesfful.
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::submit_answer())]
		pub fn submit_answer(origin: OriginFor<T>, guess: u32, game_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			let game_info = Self::game_info(game_id).ok_or(Error::<T>::NoActiveGame)?;
			ensure!(signer == game_info.player, Error::<T>::NoThePlayer);
			let property_id = game_info.property.id;
			let result: u32 = Self::test_prices(property_id).ok_or(Error::<T>::NoProperty)?;
			let difference_value = ((result as i32)
				.checked_sub(guess as i32)
				.ok_or(Error::<T>::ArithmeticUnderflow)?)
			.checked_mul(1000)
			.ok_or(Error::<T>::MultiplyError)?
			.checked_div(result as i32)
			.ok_or(Error::<T>::DivisionError)?
			.abs();
			Self::check_result(difference_value.try_into().unwrap(), game_id)?;
			Self::deposit_event(Event::<T>::AnswerSubmitted { player: signer, game_id });
			Ok(())
		}

		/// Lists a nft from the user.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `collection_id`: The collection id of the nft that will be listed.
		/// - `item_id`: The item id of the nft that will be listed.
		///
		/// Emits `NftListed` event when succesfful.
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::list_nft())]
		pub fn list_nft(
			origin: OriginFor<T>,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			ensure!(
				pallet_nfts::Pallet::<T>::owner(collection_id.into(), item_id.into()) ==
					Some(signer.clone()),
				Error::<T>::NoPermission
			);
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::unlock_item_transfer(
				pallet_origin,
				collection_id.into(),
				item_id.into(),
			)?;
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let listing_info = ListingInfo { owner: signer.clone(), collection_id, item_id };
			let mut listing_id = Self::next_lising_id();
			Listings::<T>::insert(listing_id, listing_info);
			listing_id = listing_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextListingId::<T>::put(listing_id);
			Self::deposit_event(Event::<T>::NftListed { owner: signer, collection_id, item_id });
			Ok(())
		}

		/// Delists a nft from the user.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing id of the listing.
		///
		/// Emits `NftDelisted` event when succesfful.
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::delist_nft())]
		pub fn delist_nft(origin: OriginFor<T>, listing_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let listing_info =
				Listings::<T>::take(listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_info.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::lock_item_transfer(
				pallet_origin,
				listing_info.collection_id.into(),
				listing_info.item_id.into(),
			)?;
			Self::deposit_event(Event::<T>::NftDelisted {
				owner: signer,
				collection_id: listing_info.collection_id,
				item_id: listing_info.item_id,
			});
			Ok(())
		}

		/// Makes an offer for a nft listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `listing_id`: The listing id of the listing.
		/// - `collection_id`: The collection id of the nft that will be offered.
		/// - `item_id`: The item id of the nft that will be offered.
		///
		/// Emits `OfferMade` event when succesfful.
		#[pallet::call_index(7)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn make_offer(
			origin: OriginFor<T>,
			listing_id: u32,
			collection_id: CollectionId<T>,
			item_id: ItemId<T>,
		) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			ensure!(Self::listings(listing_id).is_some(), Error::<T>::ListingDoesNotExist);
			let pallet_lookup = <T::Lookup as StaticLookup>::unlookup(Self::account_id());
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::unlock_item_transfer(
				pallet_origin,
				collection_id.into(),
				item_id.into(),
			)?;
			pallet_nfts::Pallet::<T>::transfer(
				origin,
				collection_id.into(),
				item_id.into(),
				pallet_lookup,
			)?;
			let offer_info =
				OfferInfo { owner: signer.clone(), listing_id, collection_id, item_id };
			let offer_id = Self::next_offer_id();
			Offers::<T>::insert(offer_id, offer_info);
			let offer_id = offer_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
			NextOfferId::<T>::put(offer_id);
			Self::deposit_event(Event::<T>::OfferMade {
				owner: signer,
				listing_id,
				collection_id,
				item_id,
			});
			Ok(())
		}

		/// Withdraw an offer.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `offer_id`: The id of the offer.
		///
		/// Emits `OfferWithdrawn` event when succesfful.
		#[pallet::call_index(8)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::make_offer())]
		pub fn withdraw_offer(origin: OriginFor<T>, offer_id: u32) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Self::offers(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			ensure!(offer_details.owner == signer, Error::<T>::NoPermission);
			pallet_nfts::Pallet::<T>::do_transfer(
				offer_details.collection_id.into(),
				offer_details.item_id.into(),
				signer.clone(),
				|_, _| Ok(()),
			)?;
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			pallet_nfts::Pallet::<T>::lock_item_transfer(
				pallet_origin,
				offer_details.collection_id.into(),
				offer_details.item_id.into(),
			)?;
			Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			Self::deposit_event(Event::<T>::OfferWithdrawn { owner: signer, offer_id });
			Ok(())
		}

		/// Handles an offer for a nft listing.
		///
		/// The origin must be Signed and the sender must have sufficient funds free.
		///
		/// Parameters:
		/// - `offer_id`: The id of the offer.
		/// - `offer`: Must be either Accept or Reject.
		///
		/// Emits `OfferHandeld` event when succesfful.
		#[pallet::call_index(9)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::handle_offer())]
		pub fn handle_offer(origin: OriginFor<T>, offer_id: u32, offer: Offer) -> DispatchResult {
			let signer = ensure_signed(origin.clone())?;
			let offer_details = Offers::<T>::take(offer_id).ok_or(Error::<T>::OfferDoesNotExist)?;
			let listing_details =
				Self::listings(offer_details.listing_id).ok_or(Error::<T>::ListingDoesNotExist)?;
			ensure!(listing_details.owner == signer, Error::<T>::NoPermission);
			let pallet_origin: OriginFor<T> = RawOrigin::Signed(Self::account_id()).into();
			if offer == Offer::Accept {
				pallet_nfts::Pallet::<T>::do_transfer(
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
					offer_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin.clone(),
					listing_details.collection_id.into(),
					listing_details.item_id.into(),
				)?;
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					listing_details.owner.clone(),
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin,
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
				)?;
				Listings::<T>::take(offer_details.listing_id)
					.ok_or(Error::<T>::ListingDoesNotExist)?;

				Self::swap_user_points(offer_details.owner.clone(), listing_details.collection_id, offer_details.collection_id)?;
				Self::swap_user_points(signer.clone(), offer_details.collection_id, listing_details.collection_id)?;
			} else {
				pallet_nfts::Pallet::<T>::do_transfer(
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
					offer_details.owner,
					|_, _| Ok(()),
				)?;
				pallet_nfts::Pallet::<T>::lock_item_transfer(
					pallet_origin,
					offer_details.collection_id.into(),
					offer_details.item_id.into(),
				)?;
			}
			Self::deposit_event(Event::<T>::OfferHandeld { offer_id, offer });
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight({0})]
		pub fn submit_price(
			origin: OriginFor<T>,
			property: PropertyInfoData<T>,
		) -> DispatchResultWithPostInfo {
			// Retrieve sender of the transaction.
			let who = ensure_signed(origin)?;
			// Add the price to the on-chain list.
			// Self::add_price(Some(who), price);
			TestProperties::<T>::put(property.clone());
			//Self::deposit_event(Event::NewPrice { price: price.id.clone(), who });
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Get the account id of the pallet
		pub fn account_id() -> AccountIdOf<T> {
			<T as pallet::Config>::PalletId::get().into_account_truncating()
		}

		/// checks if the signer has enough points to start a game.
		fn check_enough_points(
			signer: AccountIdOf<T>,
			game_type: DifficultyLevel,
		) -> DispatchResult {
			if game_type == DifficultyLevel::Pro {
				ensure!(
					Self::users(signer.clone())
						.ok_or(Error::<T>::UserNotRegistered)?
						.practise_rounds > 0,
					Error::<T>::NoPractise
				);
				ensure!(
					Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.points >= 50,
					Error::<T>::NotEnoughPoints
				);
			} else if game_type == DifficultyLevel::Player {
				ensure!(
					Self::users(signer.clone())
						.ok_or(Error::<T>::UserNotRegistered)?
						.practise_rounds > 0,
					Error::<T>::NoPractise
				);
				ensure!(
					Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.points >= 25,
					Error::<T>::NotEnoughPoints
				);
			} else {
				ensure!(
					Self::users(signer).ok_or(Error::<T>::UserNotRegistered)?.practise_rounds < 5,
					Error::<T>::TooManyPractise
				);
			}
			Ok(())
		}

		/// checks the answer and distributes the rewards accordingly.
		fn check_result(difference: u16, game_id: u32) -> DispatchResult {
			let game_info = GameInfo::<T>::take(game_id).ok_or(Error::<T>::NoActiveGame)?;
			if game_info.difficulty == DifficultyLevel::Pro {
				match difference {
					0..=10 => {
						let (hashi, _) = T::GameRandomness::random(&[game_id as u8]);
						let u32_value = u32::from_le_bytes(
							hashi.as_ref()[4..8]
								.try_into()
								.map_err(|_| Error::<T>::ConversionError)?,
						);
						let random_number = (u32_value % 8)
							.checked_add(
								8 * (Self::current_round()
									.checked_sub(1)
									.ok_or(Error::<T>::ArithmeticUnderflow)?),
							)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						let collection_id: <T as pallet::Config>::CollectionId =
							random_number.into();
						let next_item_id = Self::next_color_id(collection_id);
						let item_id: ItemId<T> = next_item_id.into();
						let next_item_id =
							next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						NextColorId::<T>::insert(collection_id, next_item_id);
						pallet_nfts::Pallet::<T>::do_mint(
							collection_id.into(),
							item_id.into(),
							Some(Self::account_id()),
							game_info.player.clone(),
							Self::default_item_config(),
							|_, _| Ok(()),
						)?;
						let pallet_origin: OriginFor<T> =
							RawOrigin::Signed(Self::account_id()).into();
						pallet_nfts::Pallet::<T>::lock_item_transfer(
							pallet_origin,
							collection_id.into(),
							item_id.into(),
						)?;
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						let color = Self::collection_color(collection_id)
							.ok_or(Error::<T>::CollectionUnknown)?;
						user.add_nft_color(color.clone())?;
						let points = user.calculate_points(color);
						user.points = user
							.points
							.checked_add(points)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user.clone());
						if user.has_four_of_all_colors() {
							Self::end_game(game_info.player.clone());
						}
					},
					11..=30 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(50).ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					31..=50 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(30).ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					51..=100 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(10).ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					101..=150 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					151..=200 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					201..=250 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(30).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					251..=300 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(40).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					_ => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(50).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
				}
			} else if game_info.difficulty == DifficultyLevel::Player {
				match difference {
					0..=10 => {
						let (hashi, _) = T::GameRandomness::random(&[game_id as u8]);
						let u32_value = u32::from_le_bytes(
							hashi.as_ref()[4..8]
								.try_into()
								.map_err(|_| Error::<T>::ConversionError)?,
						);
						let random_number = (u32_value % 8)
							.checked_add(
								8 * (Self::current_round()
									.checked_sub(1)
									.ok_or(Error::<T>::ArithmeticUnderflow)?),
							)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						let collection_id: <T as pallet::Config>::CollectionId =
							random_number.into();
						let next_item_id = Self::next_color_id(collection_id);
						let item_id: ItemId<T> = next_item_id.into();
						let next_item_id =
							next_item_id.checked_add(1).ok_or(Error::<T>::ArithmeticOverflow)?;
						NextColorId::<T>::insert(collection_id, next_item_id);
						pallet_nfts::Pallet::<T>::do_mint(
							collection_id.into(),
							item_id.into(),
							Some(Self::account_id()),
							game_info.player.clone(),
							Self::default_item_config(),
							|_, _| Ok(()),
						)?;
						let pallet_origin: OriginFor<T> =
							RawOrigin::Signed(Self::account_id()).into();
						pallet_nfts::Pallet::<T>::lock_item_transfer(
							pallet_origin,
							collection_id.into(),
							item_id.into(),
						)?;
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						let color = Self::collection_color(collection_id)
							.ok_or(Error::<T>::CollectionUnknown)?;
						user.add_nft_color(color.clone())?;
						let points = user.calculate_points(color);
						user.points = user
							.points
							.checked_add(points)
							.ok_or(Error::<T>::ArithmeticOverflow)?;
						Users::<T>::insert(game_info.player.clone(), user.clone());
						if user.has_four_of_all_colors() {
							Self::end_game(game_info.player.clone());
						}
					},
					11..=30 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					31..=50 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					51..=100 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_add(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					101..=150 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					151..=200 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					201..=250 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(15).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					251..=300 => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(20).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
					_ => {
						let mut user = Self::users(game_info.player.clone())
							.ok_or(Error::<T>::UserNotRegistered)?;
						user.points =
							user.points.checked_sub(25).ok_or(Error::<T>::ArithmeticUnderflow)?;
						Users::<T>::insert(game_info.player.clone(), user);
					},
				}
			} else {
				let mut user =
					Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user.points = user.points.checked_add(5).ok_or(Error::<T>::ArithmeticUnderflow)?;
				user.practise_rounds =
					user.practise_rounds.checked_add(1).ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(game_info.player.clone(), user);
			}
			let mut user =
				Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
			Self::update_leaderboard(game_info.player, user.points)?;
			Ok(())
		}

		fn update_leaderboard(user_id: AccountIdOf<T>, new_points: u32) -> DispatchResult {
			let mut leaderboard = Self::leaderboard();
			let leaderboard_size = leaderboard.len();
		
			if let Some((_, user_points)) = leaderboard.iter_mut().find(|(id, _)| *id == user_id) {
				*user_points = new_points;
				leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
				return Ok(());
			}
			if new_points > 0 && (leaderboard_size < 10 || new_points > leaderboard.last().map(|(_, points)| *points).unwrap_or(0)) {
				if leaderboard.len() >= 10 {
					leaderboard.pop();
				}
				leaderboard.try_push((user_id, new_points)).map_err(|_| Error::<T>::InvalidIndex)?;
				leaderboard.sort_by(|a, b| b.1.cmp(&a.1));
			}
			Ok(())
		}

		fn swap_user_points(nft_holder: AccountIdOf<T>, collection_id_add: CollectionId<T>, collection_id_sub: CollectionId<T>) -> DispatchResult {
				let mut user = Self::users(nft_holder.clone())
					.ok_or(Error::<T>::UserNotRegistered)?;
				let color_add = Self::collection_color(collection_id_add)
					.ok_or(Error::<T>::CollectionUnknown)?;
				let color_sub = Self::collection_color(collection_id_sub)
					.ok_or(Error::<T>::CollectionUnknown)?;
				user.add_nft_color(color_add.clone())?;
				let points = user.calculate_points(color_add);
				user.points =
				user.points.checked_add(points).ok_or(Error::<T>::ArithmeticOverflow)?;
				user.sub_nft_color(color_sub.clone())?;
				let points = user.subtracting_calculate_points(color_sub);
				user.points =
				user.points.checked_sub(points).ok_or(Error::<T>::ArithmeticOverflow)?; 
				Users::<T>::insert(nft_holder.clone(), user.clone());
				Self::update_leaderboard(nft_holder.clone(), user.points)?;
				if user.has_four_of_all_colors() {
					Self::end_game(nft_holder);
				}
			Ok(())
		}

		/// Handles the case if the player did not answer on time.
		fn no_answer_result(game_info: GameData<T>) -> DispatchResult {
			if game_info.difficulty == DifficultyLevel::Pro {
				let mut user =
					Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user.points = user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(game_info.player.clone(), user);
			} else if game_info.difficulty == DifficultyLevel::Player {
				let mut user =
					Self::users(game_info.player.clone()).ok_or(Error::<T>::UserNotRegistered)?;
				user.points = user.points.checked_sub(10).ok_or(Error::<T>::ArithmeticUnderflow)?;
				Users::<T>::insert(game_info.player.clone(), user);
			} else {
			}
			Ok(())
		}

		fn end_game(winner: AccountIdOf<T>) -> DispatchResult {
			RoundActive::<T>::put(false);
			RoundChampion::<T>::insert(Self::current_round(), winner);
			Ok(())
		}

		/// Set the default collection configuration for creating a collection.
		fn default_collection_config() -> CollectionConfig<
			BalanceOf<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			Self::collection_config_from_disabled_settings(
				CollectionSetting::DepositRequired.into(),
			)
		}

		fn collection_config_from_disabled_settings(
			settings: BitFlags<CollectionSetting>,
		) -> CollectionConfig<
			BalanceOf<T>,
			BlockNumberFor<T>,
			<T as pallet_nfts::Config>::CollectionId,
		> {
			CollectionConfig {
				settings: CollectionSettings::from_disabled(settings),
				max_supply: None,
				mint_settings: MintSettings::default(),
			}
		}

		/// Set the default item configuration for minting a nft.
		fn default_item_config() -> ItemConfig {
			ItemConfig { settings: ItemSettings::all_enabled() }
		}
	}

	impl<T: Config> Pallet<T> {
		/// Fetch current price and return the result in cents.
	fn fetch_property() -> Result<PropertyInfoData<T>, http::Error> {
		// We want to keep the offchain worker execution time reasonable, so we set a hard-coded
		// deadline to 2s to complete the external call.
		// You can also wait indefinitely for the response, however you may still get a timeout
		// coming from the host machine.
		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
		// Initiate an external HTTP GET request.
		// This is using high-level wrappers from `sp_runtime`, for the low-level calls that
		// you can find in `sp_io`. The API is trying to be similar to `request`, but
		// since we are running in a custom WASM execution environment we can't simply
		// import the library here.

		let request = http::Request::get(
			"https://ipfs.io/ipfs/QmZ3Dn5B2UMuv9PFr1Ba3NGSKft2rwToBKCPaCTCmSab4k?filename=testing_data.json"
		);

		// We set the deadline for sending of the request, note that awaiting response can
		// have a separate deadline. Next we send the request, before that it's also possible
		// to alter request headers or stream body content in case of non-GET requests.
		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;

		// The request is already being processed by the host, we are free to do anything
		// else in the worker (we can send multiple concurrent requests too).
		// At some point however we probably want to check the response though,
		// so we can block current thread and wait for it to finish.
		// Note that since the request is being driven by the host, we don't have to wait
		// for the request to have it complete, we will just not read the response.
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		// Let's check the status code before we proceed to reading the response.
		if response.code != 200 {
			log::warn!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown)
		}

		// Next we want to fully read the response body and collect it to a vector of bytes.
		// Note that the return object allows you to read the body in chunks as well
		// with a way to control the deadline.
		let body = response.body().collect::<Vec<u8>>();

		// Create a str slice from the body.
		let body_str = sp_std::str::from_utf8(&body).map_err(|_| {
			log::warn!("No UTF8 body");
			http::Error::Unknown
		})?;

		let property = match Self::parse_property(body_str) {
			Some(property) => Ok(property),
			None => {
				log::warn!("Unable to extract price from the response: {:?}", body_str);
				Err(http::Error::Unknown)
			},
		}?;

		// log::warn!("Got property: {:?} cents", price);

		Ok(property)
	}

	/// Parse the price from the given JSON string using `lite-json`.
	///
	/// Returns `None` when parsing failed or `Some(price in cents)` when parsing is successful.
	fn parse_property(property_str: &str) -> Option<PropertyInfoData<T>> {
		let val = lite_json::parse_json(property_str);
		let id = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'id' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("id".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::Number(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};
		let val = lite_json::parse_json(property_str);
		let bedrooms = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'bedrooms' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("bedrooms".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::Number(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};
		let val = lite_json::parse_json(property_str);
		let bathrooms = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'bathrooms' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("bathrooms".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::Number(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let summary = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'summary' field in the first object
						if let Some((_, v)) =
							obj.into_iter().find(|(k, _)| k.iter().copied().eq("summary".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let propertySubType = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'propertySubType' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("propertySubType".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let firstVisibleDate = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'firstVisibleDate' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("firstVisibleDate".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let displaySize = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'displaySize' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("displaySize".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let displayAddress = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'displayAddress' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("displayAddress".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let val = lite_json::parse_json(property_str);
		let propertyImages = match val.ok()? {
			JsonValue::Array(mut arr) => {
				// Check if the array has at least one element
				if let Some(obj) = arr.pop() {
					// Check if the first element is an object
					if let JsonValue::Object(obj) = obj {
						// Find the 'propertyImages' field in the first object
						if let Some((_, v)) = obj
							.into_iter()
							.find(|(k, _)| k.iter().copied().eq("propertyImages".chars()))
						{
							// Check if the value associated with 'id' is a number
							if let JsonValue::String(number) = v {
								number
							} else {
								return None;
							}
						} else {
							return None;
						}
					} else {
						return None;
					}
				} else {
					return None;
				}
			},
			_ => return None,
		};

		let id = id.integer as u32;
		let bedrooms = bedrooms.integer as u32;
		let bathrooms = bathrooms.integer as u32;
		let summary: &str = &summary.iter().collect::<String>();
		let propertySubType: &str = &propertySubType.iter().collect::<String>();
		let firstVisibleDate: &str = &firstVisibleDate.iter().collect::<String>();
		let displaySize: &str = &displaySize.iter().collect::<String>();
		let displayAddress: &str = &displayAddress.iter().collect::<String>();
		// let propertyImages: &str = &propertyImages.iter().collect::<String>();

		let property = PropertyInfoData {
			id,
			bedrooms,
			bathrooms,
			summary: summary.as_bytes().to_vec().try_into().unwrap(),
			property_sub_type: propertySubType.as_bytes().to_vec().try_into().unwrap(),
			first_visible_date: firstVisibleDate.as_bytes().to_vec().try_into().unwrap(),
			display_size: displaySize.as_bytes().to_vec().try_into().unwrap(),
			display_address: displayAddress.as_bytes().to_vec().try_into().unwrap(),
			// property_images: propertyImages.as_bytes().to_vec().try_into().unwrap(),
		};

		Some(property)

		// Some(price.integer as u32 * 100 + (price.fraction / 10_u64.pow(exp)) as u32)
	}

	fn fetch_property_and_send_signed() -> Result<(), &'static str> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			return Err(
				"No local accounts available. Consider adding one via `author_insertKey` RPC.",
			)
		}
		// Make an external HTTP request to fetch the current price.
		// Note this call will block until response is received.
		let property = Self::fetch_property().map_err(|_| "Failed to fetch price")?;

		// Using `send_signed_transaction` associated type we create and submit a transaction
		// representing the call, we've just created.
		// Submit signed will return a vector of results for all accounts that were found in the
		// local keystore with expected `KEY_TYPE`.
		let results = signer.send_signed_transaction(|_account| {
			// Received price is wrapped into a call to `submit_price` public function of this
			// pallet. This means that the transaction, when executed, will simply call that
			// function passing `price` as an argument.
			Call::submit_price { property: property.clone() }
		});

		for (acc, res) in &results {
			match res {
				Ok(()) => log::info!("[{:?}] Submitted price of {:?} cents", acc.id, property),
				Err(e) => log::error!("[{:?}] Failed to submit transaction: {:?}", acc.id, e),
			}
		}

		Ok(())
	}

	}
}
