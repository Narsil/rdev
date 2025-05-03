fn main() {
    if let Ok(val) = std::env::var("XDG_SESSION_TYPE") {
        println!("cargo:rustc-cfg=feature=\"{val}\"");
    }
}
