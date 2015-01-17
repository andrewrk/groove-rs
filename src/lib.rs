#![allow(unstable)]
#![allow(missing_copy_implementations)]
extern crate libc;

use std::str::Utf8Error;
use std::option::Option;
use std::result::Result;
use libc::{c_int, uint64_t, c_char, c_void, c_double};
use std::ffi::CString;

#[link(name="groove")]
extern {
    fn groove_init() -> c_int;
    fn groove_finish();
    fn groove_set_logging(level: c_int);
    fn groove_channel_layout_count(channel_layout: uint64_t) -> c_int;
    fn groove_channel_layout_default(count: c_int) -> uint64_t;
    fn groove_sample_format_bytes_per_sample(format: c_int) -> c_int;
    fn groove_version_major() -> c_int;
    fn groove_version_minor() -> c_int;
    fn groove_version_patch() -> c_int;
    fn groove_version() -> *const c_char;

    fn groove_file_open(filename: *const c_char) -> *mut GrooveFile;
    fn groove_file_close(file: *mut GrooveFile);
    fn groove_file_duration(file: *mut GrooveFile) -> c_double;
    fn groove_file_metadata_get(file: *mut GrooveFile, key: *const c_char,
                                prev: *const c_void, flags: c_int) -> *mut c_void;
    fn groove_file_metadata_set(file: *mut GrooveFile, key: *const c_char,
                                value: *const c_char, flags: c_int) -> c_int;
    fn groove_file_save(file: *mut GrooveFile) -> c_int;

    fn groove_tag_key(tag: *mut c_void) -> *const c_char;
    fn groove_tag_value(tag: *mut c_void) -> *const c_char;

    fn groove_playlist_create() -> *mut GroovePlaylist;
    fn groove_playlist_insert(playlist: *mut GroovePlaylist, file: *mut GrooveFile,
                              gain: c_double, peak: c_double,
                              next: *mut GroovePlaylistItem) -> *mut GroovePlaylistItem;
    fn groove_playlist_destroy(playlist: *mut GroovePlaylist);

    fn groove_encoder_create() -> *mut GrooveEncoder;
    fn groove_encoder_destroy(encoder: *mut GrooveEncoder);
}

/// all fields are read-only. modify with methods
#[repr(C)]
struct GroovePlaylistItem {
    file: *mut GrooveFile,

    gain: c_double,
    peak: c_double,

    /// A GroovePlaylist is a doubly linked list. Use these fields to
    /// traverse the list.
    prev: *mut GroovePlaylistItem,
    next: *mut GroovePlaylistItem,
}

pub struct PlaylistItem {
    groove_playlist_item: *mut GroovePlaylistItem,
}

impl PlaylistItem {
    /// A volume adjustment in float format to apply to the file when it plays.
    /// This is typically used for loudness compensation, for example ReplayGain.
    /// To convert from dB to float, use exp(log(10) * 0.05 * dB_value)
    pub fn gain(&self) -> f64 {
        unsafe {
            (*self.groove_playlist_item).gain
        }
    }

    /// The sample peak of this playlist item is assumed to be 1.0 in float
    /// format. If you know for certain that the peak is less than 1.0, you
    /// may set this value which may allow the volume adjustment to use
    /// a pure amplifier rather than a compressor. This results in slightly
    /// better audio quality.
    pub fn peak(&self) -> f64 {
        unsafe {
            (*self.groove_playlist_item).peak
        }
    }
}

/// a GroovePlaylist keeps its sinks full.
/// all fields are read-only. modify using methods.
#[repr(C)]
struct GroovePlaylist {
    /// doubly linked list which is the playlist
    head: *mut GroovePlaylistItem,
    tail: *mut GroovePlaylistItem,

    gain: c_double,
}

/// a playlist keeps its sinks full.
pub struct Playlist {
    groove_playlist: *mut GroovePlaylist,
}
impl Drop for Playlist {
    fn drop(&mut self) {
        unsafe { groove_playlist_destroy(self.groove_playlist) }
    }
}

