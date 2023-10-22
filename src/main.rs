use std::fmt::Display;

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
            println!("Do something graceful in failure =P");
            "failure"
        }
    };

    Ok(SongInfo {
        artist: artist,
        album: album,
        title: title
    })
}

#[cfg(test)]
mod tests {
    use id3::{Tag,TagLike};

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
}
