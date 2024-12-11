
#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::storage]
	pub(super) type Providers<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, bool, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {

		/// A provider is registered.
		ProviderRegistered {
			/// The account of the newly registered provider.
			who: T::AccountId,
		},
	}


	#[pallet::error]
	pub enum Error<T> {
		/// The address is already registerered
		ProviderAlreadyRegisered
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::call_index(0)]
		#[pallet::weight({10000})]
		pub fn register_provider(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Check if the provider's address exists in the registry
			ensure!(!Providers::<T>::contains_key(&who), Error::<T>::ProviderAlreadyRegisered);

			Providers::<T>::insert(&who, true);

			Self::deposit_event(Event::ProviderRegistered { who });

			Ok(())
		}

	}
}
