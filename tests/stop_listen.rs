use rdev::{listen, stop_listen};
use serial_test::serial;
use std::{
    thread::{self, spawn},
    time::Duration,
};
#[test]
#[serial]
fn test_stop() {
    eprintln!("hello");
    spawn(|| {
        if let Err(error) = listen(|event| {
            println!("My callback {:?}", event);
        }) {
            println!("Error: {:?}", error)
        }
    });
    thread::sleep(Duration::from_secs(5));
    spawn(|| {
        stop_listen();
    });
    thread::sleep(Duration::from_secs(5));
}