impl Playlist {
    pub fn new() -> Self {
        unsafe {
            Playlist { groove_playlist: groove_playlist_create() }
        }
    }

    /// volume adjustment in float format which applies to all playlist items
    /// and all sinks. defaults to 1.0.
    pub fn gain(&self) -> f64 {
        unsafe {
            (*self.groove_playlist).gain
        }
    }

    pub fn iter(&self) -> PlaylistIterator {
        unsafe {
            PlaylistIterator { curr: (*self.groove_playlist).head }
        }
    }

    /// once you add a file to the playlist, you must not destroy it until you first
    /// remove it from the playlist.
    /// gain: see PlaylistItem. use 1.0 for no adjustment.
    /// peak: see PlaylistItem. use 1.0 for no adjustment.
    /// returns the newly created playlist item.
    pub fn append(&self, file: &File, gain: f64, peak: f64) -> PlaylistItem {
        unsafe {
            let inserted_item = groove_playlist_insert(self.groove_playlist, file.groove_file,
                                                       gain, peak, std::ptr::null_mut());
            if inserted_item.is_null() {
                panic!("out of memory");
            } else {
                PlaylistItem {groove_playlist_item: inserted_item}
            }
        }
    }

    /// once you add a file to the playlist, you must not destroy it until you first
    /// remove it from the playlist.
    /// before: the item to insert before.
    /// gain: see Groove. use 1.0 for no adjustment.
    /// peak: see Groove. use 1.0 for no adjustment.
    /// returns the newly created playlist item.
    pub fn insert(&self, file: &File, gain: f64, peak: f64, before: &PlaylistItem) -> PlaylistItem {
        unsafe {
            let inserted_item = groove_playlist_insert(self.groove_playlist, file.groove_file,
                                                       gain, peak, before.groove_playlist_item);
            if inserted_item.is_null() {
                panic!("out of memory");
            } else {
                PlaylistItem {groove_playlist_item: inserted_item}
            }
        }
    }
}

pub struct PlaylistIterator {
    curr: *mut GroovePlaylistItem,
}

impl Iterator for PlaylistIterator {
    type Item = PlaylistItem;

    fn next(&mut self) -> Option<PlaylistItem> {
        unsafe {
            if self.curr.is_null() {
                Option::None
            } else {
                let prev = self.curr;
                self.curr = (*self.curr).next;
                Option::Some(PlaylistItem {groove_playlist_item: prev})
            }
        }
    }
}

#[repr(C)]
struct GrooveFile {
    dirty: c_int,
    filename: *const c_char,
}

pub struct File {
    groove_file: *mut GrooveFile,
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe { groove_file_close(self.groove_file) }
    }
}

impl File {
    pub fn filename(&self) -> Path {
        unsafe {
            let slice = std::ffi::c_str_to_bytes(&(*self.groove_file).filename);
            Path::new(slice)
        }
    }
    /// whether the file has pending edits
    pub fn is_dirty(&self) -> bool {
        unsafe {
            (*self.groove_file).dirty == 1
        }
    }
    /// main audio stream duration in seconds. note that this relies on a
    /// combination of format headers and heuristics. It can be inaccurate.
    /// The most accurate way to learn the duration of a file is to use
    /// GrooveLoudnessDetector
    pub fn duration(&self) -> f64 {
        unsafe {
            groove_file_duration(self.groove_file)
        }
    }

