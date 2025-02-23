#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut reader = std::io::Cursor::new(data);
    _ = mp4ameta::Tag::read_from(&mut reader);
});
