use crate::Tag;

#[test]
fn test_sample() {
    let tag_sample = Tag::read_from_path("./test/sample.m4a").unwrap();
    println!("{:?}", tag_sample);
}
