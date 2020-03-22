#[macro_use]
extern crate lazy_static;

use rdev::listen;
use async_std::task::block_on;
use async_std::task;
use async_std::sync::{channel, Receiver, Sender};

lazy_static! {
    static ref MY_CHANNEL: (Sender<i32>, Receiver<i32>) = channel(1);
}

fn callback(r: i32) {
    let (sender, _) = &*MY_CHANNEL;
    block_on(sender.send(r))
}

extern "C" fn c_callback(r: i32) {
    callback(r);
}

async fn events() -> Receiver<Event>{
    let (_, receiver) = &*MY_CHANNEL;
    task::spawn(async move {
        unsafe{
            listen(c_callback)
        }
    });
    receiver.to_owned()
}
