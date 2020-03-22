use rdev::events;

#[async_std::main]
async fn main() {
    let receiver = events().await;
    while let Some(event) = receiver.recv().await {
        println!("Received event : {:?}", event);
    }
}