    pub fn metadata_get(&self, key: &str, case_sensitive: bool) -> Option<Tag> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::from_slice(key.as_bytes());
        unsafe {
            let tag = groove_file_metadata_get(self.groove_file, c_tag_key.as_ptr(),
                                               std::ptr::null(), flags);
            if tag.is_null() {
                Option::None
            } else {
                Option::Some(Tag {groove_tag: tag})
            }
        }
    }

    pub fn metadata_iter(&self) -> MetadataIterator {
        MetadataIterator { file: self, curr: std::ptr::null() }
    }

    pub fn metadata_set(&self, key: &str, value: &str, case_sensitive: bool) -> Result<(), i32> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::from_slice(key.as_bytes());
        let c_tag_value = CString::from_slice(value.as_bytes());
        unsafe {
            let err_code = groove_file_metadata_set(self.groove_file, c_tag_key.as_ptr(),
                                                    c_tag_value.as_ptr(), flags);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    pub fn metadata_delete(&self, key: &str, case_sensitive: bool) -> Result<(), i32> {
        let flags: c_int = if case_sensitive {TAG_MATCH_CASE} else {0};
        let c_tag_key = CString::from_slice(key.as_bytes());
        unsafe {
            let err_code = groove_file_metadata_set(self.groove_file, c_tag_key.as_ptr(),
                                                    std::ptr::null(), flags);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }

    /// write changes made to metadata to disk.
    pub fn save(&self) -> Result<(), i32> {
        unsafe {
            let err_code = groove_file_save(self.groove_file);
            if err_code >= 0 {
                Result::Ok(())
            } else {
                Result::Err(err_code as i32)
            }
        }
    }
}

pub struct MetadataIterator<'a> {
    file: &'a File,
    curr: *const c_void,
}

impl<'a> Iterator for MetadataIterator<'a> {
    type Item = Tag<'a>;
    fn next(&mut self) -> Option<Tag> {
        let c_tag_key = CString::from_slice("".as_bytes());
        unsafe {
            let tag = groove_file_metadata_get(self.file.groove_file, c_tag_key.as_ptr(),
                                               self.curr, 0);
            self.curr = tag;
            if tag.is_null() {
                Option::None
            } else {
                Option::Some(Tag {groove_tag: tag})
            }
        }
    }
}

pub enum Log {
    Quiet,
    Error,
    Warning,
    Info,
}
impl Copy for Log {}


pub enum ChannelLayout {
    FrontLeft,
    FrontRight,
    FrontCenter,
    LayoutMono,
    LayoutStereo,
}
impl Copy for ChannelLayout {}

const CH_FRONT_LEFT    :uint64_t = 0x00000001;
const CH_FRONT_RIGHT   :uint64_t = 0x00000002;
const CH_FRONT_CENTER  :uint64_t = 0x00000004;
const CH_LAYOUT_MONO   :uint64_t = CH_FRONT_CENTER;
const CH_LAYOUT_STEREO :uint64_t = CH_FRONT_LEFT|CH_FRONT_RIGHT;

impl ChannelLayout {
    /// get the default channel layout based on the channel count
    pub fn default(count: i32) -> Self {
        let x = unsafe { groove_channel_layout_default(count) };
        ChannelLayout::from_groove(x)
    }

    /// Get the channel count for the channel layout
    pub fn count(&self) -> i32 {
        unsafe { groove_channel_layout_count(self.to_groove()) as i32 }
    }

    fn to_groove(&self) -> uint64_t {
        match *self {
            ChannelLayout::FrontLeft    => CH_FRONT_LEFT,
            ChannelLayout::FrontRight   => CH_FRONT_RIGHT,
            ChannelLayout::FrontCenter  => CH_FRONT_CENTER,
            ChannelLayout::LayoutMono   => CH_LAYOUT_MONO,
            ChannelLayout::LayoutStereo => CH_LAYOUT_STEREO,
        }
    }

    fn from_groove(x: uint64_t) -> Self {
        match x {
            CH_FRONT_LEFT     => ChannelLayout::FrontLeft,
            CH_FRONT_RIGHT    => ChannelLayout::FrontRight,
            CH_FRONT_CENTER   => ChannelLayout::FrontCenter,
            CH_LAYOUT_STEREO  => ChannelLayout::LayoutStereo,
            _                 => panic!("invalid channel layout"),
        }
    }
}

