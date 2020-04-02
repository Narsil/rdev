use lazy_static::lazy_static;
use rdev::{grab, listen, simulate, Event, EventType, Key};
use serial_test::serial;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref EVENT_CHANNEL: (Mutex<Sender<Event>>, Mutex<Receiver<Event>>) = {
        let (send, recv) = channel();
        (Mutex::new(send), Mutex::new(recv))
    };
}

fn send_event(event: Event) {
    EVENT_CHANNEL
        .0
        .lock()
        .expect("Failed to unlock Mutex")
        .send(event)
        .expect("Receiving end of EVENT_CHANNEL was closed");
}

fn grab_tab(event: Event) -> Option<Event> {
    match event.event_type {
        EventType::KeyPress(Key::Tab) => None,
        _ => Some(event),
    }
}

#[test]
#[serial]
fn test_grab() {
    // spawn new thread because listen blocks
    let _listener = thread::spawn(move || {
        listen(send_event).expect("Could not listen");
    });
    // Make sure grab ends up on top of listen so it can properly discard.
    thread::sleep(Duration::from_secs(1));
    let _grab = thread::spawn(move || {
        grab(grab_tab).expect("Could not listen");
    });

    let recv = EVENT_CHANNEL.1.lock().expect("Failed to unlock Mutex");

    // Wait for listen to start
    thread::sleep(Duration::from_secs(1));

    let event_type = EventType::KeyPress(Key::KeyS);
    let event_type2 = EventType::KeyRelease(Key::KeyS);
    let result = simulate(&event_type);
    assert!(result.is_ok());
    let result = simulate(&event_type2);
    assert!(result.is_ok());
    let timeout = Duration::from_secs(1);
    match recv.recv_timeout(timeout) {
        Ok(event1) => assert_eq!(event1.event_type, event_type),
        Err(err) => panic!("{:?}", err),
    }
    match recv.recv_timeout(timeout) {
        Ok(event2) => assert_eq!(event2.event_type, event_type2),
        Err(err) => panic!("{:?}", err),
    }
    let tab = EventType::KeyPress(Key::Tab);
    let result = simulate(&tab);
    assert!(result.is_ok());
    match recv.recv_timeout(timeout) {
        Ok(event) => panic!("We should not receive event : {:?}", event),
        Err(err) => assert_eq!(err, RecvTimeoutError::Timeout),
    }
}
