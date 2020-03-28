use rdev::{listen, Event};

fn main() {
    // This will block.
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
    match event.name {
        Some(_) => (), // println!("User wrote {:?}", string),
        None => (),
    }
}
