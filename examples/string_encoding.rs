use mp4ameta::{Data, Tag};

/// Reencode all utf-16 encoded strings in utf-8.
fn main() {
    let mut tag = Tag::read_from_path("music.m4a").expect("error reading tag");

    tag.data_mut().for_each(|(_, d)| {
        if let Data::Utf16(s) = d {
            let value = std::mem::take(s);
            *d = Data::Utf8(value);
        }
    });

    tag.write_to_path("music.m4a").expect("error writing tag");
}
