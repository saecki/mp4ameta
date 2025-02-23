use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mp4ameta::{
    AdvisoryRating, ChannelConfig, Chapter, Data, Fourcc, Img, MediaType, SampleRate, Tag, Userdata,
};
use walkdir::WalkDir;

const EXTENSIONS: [&str; 6] = [".m4a", ".m4b", ".m4p", ".m4v", ".mp4", ".3gp"];

/// Allows for some rounding errors of the start duration. These issues appear when chapters are
/// written because the timescale of the file is used.
#[derive(Debug)]
struct CmpChapter {
    /// The start of the chapter.
    pub start: Duration,
    /// The title of the chapter.
    pub title: String,
}

impl CmpChapter {
    fn new(start: Duration, title: impl Into<String>) -> Self {
        Self { start, title: title.into() }
    }
}

impl PartialEq<Chapter> for CmpChapter {
    fn eq(&self, other: &Chapter) -> bool {
        const EPSILON: f32 = 0.01;
        (self.start.as_secs_f32() - other.start.as_secs_f32()) < EPSILON
            && self.title == other.title
    }
}

fn read_dir(path: &str, fun: impl Fn(&Path, &Tag)) {
    for d in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
    {
        let filename = d.file_name().to_str().unwrap();
        let mut mp4file = false;
        for e in EXTENSIONS.iter() {
            if filename.ends_with(e) {
                mp4file = true;
            }
        }

        if !mp4file {
            continue;
        }

        let filepath = d.into_path();

        println!("{}:", filepath.display());
        let tag = Tag::read_from_path(&filepath).unwrap();
        println!("{}", tag);
        fun(&filepath, &tag);
    }
}

#[track_caller]
fn use_sample_file<'a>(in_file: &str, target_file: &'a str) -> &'a str {
    println!("copying `{in_file}` to `{target_file}`...");
    std::fs::copy(in_file, target_file).unwrap();
    target_file
}

#[track_caller]
fn write_tag<'a>(tag: &Userdata, target_file: &str) {
    println!("writing to `{target_file}`...");
    tag.write_to_path(target_file).unwrap();
}

#[track_caller]
fn read_tag<'a>(target_file: &str) -> Tag {
    println!("reading from `{target_file}`...");
    Tag::read_from_path(target_file).unwrap()
}

fn get_tag_1() -> Tag {
    let mut tag = Tag::default();
    tag.set_advisory_rating(AdvisoryRating::Explicit);
    tag.set_album("TEST ALBUM");
    tag.set_album_artist("TEST ALBUM ARTIST");
    tag.set_artist("TEST ARTIST");
    tag.set_bpm(132);
    tag.set_category("TEST CATEGORY");
    tag.set_comment("TEST COMMENT");
    tag.set_compilation();
    tag.set_composer("TEST COMPOSER");
    tag.set_copyright("TEST COPYRIGHT");
    tag.set_description("TEST DESCRIPTION");
    tag.set_disc(1, 2);
    tag.set_encoder("Lavf58.29.100");
    tag.set_gapless_playback();
    tag.set_genre("Hard Rock");
    tag.set_grouping("TEST GROUPING");
    tag.set_keyword("TEST KEYWORD");
    tag.set_lyrics("TEST LYRICS");
    tag.set_media_type(MediaType::Normal);
    tag.set_title("TEST TITLE");
    tag.set_track(7, 13);
    tag.set_year("2013");
    tag.set_artwork(Img::png(fs::read("files/artwork.png").unwrap()));
    tag.set_isrc("TEST ISRC");
    tag.set_lyricist("TEST LYRICIST");
    tag
}

