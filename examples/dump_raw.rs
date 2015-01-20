#![allow(unstable)]
extern crate groove;

use std::option::Option;
use std::iter::range_step;

// dump raw audio samples to stdout

fn main() {
    let args = std::os::args_as_bytes();

    groove::set_logging(groove::Log::Info);

    let playlist = groove::Playlist::new();
    let sink = groove::Sink::new();
    sink.set_audio_format(groove::AudioFormat {
        sample_rate: 44100,
        channel_layout: groove::ChannelLayout::LayoutStereo,
        sample_fmt: groove::SampleFormat {
            sample_type: groove::SampleType::S16,  
            planar: false,
        },
    });
    sink.attach(&playlist).ok().expect("error attaching sink");

    let input_filename = args[1].as_slice();
    match groove::File::open(&Path::new(input_filename)) {
        Option::Some(file) => {
            playlist.append(&file, 1.0, 1.0);
        },
        Option::None => panic!("could not open file"),
    }

    loop {
        match sink.buffer_get_blocking() {
            Option::Some(decoded_buffer) => {
                let buf = decoded_buffer.as_slice_i16();
                for i in range_step(0, buf.len(), 2) {
                    println!("{} {}", buf[i], buf[i + 1]);
                }
            },
            Option::None => break,
        }
    }
}
