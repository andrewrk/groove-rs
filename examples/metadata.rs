#![allow(unstable)]
extern crate groove;

// read or update metadata in a media file

fn main() {
    let mut stderr = std::io::stderr();
    let args = std::os::args();
    let exe = args[0].as_slice();

    if args.len() < 2 {
        print_usage(&mut stderr, exe);
        std::os::set_exit_status(1);
        return;
    }
    let _ = writeln!(&mut stderr, "Using libgroove version v{}", groove::version());

    let filename = args[1].as_slice();
    groove::init();
    groove::set_logging(groove::Log::Info);

    {
        let file = groove::file_open(filename).expect("error opening file");
        println!("duration={}", file.duration());
    }

    groove::finish();
}

fn print_usage(stderr: &mut std::io::LineBufferedWriter<std::io::stdio::StdWriter>, exe: &str) {
    let _ = write!(stderr, "Usage: {} <file> [--update key value] [--delete key]\n", exe);
    let _ = write!(stderr, "Repeat --update and --delete as many times as you need to.\n");
}
