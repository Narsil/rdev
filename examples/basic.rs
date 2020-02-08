use rdev::{listen, Event};

fn main() {
    listen(callback);
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
}
