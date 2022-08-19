#[cfg(test)]
pub mod tests_wrapping_uwrapping {
    use crate::cutils::strip_parentheses_as_break_line;
    use crate::lib::constants as CConsts;
    use crate::lib::utils::cutils as cutils;
    use crate::lib::utils::version_handler;
    use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::{calc_coinbased_output_maturation_date};

    #[test]
    pub fn test_strip_break_1() {
        assert_eq!(strip_parentheses_as_break_line("(hello)(world)".to_string()), "hello".to_string());
        assert_eq!(strip_parentheses_as_break_line("(hello)".to_string()), "hello".to_string());
    }
}
