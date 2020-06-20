use crate::Tag;
use std::fs;

#[test]
fn test_sample_files() {
    for f in fs::read_dir("./test/").unwrap() {
        let filename: String = f.unwrap().path().to_str().unwrap().into();
        let tag_sample = Tag::read_from_path(&filename).unwrap();
        println!("{}:\n{:?}", &filename, tag_sample);
    }
}