const SAMPLE_FMT_NONE: i32 = -1;
const SAMPLE_FMT_U8:   i32 =  0;
const SAMPLE_FMT_S16:  i32 =  1;
const SAMPLE_FMT_S32:  i32 =  2;
const SAMPLE_FMT_FLT:  i32 =  3;
const SAMPLE_FMT_DBL:  i32 =  4;

const SAMPLE_FMT_U8P:  i32 =  5;
const SAMPLE_FMT_S16P: i32 =  6;
const SAMPLE_FMT_S32P: i32 =  7;
const SAMPLE_FMT_FLTP: i32 =  8;
const SAMPLE_FMT_DBLP: i32 =  9;

/// how to organize bits which represent audio samples
pub struct SampleFormat {
    sample_type: SampleType,
    /// planar means non-interleaved
    planar: bool,
}
impl Copy for SampleFormat {}

pub enum SampleType {
    NoType,
    /// unsigned 8 bits
    U8,
    /// signed 16 bits
    S16,
    /// signed 32 bits
    S32,
    /// float (32 bits)
    Flt,
    /// double (64 bits)
    Dbl,
}
impl Copy for SampleType {}

impl SampleFormat {
    fn to_groove(&self) -> i32 {
        match (self.sample_type, self.planar) {
            (SampleType::NoType, false) => SAMPLE_FMT_NONE,
            (SampleType::U8,     false) => SAMPLE_FMT_U8,
            (SampleType::S16,    false) => SAMPLE_FMT_S16,
            (SampleType::S32,    false) => SAMPLE_FMT_S32,
            (SampleType::Flt,    false) => SAMPLE_FMT_FLT,
            (SampleType::Dbl,    false) => SAMPLE_FMT_DBL,

            (SampleType::NoType, true)  => SAMPLE_FMT_NONE,
            (SampleType::U8,     true)  => SAMPLE_FMT_U8P,
            (SampleType::S16,    true)  => SAMPLE_FMT_S16P,
            (SampleType::S32,    true)  => SAMPLE_FMT_S32P,
            (SampleType::Flt,    true)  => SAMPLE_FMT_FLTP,
            (SampleType::Dbl,    true)  => SAMPLE_FMT_DBLP,
        }
    }

    fn from_groove(groove_sample_format: i32) -> SampleFormat {
        match groove_sample_format {
            SAMPLE_FMT_NONE => SampleFormat { sample_type: SampleType::NoType, planar: false },
            SAMPLE_FMT_U8   => SampleFormat { sample_type: SampleType::U8,     planar: false },
            SAMPLE_FMT_S16  => SampleFormat { sample_type: SampleType::S16,    planar: false },
            SAMPLE_FMT_S32  => SampleFormat { sample_type: SampleType::S32,    planar: false },
            SAMPLE_FMT_FLT  => SampleFormat { sample_type: SampleType::Flt,    planar: false },
            SAMPLE_FMT_DBL  => SampleFormat { sample_type: SampleType::Dbl,    planar: false },

            SAMPLE_FMT_U8P  => SampleFormat { sample_type: SampleType::U8,     planar: true },
            SAMPLE_FMT_S16P => SampleFormat { sample_type: SampleType::S16,    planar: true },
            SAMPLE_FMT_S32P => SampleFormat { sample_type: SampleType::S32,    planar: true },
            SAMPLE_FMT_FLTP => SampleFormat { sample_type: SampleType::Flt,    planar: true },
            SAMPLE_FMT_DBLP => SampleFormat { sample_type: SampleType::Dbl,    planar: true },

            _ => panic!("invalid sample format value"),
        }
    }

    pub fn bytes_per_sample(&self) -> i32 {
        unsafe { groove_sample_format_bytes_per_sample(self.to_groove()) }
    }
}

pub struct Tag<'a> {
    groove_tag: *mut c_void,
}

