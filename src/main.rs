fn main(){}

mod helpers {
    pub fn check_number_of_args(args: &impl ExactSizeIterator) -> bool {
        args.len() == 2
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::check_number_of_args;

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
}
