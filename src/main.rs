use std::{fmt::Display, path::{Path, PathBuf}, fs::create_dir_all, io::ErrorKind};

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


fn create_song_dir(outdir: &impl AsRef<Path>,  artist: &str, album: &str) -> Result<PathBuf, ErrorKind> {
    let mut outdir_path = outdir.as_ref().to_path_buf();
    outdir_path.push(artist);
    outdir_path.push(album);
    create_dir_all(outdir_path.clone()).unwrap();
    Ok(outdir_path)
}

#[cfg(test)]
mod tests {
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
}