impl<'a> Tag<'a> {
    pub fn key(&self) -> Result<&'a str, Utf8Error> {
        unsafe {
            let key_c_str = groove_tag_key(self.groove_tag);
            let slice = std::ffi::c_str_to_bytes(&key_c_str);
            match std::str::from_utf8(slice) {
                Result::Ok(s) => Result::Ok(std::mem::transmute::<&str, &'a str>(s)),
                Result::Err(err) => Result::Err(err),
            }
        }
    }
    pub fn value(&self) -> Result<&'a str, Utf8Error> {
        unsafe {
            let val_c_str = groove_tag_value(self.groove_tag);
            let slice = std::ffi::c_str_to_bytes(&val_c_str);
            match std::str::from_utf8(slice) {
                Result::Ok(s) => Result::Ok(std::mem::transmute::<&str, &'a str>(s)),
                Result::Err(err) => Result::Err(err),
            }
        }
    }
}

#[repr(C)]
struct GrooveAudioFormat {
    sample_rate: c_int,
    channel_layout: uint64_t,
    sample_fmt: c_int,
}

pub struct AudioFormat {
    sample_rate: i32,
    channel_layout: ChannelLayout,
    sample_fmt: SampleFormat,
}
impl Copy for AudioFormat {}

impl AudioFormat {
    fn from_groove(groove_audio_format: &GrooveAudioFormat) -> Self {
        AudioFormat {
            sample_rate: groove_audio_format.sample_rate as i32,
            channel_layout: ChannelLayout::from_groove(groove_audio_format.channel_layout),
            sample_fmt: SampleFormat::from_groove(groove_audio_format.sample_fmt),
        }
    }
    fn to_groove(&self) -> GrooveAudioFormat {
        GrooveAudioFormat {
            sample_rate: self.sample_rate as c_int,
            channel_layout: self.channel_layout.to_groove(),
            sample_fmt: self.sample_fmt.to_groove(),
        }
    }
}

#[repr(C)]
struct GrooveEncoder {
    target_audio_format: GrooveAudioFormat,
    bit_rate: c_int,
    format_short_name: *const c_char,
    codec_short_name: *const c_char,
    filename: *const c_char,
    mime_type: *const c_char,

    /// how big the sink buffer should be, in sample frames.
    /// groove_encoder_create defaults this to 8192
    sink_buffer_size: c_int,

    /// how big the encoded audio buffer should be, in bytes
    /// groove_encoder_create defaults this to 16384
    encoded_buffer_size: c_int,

    /// This volume adjustment to make to this player.
    /// It is recommended that you leave this at 1.0 and instead adjust the
    /// gain of the underlying playlist.
    /// If you want to change this value after you have already attached the
    /// sink to the playlist, you must use groove_encoder_set_gain.
    /// float format. Defaults to 1.0
    gain: c_double,

    /// read-only. set when attached and cleared when detached
    playlist: *mut GroovePlaylist,

    actual_audio_format: GrooveAudioFormat,
}

/// attach an Encoder to a playlist to keep a buffer of encoded audio full.
/// for example you could use it to implement an http audio stream
pub struct Encoder {
    groove_encoder: *mut GrooveEncoder,
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe { groove_encoder_destroy(self.groove_encoder) }
    }
}

impl Encoder {
    pub fn new() -> Self {
        unsafe {
            Encoder { groove_encoder: groove_encoder_create() }
        }
    }

    /// The desired audio format to encode.
    /// groove_encoder_create defaults these to 44100 Hz,
    /// signed 16-bit int, stereo.
    /// These are preferences; if a setting cannot be used, a substitute will be
    /// used instead. actual_audio_format is set to the actual values.
    pub fn set_target_audio_format(&self, target_audio_format: AudioFormat) {
        unsafe {
            (*self.groove_encoder).target_audio_format = target_audio_format.to_groove();
        }
    }
    pub fn get_target_audio_format(&self) -> AudioFormat {
        unsafe {
            AudioFormat::from_groove(&(*self.groove_encoder).target_audio_format)
        }
    }

