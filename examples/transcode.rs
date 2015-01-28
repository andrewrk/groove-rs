#![allow(unstable)]
extern crate groove;

use std::option::Option;
use std::old_io::File;

// transcode one or more files into one output file

fn main() {
    let mut stderr = std::old_io::stderr();
    let args = std::os::args();
    let exe = args[0].as_slice();

    let mut bit_rate_k = 320;
    let mut format_option = Option::None;
    let mut codec_option = Option::None;
    let mut mime_option = Option::None;
    let mut output_file_name_option = Option::None;

    groove::set_logging(groove::Log::Info);

    let playlist = groove::Playlist::new();

    let mut i = 1;
    while i < args.len() {
        let full_arg = args[i].as_slice();
        if full_arg.starts_with("--") {
            let arg = &full_arg[2..];
            if i + 1 >= args.len() {
                print_usage(&mut stderr, exe);
                std::os::set_exit_status(1);
                return;
            } else if arg == "bitrate" {
                i += 1;
                bit_rate_k = args[i].parse().unwrap();
            } else if arg == "format" {
                i += 1;
                format_option = Option::Some(args[i].as_slice());
            } else if arg == "codec" {
                i += 1;
                codec_option = Option::Some(args[i].as_slice());
            } else if arg == "mime" {
                i += 1;
                mime_option = Option::Some(args[i].as_slice());
            } else if arg == "output" {
                i += 1;
                output_file_name_option = Option::Some(args[i].as_slice());
            } else {
                print_usage(&mut stderr, exe);
                std::os::set_exit_status(1);
                return;
            }
        } else {
            match groove::File::open(&Path::new(full_arg.as_bytes())) {
                Option::Some(file) => {
                    playlist.append(&file, 1.0, 1.0);
                },
                Option::None => {
                    let _ = writeln!(&mut stderr, "Error opening input file {}", full_arg);
                    std::os::set_exit_status(1);
                    return;
                },
            }
        }
        i += 1;
    }
    let output_file_name = match output_file_name_option {
        Option::Some(file_name) => file_name,
        Option::None => {
            print_usage(&mut stderr, exe);
            std::os::set_exit_status(1);
            return;
        },
    };
    let encoder = groove::Encoder::new();
    encoder.set_bit_rate(bit_rate_k * 1000);
    match format_option {
        Option::Some(format) => encoder.set_format_short_name(format),
        _ => {},
    }
    match codec_option {
        Option::Some(codec) => encoder.set_codec_short_name(codec),
        _ => {},
    }
    match mime_option {
        Option::Some(mime) => encoder.set_mime_type(mime),
        _ => {},
    }
    encoder.set_filename(output_file_name);

    if playlist.len() == 1 {
        encoder.set_target_audio_format(playlist.first().file().audio_format());

        // copy metadata
        for tag in playlist.first().file().metadata_iter() {
            let k = tag.key().ok().unwrap();
            let v = tag.value().ok().unwrap();
            encoder.metadata_set(k, v, false).ok().expect("unable to set metadata");
        }
    }

    encoder.attach(&playlist).ok().expect("error attaching encoder");

    let mut f = match File::create(&Path::new(output_file_name)) {
        Err(_) => {
            let _ = writeln!(&mut stderr, "Error opening output file {}", output_file_name);
            std::os::set_exit_status(1);
            return;
        },
        Ok(file) => file,
    };

    loop {
        match encoder.buffer_get_blocking() {
            Option::Some(buf) => {
                f.write_all(buf.as_vec()).ok().expect("write error");
            },
            Option::None => break,
        }
    }

    groove::finish();
}

fn print_usage(stderr: &mut std::old_io::LineBufferedWriter<std::old_io::stdio::StdWriter>, exe: &str) {
    let _ = write!(stderr, "Usage: {} file1 [file2 ...] --output outputfile [--bitrate 320] [--format name] [--codec name] [--mime mimetype]\n", exe);
}
