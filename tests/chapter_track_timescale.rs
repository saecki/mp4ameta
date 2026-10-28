use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use mp4ameta::{Chapter, Tag};

/// Chapter tracks written by other tools commonly run at the audio sample rate
/// while the movie header uses a different timescale (600 Hz or 1000 Hz here).
const MEDIA_TIMESCALE: u32 = 44_100;

/// A parsed MP4 atom.
#[derive(Clone)]
struct Atom {
    /// Offset of the size field.
    start: usize,
    /// Size of the atom header (8 bytes, or 16 for 64-bit sizes).
    header: usize,
    /// Offset one past the atom.
    end: usize,
    fourcc: [u8; 4],
}

fn be_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes(buf[offset..offset + 4].try_into().expect("4-byte read"))
}

fn be_u64(buf: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes(buf[offset..offset + 8].try_into().expect("8-byte read"))
}

fn atoms_in(buf: &[u8], start: usize, end: usize) -> Vec<Atom> {
    let mut atoms = Vec::new();
    let mut pos = start;
    while pos + 8 <= end {
        let size = be_u32(buf, pos) as usize;
        let fourcc: [u8; 4] = buf[pos + 4..pos + 8].try_into().expect("fourcc");
        let (size, header) = match size {
            0 => (end - pos, 8),
            1 => (be_u64(buf, pos + 8) as usize, 16),
            size => (size, 8),
        };
        assert!(size >= header && pos + size <= end, "malformed atom {fourcc:?}");
        atoms.push(Atom { start: pos, header, end: pos + size, fourcc });
        pos += size;
    }
    atoms
}

impl Atom {
    fn child(&self, buf: &[u8], fourcc: &[u8; 4]) -> Option<Atom> {
        atoms_in(buf, self.start + self.header, self.end)
            .into_iter()
            .find(|atom| &atom.fourcc == fourcc)
    }
}

fn descend(buf: &[u8], mut atom: Atom, path: &[&[u8; 4]]) -> Option<Atom> {
    for fourcc in path {
        atom = atom.child(buf, fourcc)?;
    }
    Some(atom)
}

/// Locate the chapter track referenced by a `tref`/`chap` entry.
fn chapter_track(buf: &[u8]) -> Atom {
    let moov = atoms_in(buf, 0, buf.len())
        .into_iter()
        .find(|atom| &atom.fourcc == b"moov")
        .expect("moov atom");
    let traks: Vec<Atom> = atoms_in(buf, moov.start + moov.header, moov.end)
        .into_iter()
        .filter(|atom| &atom.fourcc == b"trak")
        .collect();

    let chapter_id = traks
        .iter()
        .find_map(|trak| {
            let chap = descend(buf, trak.clone(), &[b"tref", b"chap"])?;
            // `tref` children hold raw track IDs; unlike full boxes they carry
            // no version/flags word.
            Some(be_u32(buf, chap.start + chap.header))
        })
        .expect("chap track reference");

    let track_id = |trak: &Atom| {
        let tkhd = trak.child(buf, b"tkhd").expect("tkhd atom");
        let version = buf[tkhd.start + tkhd.header];
        let offset = tkhd.start + tkhd.header + if version == 0 { 12 } else { 20 };
        be_u32(buf, offset)
    };

    traks.into_iter().find(|trak| track_id(trak) == chapter_id).expect("chapter track")
}

/// Return `(timescale, mdhd duration, sum of stts sample durations)`.
fn chapter_track_clock(path: &Path) -> (u32, u64, u64) {
    let buf = fs::read(path).expect("read file");
    let trak = chapter_track(&buf);
    let mdhd = descend(&buf, trak.clone(), &[b"mdia", b"mdhd"]).expect("mdhd atom");
    let stts = descend(&buf, trak, &[b"mdia", b"minf", b"stbl", b"stts"]).expect("stts atom");

    let version = buf[mdhd.start + mdhd.header];
    assert_eq!(version, 0, "chapter track mdhd is version 0");
    let timescale = be_u32(&buf, mdhd.start + mdhd.header + 12);
    let duration = be_u32(&buf, mdhd.start + mdhd.header + 16) as u64;

    let entries = be_u32(&buf, stts.start + stts.header + 4) as usize;
    let mut total = 0u64;
    for entry in 0..entries {
        let offset = stts.start + stts.header + 8 + entry * 8;
        total += be_u32(&buf, offset) as u64 * be_u32(&buf, offset + 4) as u64;
    }

    (timescale, duration, total)
}

