use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mp4ameta::{
    AdvisoryRating, ChannelConfig, Chapter, Data, Fourcc, Img, MediaType, SampleRate, Tag,
    WriteChapters, WriteConfig, STANDARD_GENRES,
};
use walkdir::WalkDir;

const EXTENSIONS: [&str; 6] = [".m4a", ".m4b", ".m4p", ".m4v", ".mp4", ".3gp"];

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

    tag.add_chapter(Chapter::new(Duration::ZERO, "CHAPTER 1"));
    tag.add_chapter(Chapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"));
    tag.add_chapter(Chapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"));

    tag
}

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

    let mut chapters = tag.chapters();
    assert_eq!(chapters.next(), Some(&Chapter::new(Duration::ZERO, "CHAPTER 1")));
    assert_eq!(chapters.next(), Some(&Chapter::new(Duration::new(234, 324_000_000), "CHAPTER 2")));
    assert_eq!(chapters.next(), Some(&Chapter::new(Duration::new(553, 946_000_000), "CHAPTER 3")));
    assert_eq!(chapters.next(), None);
}

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

fn assert_readonly(tag: &Tag) {
    assert_eq!(tag.duration(), Duration::from_millis(486));
    assert_eq!(tag.filetype(), "M4A \u{0}\u{0}\u{2}\u{0}isomiso2");
    assert_eq!(tag.channel_config(), Some(ChannelConfig::Mono));
    assert_eq!(tag.sample_rate(), Some(SampleRate::Hz44100));
    assert_eq!(tag.avg_bitrate(), Some(64776));
    assert_eq!(tag.max_bitrate(), Some(69000));
}

