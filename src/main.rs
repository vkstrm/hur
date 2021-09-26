use hur;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    hur::process(&args);
}