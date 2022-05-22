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
		pub fees: Option<BalanceOf<T>>,
		pub owner_index: u8,
		pub route: BoundedVec<T::AccountId, T::MaxSize>,
		pub status: ShipmentStatus,
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
	#[pallet::without_storage_info]
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
		InvalidUID,
		InvalidRoute,
		UIDNotFound,
		ShipmentAlreadyExists,
		ShipmentKeyAlreadyExists,
		ShipmentNotFound,
		TransitPointAlreadyExists,
		TransitPointNotFound,
		UnauthorizedCaller,
	}

	#[pallet::storage]
	pub(super) type NodeUID<T:Config> = StorageValue<
		_,
		u8,
		ValueQuery,
	>;

	// uid -> key
	#[pallet::storage]
	pub(super) type UidToKey<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		[u8; 16],
		OptionQuery,
	>;

	// shipment_uid -> shipment map
	#[pallet::storage]
	pub(super) type UidToShipment<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		Shipment<T>,
	>;

	#[pallet::storage]
	pub(super) type ShipmentCount<T:Config> = StorageValue<
		_,
		u64,
		ValueQuery,
	>;

	// transit_node -> node_uid map
	#[pallet::storage]
	pub(super) type TransitNodeToUid<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u8,
		OptionQuery,
	>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_new_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(!TransitNodeToUid::<T>::contains_key(&transit_node), Error::<T>::TransitPointAlreadyExists);

			let uid = NodeUID::<T>::get();
			let new_uid = uid.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			TransitNodeToUid::<T>::insert(&transit_node, &new_uid);
			NodeUID::<T>::put(new_uid);

			Self::deposit_event(Event::TransitPointCreated(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(TransitNodeToUid::<T>::contains_key(&transit_node), Error::<T>::TransitPointNotFound);

			TransitNodeToUid::<T>::remove(&transit_node);

			Self::deposit_event(Event::TransitPointRemoved(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_shipment(origin: OriginFor<T>, route_vec: BoundedVec<T::AccountId, T::MaxSize>) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;
			ensure!(TransitNodeToUid::<T>::contains_key(&transit_node), Error::<T>::UnauthorizedCaller); // check if this is called by Transit Node
			ensure!(route_vec.len() > 1, Error::<T>::InvalidRoute);

			let uid = ShipmentCount::<T>::get();
			let new_uid = uid.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			let shipment = Shipment::<T> {
				creator: transit_node.clone(),
				fees: None, // Todo: Calculate fees based on the route
				owner_index: 1,
				route: route_vec,
				status: ShipmentStatus::InTransit
			};

			ensure!(!UidToShipment::<T>::contains_key(&new_uid), Error::<T>::ShipmentAlreadyExists);
			UidToShipment::<T>::insert(&new_uid, &shipment);

			let key = Self::gen_key(); // Todo: How will next destination know/get this key value?
			UidToKey::<T>::insert(&new_uid, &key);

			ShipmentCount::<T>::put(new_uid);

			Self::deposit_event(Event::ShipmentCreated(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_shipment(origin: OriginFor<T>, uid: u64, key: [u8; 16]) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;
			ensure!(TransitNodeToUid::<T>::contains_key(&transit_node), Error::<T>::UnauthorizedCaller);
			ensure!(UidToKey::<T>::contains_key(&uid), Error::<T>::UIDNotFound);
			ensure!(UidToKey::<T>::get(&uid).unwrap() == key, Error::<T>::InvalidUID);
			ensure!(UidToShipment::<T>::contains_key(uid), Error::<T>::ShipmentNotFound);

			let mut shipment = UidToShipment::<T>::get(uid).unwrap();
			ensure!(&transit_node == shipment.route.get(shipment.owner_index as usize).unwrap(), Error::<T>::UnauthorizedCaller);
			UidToKey::<T>::remove(&uid);

			match shipment.owner_index == shipment.route.len() as u8 - 1 {
				true => {
					// Shipment has reached end destination
					shipment.status = ShipmentStatus::Delivered;
					Self::deposit_event(Event::ShipmentReceived(transit_node));
				},
				false => {
					// Shipment is still in transit
					shipment.owner_index = shipment.owner_index + 1;
					let new_key = Self::gen_key(); // Todo: How will next destination know/get this key value?
					UidToKey::<T>::insert(&uid, &new_key);
					Self::deposit_event(Event::ShipmentUpdated(transit_node));
				}
			}
			Ok(())
		}
	}

	// Helpful functions
	impl<T: Config> Pallet<T> {

		fn gen_key() -> [u8; 16] {
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