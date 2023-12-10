fn main(){}

mod helpers {
    pub fn check_number_of_args(args: &impl ExactSizeIterator) -> bool {
        args.len() >= 2
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
}
