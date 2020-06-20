use crate::Tag;

#[test]
fn test_tag_ride_all_night() {
    let tag_ride_all_night = Tag::read_from_path("./test_files/00 I Ride All Night (Shake Shake Shake).m4a").unwrap();
    println!("{:?}", tag_ride_all_night);
}

#[test]
fn test_tag_i_seen_you() {
    let tag_i_seen_you = Tag::read_from_path("./test_files/00 I Ride All Night (Shake Shake Shake).m4a").unwrap();
    println!("{:?}", tag_i_seen_you);
}
