mod helpers{
    use std::path::{Path, PathBuf};
    use std::fs::{File, create_dir};
    use std::iter::zip;

    use id3::{Tag, TagLike};

    pub fn create_dir_with_song_files(
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
            File::create(song_file_path.clone()).unwrap();
            tag.write_to_path(song_file_path, id3::Version::Id3v24).unwrap();
        }
        for other in non_songs.iter() {
            File::create(dir_path.join(other)).unwrap();
        }
        Ok(dir_path)
    }
}

mod tests{
    use std::path::PathBuf;
    use std::iter::zip;
    use std::fs::read_dir;
    use tempfile::tempdir;

    use crate::helpers::create_dir_with_song_files;

    use mp3_mover::run;

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
        let res = run(indir.as_ref(), outdir.as_ref());

        // Check output dir has expected subdirs
        let read_iter = read_dir(outdir.path()).unwrap();
        let pathbuf_iter = read_iter.map(|entry| entry.unwrap().path());
        let subdirs: Vec<PathBuf> = pathbuf_iter.collect();
        for artist in artists {
            assert_eq!(
                subdirs.contains(&outdir.path().join(artist)), true
            );
        }

        // Check all artist subdirs have expected album subdirs
        let artist_album_iter = zip(artists, albums);
        for (artist, album) in artist_album_iter {
            let album_path = outdir.as_ref().to_path_buf().join(artist).join(album);
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
            assert_eq!(song_path.try_exists().unwrap(), true);
        }
    }
}
