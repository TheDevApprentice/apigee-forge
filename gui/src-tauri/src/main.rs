fn main() {
    if let Err(error) = gui_lib::run() {
        eprintln!("failed to start Apigee Forge: {error}");
        std::process::exit(1);
    }
}
