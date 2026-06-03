fn main() {
    let mut key = String::new();
    println!("Paste your base32 encoded secret key");
    std::io::stdin()
        .read_line(&mut key)
        .expect("Failed to read key from stdin");
    let key = if let Some(key) = totp6238::Key::from_base32(&key) {
        key
    } else {
        println!("not vaild base32 text");
        return;
    };
    let config = Default::default();
    let code = totp6238::totp_now(&key, config)
        .expect("Can only fail if asked for code before unix epoch, or 17,000 eons in the future");
    assert!(code.is_valid());
    println!("{code}");
}
