use id3::{Tag, TagLike};

fn check_tag_info(tag: &Tag) -> (&str, &str, &str) {
    let title = match tag.title() {
        Some(val) => val,
        None => {
            println!("Do something graceful in failure =P");
            "failure"
        }
    };

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

    (title, artist, album)
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
        let expected = (
            String::from(dummy_title),
            String::from(dummy_artist),
            String::from(dummy_album),
        );
        let result = check_tag_info(&tag);
        assert_eq!(result.0, expected.0);
        assert_eq!(result.1, expected.1);
        assert_eq!(result.2, expected.2);
    }
}
