use http;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    http::perform(&args);
}