fn get_tag_2() -> Tag {
    let mut tag = Tag::default();
    tag.set_advisory_rating(AdvisoryRating::Inoffensive);
    tag.set_album("NEW ALBUM");
    tag.set_album_artist("NEW ALBUM ARTIST");
    tag.set_artist("NEW ARTIST");
    tag.set_bpm(98);
    tag.set_category("NEW CATEGORY");
    tag.set_comment("NEW COMMENT");
    tag.set_compilation();
    tag.set_composer("NEW COMPOSER");
    tag.set_copyright("NEW COPYRIGHT");
    tag.set_description("NEW DESCRIPTION");
    tag.set_disc(2, 0);
    tag.set_encoder("Lavf58.12.100");
    tag.set_gapless_playback();
    tag.set_genre("Hard Rock");
    tag.set_grouping("NEW GROUPING");
    tag.set_keyword("NEW KEYWORD");
    tag.set_lyrics("NEW LYRICS");
    tag.set_media_type(MediaType::AudioBook);
    tag.set_title("NEW TITLE");
    tag.set_track(3, 7);
    tag.set_year("1998");
    tag.set_artwork(Img::jpeg(b"NEW ARTWORK".to_vec()));
    tag.set_isrc("NEW ISRC");
    tag.set_lyricist("NEW LYRICIST");

    tag.chapter_list_mut().extend([
        Chapter::new(Duration::ZERO, "CHAPTER 1"),
        Chapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"),
        Chapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"),
    ]);
    tag.chapter_track_mut().extend([
        Chapter::new(Duration::ZERO, "CHAPTER 1"),
        Chapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"),
        Chapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"),
    ]);

    tag
}

#[track_caller]
fn assert_tag_1(tag: &Tag) {
    assert_eq!(tag.advisory_rating(), Some(AdvisoryRating::Explicit));
    assert_eq!(tag.album(), Some("TEST ALBUM"));
    assert_eq!(tag.album_artist(), Some("TEST ALBUM ARTIST"));
    assert_eq!(tag.artist(), Some("TEST ARTIST"));
    assert_eq!(tag.bpm(), Some(132));
    assert_eq!(tag.category(), Some("TEST CATEGORY"));
    assert_eq!(tag.comment(), Some("TEST COMMENT"));
    assert_eq!(tag.compilation(), true);
    assert_eq!(tag.composer(), Some("TEST COMPOSER"));
    assert_eq!(tag.copyright(), Some("TEST COPYRIGHT"));
    assert_eq!(tag.description(), Some("TEST DESCRIPTION"));
    assert_eq!(tag.disc(), (Some(1), Some(2)));
    assert_eq!(tag.disc_number(), Some(1));
    assert_eq!(tag.total_discs(), Some(2));
    assert_eq!(tag.encoder(), Some("Lavf58.29.100"));
    assert_eq!(tag.gapless_playback(), true);
    assert_eq!(tag.genre(), Some("Hard Rock"));
    assert_eq!(tag.grouping(), Some("TEST GROUPING"));
    assert_eq!(tag.keyword(), Some("TEST KEYWORD"));
    assert_eq!(tag.lyrics(), Some("TEST LYRICS"));
    assert_eq!(tag.media_type(), Some(MediaType::Normal));
    assert_eq!(tag.title(), Some("TEST TITLE"));
    assert_eq!(tag.track(), (Some(7), Some(13)));
    assert_eq!(tag.track_number(), Some(7));
    assert_eq!(tag.total_tracks(), Some(13));
    assert_eq!(tag.year(), Some("2013"));
    assert_eq!(tag.artwork(), Some(Img::png(fs::read("files/artwork.png").unwrap().as_slice())));
    assert_eq!(tag.isrc(), Some("TEST ISRC"));
    assert_eq!(tag.lyricist(), Some("TEST LYRICIST"));
}

