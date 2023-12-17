fn main(){}

struct Config {
    input_path: String,
    output_path: String,
}

impl Config {
    fn new(args: &[String]) -> Config {
        let input_path = args[1].clone();
        let output_path = args[2].clone();
        Config { input_path, output_path }
    }
}

mod helpers {
    use std::{path::PathBuf, fs::{read_dir, create_dir_all}};

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

        let contents = read_dir(input_path).unwrap();
        if contents.count() == 0 {
            return false
        }

        true
    }

    pub fn is_output_dir_arg_valid(arg: &str) -> bool {
        let output_path = PathBuf::from(arg);

        if !output_path.exists() {
            return match create_dir_all(output_path) {
                Ok(_) => true,
                Err(e) => {
                    println!("Error when trying to create output dir: {:?}", e);
                    false
                },
            };
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use tempfile::tempdir;

    use crate::Config;
    use crate::helpers::{check_number_of_args, is_input_dir_arg_valid, is_output_dir_arg_valid};

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

    #[test]
    fn input_dir_arg_is_empty() {
        let indir = tempdir().unwrap();
        let indir_str = indir.as_ref().to_str().unwrap();
        let is_valid = is_input_dir_arg_valid(indir_str);
        assert_eq!(is_valid, false);
    }

    #[test]
    fn output_dir_arg_doesnt_exist_but_can_create() {
        let outdir = tempdir().unwrap();
        let subdir = "subdir";
        let outdir_path = outdir.as_ref().join(subdir);
        let outdir_str = outdir_path.to_str().unwrap();
        let is_valid = is_output_dir_arg_valid(outdir_str);
        assert_eq!(is_valid, true);
        assert_eq!(outdir_path.exists(), true);
    }

    #[test]
    fn config_instance_has_correct_args() {
        let program_name = "/path/to/program";
        let input_path = "/path/to/input";
        let output_path = "/path/to/output";
        let dummy_args = vec![
            program_name.to_string(),
            input_path.to_string(),
            output_path.to_string(),
        ];
        let config = Config::new(&dummy_args);
        assert_eq!(config.input_path, input_path);
        assert_eq!(config.output_path, output_path);
    }
}