    /// Select encoding quality by choosing a target bit rate in bits per
    /// second. Note that typically you see this expressed in "kbps", such
    /// as 320kbps or 128kbps. Surprisingly, in this circumstance 1 kbps is
    /// 1000 bps, *not* 1024 bps as you would expect.
    /// groove_encoder_create defaults this to 256000
    pub fn set_bit_rate(&self, rate: i32) {
        unsafe {
            (*self.groove_encoder).bit_rate = rate;
        }
    }
    pub fn get_bit_rate(&self) -> i32 {
        unsafe {
            (*self.groove_encoder).bit_rate
        }
    }

    /// optional - choose a short name for the format
    /// to help libgroove guess which format to use
    /// use `avconv -formats` to get a list of possibilities
    pub fn set_format_short_name(&self, format: &str) {
        let format_c_str = CString::from_slice(format.as_bytes());
        unsafe {
            (*self.groove_encoder).format_short_name = format_c_str.as_ptr();
        }
    }

    /// optional - choose a short name for the codec
    /// to help libgroove guess which codec to use
    /// use `avconv -codecs` to get a list of possibilities
    pub fn set_codec_short_name(&self, codec: &str) {
        let codec_c_str = CString::from_slice(codec.as_bytes());
        unsafe {
            (*self.groove_encoder).codec_short_name = codec_c_str.as_ptr();
        }
    }

    /// optional - provide an example filename
    /// to help libgroove guess which format/codec to use
    pub fn set_filename(&self, filename: &str) {
        let filename_c_str = CString::from_slice(filename.as_bytes());
        unsafe {
            (*self.groove_encoder).filename = filename_c_str.as_ptr();
        }
    }

    /// optional - provide a mime type string
    /// to help libgroove guess which format/codec to use
    pub fn set_mime_type(&self, mime_type: &str) {
        let mime_type_c_str = CString::from_slice(mime_type.as_bytes());
        unsafe {
            (*self.groove_encoder).mime_type = mime_type_c_str.as_ptr();
        }
    }

    /// set to the actual format you get when you attach to a
    /// playlist. ideally will be the same as target_audio_format but might
    /// not be.
    pub fn get_actual_audio_format(&self) -> AudioFormat {
        unsafe {
            AudioFormat::from_groove(&(*self.groove_encoder).actual_audio_format)
        }
    }
}

/// call once at the beginning of your program from the main thread
/// returns 0 on success, < 0 on error
pub fn init() -> isize {
    unsafe { groove_init() as isize }
}

/// call at the end of your program to clean up. after calling this
/// you may no longer use this API.
pub fn finish() {
    unsafe { groove_finish() }
}

/// enable/disable logging of errors
pub fn set_logging(level: Log) {
    let c_level: c_int = match level {
        Log::Quiet   => -8,
        Log::Error   => 16,
        Log::Warning => 24,
        Log::Info    => 32,
    };
    unsafe { groove_set_logging(c_level) }
}

pub fn version_major() -> i32 {
    unsafe { groove_version_major() }
}

pub fn version_minor() -> i32 {
    unsafe { groove_version_minor() }
}

pub fn version_patch() -> i32 {
    unsafe { groove_version_patch() }
}

/// get a string which represents the version number of libgroove
pub fn version() -> &'static str {
    unsafe {
        let version = groove_version();
        let slice = std::ffi::c_str_to_bytes(&version);
        std::mem::transmute::<&str, &'static str>(std::str::from_utf8(slice).unwrap())
    }
}

/// open a file on disk and prepare to stream audio from it
pub fn file_open(filename: &str) -> Option<File> {
    let c_filename = CString::from_slice(filename.as_bytes());
    unsafe {
        let groove_file = groove_file_open(c_filename.as_ptr());
        match groove_file.is_null() {
            true  => Option::None,
            false => Option::Some(File { groove_file: groove_file }),
        }
    }
}

const TAG_MATCH_CASE: c_int = 1;
