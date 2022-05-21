#![cfg_attr(not(feature = "std"), no_std)]

  pub use pallet::*;

  #[frame_support::pallet]
  pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Randomness},
	};
	use frame_system::pallet_prelude::*;
	use scale_info::TypeInfo;
	use sp_io::hashing::blake2_128;
	use sp_runtime::ArithmeticError;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Shipment<T: Config> {
		pub creator: T::AccountId,
		owner: T::AccountId,
		pub fees: Option<BalanceOf<T>>,
		pub status: ShipmentStatus,
		pub route: BoundedVec<T::AccountId, T::MaxSize>,
	}

	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum ShipmentStatus {
		InTransit,
		Delivered,
		Failed,
		Unavailable,
	}

	// The struct on which we build all of our Pallet logic.
	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    /* Placeholder for defining custom types. */

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId>;
		type KeyRandomNess: Randomness<Self::Hash, Self::BlockNumber>;
		type MaxSize: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		TransitPointCreated(T::AccountId),
		TransitPointRemoved(T::AccountId),
		ShipmentCreated(T::AccountId),
		ShipmentUpdated(T::AccountId),
		ShipmentReceived(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidRoute,
		OwnerNotFound,
		ShipmentNotFound,
		TransitPointAlreadyExists,
		TransitPointNotFound,
		UnauthorizedCaller,
	}

	// transit_node -> node_uid map
	#[pallet::storage]
	pub(super) type TransitNodes<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u8,
		OptionQuery,
	>;

	#[pallet::storage]
	pub(super) type NodeUID<T:Config> = StorageValue<
		_,
		u8,
		ValueQuery,
	>;

	#[pallet::storage]
	pub(super) type ShipmentUID<T:Config> = StorageValue<
		_,
		u64,
		ValueQuery,
	>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_new_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(!TransitNodes::<T>::contains_key(&transit_node), Error::<T>::TransitPointAlreadyExists);

			let uid = NodeUID::<T>::get();
			let new_uid = uid.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			TransitNodes::<T>::insert(&transit_node, &new_uid);
			NodeUID::<T>::put(new_uid);

			Self::deposit_event(Event::TransitPointCreated(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(TransitNodes::<T>::contains_key(&transit_node), Error::<T>::TransitPointNotFound);

			TransitNodes::<T>::remove(&transit_node);

			Self::deposit_event(Event::TransitPointRemoved(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_shipment(origin: OriginFor<T>, route_vec: BoundedVec<T::AccountId, T::MaxSize>) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;
			ensure!(TransitNodes::<T>::contains_key(&transit_node), Error::<T>::UnauthorizedCaller);
			ensure!(route_vec.len() > 1, Error::<T>::InvalidRoute);

			let uid = ShipmentUID::<T>::get();
			let new_uid = uid.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			let shipment = Shipment::<T> {
				creator: transit_node.clone(),
				owner: route_vec.get(1).unwrap().clone(),
				fees: None, // Todo: Calculate fees based on the route
				status: ShipmentStatus::InTransit,
				route: route_vec
			};

			//create Restricted Key => Create a type first so the shipment uid maps to the type (TO-DO)
			// Todo: store shipment (to map?)

			Self::deposit_event(Event::ShipmentCreated(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_shipment(origin: OriginFor<T>, uid: u64) -> DispatchResult {
			// This function will take a key parameter but we
			// don't know what the type will be. I'm working on it
			let transit_node = ensure_signed(origin)?;
			ensure!(TransitNodes::<T>::contains_key(&transit_node), Error::<T>::UnauthorizedCaller);

			// Match the input key with the one stored on the blockchain (Not implemented yet. see the first comment in this function)

			// Check if the current transit node is the destination. If it is,  change the status of the shipment.

			// Otherwise , generate a new key and a new owner

			Ok(())
		}
	}

	// Helpful functions
	impl<T: Config> Pallet<T> {

		fn gen_key(&self) -> [u8; 16] {
			let payload = (
				T::KeyRandomNess::random(&b"key"[..]).0,
				<frame_system::Pallet<T>>::extrinsic_index().unwrap_or_default(),
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		fn set_fees(&self) {}

		fn route(&mut self) {}

		fn get_transit_nodes() {}

		fn get_transit_status() {}
	}
  }