use rdev::{listen, Event};

fn main() {
    // listen(&callback);
    listen(&|e| println!("Event {:?}", e));
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
    match event.name {
        Some(string) => println!("User wrote {:?}", string),
        None => (),
    }
}
