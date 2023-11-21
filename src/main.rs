use std::{fmt::Display, path::{Path, PathBuf}, fs::{create_dir_all, rename, read_dir, DirEntry}};

use glob::{Paths, PatternError, glob};
use id3::{Tag, TagLike};

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

fn check_tag_info(tag: &Tag) -> Result<SongInfo, MissingSongInfo> {
    let title = tag.title();

    let artist = match tag.artist() {
        Some(val) => val,
        None => {
            return Err(
                MissingSongInfo{ missing_field: String::from("artist") }
            )
        }
    };

    let album = match tag.album() {
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


fn create_song_dir(outdir: &impl AsRef<Path>,  artist: &str, album: &str) -> std::io::Result<PathBuf> {
    let mut outdir_path = outdir.as_ref().to_path_buf();
    outdir_path.push(artist);
    outdir_path.push(album);
    create_dir_all(outdir_path.clone()).unwrap();
    Ok(outdir_path)
}


fn move_song_file(filepath: &PathBuf, song_info: &SongInfo, outdir: &PathBuf) -> std::io::Result<()> {
    let mut outdir_path = outdir.clone();
    let mut filename = String::new();
    outdir_path.push(song_info.artist);
    outdir_path.push(song_info.album);
    match song_info.title {
        None => {
            filename.push_str(filepath.as_path().file_name().unwrap().to_str().unwrap());
        },
        Some(title) => {
            filename.push_str(title);
            filename.push_str(".mp3");
        }
    }
    outdir_path.push(filename);
    rename(filepath, outdir_path)
}

fn find_song_files(dir: &PathBuf) -> Result<Paths, PatternError> {
    let pattern = "*.mp3";
    glob(dir.join(pattern).to_str().unwrap())
}

fn process_input_dir(indir: &Path, outdir: &Path) -> std::io::Result<bool> {
    let contents = read_dir(indir).unwrap();
    for child in contents {
        let elem = child.unwrap();
        if !elem.path().is_dir() {
            continue;
        }
        let _ = check_song_files(&elem, outdir);
    }
    Ok(true)
}


fn check_song_files(dir_entry: &DirEntry, outdir: &Path) -> std::io::Result<bool> {
    let song_file_paths = find_song_files(&dir_entry.path()).unwrap();
    for glob_res in song_file_paths {
        let path = glob_res.unwrap();
        let tag = match Tag::read_from_path(path.clone()) {
            Ok(val) => val,
            Err(_) => {
                println!(
                    "The file {:?} has no tag, it has been left unmodified",
                    path,
                );
                continue;
            }
        };
        let tag_info = check_tag_info(&tag);
        match tag_info {
            Ok(song_info) => {
                let created_dir = create_song_dir(
                    &outdir, song_info.artist, song_info.album
                ).unwrap();
                let moved_song_file = move_song_file(
                    &path, &song_info, &outdir.to_path_buf()
                ).unwrap();
            },
            Err(e) => {
                continue;
            }
        }
    }
    Ok(true)
}


#[cfg(test)]
mod tests {
    use std::{fs::{File, create_dir, read_dir}, iter::zip};

    use glob::GlobResult;
    use id3::{Tag,TagLike};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn correct_tag() {
        let mut tag = Tag::new();
        let dummy_title = "Dummy Title";
        let dummy_artist = "Dummy Artist";
        let dummy_album = "Dummy Album";
        tag.set_title(String::from(dummy_title));
        tag.set_artist(String::from(dummy_artist));
        tag.set_album(String::from(dummy_album));
        let result = check_tag_info(&tag).unwrap();
        assert_eq!(result.title, Some(dummy_title));
        assert_eq!(result.artist, dummy_artist);
        assert_eq!(result.album, dummy_album);
    }

    #[test]
    fn tag_with_missing_title() {
        let mut tag = Tag::new();
        let dummy_artist = "Dummy Artist";
        let dummy_album = "Dummy Album";
        tag.set_artist(dummy_artist);
        tag.set_album(dummy_album);
        let result = check_tag_info(&tag).unwrap();
        assert_eq!(result.title, None);
        assert_eq!(result.artist, dummy_artist);
        assert_eq!(result.album, dummy_album);
    }

    #[test]
    fn tag_with_missing_artist() {
        let mut tag = Tag::new();
        let dummy_title = "Dummy Title";
        let dummy_album = "Dummy Album";
        tag.set_title(dummy_title);
        tag.set_album(dummy_album);
        let result = check_tag_info(&tag).unwrap_err();
        assert_eq!(result.to_string(), "Missing field: artist");
    }

    #[test]
    fn tag_with_missing_album() {
        let mut tag = Tag::new();
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
        let res = create_song_dir(&outdir, &artist, &album).unwrap();
        let mut outdir_path = outdir.as_ref().to_path_buf();
        outdir_path.push(artist);
        outdir_path.push(album);
        let was_correct_dir_created = outdir_path.try_exists().unwrap();
        assert_eq!(
            was_correct_dir_created,
            true,
            "Expected dir: {:?}. Created dir: {:?}",
            outdir_path,
            res,
        );
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

    fn create_dir_with_song_files(
        dir_name: &str,
        parent_dir: &Path,
        songs: &[&str],
        song_titles: &[&str],
        non_songs: &[&str],
        artists: &[&str],
        albums: &[&str],
    ) -> std::io::Result<PathBuf> {
        let parent_dir_path = parent_dir.to_path_buf();
        let dir_path = parent_dir_path.join(dir_name);
        create_dir(dir_path.clone()).unwrap();
        for (i, (artist, album)) in zip(artists, albums).enumerate() {
            let mut tag = Tag::new();
            tag.set_title(song_titles[i]);
            tag.set_artist(*artist);
            tag.set_album(*album);
            let song_file_path = dir_path.join(songs[i]);
            //println!("Created file {:?} with song title {}", song_file_path, song_titles[i]);
            File::create(song_file_path.clone()).unwrap();
            tag.write_to_path(song_file_path, id3::Version::Id3v24).unwrap();
        }
        for other in non_songs.iter() {
            File::create(dir_path.join(other)).unwrap();
        }
        Ok(dir_path)
    }

    #[test]
    fn find_song_files_in_two_dirs() {
        // Setup input dir with 2 subdirs containing song files

        // Stuff common to both dirs first
        let indir = tempdir().unwrap();
        let artists = ["Artist1", "Artist2", "Artist3"];
        let albums = ["Album1", "Album2", "Album3"];

        // Stuff for dir one
        let dir_one = "F00";
        let dir_one_song_files = ["A.mp3", "B.mp3", "C.mp3"];
        let dir_one_song_titles = ["Song1", "Song2", "Song3"];
        let dir_one_non_song_files = ["D.mp4", "E.jpg"];
        let dir_one_path = create_dir_with_song_files(
            dir_one,
            indir.as_ref(),
            &dir_one_song_files,
            &dir_one_song_titles,
            &dir_one_non_song_files,
            &artists,
            &albums,
        ).unwrap();

        // Stuff for dir two
        let dir_two = "F11";
        let dir_two_song_files = ["F.mp3", "G.mp3", "H.mp3"];
        let dir_two_song_titles = ["Song4", "Song5", "Song6"];
        let dir_two_non_song_files = ["I.mp4", "J.jpg"];
        let dir_two_path = create_dir_with_song_files(
            dir_two,
            indir.as_ref(),
            &dir_two_song_files,
            &dir_two_song_titles,
            &dir_two_non_song_files,
            &artists,
            &albums,
        ).unwrap();

        // Setup output dir to place renamed song files in
        let outdir = tempdir().unwrap();

        // Run function to search through all subdirs in input dir and rename+move song files into
        // the output dir
        let res = process_input_dir(indir.as_ref(), outdir.as_ref());

        // Check output dir has expected subdirs
        let read_iter = read_dir(outdir.path()).unwrap();
        let pathbuf_iter = read_iter.map(|entry| entry.unwrap().path());
        let subdirs: Vec<PathBuf> = pathbuf_iter.collect();
        for artist in artists {
            //println!("{:?}", outdir.path().join(artist));
            assert_eq!(
                subdirs.contains(&outdir.path().join(artist)), true
            );
        }

        // Check all artist subdirs have expected album subdirs
        let artist_album_iter = zip(artists, albums);
        for (artist, album) in artist_album_iter {
            let album_path = outdir.as_ref().to_path_buf().join(artist).join(album);
            //println!("{:?}", album_path);
            assert_eq!(album_path.try_exists().unwrap(), true);
        }

        // Check all album subdirs contain expected renamed song files
        let dir_one_expected_song_files = dir_one_song_titles.map(
            |s| format!("{}.mp3", s)
        );
        let dir_one_iter = zip(zip(dir_one_expected_song_files, artists), albums);
        for ((song, artist), album) in dir_one_iter {
            let song_path = outdir.as_ref().to_path_buf()
                .join(artist)
                .join(album)
                .join(song);
            println!("Seeing if this path exists {:?}", song_path);
            assert_eq!(song_path.try_exists().unwrap(), true);
        }
        let dir_two_expected_song_files = dir_two_song_titles.map(
            |s| format!("{}.mp3", s)
        );
        let dir_two_iter = zip(zip(dir_two_expected_song_files, artists), albums);
        for ((song, artist), album) in dir_two_iter {
            let song_path = outdir.as_ref().to_path_buf()
                .join(artist)
                .join(album)
                .join(song);
            //println!("{:?}", song_path);
            assert_eq!(song_path.try_exists().unwrap(), true);
        }
    }
}