#[test]
fn collection() {
    if let Some(path) = std::env::args().skip_while(|a| a != "collection").skip(1).next() {
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
fn read_sample() {
    let tag = Tag::read_from_path("files/sample.m4a").unwrap();

    assert_tag_1(&tag);
    assert_readonly(&tag);
}

#[test]
fn read_sample_multi_data() {
    let tag = Tag::read_from_path("files/sample-multi-data.m4a").unwrap();

    assert_tag_3(&tag);
    assert_readonly(&tag);
}

#[test]
fn write() {
    let tag = get_tag_2();

    let _ = std::fs::remove_file("target/write.m4a");
    println!("copying files/sample.m4a to target/write.m4a...");
    std::fs::copy("files/sample.m4a", "target/write.m4a").unwrap();

    println!("writing...");
    tag.write_to_path("target/write.m4a").unwrap();

    println!("reading...");
    let tag = Tag::read_from_path("target/write.m4a").unwrap();
    assert_tag_2(&tag);
    assert_readonly(&tag);
}

#[test]
fn write_same() {
    let tag = get_tag_1();

    let _ = std::fs::remove_file("target/write_same.m4a");
    println!("copying files/sample.m4a to target/write_same.m4a...");
    std::fs::copy("files/sample.m4a", "target/write_same.m4a").unwrap();

    println!("writing...");
    tag.write_to_path("target/write_same.m4a").unwrap();

    println!("reading...");
    let tag = Tag::read_from_path("target/write_same.m4a").unwrap();
    assert_tag_1(&tag);
    assert_readonly(&tag);
}

#[test]
fn write_bigger() {
    let mut tag = get_tag_2();
    let data: Vec<u8> = (0..2048).map(|n| (n % 255) as u8).collect();
    tag.add_data(Fourcc(*b"test"), Data::Reserved(data));

    let _ = std::fs::remove_file("target/write_bigger.m4a");
    println!("copying files/sample.m4a to target/write_bigger.m4a...");
    std::fs::copy("files/sample.m4a", "target/write_bigger.m4a").unwrap();

    println!("writing...");
    tag.write_to_path("target/write_bigger.m4a").unwrap();

    println!("reading...");
    let tag = Tag::read_from_path("target/write_bigger.m4a").unwrap();
    assert_tag_2(&tag);
    assert_readonly(&tag);
}

#[test]
fn write_empty() {
    let tag = Tag::default();

    let _ = std::fs::remove_file("target/write_empty.m4a");
    println!("copying files/sample.m4a to target/write_empty.m4a...");
    std::fs::copy("files/sample.m4a", "target/write_empty.m4a").unwrap();

    println!("writing...");
    tag.write_to_path("target/write_empty.m4a").unwrap();

    println!("reading...");
    let tag = Tag::read_from_path("target/write_empty.m4a").unwrap();
    assert!(tag.is_empty());
    assert_readonly(&tag);
}

#[test]
fn dump_1() {
    let tag = get_tag_1();

    let _ = std::fs::remove_file("target/dump_1.m4a");
    println!("dumping to target/dump_1.m4a...");
    tag.dump_to_path("target/dump_1.m4a").unwrap();

    println!("reading target/dump_1.m4a....");
    let tag = Tag::read_from_path("target/dump_1.m4a").unwrap();
    assert_tag_1(&tag);
}

#[test]
fn dump_2() {
    let tag = get_tag_2();

    let _ = std::fs::remove_file("target/dump_2.m4a");
    println!("dumping to target/dump_2.m4a...");
    tag.dump_to_path("target/dump_2.m4a").unwrap();

    println!("reading target/dump_2.m4a...");
    let tag = Tag::read_from_path("target/dump_2.m4a").unwrap();
    assert_tag_2(&tag);
}

#[test]
fn dump_chapter_list() {
    let mut tag = Tag::default();
    let chapters = [
        Chapter::new(Duration::ZERO, "CHAPTER 1"),
        Chapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"),
        Chapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"),
    ];

    tag.add_all_chapters(chapters.clone());

    let _ = std::fs::remove_file("target/dump_chapter_list.m4a");
    println!("dumping to target/dump_chapter_list.m4a...");
    let cfg = WriteConfig {
        write_chapters: WriteChapters::ChapterList,
        ..Default::default()
    };
    tag.dump_with_path("target/dump_chapter_list.m4a", &cfg).unwrap();

    println!("reading target/dump_chapter_list.m4a...");
    let tag = Tag::read_from_path("target/dump_chapter_list.m4a").unwrap();

    let mut chapters_iter = tag.chapters();
    assert_eq!(chapters_iter.next(), Some(&chapters[0]));
    assert_eq!(chapters_iter.next(), Some(&chapters[1]));
    assert_eq!(chapters_iter.next(), Some(&chapters[2]));
    assert_eq!(chapters_iter.next(), None);
}

#[test]
fn dump_chapter_track() {
    let mut tag = Tag::default();
    let chapters = [
        Chapter::new(Duration::ZERO, "CHAPTER 1"),
        Chapter::new(Duration::new(234, 324_000_000), "CHAPTER 2"),
        Chapter::new(Duration::new(553, 946_000_000), "CHAPTER 3"),
    ];

    tag.add_all_chapters(chapters.clone());

    let _ = std::fs::remove_file("target/dump_chapter_track.m4a");
    println!("dumping to target/dump_chapter_track.m4a...");
    let cfg = WriteConfig {
        write_chapters: WriteChapters::ChapterTrack,
        ..Default::default()
    };
    tag.dump_with_path("target/dump_chapter_track.m4a", &cfg).unwrap();

    println!("reading target/dump_chapter_track.m4a...");
    let tag = Tag::read_from_path("target/dump_chapter_track.m4a").unwrap();

    let mut chapters_iter = tag.chapters();
    assert_eq!(chapters_iter.next(), Some(&chapters[0]));
    assert_eq!(chapters_iter.next(), Some(&chapters[1]));
    assert_eq!(chapters_iter.next(), Some(&chapters[2]));
    assert_eq!(chapters_iter.next(), None);
}

#[test]
fn multiple_values() {
    let mut tag = Tag::default();

    tag.add_artist("1");
    tag.add_artist("2");
    tag.add_artist("3");
    tag.add_artist("4");

    assert_eq!(tag.artist(), Some("1"));
    {
        let mut artists = tag.artists();
        assert_eq!(artists.next(), Some("1"));
        assert_eq!(artists.next(), Some("2"));
        assert_eq!(artists.next(), Some("3"));
        assert_eq!(artists.next(), Some("4"));
        assert_eq!(artists.next(), None);
    }

    tag.set_artist("5");

    assert_eq!(tag.artist(), Some("5"));
    {
        let mut artists = tag.artists();
        assert_eq!(artists.next(), Some("5"));
        assert_eq!(artists.next(), None);
    }

    tag.add_artist("6");
    tag.add_artist("7");

    assert_eq!(tag.artist(), Some("5"));
    {
        let mut artists = tag.artists();
        assert_eq!(artists.next(), Some("5"));
        assert_eq!(artists.next(), Some("6"));
        assert_eq!(artists.next(), Some("7"));
        assert_eq!(artists.next(), None);
    }

    tag.remove_artists();

    assert_eq!(tag.artists().next(), None);
    assert_eq!(tag.artist(), None);
}

#[test]
fn genre_handling() {
    let mut tag = Tag::default();
    assert_eq!(tag.genre(), None);
    assert_eq!(tag.standard_genre(), None);
    assert_eq!(tag.custom_genre(), None);

    let standard_name = STANDARD_GENRES[38];
    tag.set_genre(standard_name);
    assert_eq!(tag.genre(), Some(standard_name));
    assert_eq!(tag.standard_genre(), None);
    assert_eq!(tag.custom_genre(), Some(standard_name));

    tag.set_genre("CUSTOM GENRE");
    assert_eq!(tag.genre(), Some("CUSTOM GENRE"));
    assert_eq!(tag.standard_genre(), None);
    assert_eq!(tag.custom_genre(), Some("CUSTOM GENRE"));

    tag.remove_genres();
    assert_eq!(tag.genre(), None);
    assert_eq!(tag.genres().next(), None);

    let (code1, name1) = (7, STANDARD_GENRES[6]);
    let (code2, name2) = (24, STANDARD_GENRES[23]);
    tag.add_custom_genre("GENRE 1");
    tag.add_standard_genre(code1);
    tag.add_custom_genre("GENRE 2");
    tag.add_standard_genre(code2);

    {
        let mut genres = tag.genres();
        assert_eq!(genres.next(), Some(name1));
        assert_eq!(genres.next(), Some(name2));
        assert_eq!(genres.next(), Some("GENRE 1"));
        assert_eq!(genres.next(), Some("GENRE 2"));
        assert_eq!(genres.next(), None);

        let mut standard_genres = tag.standard_genres();
        assert_eq!(standard_genres.next(), Some(code1));
        assert_eq!(standard_genres.next(), Some(code2));
        assert_eq!(genres.next(), None);

        let mut custom_genres = tag.custom_genres();
        assert_eq!(custom_genres.next(), Some("GENRE 1"));
        assert_eq!(custom_genres.next(), Some("GENRE 2"));
        assert_eq!(genres.next(), None);
    }

    tag.remove_standard_genres();
    assert_eq!(tag.standard_genres().next(), None);
    assert_eq!(tag.genres().next(), Some("GENRE 1"));

    tag.remove_custom_genres();
    assert_eq!(tag.custom_genres().next(), None);
    assert_eq!(tag.genres().next(), None);
}

#[test]
fn track_disc_handling() {
    let track_number = 4u16;
    let total_tracks = 16u16;
    let disc_number = 2u16;
    let total_discs = 3u16;

    let mut tag = Tag::default();
    assert_eq!(tag.track(), (None, None));
    assert_eq!(tag.track_number(), None);
    assert_eq!(tag.total_tracks(), None);
    assert_eq!(tag.disc(), (None, None));
    assert_eq!(tag.disc_number(), None);
    assert_eq!(tag.total_discs(), None);

    tag.set_track_number(track_number);
    tag.set_total_tracks(total_tracks);
    tag.set_disc_number(disc_number);
    tag.set_total_discs(total_discs);

    assert_eq!(tag.track(), (Some(track_number), Some(total_tracks)));
    assert_eq!(tag.track_number(), Some(track_number));
    assert_eq!(tag.total_tracks(), Some(total_tracks));
    assert_eq!(tag.disc(), (Some(disc_number), Some(total_discs)));
    assert_eq!(tag.disc_number(), Some(disc_number));
    assert_eq!(tag.total_discs(), Some(total_discs));

    tag.remove_track_number();
    tag.remove_disc_number();

    assert_eq!(tag.track(), (None, Some(total_tracks)));
    assert_eq!(tag.track_number(), None);
    assert_eq!(tag.total_tracks(), Some(total_tracks));
    assert_eq!(tag.disc(), (None, Some(total_discs)));
    assert_eq!(tag.disc_number(), None);
    assert_eq!(tag.total_discs(), Some(total_discs));

    tag.remove_total_tracks();
    tag.remove_total_discs();

    assert_eq!(tag.track(), (None, None));
    assert_eq!(tag.track_number(), None);
    assert_eq!(tag.total_tracks(), None);
    assert_eq!(tag.disc(), (None, None));
    assert_eq!(tag.disc_number(), None);
    assert_eq!(tag.total_discs(), None);
}

#[test]
fn work_movement_handling() {
    let movement = "TEST MOVEMENT";
    let index = 1;
    let count = 8;
    let work = "TEST WORK";

    let mut tag = Tag::default();
    assert_eq!(tag.movement(), None);
    assert_eq!(tag.movement_count(), None);
    assert_eq!(tag.movement_index(), None);
    assert_eq!(tag.show_movement(), false);
    assert_eq!(tag.work(), None);

    tag.set_movement(movement);
    tag.set_movement_count(count);
    tag.set_movement_index(index);
    tag.set_show_movement();
    tag.set_work(work);

    assert_eq!(tag.movement(), Some(movement));
    assert_eq!(tag.movement_count(), Some(count));
    assert_eq!(tag.movement_index(), Some(index));
    assert_eq!(tag.show_movement(), true);
    assert_eq!(tag.work(), Some(work));
}

#[test]
fn chapter_handling() {
    let chapter1 = Chapter::new(Duration::from_secs(0), "CHAPTER 1");
    let chapter2 = Chapter::new(Duration::from_secs(1), "CHAPTER 2");
    let chapter3 = Chapter::new(Duration::from_secs(2), "CHAPTER 3");

    let mut tag = Tag::default();
    assert_eq!(tag.chapters().next(), None);

    tag.add_chapter(chapter2.clone());
    let mut chapters = tag.chapters();
    assert_eq!(chapters.next(), Some(&chapter2));
    assert_eq!(chapters.next(), None);
    drop(chapters);

    tag.add_chapter(chapter1.clone());
    let mut chapters = tag.chapters();
    assert_eq!(chapters.next(), Some(&chapter1));
    assert_eq!(chapters.next(), Some(&chapter2));
    assert_eq!(chapters.next(), None);
    drop(chapters);

    tag.add_chapter(chapter3.clone());
    let mut chapters = tag.chapters();
    assert_eq!(chapters.next(), Some(&chapter1));
    assert_eq!(chapters.next(), Some(&chapter2));
    assert_eq!(chapters.next(), Some(&chapter3));
    assert_eq!(chapters.next(), None);
}

#[test]
fn tag_destructuring() {
    let mut tag = Tag::default();

    tag.set_album("TEST ALBUM");
    tag.set_album_artist("TEST ALBUM ARTIST");
    tag.set_artist("TEST ARTIST");
    tag.set_category("TEST CATEGORY");
    tag.set_comment("TEST COMMENT");
    tag.set_composer("TEST COMPOSER");
    tag.set_copyright("TEST COPYRIGHT");
    tag.set_description("TEST DESCRIPTION");
    tag.set_encoder("Lavf58.29.100");
    tag.set_genre("TEST GENRE");
    tag.set_grouping("TEST GROUPING");
    tag.set_keyword("TEST KEYWORD");
    tag.set_lyrics("TEST LYRICS");
    tag.set_title("TEST TITLE");
    tag.set_year("2013");
    tag.set_artwork(Img::png(b"TEST ARTWORK".to_vec()));

    assert_eq!(tag.take_album(), Some("TEST ALBUM".to_string()));
    assert_eq!(tag.take_album_artist(), Some("TEST ALBUM ARTIST".to_string()));
    assert_eq!(tag.take_artist(), Some("TEST ARTIST".to_string()));
    assert_eq!(tag.take_category(), Some("TEST CATEGORY".to_string()));
    assert_eq!(tag.take_comment(), Some("TEST COMMENT".to_string()));
    assert_eq!(tag.take_composer(), Some("TEST COMPOSER".to_string()));
    assert_eq!(tag.take_copyright(), Some("TEST COPYRIGHT".to_string()));
    assert_eq!(tag.take_description(), Some("TEST DESCRIPTION".to_string()));
    assert_eq!(tag.take_encoder(), Some("Lavf58.29.100".to_string()));
    assert_eq!(tag.take_genre(), Some("TEST GENRE".to_string()));
    assert_eq!(tag.take_grouping(), Some("TEST GROUPING".to_string()));
    assert_eq!(tag.take_keyword(), Some("TEST KEYWORD".to_string()));
    assert_eq!(tag.take_lyrics(), Some("TEST LYRICS".to_string()));
    assert_eq!(tag.take_title(), Some("TEST TITLE".to_string()));
    assert_eq!(tag.take_year(), Some("2013".to_string()));
    assert_eq!(tag.take_artwork(), Some(Img::png(b"TEST ARTWORK".to_vec())));

    assert_eq!(tag.album(), None);
    assert_eq!(tag.album_artist(), None);
    assert_eq!(tag.artist(), None);
    assert_eq!(tag.category(), None);
    assert_eq!(tag.comment(), None);
    assert_eq!(tag.composer(), None);
    assert_eq!(tag.copyright(), None);
    assert_eq!(tag.description(), None);
    assert_eq!(tag.encoder(), None);
    assert_eq!(tag.genre(), None);
    assert_eq!(tag.grouping(), None);
    assert_eq!(tag.keyword(), None);
    assert_eq!(tag.lyrics(), None);
    assert_eq!(tag.title(), None);
    assert_eq!(tag.year(), None);
    assert_eq!(tag.artwork(), None);
}
