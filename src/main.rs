#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("coomer only runs on macOS");
    std::process::exit(1);
}

#[cfg(target_os = "macos")]
fn main() {
    if let Err(e) = coomer::run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
