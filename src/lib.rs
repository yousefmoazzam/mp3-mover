pub mod config;

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::fs::{create_dir_all, rename, read_dir, DirEntry};

use audiotags::{AudioTag,FlacTag,Id3v2Tag,Mp4Tag,TagType};
use glob::{Paths, glob};
use log::{info, warn};

use config::Config;

#[derive(Debug)]
struct SongInfo<'a> {
    artist: &'a str,
    album: &'a str,
    title: Option<&'a str>,
}

#[derive(Debug)]
struct MissingSongInfo {
    missing_field: String,
}

impl Display for MissingSongInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Missing field: {}", self.missing_field)
    }
}

fn check_tag_info(tag: &impl AudioTag) -> Result<SongInfo, MissingSongInfo> {
    let title = tag.title();

    let artist = match tag.artist() {
        Some(val) => val,
        None => {
            return Err(
                MissingSongInfo{ missing_field: String::from("artist") }
            )
        }
    };

    let album = match tag.album_title() {
        Some(val) => val,
        None => {
            return Err(
                MissingSongInfo { missing_field: String::from("album") }
            )
        }
    };

    Ok(SongInfo {
        artist: artist,
        album: album,
        title: title
    })
}


fn create_song_dir(outdir: &impl AsRef<Path>,  artist: &str, album: &str) -> std::io::Result<()> {
    let mut outdir_path = outdir.as_ref().to_path_buf();
    outdir_path.push(artist);
    outdir_path.push(album);
    create_dir_all(outdir_path.clone())
}


fn move_song_file(filepath: &PathBuf, song_info: &SongInfo, outdir: &PathBuf) -> std::io::Result<()> {
    let mut outdir_path = outdir.clone();
    let mut filename = String::new();
    outdir_path.push(song_info.artist);
    outdir_path.push(song_info.album);
    match song_info.title {
        None => {
            let original_filename = filepath.as_path().file_name()
                .expect("Audio file path from globbing shouldn't return a directory");
            let lossy_filename = original_filename.to_string_lossy();
            filename.push_str(&lossy_filename);
        },
        Some(title) => {
            filename.push_str(title);
            filename.push_str(".mp3");
        }
    }
    outdir_path.push(filename);
    info!("Renaming {:?} to {:?}", filepath, outdir_path);
    rename(filepath, outdir_path)
}

fn find_song_files(dir: &PathBuf) -> Option<Paths> {
    let pattern = "*.mp3";
    let full_pattern = dir.join(pattern);
    let glob_str = full_pattern.to_str()?;
    Some(glob(glob_str)
        .expect("Glob pattern is hardcoded so should not be an invalid pattern"))
}

pub fn run(config: Config) -> std::io::Result<()> {
    let contents = read_dir(config.input_path)?;
    for child in contents {
        let elem = match child {
            Ok(val) => val,
            Err(e) => {
                warn!("During input dir contents reading encountered error {:?}; moving on", e);
                continue;
            }
        };
        if !elem.path().is_dir() {
            continue;
        }
        check_song_files(&elem, Path::new(&config.output_path))?;
    }
    Ok(())
}


fn check_song_files(dir_entry: &DirEntry, outdir: &Path) -> std::io::Result<()> {
    let song_file_paths = match find_song_files(&dir_entry.path()) {
        Some(val) => val,
        None => {
            warn!(
                "Found invalid unicode in glob string for dir: {:?}; ignoring an moving on",
                dir_entry.path()
            );
            return Ok(())
        }
    };

    for glob_res in song_file_paths {
        let path = match glob_res {
            Ok(val) => val,
            Err(e) => {
                warn!("Found a glob error {:?}, ignoring and moving onto next match", e);
                continue;
            }
        };
        let file_extension = match path.extension() {
            Some(val) => val.to_str().expect("Expected valid unicode in file extension"),
            None => {
                info!(
                    "The file {:?} has no file extension so the appropriate tag cannot be inferred, skipping",
                    path,
                );
                continue
            }
        };
        let tag_type = match file_extension {
            "flac" => TagType::Flac,
            "mp3" => TagType::Id3v2,
            "mp4" => TagType::Mp4,
            _ => {
                info!(
                    "Unsupported file extension encountered for file {:?}, skipping",
                    path,
                );
                continue
            }
        };
        match tag_type {
            TagType::Flac => {
                check_song_file_tag_info(
                    &FlacTag::read_from_path(path.clone()).unwrap(), &path, outdir
                )?
            },
            TagType::Id3v2 => {
                check_song_file_tag_info(
                    &Id3v2Tag::read_from_path(path.clone()).unwrap(), &path, outdir
                )?
            },
            TagType::Mp4 => {
                check_song_file_tag_info(
                    &Mp4Tag::read_from_path(path.clone()).unwrap(), &path, outdir
                )?
            },
        };
    }
    Ok(())
}


