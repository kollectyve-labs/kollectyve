#![cfg(feature = "runtime-benchmarks")]

use super::{Pallet as KumulusBootstrap, *};
use frame::deps::frame_support::assert_ok;
use frame::{deps::frame_benchmarking::v2::*, prelude::*};

#[benchmarks]
mod benchmarks {
    use super::*;
    #[cfg(test)]
    use crate::pallet::Pallet as KumulusBootstrap;
    use frame_system::RawOrigin;
    
    #[benchmark]
    fn register_bootstrapper() {
        let caller: T::AccountId = whitelisted_caller();
        
        #[extrinsic_call]
        register_bootstrapper(RawOrigin::Signed(caller.clone()));

        assert_eq!(Bootstrappers::<T>::get(caller), Some(true));
    }
    

    impl_benchmark_test_suite!(KumulusBootstrap, crate::mock::new_test_ext(), crate::mock::Test);
}