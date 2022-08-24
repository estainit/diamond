#[cfg(test)]
pub mod tests_wrapping_uwrapping {
    use crate::cutils::strip_parentheses_as_break_line;

    #[test]
    pub fn test_strip_break_1() {
        assert_eq!(strip_parentheses_as_break_line("(hello)(world)".to_string()), "hello".to_string());
        assert_eq!(strip_parentheses_as_break_line("(hello)".to_string()), "hello".to_string());
    }
}
