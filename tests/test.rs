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