#[track_caller]
fn assert_tag_2(tag: &Tag) {
    assert_eq!(tag.advisory_rating(), Some(AdvisoryRating::Inoffensive));
    assert_eq!(tag.album(), Some("NEW ALBUM"));
    assert_eq!(tag.album_artist(), Some("NEW ALBUM ARTIST"));
    assert_eq!(tag.artist(), Some("NEW ARTIST"));
    assert_eq!(tag.bpm(), Some(98));
    assert_eq!(tag.category(), Some("NEW CATEGORY"));
    assert_eq!(tag.comment(), Some("NEW COMMENT"));
    assert_eq!(tag.compilation(), true);
    assert_eq!(tag.composer(), Some("NEW COMPOSER"));
    assert_eq!(tag.copyright(), Some("NEW COPYRIGHT"));
    assert_eq!(tag.description(), Some("NEW DESCRIPTION"));
    assert_eq!(tag.disc(), (Some(2), None));
    assert_eq!(tag.disc_number(), Some(2));
    assert_eq!(tag.total_discs(), None);
    assert_eq!(tag.encoder(), Some("Lavf58.12.100"));
    assert_eq!(tag.gapless_playback(), true);
    assert_eq!(tag.genre(), Some("Hard Rock"));
    assert_eq!(tag.grouping(), Some("NEW GROUPING"));
    assert_eq!(tag.keyword(), Some("NEW KEYWORD"));
    assert_eq!(tag.lyrics(), Some("NEW LYRICS"));
    assert_eq!(tag.media_type(), Some(MediaType::AudioBook));
    assert_eq!(tag.title(), Some("NEW TITLE"));
    assert_eq!(tag.track(), (Some(3), Some(7)));
    assert_eq!(tag.track_number(), Some(3));
    assert_eq!(tag.total_tracks(), Some(7));
    assert_eq!(tag.year(), Some("1998"));
    assert_eq!(tag.artwork(), Some(Img::jpeg(&b"NEW ARTWORK"[..])));
    assert_eq!(tag.isrc(), Some("NEW ISRC"));
    assert_eq!(tag.lyricist(), Some("NEW LYRICIST"));

    assert_eq!(
        [
            CmpChapter::new(Duration::ZERO, "CHAPTER 1"),
            CmpChapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"),
            CmpChapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"),
        ],
        tag.chapter_list(),
    );
    assert_eq!(
        [
            CmpChapter::new(Duration::ZERO, "CHAPTER 1"),
            CmpChapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"),
            CmpChapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"),
        ],
        tag.chapter_track(),
    );
}

#[track_caller]
fn assert_tag_3(tag: &Tag) {
    assert_eq!(tag.advisory_rating(), Some(AdvisoryRating::Explicit));
    assert_eq!(tag.album(), Some("TEST ALBUM"));
    assert_eq!(tag.album_artist(), Some("TEST ALBUM ARTIST"));

    let mut artists = tag.artists();
    assert_eq!(artists.next(), Some("ARTIST 1"));
    assert_eq!(artists.next(), Some("ARTIST 2"));
    assert_eq!(artists.next(), Some("ARTIST 3"));
    assert_eq!(artists.next(), None);

    assert_eq!(tag.bpm(), Some(132));
    assert_eq!(tag.category(), Some("TEST CATEGORY"));
    assert_eq!(tag.comment(), Some("TEST COMMENT"));
    assert_eq!(tag.compilation(), true);
    assert_eq!(tag.composer(), Some("TEST COMPOSER"));
    assert_eq!(tag.copyright(), Some("TEST COPYRIGHT"));
    assert_eq!(tag.description(), Some("TEST DESCRIPTION"));
    assert_eq!(tag.disc(), (Some(1), Some(2)));
    assert_eq!(tag.disc_number(), Some(1));
    assert_eq!(tag.total_discs(), Some(2));
    assert_eq!(tag.encoder(), Some("Lavf58.29.100"));
    assert_eq!(tag.gapless_playback(), true);

    let mut genres = tag.genres();
    assert_eq!(genres.next(), Some("GENRE 1"));
    assert_eq!(genres.next(), Some("GENRE 2"));
    assert_eq!(genres.next(), Some("GENRE 3"));
    assert_eq!(genres.next(), Some("GENRE 4"));
    assert_eq!(genres.next(), None);

    assert_eq!(tag.grouping(), Some("TEST GROUPING"));
    assert_eq!(tag.keyword(), Some("TEST KEYWORD"));
    assert_eq!(tag.lyrics(), Some("TEST LYRICS"));
    assert_eq!(tag.media_type(), Some(MediaType::Normal));
    assert_eq!(tag.title(), Some("TEST TITLE"));
    assert_eq!(tag.track(), (Some(7), Some(13)));
    assert_eq!(tag.track_number(), Some(7));
    assert_eq!(tag.total_tracks(), Some(13));
    assert_eq!(tag.year(), Some("2013"));
    assert_eq!(tag.artwork(), Some(Img::png(fs::read("files/artwork.png").unwrap().as_slice())));
    assert_eq!(tag.isrc(), Some("TEST ISRC"));

    let mut lyricists = tag.lyricists();
    assert_eq!(lyricists.next(), Some("LYRICIST 1"));
    assert_eq!(lyricists.next(), Some("LYRICIST 2"));
    assert_eq!(lyricists.next(), Some("LYRICIST 3"));
    assert_eq!(lyricists.next(), Some("LYRICIST 4"));
    assert_eq!(lyricists.next(), Some("LYRICIST 5"));
    assert_eq!(lyricists.next(), None);
}

