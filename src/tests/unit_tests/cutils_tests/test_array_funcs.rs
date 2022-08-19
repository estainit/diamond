#[cfg(test)]
pub mod test_array_funcs {
    use crate::lib::constants as CConsts;
    use crate::lib::utils::cutils as cutils;
    use crate::lib::utils::version_handler;
    use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::{calc_coinbased_output_maturation_date};

    #[test]
    pub fn test_array_diff() {
        assert_eq!(
            cutils::array_diff(&vec!["a".to_string(), "b".to_string(), "c".to_string()], &vec!["b".to_string()]),
            vec!["a".to_string(), "c".to_string()]
        );
    }
}
