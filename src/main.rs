fn main(){}

mod helpers {
    use std::path::PathBuf;

    pub fn check_number_of_args(args: &impl ExactSizeIterator) -> bool {
        args.len() == 2
    }

    pub fn is_input_dir_arg_valid(input: &str) -> bool {
        let input_path = PathBuf::from(input);

        if !input_path.exists() {
            return false
        }

        if !input_path.is_dir() {
            return false
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;

    use crate::helpers::{check_number_of_args, is_input_dir_arg_valid};

    #[test]
    fn not_enough_cli_args() {
        let dummy_args = ["/path/to/some/dir".to_string()];
        let iter = dummy_args.iter();
        let is_correct_no_of_args = check_number_of_args(&iter);
        assert_eq!(is_correct_no_of_args, false);
    }

    #[test]
    fn too_many_cli_args() {
        let dummy_args = ["/input/dir/path", "/ouput/dir/path", "extra arg"];
        let dummy_args = dummy_args.map(|s| s.to_string());
        let is_correct_no_of_args = check_number_of_args(&dummy_args.iter());
        assert_eq!(is_correct_no_of_args, false);
    }

    #[test]
    fn input_dir_arg_doesnt_exist() {
       let nonexistent_path = "/tmp/nonexistent-dir";
       let is_valid = is_input_dir_arg_valid(&nonexistent_path);
       assert_eq!(is_valid, false);
    }

    #[test]
    fn input_dir_arg_isnt_a_dir() {
        let indir = tempdir().unwrap();
        let indir_path = indir.as_ref().to_path_buf();
        let file_not_dir_path = indir_path.join("blah.txt");
        File::create(file_not_dir_path.clone()).unwrap();
        let is_valid = is_input_dir_arg_valid(&file_not_dir_path.to_str().unwrap());
        assert_eq!(is_valid, false);
    }
}