#[track_caller]
fn assert_readonly(tag: &Tag) {
    assert_eq!(tag.duration(), Duration::from_millis(486));
    assert_eq!(tag.filetype(), "M4A \u{0}\u{0}\u{2}\u{0}isomiso2");
    assert_eq!(tag.channel_config(), Some(ChannelConfig::Mono));
    assert_eq!(tag.sample_rate(), Some(SampleRate::Hz44100));
    assert_eq!(tag.avg_bitrate(), Some(64776));
    assert_eq!(tag.max_bitrate(), Some(69000));
}

#[test]
fn chaptered() {
    let tag = read_tag("files/sample-chaptered.m4a");
    let chapters = [
        Chapter::new(Duration::ZERO, "The Pledge"),
        Chapter::new(Duration::new(7 * 60 + 28, 1_000_000), "The Turn"),
        Chapter::new(Duration::new(64 * 60 + 44, 0), "The Prestige"),
    ];
    assert_eq!(tag.chapter_list(), chapters);
    assert_eq!(tag.chapter_track(), chapters);
}

#[test]
fn collection() {
    if let Some(path) = std::env::args().skip_while(|a| a != "collection").nth(1) {
        println!("Testing collection at {}", &path);
        read_dir(&path, |_, _| {});
    } else {
        println!("Skipping collection test since no path was provided.");
    }
}

#[test]
fn sample_files() {
    let _ = fs::remove_dir_all("target/files");
    let _ = fs::create_dir("target/files");

    read_dir("files", |p, t| {
        let file_name = p.file_name().unwrap().to_owned();
        let mut path = PathBuf::from("target/files");
        path.push(file_name);

        println!("copying {} to {}...", p.display(), path.display());
        fs::copy(p, &path).unwrap();

        println!("writing empty tag");
        Tag::default().write_to_path(&path).unwrap();
        println!("reading empty tag");
        let tag = Tag::read_from_path(&path).unwrap();
        assert!(tag.is_empty());
        assert_eq!(tag.audio_info(), t.audio_info());

        println!("writing sample tag 1");
        get_tag_1().write_to_path(&path).unwrap();
        println!("reading sample tag 1");
        let tag = Tag::read_from_path(&path).unwrap();
        assert_tag_1(&tag);
        assert_eq!(tag.audio_info(), t.audio_info());

        println!("writing sample tag 2");
        get_tag_2().write_to_path(&path).unwrap();
        println!("reading sample tag 2");
        let tag = Tag::read_from_path(&path).unwrap();
        assert_tag_2(&tag);
        assert_eq!(tag.audio_info(), t.audio_info());
        println!();
    });
}

#[test]
fn chapter_list_title_truncation() {
    let target_file = use_sample_file("files/sample.m4a", "target/chapter_list_title.m4a");

    let mut tag = Userdata::default();
    tag.chapter_list_mut().extend([
        Chapter::new(Duration::ZERO, "a".repeat(255) + "truncated"),
        Chapter::new(Duration::from_millis(20), "after"),
    ]);
    write_tag(&tag, target_file);

    let tag = read_tag(target_file);
    assert_eq!(
        tag.chapter_list(),
        [
            Chapter::new(Duration::ZERO, "a".repeat(255)),
            Chapter::new(Duration::from_millis(20), "after"),
        ],
    );
}

#[test]
fn chapter_track_title_truncation() {
    let target_file = use_sample_file("files/sample.m4a", "target/chapter_track_title.m4a");

    let mut tag = Userdata::default();
    tag.chapter_track_mut().extend([
        Chapter::new(Duration::ZERO, "a".repeat(65535) + "truncated"),
        Chapter::new(Duration::from_millis(20), "after"),
    ]);
    write_tag(&tag, target_file);

    let tag = read_tag(target_file);
    assert_eq!(
        tag.chapter_track(),
        [
            Chapter::new(Duration::ZERO, "a".repeat(65535)),
            Chapter::new(Duration::from_millis(20), "after"),
        ],
    );
}

