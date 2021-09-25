use hur;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    hur::handle_arguments(&args);
}