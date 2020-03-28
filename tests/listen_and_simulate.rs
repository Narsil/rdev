extern crate rdev;
extern crate tokio;
use rdev::{simulate, EventType, Key};
use std::time::Duration;
use lazy_static::lazy_static;
use rdev::{listen, Event};
use std::thread;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Receiver, Sender};

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

#[tokio::test]
async fn test_listen_and_simulate() {
    // spawn new thread because listen blocks
    let _listener = thread::spawn(move || {
        listen(send_event).expect("Could not listen");
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
    let timeout =  Duration::from_secs(1);
    match recv.recv_timeout(timeout){
        Ok(event1) => {
            assert_eq!(event1.event_type, event_type);
        }
        Err(err) => assert!(false, format!("Timeout error : {:?}", err))
    }
    match recv.recv_timeout(timeout){
        Ok(event2) => {
            assert_eq!(event2.event_type, event_type2);
        }
        Err(err) => assert!(false, format!("Timeout error : {:?}", err))
    }


}
