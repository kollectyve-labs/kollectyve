#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub mod weights;
use crate::weights::WeightInfo;

#[frame::pallet]
pub mod pallet {
    use super::*;
    use frame::prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type WeightInfo: WeightInfo;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The Bootstrapper already exist.
        BootstrapperAlreadyRegistered,
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Bootstrapper added.
        BootstrapperAdded {
            who: T::AccountId,
        },
    }

    #[pallet::storage]
    pub(super) type Bootstrappers<T: Config> =
    StorageMap<_, Blake2_128Concat, T::AccountId, bool, OptionQuery>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        // TODO: Simple Bootstrap mechanism to be improved
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_bootstrapper())]
        pub fn register_bootstrapper(
            origin: OriginFor<T>) -> DispatchResult {
            let bootstrapper = ensure_signed(origin)?;

            // Check if the account is already a bootstrapper
            ensure!(
                !Bootstrappers::<T>::contains_key(&bootstrapper),
                Error::<T>::BootstrapperAlreadyRegistered
            );

            Bootstrappers::<T>::insert(&bootstrapper, true);

            Self::deposit_event(Event::BootstrapperAdded {
                who: bootstrapper
            });

            Ok(())
        }
    }



}