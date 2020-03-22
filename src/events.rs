use crate::{listen, Event};
use async_std::sync::{channel, Receiver, Sender};
use async_std::task;
use async_std::task::block_on;

lazy_static! {
    static ref MY_CHANNEL: (Sender<Event>, Receiver<Event>) = channel(1);
}

fn callback(r: Event) {
    let (sender, _) = &*MY_CHANNEL;
    block_on(sender.send(r))
}

pub async fn events() -> Receiver<Event> {
    let (_, receiver) = &*MY_CHANNEL;
    task::spawn(async move { listen(callback) });
    receiver.to_owned()
}