fn check_song_file_tag_info(tag: &impl AudioTag, file_path: &PathBuf, outdir: &Path) -> std::io::Result<()> {
    let tag_info = check_tag_info(tag);
    match tag_info {
        Ok(song_info) => {
            create_song_dir(&outdir, song_info.artist, song_info.album)?;
            move_song_file(&file_path, &song_info, &outdir.to_path_buf())?;
        },
        Err(e) => {
            info!("Song file {:?} has missing field {}", file_path, e.missing_field);
        }
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::fs::File;

    use glob::GlobResult;
    use id3::{Tag,TagLike};
    use audiotags::{AudioTagEdit,Id3v2Tag};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn correct_tag() {
        let mut tag = Id3v2Tag::new();
        let dummy_title = "Dummy Title";
        let dummy_artist = "Dummy Artist";
        let dummy_album = "Dummy Album";
        tag.set_title(dummy_title);
        tag.set_artist(dummy_artist);
        tag.set_album_title(dummy_album);
        let result = check_tag_info(&tag).unwrap();
        assert_eq!(result.title, Some(dummy_title));
        assert_eq!(result.artist, dummy_artist);
        assert_eq!(result.album, dummy_album);
    }

    #[test]
    fn tag_with_missing_title() {
        let mut tag = Id3v2Tag::new();
        let dummy_artist = "Dummy Artist";
        let dummy_album = "Dummy Album";
        tag.set_artist(dummy_artist);
        tag.set_album_title(dummy_album);
        let result = check_tag_info(&tag).unwrap();
        assert_eq!(result.title, None);
        assert_eq!(result.artist, dummy_artist);
        assert_eq!(result.album, dummy_album);
    }

    #[test]
    fn tag_with_missing_artist() {
        let mut tag = Id3v2Tag::new();
        let dummy_title = "Dummy Title";
        let dummy_album = "Dummy Album";
        tag.set_title(dummy_title);
        tag.set_album_title(dummy_album);
        let result = check_tag_info(&tag).unwrap_err();
        assert_eq!(result.to_string(), "Missing field: artist");
    }

    #[test]
    fn tag_with_missing_album() {
        let mut tag = Id3v2Tag::new();
        let dummy_title = "Dummy Title";
        let dummy_artist = "Dummy Artist";
        tag.set_title(dummy_title);
        tag.set_artist(dummy_artist);
        let result = check_tag_info(&tag).unwrap_err();
        assert_eq!(result.to_string(), "Missing field: album");
    }

    #[test]
    fn correct_dir_created_for_song() {
        let outdir = tempdir().unwrap();
        let artist = "Dummy Artist";
        let album = "Dummy Album";
        let _ = create_song_dir(&outdir, &artist, &album).unwrap();
        let mut outdir_path = outdir.as_ref().to_path_buf();
        outdir_path.push(artist);
        outdir_path.push(album);
        let was_correct_dir_created = outdir_path.try_exists().unwrap();
        assert_eq!(was_correct_dir_created, true);
    }

    #[test]
    fn move_song_with_title_info() {
        // Setup directory that song file should be moved into
        let outdir = tempdir().unwrap();
        let mut outdir_path = outdir.as_ref().to_path_buf();
        let song_info = SongInfo {
            artist: "Dummy Artist",
            album: "Dummy Album",
            title: Some("Dummy Title"),
        };
        outdir_path.push(song_info.artist);
        outdir_path.push(song_info.album);
        create_dir_all(outdir_path.clone()).unwrap();
        // Setup directory where dummy file originally exists before attempted move, and setup
        // dummy file
        let original_parent_dir = tempdir().unwrap();
        let original_filename = "ABCD.mp3";
        let mut original_filepath = original_parent_dir.as_ref().to_path_buf();
        original_filepath.push(original_filename);
        // Create dummy song file
        File::create(original_filepath.clone()).unwrap();
        // Attempt to move dummy file to the created output dir
        let _ = move_song_file(
            &original_filepath,
            &song_info,
            &outdir.as_ref().to_path_buf(),
        ).unwrap();
        // Define the expected path of the moved + renamed file
        let mut expected_new_filepath = outdir_path.clone();
        let expected_filename = format!("{}.mp3", song_info.title.unwrap());
        expected_new_filepath.push(expected_filename);
        // Check if the dummy file was moved correctly
        let was_file_moved_correctly = expected_new_filepath.try_exists().unwrap();
        assert_eq!(
            was_file_moved_correctly,
            true,
            "Expected filepath: {:?}",
            expected_new_filepath,
        )
    }

    #[test]
    fn move_song_without_title_info() {
        let outdir = tempdir().unwrap();
        let mut outdir_path = outdir.as_ref().to_path_buf();
        let song_info = SongInfo {
            artist: "Dummy Artist",
            album: "Dummy Album",
            title: None,
        };
        outdir_path.push(song_info.artist);
        outdir_path.push(song_info.album);
        create_dir_all(outdir_path.clone()).unwrap();
        // Setup directory where dummy file originally exists before attempted move, and setup
        // dummy file
        let original_parent_dir = tempdir().unwrap();
        let original_filename = "ABCD.mp3";
        let mut original_filepath = original_parent_dir.as_ref().to_path_buf();
        original_filepath.push(original_filename);
        // Create dummy song file
        File::create(original_filepath.clone()).unwrap();
        // Attempt to move dummy file to the created output dir
        let _ = move_song_file(
            &original_filepath,
            &song_info,
            &outdir.as_ref().to_path_buf(),
        ).unwrap();
        // Define the expected path of the moved + renamed file
        let mut expected_new_filepath = outdir_path.clone();
        expected_new_filepath.push(original_filename);
        // Check if the dummy file was moved correctly
        let was_file_moved_correctly = expected_new_filepath.try_exists().unwrap();
        assert_eq!(
            was_file_moved_correctly,
            true,
            "Expected filepath: {:?}",
            expected_new_filepath,
        )
    }

    #[test]
    fn find_three_song_files_in_dir() {
        // Setup input directory containing all MP3 files to be checked and moved
        let indir = tempdir().unwrap();
        let indir_path = indir.as_ref().to_path_buf();
        // Create dummy song files and associated dummy tag info
        let songs = ["AATV.mp3", "AKLO.mp3", "ALZH.mp3"];
        let mut tag = Tag::new();
        let album = "Some Album";
        let artist = "Some Artist";
        for (i, song) in songs.iter().enumerate() {
            tag.set_title(format!("{} {}", "Song", i.to_string()));
            tag.set_artist(artist);
            tag.set_album(album);
            File::create(indir_path.join(song)).unwrap();
            tag.write_to_path(indir_path.join(song), id3::Version::Id3v24).unwrap();
        }
        // Create some unsupported audio files in the input dir too
        let other_files = ["A.m4a", "B.mp4", "C.flac"];
        for unsupported_song_file in other_files.iter() {
            File::create(indir_path.join(unsupported_song_file)).unwrap();
        }
        // Call function to check for supported song files
        let song_files: Vec<GlobResult> = find_song_files(&indir_path).unwrap().collect();
        // Check that we got the three MP3 files and none of the other unsupported files
        let supported_song_filepaths = songs.map(|song| indir_path.join(song));
        assert_eq!(song_files.len(), 3);
        for filepath in song_files {
            let matched_song_filepath = filepath.unwrap();
            assert_eq!(supported_song_filepaths.contains(&matched_song_filepath), true);
        }
        let unsupported_song_filepaths = other_files.map(|song| indir_path.join(song));
        for filepath in unsupported_song_filepaths {
            assert_eq!(
                supported_song_filepaths.contains(&filepath), false
            );
        }
    }

    #[test]
    fn find_no_song_files_in_dir() {
        let indir = tempdir().unwrap();
        let indir_path = indir.as_ref().to_path_buf();
        // Create some unsupported files in the input dir
        let files = ["A.jpg", "B.mp4", "C.txt"];
        for filename in files.iter() {
            File::create(indir_path.join(filename)).unwrap();
        }
        // Call function to check for supported files
        let song_files: Vec<GlobResult> = find_song_files(&indir_path).unwrap().collect();
        assert_eq!(song_files.is_empty(), true);
    }
}

