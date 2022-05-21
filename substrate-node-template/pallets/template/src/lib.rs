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
	  //type AccountIdOf<T> = <T as system::Trait>::AccountId;

	  //Shipment Struct

	  #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	  #[scale_info(skip_type_params(T))]
	  pub struct Shipment<T: Config> {
		pub creator: T::AccountId,
		owner: T::AccountId,
		pub fees: Option<BalanceOf<T>>,
		pub status: ShipmentStatus,
		pub route: BoundedVec<T::AccountId,T::MaxSize>,
	  }

	  // Shipment Status enum
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

      // TODO: Update the `config` block below
      #[pallet::config]
      pub trait Config: frame_system::Config {
          type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		  type Currency: Currency<Self::AccountId>;
		  type MaxSize: Get<u32>;
      }

      // TODO: Update the `event` block below
      #[pallet::event]
      #[pallet::generate_deposit(pub(super) fn deposit_event)]
      pub enum Event<T: Config> {
		  /// Event emitted when a new Transit Point is created
		  TransitPointCreated(T::AccountId),
		  /// Event emitted when a Transit Point is removed
		  TransitPointRemoved(T::AccountId),
		  /// Event emitted when a new shipment is created
		  ShipmentInit(T::AccountId),
		  /// Event emitted when shipment keys and owners are updated
		  ShipmentUpdated(T::AccountId),
		  /// Event emitted when shipment is received at the destination
		  ShipmentReceived(T::AccountId),
	  }

      // TODO: Update the `error` block below
      #[pallet::error]
      pub enum Error<T> {
		  /// Transit Point already exists
		  TransitPointExists,
		  /// Transit Point does not exist
		  TransitPointNotFound,
		  /// Not Authorized to Create Shipment
		  UnAuthorizedCaller,
	  }

      // TODO: add #[pallet::storage] block

	  #[pallet::storage]
	  pub(super) type TransitNodes<T:Config>=StorageMap<
	      _,
		  Blake2_128Concat,
		  T::AccountId,
		  u8,
		  OptionQuery,
	  >;

	  #[pallet::storage]
	  pub(super) type NodeUID<T:Config>=StorageValue<
	      _,
		  u8,
		  ValueQuery,
	  >;

	  #[pallet::storage]
	  pub(super) type ShipmentUID<T:Config>=StorageValue<
	      _,
		  u64,
		  ValueQuery,
	  >;

      // TODO: Update the `call` block below
      #[pallet::call]
      impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_new_transit_node(origin: OriginFor<T>, transit_node: T::AccountId)
		-> DispatchResult {
			ensure_root(origin)?;

			ensure!(!TransitNodes::<T>::contains_key(&transit_node), Error::<T>::TransitPointNotFound);

			let ncount = NodeUID::<T>::get();
			let ncountp1 = ncount.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			TransitNodes::<T>::insert(&transit_node,&ncountp1);
			NodeUID::<T>::put(ncountp1);

			Self::deposit_event(Event::TransitPointCreated(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_transit_node(origin: OriginFor<T>, transit_node: T::AccountId)
		-> DispatchResult {
			ensure_root(origin)?;

			ensure!(TransitNodes::<T>::contains_key(&transit_node), Error::<T>::TransitPointExists);

			TransitNodes::<T>::remove(&transit_node);

			Self::deposit_event(Event::TransitPointRemoved(transit_node));
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn create_shipment(origin: OriginFor<T>,route_vec: BoundedVec<T::AccountId,T::MaxSize>) 
		-> DispatchResult {
			let who = ensure_signed(origin)?;

			ensure!(TransitNodes::<T>::contains_key(&who), Error::<T>::UnAuthorizedCaller);

			// More checks needed ?

			// Set uid

			let ncount = ShipmentUID::<T>::get();
			let ncountp1 = ncount.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			let index = 1;
			let next_owner = route_vec.get(index);

			let shipment = Shipment::<T> {creator: who, owner: next_owner, fees: None, status: ShipmentStatus::InTransit,route: route_vec};

			// Set caller as the creator

			// Set the route

			// Calculate Shipment fees based on the route

			// Create the shipment

			//create Restricted Key => Create a type first so the shipment uid maps to the type (TO-DO)

			//Set next Transit Node as the owner

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn update_shipment(origin: OriginFor<T>,uid: u64 )
		-> DispatchResult {
			// This function will take a key parameter but we
			// don't know what the type will be. I'm working on it
			let who = ensure_signed(origin)?;

			ensure!(TransitNodes::<T>::contains_key(&who), Error::<T>::UnAuthorizedCaller);

			// Check if the shipment owner is the one who is calling the function. Transaction fails otherwise

			// Match the input key with the one stored on the blockchain (Not implemented yet. see the first comment in this function)

			// Check if the current transit node is the destination. If it is,  change the status of the shipment.

			// Otherwise , generate a new key and a new owner

			Ok(())
		}

		// More functions

		// Fees

		// Routing
		
		// Key generation functions

		// Getter functions for the transit nodes

		// Getter functions for the customers
	}

  }