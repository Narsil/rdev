use rdev::display_size;
fn main() {
    let (w, h) = display_size();

    println!("Your screen is {:?}x{:?}", w, h);
}
