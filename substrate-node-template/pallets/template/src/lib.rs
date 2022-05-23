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
	use sp_std::vec::Vec;

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
		InvalidShipmentUID,
		InvalidRoute,
		InvalidKey,
		KeyNotFound,
		ShipmentAlreadyExists,
		ShipmentKeyAlreadyExists,
		ShipmentNotFound,
		TransitPointAlreadyExists,
		TransitNodesOverFlow,
		TransitPointNotFound,
		UIDNotFound,
		UnauthorizedCaller,
		CallerIsNotFirstNode
	}

	#[pallet::storage]
	#[pallet::getter(fn count_for_transit_point)]
	pub(super) type CountForTransitPoints<T:Config> = StorageValue<
		_,
		u8,
		ValueQuery,
	>;

	// shipment_uid -> key map
	#[pallet::storage]
	#[pallet::getter(fn shipment_uid_to_key)]
	pub(super) type ShipmentUidToKey<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		[u8; 16],
		OptionQuery,
	>;

	// shipment_uid -> shipment map
	#[pallet::storage]
	#[pallet::getter(fn uid_to_shipment)]
	pub(super) type UidToShipment<T:Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		Shipment<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn count_for_shipment)]
	pub(super) type ShipmentCount<T:Config> = StorageValue<
		_,
		u64,
		ValueQuery,
	>;

	// transit_node -> node_uid map
	#[pallet::storage]
	#[pallet::getter(fn transit_nodes)]
	pub(super) type TransitNodes<T:Config> = StorageValue<
		_,
		Vec<T::AccountId>,
		ValueQuery,
	>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_new_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(!Self::transit_nodes().contains(&transit_node), Error::<T>::TransitPointAlreadyExists);

			let transit_point_counts = Self::count_for_transit_point().checked_add(1).ok_or(ArithmeticError::Overflow)?;
			let mut new_transit_nodes = Self::transit_nodes();
			new_transit_nodes.push(transit_node.clone());
			CountForTransitPoints::<T>::put(transit_point_counts);
			TransitNodes::<T>::put(new_transit_nodes);

			Self::deposit_event(Event::TransitPointCreated(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_transit_node(origin: OriginFor<T>, transit_node: T::AccountId) -> DispatchResult {

			ensure_root(origin)?;
			ensure!(Self::transit_nodes().contains(&transit_node), Error::<T>::TransitPointNotFound);

			let transit_point_counts = Self::count_for_transit_point().checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let mut new_transit_nodes = Self::transit_nodes();
			new_transit_nodes.retain(|nodes| *nodes == transit_node);

			CountForTransitPoints::<T>::put(transit_point_counts);
			TransitNodes::<T>::put(new_transit_nodes);

			Self::deposit_event(Event::TransitPointRemoved(transit_node));

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_shipment(origin: OriginFor<T>, route_vec: BoundedVec<T::AccountId, T::MaxSize>) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;
			ensure!(route_vec[0] == transit_node, Error::<T>::CallerIsNotFirstNode);
			ensure!(route_vec.len() > 1, Error::<T>::InvalidRoute);
			ensure!(route_vec.iter().all(|node| Self::transit_nodes().contains(&node)), Error::<T>::InvalidRoute);

			let shipment_uid = ShipmentCount::<T>::get();
			let new_shipment_uid = shipment_uid.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			let shipment = Shipment::<T> {
				creator: transit_node.clone(),
				fees: None, // Todo: Calculate fees based on the route
				owner_index: 1,
				route: route_vec,
				status: ShipmentStatus::InTransit
			};

			ensure!(!UidToShipment::<T>::contains_key(&new_shipment_uid), Error::<T>::ShipmentAlreadyExists);
			UidToShipment::<T>::insert(&new_shipment_uid, &shipment);

			let key = Self::gen_key();
			ShipmentUidToKey::<T>::insert(&new_shipment_uid, &key);

			ShipmentCount::<T>::put(new_shipment_uid);

			Self::deposit_event(Event::ShipmentCreated(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_shipment(origin: OriginFor<T>, shipment_uid: u64, key: [u8; 16]) -> DispatchResult {

			let transit_node = ensure_signed(origin)?;
			ensure!(ShipmentUidToKey::<T>::contains_key(&shipment_uid), Error::<T>::UIDNotFound);
			ensure!(Self::shipment_uid_to_key(&shipment_uid).unwrap() == key, Error::<T>::InvalidKey);
			ensure!(UidToShipment::<T>::contains_key(shipment_uid), Error::<T>::ShipmentNotFound);

			let mut shipment = Self::uid_to_shipment(shipment_uid).ok_or(Error::<T>::ShipmentNotFound)?;
			ensure!(&transit_node == shipment.route.get(shipment.owner_index as usize).unwrap(), Error::<T>::UnauthorizedCaller);
			ShipmentUidToKey::<T>::remove(&shipment_uid);

			match shipment.owner_index == shipment.route.len() as u8 - 1 {
				true => {
					// Shipment has reached end destination
					shipment.status = ShipmentStatus::Delivered;
					UidToShipment::<T>::insert(&shipment_uid, &shipment);
					Self::deposit_event(Event::ShipmentReceived(transit_node));
				},
				false => {
					// Shipment is still in transit
					shipment.owner_index = shipment.owner_index + 1;
					let new_key = Self::gen_key();
					ShipmentUidToKey::<T>::insert(&shipment_uid, &new_key);
					UidToShipment::<T>::insert(&shipment_uid, &shipment);
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

		// fn set_fees() {}

		// fn route() {}

		// fn get_transit_nodes() {}

		// fn get_transit_status() {}

		// I think we should be looking to develop an encode decode algorithm. Public key is encoded but only the decoded private key
		// can be used to call the update function.

		// fn get_key(origin: OriginFor<T>, shipment_uid: u64) -> Result<[u8; 16], Error<T>> {
		// 	let transit_node = match ensure_signed(origin) {
		// 		Ok(val) => val,
		// 		Err(_) => return Err(Error::<T>::UnauthorizedCaller)
		// 	};
		// 	ensure!(Self::transit_nodes().contains(&transit_node), Error::<T>::UnauthorizedCaller);
		// 	ensure!(ShipmentUidToKey::<T>::contains_key(&shipment_uid), Error::<T>::UIDNotFound);

		// 	return ShipmentUidToKey::<T>::get(&shipment_uid).ok_or(Error::<T>::KeyNotFound);
		// }
	}
  }
