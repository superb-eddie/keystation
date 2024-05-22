fn main() {
    println!(
        "{}",
        std::env::var("TARGET").unwrap()
    );
}