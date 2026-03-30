use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn test_log_valid_control_event() {
    new_test_ext().execute_with(|| {
        System::set_block_number(1);

        let event_id = [1u8; 32];
        let rtu_id = b"RTU-001".to_vec();
        let event_name = b"BREAKER_OPEN".to_vec();
        let event_desc = b"Open main breaker at substation A".to_vec();
        let event_timestamp = 1680000000000;

        // Log the event
        assert_ok!(Dwntp::log_control_event(
            RuntimeOrigin::signed(1),
            event_id,
            rtu_id.clone(),
            event_name.clone(),
            event_desc.clone(),
            event_timestamp,
        ));

        // Verify it was stored correctly
        let stored_event = Dwntp::control_events(&event_id).expect("Event should be in storage");
        assert_eq!(stored_event.source_mtu, 1);
        assert_eq!(stored_event.rtu_id.into_inner(), rtu_id);
        assert_eq!(stored_event.event_name.into_inner(), event_name);
        assert_eq!(stored_event.event_description.into_inner(), event_desc);
        assert_eq!(stored_event.event_timestamp, event_timestamp);
        assert_eq!(stored_event.on_chain_timestamp, 1690000000000); // From MockTime

        // Verify the event was emitted
        System::assert_has_event(RuntimeEvent::Dwntp(Event::ControlEventLogged {
            event_id,
            source_mtu: 1,
        }));
    });
}

#[test]
fn test_duplicate_event_id_fails() {
    new_test_ext().execute_with(|| {
        let event_id = [2u8; 32];
        let rtu_id = b"RTU-002".to_vec();
        let event_name = b"SET_VOLTAGE".to_vec();
        let event_desc = b"Set to 240V".to_vec();
        let event_timestamp = 1680000000000;

        // First submission succeeds
        assert_ok!(Dwntp::log_control_event(
            RuntimeOrigin::signed(1),
            event_id,
            rtu_id.clone(),
            event_name.clone(),
            event_desc.clone(),
            event_timestamp,
        ));

        // Second submission with the same ID fails
        assert_noop!(
            Dwntp::log_control_event(
                RuntimeOrigin::signed(2),
                event_id,
                rtu_id,
                event_name,
                event_desc,
                event_timestamp,
            ),
            Error::<Test>::EventAlreadyExists
        );
    });
}

#[test]
fn test_rtu_id_length_limit() {
    new_test_ext().execute_with(|| {
        let event_id = [3u8; 32];
        let rtu_id = vec![b'A'; 65]; // Exceeds MaxRtuIdLen (64)
        let event_name = b"BREAKER_OPEN".to_vec();
        let event_desc = b"Description".to_vec();

        assert_noop!(
            Dwntp::log_control_event(
                RuntimeOrigin::signed(1),
                event_id,
                rtu_id,
                event_name,
                event_desc,
                1680000000000,
            ),
            Error::<Test>::InvalidRtuId
        );
    });
}

#[test]
fn test_event_name_length_limit() {
    new_test_ext().execute_with(|| {
        let event_id = [4u8; 32];
        let rtu_id = b"RTU-001".to_vec();
        let event_name = vec![b'B'; 65]; // Exceeds MaxEventNameLen (64)
        let event_desc = b"Description".to_vec();

        assert_noop!(
            Dwntp::log_control_event(
                RuntimeOrigin::signed(1),
                event_id,
                rtu_id,
                event_name,
                event_desc,
                1680000000000,
            ),
            Error::<Test>::InvalidEventName
        );
    });
}

#[test]
fn test_event_description_length_limit() {
    new_test_ext().execute_with(|| {
        let event_id = [5u8; 32];
        let rtu_id = b"RTU-001".to_vec();
        let event_name = b"BREAKER_OPEN".to_vec();
        let event_desc = vec![b'C'; 257]; // Exceeds MaxEventDescLen (256)

        assert_noop!(
            Dwntp::log_control_event(
                RuntimeOrigin::signed(1),
                event_id,
                rtu_id,
                event_name,
                event_desc,
                1680000000000,
            ),
            Error::<Test>::InvalidEventDescription
        );
    });
}

#[test]
fn test_unsigned_origin_fails() {
    new_test_ext().execute_with(|| {
        let event_id = [6u8; 32];
        let rtu_id = b"RTU-001".to_vec();
        let event_name = b"BREAKER_OPEN".to_vec();
        let event_desc = b"Description".to_vec();

        // Must be signed by an MTU
        assert_noop!(
            Dwntp::log_control_event(
                RuntimeOrigin::none(),
                event_id,
                rtu_id,
                event_name,
                event_desc,
                1680000000000,
            ),
            sp_runtime::traits::BadOrigin
        );
    });
}
