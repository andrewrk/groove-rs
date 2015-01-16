extern crate groove;

// read or update metadata in a media file

fn main() {
    groove::init();
    groove::set_logging(groove::Log::Info);
    println!("libgroove version {}", groove::version());
    groove::finish();
}
