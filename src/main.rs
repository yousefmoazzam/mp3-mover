use id3::{Tag, TagLike};

struct SongInfo<'a> {
    artist: &'a str,
    album: &'a str,
    title: Option<&'a str>,
}

fn check_tag_info(tag: &Tag) -> SongInfo {
    let title = tag.title();

    let artist = match tag.artist() {
        Some(val) => val,
        None => {
            println!("Do something graceful in failure =P");
            "failure"
        }
    };

    let album = match tag.album() {
        Some(val) => val,
        None => {
            println!("Do something graceful in failure =P");
            "failure"
        }
    };

    SongInfo {
        artist: artist,
        album: album,
        title: title
    }
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
        let result = check_tag_info(&tag);
        assert_eq!(result.title, Some(dummy_title));
        assert_eq!(result.artist, dummy_artist);
        assert_eq!(result.album, dummy_album);
    }
}
