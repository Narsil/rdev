use rdev::Key;

#[test]
fn convert_to_key() {
    let key0 = "Kp0";
    let dot = "Dot";
    let kpmultiply = "KpMultiply";
    let no_key = "abraca";
    assert_eq!(Key::from_str(key0).unwrap(), Key::Kp0);
    assert_eq!(Key::from_str(dot).unwrap(), Key::Dot);
    assert_eq!(Key::from_str(kpmultiply).unwrap(), Key::KpMultiply);
    assert_eq!(Key::from_str(no_key), None);
}
