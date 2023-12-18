use std::path::PathBuf;
use std::fs::{read_dir, create_dir_all};

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

    fn validate_number_of_args(args: &impl ExactSizeIterator) -> Result<(), &str> {
        if args.len() != 3 {
            return Err("Invalid number of args given")
        }
        Ok(())
    }

    fn validate_input_dir_arg(input: &str) -> Result<(), &str> {
        let input_path = PathBuf::from(input);

        if !input_path.exists() {
            return Err("Input directory arg doesn't exist")
        }

        if !input_path.is_dir() {
            return Err("Input directory arg isn't a directory")
        }

        let contents = read_dir(input_path).unwrap();
        if contents.count() == 0 {
            return Err("Input directory arg contains no subdirectories")
        }

        Ok(())
    }

    fn validate_output_dir_arg(arg: &str) -> bool {
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

    #[test]
    fn not_enough_cli_args() {
        let dummy_args = ["/path/to/progam".to_string()];
        let iter = dummy_args.iter();
        let res = Config::validate_number_of_args(&iter);
        let expected_error_message = "Invalid number of args given";
        assert_eq!(
            res.is_err_and(|e| e.to_string() == expected_error_message),
            true,
        );
    }

    #[test]
    fn too_many_cli_args() {
        let dummy_args = [
            "/path/to/program",
            "/input/dir/path",
            "/ouput/dir/path",
            "extra arg"
        ];
        let dummy_args = dummy_args.map(|s| s.to_string());
        let iter = dummy_args.iter();
        let res = Config::validate_number_of_args(&iter);
        let expected_error_message = "Invalid number of args given";
        assert_eq!(
            res.is_err_and(|e| e.to_string() == expected_error_message),
            true,
        );
    }

    #[test]
    fn input_dir_arg_doesnt_exist() {
        let nonexistent_path = "/tmp/nonexistent-dir";
        let res = Config::validate_input_dir_arg(&nonexistent_path);
        let expected_error_message = "Input directory arg doesn't exist";
        assert_eq!(
            res.is_err_and(|e| e.to_string() == expected_error_message),
            true,
        );
    }

    #[test]
    fn input_dir_arg_isnt_a_dir() {
        let indir = tempdir().unwrap();
        let indir_path = indir.as_ref().to_path_buf();
        let file_not_dir_path = indir_path.join("blah.txt");
        File::create(file_not_dir_path.clone()).unwrap();
        let res = Config::validate_input_dir_arg(
            &file_not_dir_path.to_str().unwrap()
        );
        let expected_error_message = "Input directory arg isn't a directory";
        assert_eq!(
            res.is_err_and(|e| e.to_string() == expected_error_message),
            true,
        );
    }

    #[test]
    fn input_dir_arg_is_empty() {
        let indir = tempdir().unwrap();
        let indir_str = indir.as_ref().to_str().unwrap();
        let res = Config::validate_input_dir_arg(indir_str);
        let expected_error_message = "Input directory arg contains no subdirectories";
        assert_eq!(
            res.is_err_and(|e| e.to_string() == expected_error_message),
            true,
        );
    }

    #[test]
    fn output_dir_arg_doesnt_exist_but_can_create() {
        let outdir = tempdir().unwrap();
        let subdir = "subdir";
        let outdir_path = outdir.as_ref().join(subdir);
        let outdir_str = outdir_path.to_str().unwrap();
        let is_valid = Config::validate_output_dir_arg(outdir_str);
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
