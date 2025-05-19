use crate::{mock::*, Bootstrappers, Event};
use frame::testing_prelude::*;

#[test]
fn registering_bootstrapper_event() {
    let bootstraper1: u64 = 1;

    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(KumulusBootstrap::register_bootstrapper(
            RuntimeOrigin::signed(bootstraper1)
        ));

        System::assert_last_event(Event::BootstrapperAdded { who: bootstraper1 }.into());
    });
}

#[test]
fn bootstrapper_registered() {
    let bootstraper1: u64 = 1;

    new_test_ext().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(KumulusBootstrap::register_bootstrapper(
            RuntimeOrigin::signed(bootstraper1)
        ));

        // Check if the bootsrtapper is stored
        assert_eq!(
            Bootstrappers::<Test>::get(bootstraper1).unwrap_or(false),
            true
        );

        let not_a_bootstrapper: u64 = 2;
        // Checking invalid stored bootstrapper
        assert_eq!(
            Bootstrappers::<Test>::get(not_a_bootstrapper).unwrap_or(false),
            false
        );
    });
}
