fn main() {
    let args: Vec<String> = std::env::args().collect();
    match hur::cli::handle_args(args) {
        Ok(()) => {}
        Err(error) => eprintln!("{error}"),
    }
}
