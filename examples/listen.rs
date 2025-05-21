use rdev::{Event, listen};

fn main() {
    // This will block.
    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: Event) {
    println!("My callback {:?}", event);
}
