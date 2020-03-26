use lazy_static::lazy_static;
use rdev::listen;

lazy_static! {
    static ref EVENT_CHANNEL: (Mutex<Sender<Event>>, Mutex<Receiver<Event>>) = {
        let (send, recv) = channel();
        (Mutex::new(send), Mutex::new(recv))
    };
}

fn send_event(event: Event) {
    EVENT_CHANNEL.0
        .lock()
        .expect("Failed to unlock Mutex")
        .send(event)
        .expect("Receiving end of EVENT_CHANNEL was closed");
}

fn main() {
    // spawn new thread because listen blocks
    let _listener = thread::spawn(move || {
        listen(send_event).expect("Could not listen");
    });

    let recv = EVENT_CHANNEL.1.lock().expect("Failed to unlock Mutex");
    let events = Vec::new();
    for event in recv.iter() {
        events.push(event);
        println!("Received {} events", events.len());
    }
}
