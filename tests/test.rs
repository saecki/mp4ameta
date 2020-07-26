use mp4ameta::Tag;
use std::fs;

const EXTENSIONS: [&str; 4] = [".m4a", ".m4b", ".m4p", ".m4v"];

#[test]
fn test_sample_files() {
    for f in fs::read_dir("./tests/files").unwrap() {
        let filename: String = f.unwrap().path().to_str().unwrap().into();

        let mut mp4file = false;
        for e in EXTENSIONS.iter() {
            if filename.ends_with(e) {
                mp4file = true;
                break;
            }
        }

        if !mp4file {
            continue;
        }

        println!("{}:", &filename);
        let tag_sample = Tag::read_from_path(&filename).unwrap();
        println!("{:#?}", tag_sample);
    }
}

#[test]
fn verify_sample_data() {
    let tag = Tag::read_from_path("./tests/files/sample.m4a").unwrap();

    assert_eq!(tag.album(), Some("TEST ALBUM"));
    assert_eq!(tag.album_artist(), Some("TEST ALBUM ARTIST"));
    assert_eq!(tag.artist(), Some("TEST ARTIST"));
    assert_eq!(tag.category(), Some("TEST CATEGORY"));
    assert_eq!(tag.comment(), Some("TEST COMMENT"));
    assert_eq!(tag.composer(), Some("TEST COMPOSER"));
    assert_eq!(tag.copyright(), Some("TEST COPYRIGHT"));
    assert_eq!(tag.description(), Some("TEST DESCRIPTION"));
    assert_eq!(tag.disk_number(), Some((1, 2)));
    assert_eq!(tag.encoder(), Some("Lavf58.29.100"));
    assert_eq!(tag.genre(), Some("Hard Rock"));
    assert_eq!(tag.grouping(), Some("TEST GROUPING"));
    assert_eq!(tag.keyword(), Some("TEST KEYWORD"));
    assert_eq!(tag.lyrics(), Some("TEST LYRICS"));
    assert_eq!(tag.title(), Some("TEST TITLE"));
    assert_eq!(tag.track_number(), Some((7, 13)));
    assert_eq!(tag.year(), Some("2013"));
}
    
#[test]
fn work_movement_handling() {
    let movement = "TEST MOVEMENT";
    let index = 1u16;
    let count = 8u16;
    let work = "TEST WORK";

    let mut tag = Tag::read_from_path("./tests/files/sample.m4a").unwrap();
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