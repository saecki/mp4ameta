use std::fs;

use mp4ameta::{AdvisoryRating, MediaType, Tag, Data, STANDARD_GENRES};
use walkdir::WalkDir;

const EXTENSIONS: [&str; 4] = [".m4a", ".m4b", ".m4p", ".m4v"];

#[test]
fn sample_files() {
    for d in WalkDir::new("./files")
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
        let tag_sample = Tag::read_from_path(&filepath).unwrap();
        println!("{:#?}", tag_sample);
    }
}

#[test]
fn verify_sample_data() {
    let tag = Tag::read_from_path("./files/sample.m4a").unwrap();

    assert_eq!(tag.advisory_rating(), Some(AdvisoryRating::Explicit(4)));
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
    assert_eq!(tag.artwork(), Some(&Data::Png(fs::read("./files/artwork.png").unwrap())));
    assert_eq!(tag.duration(), Some(0.48523809523809525));
    assert_eq!(tag.filetype(), Some("M4A \u{0}\u{0}\u{2}\u{0}isomiso2"));
}

#[test]
fn write_read() {
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
    tag.set_artwork(Data::Jpeg(b"NEW ARTWORK".to_vec()));

    std::fs::copy("./files/sample.m4a", "./files/temp.m4a").unwrap();
    tag.write_to_path("./files/temp.m4a").unwrap();

    let tag = Tag::read_from_path("./files/temp.m4a").unwrap();
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
    assert_eq!(tag.disc(), (Some(2), Some(0)));
    assert_eq!(tag.disc_number(), Some(2));
    assert_eq!(tag.total_discs(), Some(0));
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
    assert_eq!(tag.artwork(), Some(&Data::Jpeg(b"NEW ARTWORK".to_vec())));
    assert_eq!(tag.duration(), Some(0.48523809523809525));
    assert_eq!(tag.filetype(), Some("M4A \u{0}\u{0}\u{2}\u{0}isomiso2"));

    std::fs::remove_file("./files/temp.m4a").unwrap();
}

#[test]
fn dump_read() {
    let mut tag = Tag::default();
    tag.set_advisory_rating(AdvisoryRating::Explicit(4));
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
    tag.set_artwork(Data::Png(b"TEST ARTWORK".to_vec()));

    tag.dump_to_path("./files/temp.m4a").unwrap();

    let tag = Tag::read_from_path("./files/temp.m4a").unwrap();
    assert_eq!(tag.advisory_rating(), Some(AdvisoryRating::Explicit(4)));
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
    assert_eq!(tag.artwork(), Some(&Data::Png(b"TEST ARTWORK".to_vec())));

    std::fs::remove_file("./files/temp.m4a").unwrap();
}

#[test]
fn genre_handling() {
    let genre = STANDARD_GENRES[4];

    let mut tag = Tag::default();
    assert_eq!(tag.genre(), None);
    assert_eq!(tag.standard_genre(), None);
    assert_eq!(tag.custom_genre(), None);

    tag.set_genre(genre.1);
    assert_eq!(tag.genre(), Some(genre.1));
    assert_eq!(tag.standard_genre(), Some(genre.0));
    assert_eq!(tag.custom_genre(), None);

    tag.set_genre("CUSTOM GENRE");
    assert_eq!(tag.genre(), Some("CUSTOM GENRE"));
    assert_eq!(tag.standard_genre(), None);
    assert_eq!(tag.custom_genre(), Some("CUSTOM GENRE"));
}

#[test]
fn track_disc_handling() {
    let track_number = 4u16;
    let total_tracks= 16u16;
    let disc_number = 2u16;
    let total_discs= 3u16;

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
}

#[test]
fn work_movement_handling() {
    let movement = "TEST MOVEMENT";
    let index = 1u16;
    let count = 8u16;
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
