#![cfg_attr(not(feature = "std"), no_std)]

mod types;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_support::{
        sp_runtime::traits::AccountIdConversion,
        traits::{Currency, ExistenceRequirement, Get},
        PalletId,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::convert::TryInto;

    use crate::types::*;

    pub type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        #[pallet::constant]
        type BlocksPerWeek: Get<u32>;

        /// The payment escrow's pallet id
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// Minimum deposit required when renting a resource
        #[pallet::constant]
        type MinimumDeposit: Get<BalanceOf<Self>>;
    }

    #[pallet::storage]
    pub(super) type Providers<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ProviderInfo<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    pub(super) type Resources<T: Config> =
        StorageMap<_, Blake2_128Concat, ResourceId, Resource<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    pub(super) type NextResourceId<T: Config> = StorageValue<_, ResourceId, ValueQuery>;

    #[pallet::storage]
    pub(super) type Rentals<T: Config> =
        StorageMap<_, Blake2_128Concat, ResourceId, Rental<T::AccountId>, OptionQuery>;

    // Track category counts per provider
    #[pallet::storage]
    pub(super) type ProviderResourceCount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        ResourceCategory,
        u32,
        ValueQuery,
    >;

    #[pallet::storage]
    pub type Deposits<T: Config> =
        StorageMap<_, Blake2_128Concat, ResourceId, BalanceOf<T>, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        ProviderRegistered {
            who: T::AccountId,
        },
        ProviderStatusChanged {
            who: T::AccountId,
            status: ProviderStatus,
        },
        ResourceRegistered {
            resource_id: u32,
            provider: T::AccountId,
        },
        ResourceRented {
            resource_id: u32,
            renter: T::AccountId,
        },
        RentalCancelled {
            resource_id: u32,
            renter: T::AccountId,
        },
        PaymentClaimed {
            resource_id: u32,
            provider: T::AccountId,
            amount: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        ProviderNameTooLong,
        ProviderWebsiteTooLong,
        ProviderAlreadyRegistered,
        ProviderNotRegistered,
        ResourceNotFound,
        ResourceNotAvailable,
        NotResourceOwner,
        BlockNumberOverflow,
        NotRenter,
        RentalNotFound,
        NoPaymentDue,
        InsufficientDeposit,
        ConversionError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({20000})]
        pub fn register_provider(
            origin: OriginFor<T>,
            name: ProviderName,
            website: Option<Website>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                !Providers::<T>::contains_key(&who),
                Error::<T>::ProviderAlreadyRegistered
            );

            ensure!(name.len() <= 50, Error::<T>::ProviderNameTooLong);

            ensure!(
                website.as_ref().map_or(true, |w| w.len() <= 100),
                Error::<T>::ProviderWebsiteTooLong
            );

            let current_block = frame_system::Pallet::<T>::block_number();

            let current_block_u32: u32 = current_block
                .try_into()
                .map_err(|_| Error::<T>::BlockNumberOverflow)?;

            // Create provider info
            let provider_info = ProviderInfo {
                account: who.clone(),
                name: name.clone(),
                website,
                total_resources: 0,
                reputation_score: Default::default(),
                registration_block: current_block_u32,
                last_updated: current_block_u32,
                status: ProviderStatus::Active,
            };

            Providers::<T>::insert(&who, provider_info);

            Self::deposit_event(Event::ProviderRegistered { who: who.clone() });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight({15000})]
        pub fn update_provider_status(
            origin: OriginFor<T>,
            status: ProviderStatus,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Providers::<T>::try_mutate(&who, |maybe_provider| -> DispatchResult {
                let provider = maybe_provider
                    .as_mut()
                    .ok_or(Error::<T>::ProviderNotRegistered)?;

                provider.status = status.clone();

                let current_block = frame_system::Pallet::<T>::block_number();

                let current_block_u32: u32 = current_block
                    .try_into()
                    .map_err(|_| Error::<T>::BlockNumberOverflow)?;

                provider.last_updated = current_block_u32;

                Self::deposit_event(Event::ProviderStatusChanged {
                    who: who.clone(),
                    status,
                });

                Ok(())
            })
        }

        #[pallet::call_index(3)]
        #[pallet::weight({20000})]
        pub fn register_resource(
            origin: OriginFor<T>,
            resource: Resource<T::AccountId>,
        ) -> DispatchResult {
            let provider = ensure_signed(origin)?;
            ensure!(
                Providers::<T>::contains_key(&provider),
                Error::<T>::ProviderNotRegistered
            );

            let resource_id = NextResourceId::<T>::get();

            ProviderResourceCount::<T>::mutate(
                provider.clone(),
                resource.category.clone(),
                |count| *count += 1,
            );

            Resources::<T>::insert(resource_id, resource);

            NextResourceId::<T>::put(resource_id + 1);

            Self::deposit_event(Event::ResourceRegistered {
                resource_id,
                provider,
            });

            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight({20000})]
        pub fn rent_resource(
            origin: OriginFor<T>,
            resource_id: u32,
            billing_period: BillingPeriod,
        ) -> DispatchResult {
            let renter = ensure_signed(origin)?;

            /*
             let resource = Resources::<T>::get(resource_id).ok_or(Error::<T>::ResourceNotFound)?;
             ensure!(resource.is_available, Error::<T>::ResourceNotAvailable);

             // Calculate required deposit (2 billing periods worth)
            // let period_cost = resource.category.price(&billing_period);

             let deposit_amount: BalanceOf<T> = (period_cost * 2)
                 .try_into()
                 .map_err(|_| Error::<T>::ConversionError)?;


             ensure!(
                 deposit_amount >= T::MinimumDeposit::get(),
                 Error::<T>::InsufficientDeposit
             );

             // Transfer deposit to escrow account
             let escrow_account = Self::escrow_account();

             T::Currency::transfer(
                 &renter,
                 &escrow_account,
                 deposit_amount,
                 ExistenceRequirement::KeepAlive,
             )?;

             // Store deposit
             Deposits::<T>::insert(resource_id, deposit_amount);

             // Create rental record
             let current_block = frame_system::Pallet::<T>::block_number();

             let current_block_u32: u32 = current_block
                 .try_into()
                 .map_err(|_| Error::<T>::BlockNumberOverflow)?;

             let rental = Rental {
                 resource_id,
                 renter: renter.clone(),
                 start_block: current_block_u32,
                 billing_period,
                 last_paid_block: current_block_u32,
                 is_active: true,
             };

             // Update resource and store rental
             Resources::<T>::insert(
                 resource_id,
                 Resource {
                     is_available: false,
                     ..resource
                 },
             );

             Rentals::<T>::insert(resource_id, rental);

             Self::deposit_event(Event::ResourceRented {
                 resource_id,
                 renter,
             });
             */
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight({10000})]
        pub fn cancel_rental(origin: OriginFor<T>, resource_id: u32) -> DispatchResult {
            let renter = ensure_signed(origin)?;

            /*
            let rental = Rentals::<T>::get(resource_id).ok_or(Error::<T>::RentalNotFound)?;
            ensure!(rental.renter == renter, Error::<T>::NotRenter);

            let current_block = frame_system::Pallet::<T>::block_number();

            let current_block_u32: u32 = current_block
                .try_into()
                .map_err(|_| Error::<T>::BlockNumberOverflow)?;

            let resource = Resources::<T>::get(resource_id).ok_or(Error::<T>::ResourceNotFound)?;

            // Calculate amount to refund (if any)
            let period_length = match rental.billing_period {
                BillingPeriod::Weekly => T::BlocksPerWeek::get(),
                BillingPeriod::Monthly => T::BlocksPerWeek::get() * 4,
            };


            let period_cost = resource.category.price(&rental.billing_period);

            let blocks_used = current_block_u32.saturating_sub(rental.last_paid_block);

            if blocks_used < period_length {
                let remaining_blocks = period_length.saturating_sub(blocks_used);

                // Reimburse unused compute amount
                let reimbursement_amount: BalanceOf<T> = (period_cost
                    .saturating_mul(remaining_blocks as u128))
                .saturating_div(period_length as u128)
                .try_into()
                .map_err(|_| Error::<T>::ConversionError)?;

                let escrow_account = Self::escrow_account();

                T::Currency::transfer(
                    &escrow_account,
                    &renter,
                    reimbursement_amount,
                    ExistenceRequirement::KeepAlive,
                )?;
            }

            // Update resource availability
            Resources::<T>::mutate(resource_id, |r| {
                if let Some(res) = r {
                    res.is_available = true;
                }
            });

            // Remove rental
            Rentals::<T>::remove(resource_id);

            Self::deposit_event(Event::RentalCancelled {
                resource_id,
                renter,
            });
            */
            Ok(())
        }

        #[pallet::call_index(6)]
        #[pallet::weight({10000})]
        pub fn claim_payment(origin: OriginFor<T>, resource_id: u32) -> DispatchResult {
            let provider = ensure_signed(origin)?;

            /*

            // Get the resource and verify ownership
            let resource = Resources::<T>::get(resource_id).ok_or(Error::<T>::ResourceNotFound)?;
            ensure!(resource.provider == provider, Error::<T>::NotResourceOwner);

            // Get the rental information
            let rental = Rentals::<T>::get(resource_id).ok_or(Error::<T>::RentalNotFound)?;
            ensure!(rental.is_active, Error::<T>::RentalNotFound);

            let current_block = frame_system::Pallet::<T>::block_number();
            let current_block_u32: u32 = current_block
                .try_into()
                .map_err(|_| Error::<T>::BlockNumberOverflow)?;

            // Calculate period length in blocks
            let period_length = match rental.billing_period {
                BillingPeriod::Weekly => T::BlocksPerWeek::get(),
                BillingPeriod::Monthly => T::BlocksPerWeek::get() * 4,
            };

            // Calculate blocks since last payment
            let blocks_since_last_payment =
                current_block_u32.saturating_sub(rental.last_paid_block);

            // Only allow claiming if at least one period has passed
            ensure!(
                blocks_since_last_payment >= period_length,
                Error::<T>::NoPaymentDue
            );

            // Calculate complete periods since last payment
            let complete_periods = blocks_since_last_payment / period_length;

            // Calculate the payment amount
            let period_cost = resource.category.price(&rental.billing_period);

            let payment_amount: BalanceOf<T> = (period_cost
                .saturating_mul(complete_periods as u128))
            .try_into()
            .map_err(|_| Error::<T>::ConversionError)?;

            // Update the last paid block
            let new_last_paid_block = rental
                .last_paid_block
                .saturating_add(complete_periods.saturating_mul(period_length));

            Rentals::<T>::try_mutate(resource_id, |maybe_rental| -> DispatchResult {
                let rental = maybe_rental.as_mut().ok_or(Error::<T>::RentalNotFound)?;
                rental.last_paid_block = new_last_paid_block;
                Ok(())
            })?;

            // Transfer payment directly from escrow to provider
            let escrow_account = Self::escrow_account();

            T::Currency::transfer(
                &escrow_account,
                &provider,
                payment_amount,
                ExistenceRequirement::KeepAlive,
            )?;

            Self::deposit_event(Event::PaymentClaimed {
                resource_id,
                provider,
                amount: payment_amount,
            });
            */
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// The account ID of the escrow
        fn escrow_account() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
