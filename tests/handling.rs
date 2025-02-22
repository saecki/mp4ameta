use mp4ameta::{ident, Data, Img, Tag, STANDARD_GENRES};

#[test]
fn multiple_value_handling() {
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

    // Test if track number atom is corrected to the right size when edited.
    tag.set_data(ident::TRACK_NUMBER, Data::Reserved(vec![0, 0, 0, 1]));
    tag.set_total_tracks(2);
    assert_eq!(tag.track(), (Some(1), Some(2)));
    assert_eq!(
        tag.data_of(&ident::TRACK_NUMBER).next(),
        Some(&Data::Reserved(vec![0, 0, 0, 1, 0, 2, 0, 0]))
    );
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
