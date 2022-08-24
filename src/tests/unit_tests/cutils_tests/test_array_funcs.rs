#[cfg(test)]
pub mod test_array_funcs {
    use crate::lib::utils::cutils as cutils;

    #[test]
    pub fn test_array_diff() {
        assert_eq!(
            cutils::array_diff(&vec!["a".to_string(), "b".to_string(), "c".to_string()], &vec!["b".to_string()]),
            vec!["a".to_string(), "c".to_string()]
        );
    }
}