/// Rewrite the chapter track so its media clock differs from the movie clock,
/// like chapter tracks produced by other tools.
fn rescale_chapter_track(path: &Path) {
    let mut buf = fs::read(path).expect("read file");
    let trak = chapter_track(&buf);
    let mdhd = descend(&buf, trak.clone(), &[b"mdia", b"mdhd"]).expect("mdhd atom");
    let stts = descend(&buf, trak, &[b"mdia", b"minf", b"stbl", b"stts"]).expect("stts atom");

    let version = buf[mdhd.start + mdhd.header];
    assert_eq!(version, 0, "chapter track mdhd is version 0");
    let timescale_offset = mdhd.start + mdhd.header + 12;
    let duration_offset = mdhd.start + mdhd.header + 16;
    let old_timescale = be_u32(&buf, timescale_offset);
    assert_ne!(old_timescale, MEDIA_TIMESCALE, "fixture must start on the movie clock");

    let entries = be_u32(&buf, stts.start + stts.header + 4) as usize;
    assert!(entries > 0, "fixture has chapters");

    let mut new_duration = 0u64;
    for entry in 0..entries {
        let offset = stts.start + stts.header + 8 + entry * 8 + 4;
        let old = be_u32(&buf, offset) as u64;
        let scaled =
            (old * MEDIA_TIMESCALE as u64 + old_timescale as u64 / 2) / old_timescale as u64;
        buf[offset..offset + 4].copy_from_slice(&(scaled as u32).to_be_bytes());
        new_duration += scaled;
    }

    buf[timescale_offset..timescale_offset + 4].copy_from_slice(&MEDIA_TIMESCALE.to_be_bytes());
    buf[duration_offset..duration_offset + 4].copy_from_slice(&(new_duration as u32).to_be_bytes());

    fs::write(path, &buf).expect("write file");
}

fn chapter_starts(path: &Path) -> Vec<u64> {
    Tag::read_from_path(path)
        .expect("read tag")
        .chapter_track()
        .iter()
        .map(|chapter| chapter.start.as_millis() as u64)
        .collect()
}

#[test]
fn chapter_track_survives_metadata_only_writes() {
    let target: PathBuf =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join("chapter-track-timescale.m4a");
    fs::copy("files/sample.m4a", &target).expect("copy sample");

    // Give the file a chapter track; mp4ameta creates it on the movie clock.
    let mut tag = Tag::read_from_path(&target).expect("read sample");
    tag.chapter_track_mut().extend([
        Chapter::new(Duration::ZERO, "ONE"),
        Chapter::new(Duration::from_millis(100), "TWO"),
        Chapter::new(Duration::from_millis(200), "THREE"),
    ]);
    tag.write_to_path(&target).expect("write chapters");

    // Reclock the chapter track, as files tagged by other tools do.
    rescale_chapter_track(&target);
    assert_eq!(chapter_starts(&target), vec![0, 100, 200]);

    // A metadata-only write must not rescale the chapter track.
    let mut tag = Tag::read_from_path(&target).expect("read tag");
    tag.set_title("TITLE WRITE");
    tag.write_to_path(&target).expect("write title");
    assert_eq!(chapter_starts(&target), vec![0, 100, 200]);

    let mut tag = Tag::read_from_path(&target).expect("read tag");
    tag.set_album("ALBUM WRITE");
    tag.write_to_path(&target).expect("write album");
    assert_eq!(chapter_starts(&target), vec![0, 100, 200]);

    // The regenerated sample table must stay in the track's own clock.
    let (timescale, duration, total) = chapter_track_clock(&target);
    assert_eq!(timescale, MEDIA_TIMESCALE);
    assert_eq!(total, duration, "stts durations must fill the track duration");

    fs::remove_file(&target).expect("remove target");
}
