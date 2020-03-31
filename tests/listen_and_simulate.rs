use lazy_static::lazy_static;
use rdev::{listen, simulate, Button, Event, EventType, Key};
use std::error::Error;
use std::iter::Iterator;
use std::sync::mpsc::{channel, Receiver, Sender};
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
    println!("got event: {:?}", event);
    EVENT_CHANNEL
        .0
        .lock()
        .expect("Failed to unlock Mutex")
        .send(event)
        .expect("Receiving end of EVENT_CHANNEL was closed");
}

fn sim_then_listen(events: &mut dyn Iterator<Item = EventType>) -> Result<(), Box<dyn Error>> {
    // spawn new thread because listen blocks
    let _listener = thread::spawn(move || {
        listen(send_event).expect("Could not listen");
    });
    let tenth_sec = Duration::from_millis(100);
    thread::sleep(tenth_sec);

    let recv = EVENT_CHANNEL.1.lock()?;
    for event in events {
        simulate(&event)?;
        let recieved_event = recv.recv_timeout(tenth_sec).expect("No events to recieve");
        assert_eq!(recieved_event.event_type, event);
    }
    Ok(())
}

#[test]
fn test_listen_and_simulate() -> Result<(), Box<dyn Error>> {
    let events = vec![
        EventType::KeyPress(Key::ShiftLeft),
        EventType::KeyRelease(Key::ShiftLeft),
        EventType::ButtonPress(Button::Right),
        EventType::Wheel {
            delta_x: 0,
            delta_y: 1,
        },
    ]
    .into_iter();
    let click_events = (0..480).map(|pixel| EventType::MouseMove {
        x: pixel as f64,
        y: pixel as f64,
    });
    let events = events.chain(click_events);
    sim_then_listen(&mut events.into_iter())
}