#[test]
fn previous_chapter_track_media_data_is_removed() {
    let target_file = use_sample_file("files/sample.m4a", "target/chapter_track_doesnt_grow.m4a");

    let chapters = [
        Chapter::new(Duration::ZERO, "The Pledge"),
        Chapter::new(Duration::from_millis(135), "The Turn"),
        Chapter::new(Duration::from_millis(324), "The Prestige"),
    ];

    let mut tag = Userdata::default();
    tag.chapter_track_mut().extend(chapters.clone());
    write_tag(&tag, target_file);

    let file = std::fs::File::open(target_file).unwrap();
    let prev_size = file.metadata().unwrap().len();

    write_tag(&tag, target_file);

    let file = std::fs::File::open(target_file).unwrap();
    let new_size = file.metadata().unwrap().len();

    assert_eq!(prev_size, new_size);

    let tag = read_tag(target_file);
    assert_eq!(tag.chapter_track(), chapters);
}

#[test]
fn bench() {
    let Some(in_file) = std::env::args().skip_while(|a| a != "bench").nth(1) else {
        println!("Skipping bench test since no input file was provided.");
        return;
    };
    println!("Running bench with file: {in_file}");

    let target_file = use_sample_file(&in_file, "target/bench.m4a");
    let in_tag = read_tag(target_file);

    let start = std::time::Instant::now();
    for _ in 0..300 {
        std::fs::copy(&in_file, target_file).unwrap();

        Tag::default().write_to_path(target_file).unwrap();
        let tag = Tag::read_from_path(target_file).unwrap();
        assert!(tag.is_empty());
        assert_eq!(tag.audio_info(), in_tag.audio_info());

        get_tag_1().write_to_path(target_file).unwrap();
        let tag = Tag::read_from_path(target_file).unwrap();
        assert_tag_1(&tag);
        assert_eq!(tag.audio_info(), in_tag.audio_info());

        get_tag_2().write_to_path(target_file).unwrap();
        let tag = Tag::read_from_path(target_file).unwrap();
        assert_tag_2(&tag);
        assert_eq!(tag.audio_info(), in_tag.audio_info());
    }
    let end = std::time::Instant::now();
    let millis = end.duration_since(start).as_millis();
    println!("took: {millis}ms");
}

#[test]
fn read_sample() {
    let tag = read_tag("files/sample.m4a");
    assert_tag_1(&tag);
    assert_readonly(&tag);
}

#[test]
fn read_sample_multi_data() {
    let tag = read_tag("files/sample-multi-data.m4a");
    assert_tag_3(&tag);
    assert_readonly(&tag);
}

#[test]
fn write() {
    let target_file = use_sample_file("files/sample.m4a", "target/write.m4a");

    let tag = get_tag_2();
    write_tag(&tag, target_file);

    let tag = read_tag(target_file);
    assert_tag_2(&tag);
    assert_readonly(&tag);
}

#[test]
fn write_same() {
    let target_file = use_sample_file("files/sample.m4a", "target/write_same.m4a");

    let tag = get_tag_1();
    write_tag(&tag, target_file);

    let tag = read_tag(target_file);
    assert_tag_1(&tag);
    assert_readonly(&tag);
}

#[test]
fn write_bigger() {
    let target_file = use_sample_file("files/sample.m4a", "target/write_bigger.m4a");

    let mut tag = get_tag_2();
    let data: Vec<u8> = (0..512 * 1024).map(|n| n as u8).collect();
    tag.add_data(Fourcc(*b"test"), Data::Reserved(data));
    write_tag(&tag, target_file);

    let tag = read_tag(target_file);
    assert_tag_2(&tag);
    assert_readonly(&tag);
}

#[test]
fn write_empty() {
    let target_file = use_sample_file("files/sample.m4a", "target/write_empty.m4a");

    let tag = Tag::default();
    write_tag(&tag, target_file);

    let tag = read_tag(target_file);
    assert!(tag.is_empty());
    assert_readonly(&tag);
}
