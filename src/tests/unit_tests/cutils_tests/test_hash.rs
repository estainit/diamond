#[cfg(test)]
pub mod tests_hash {
    use crate::{cutils};

    #[test]
    pub fn test_hash_1() {
        assert!(cutils::is_valid_hash(&"c8ac43aff85bf659fb15a37ccf46b2db5b95837ce22a583a5d6235ea31b5c95d".to_string()));
    }
}