# Changelog

All notable changes to this project will be documented in this file.

## mp4ameta v0.12.0

- Implement FromStr for Fourcc
- Add Userdata struct with metadata items and chapters
- Enforce rust_2018_idioms
- Add support for chapter tracks and lists
- Replace proc-macro for userdata accessors with code generation
- [**breaking**] Remove Userdata::dump functions
- Fix some lifetime issues with userdata accessors
- Fix length computation of utf-16 strings
- Reduce number of read calls when parsing an atom head
- Avoid calling stream_position in seek_to_end
- Reduce number of read calls when parsing atom version and flags
- Reduce number number syscalls when parsing mvhd, tkhd, mdhd
- Reduce number of read calls when parsing data atom headers
- Read sample table atoms only when needed
- Allow using non-static borrowed strings in FreeformIdent
- Avoid allocating statically known data
- Rename Userdata::take_(data|bytes|strings|images) as into_* and don't use Rc
- Reduce the number of read/seek calls when parsing mp4a atoms
- Add clear_meta_items and meta_items_is_empty Userdata functions
- Migrate to 2021 edition
- Migrate to 2024 edition
- Bound all allocations to parent atom size or file size
- Avoid intermediary buffer when decoding utf-16 strings
- Add sort order tags
- Update quicktime readme links
