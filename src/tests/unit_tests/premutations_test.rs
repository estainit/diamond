#[cfg(test)]
pub mod merkel_tests_2 {
    use crate::lib::utils::permutation_handler::PermutationHandler;

    // FIXME: complete it in order to test more possibile combination in automatic style

    #[test]
    pub fn test_premutations_1() {
        let hp = PermutationHandler::new(
            &vec!["a".to_string()],
            1,
            true,
            &vec![],
            &vec![]);
        assert_eq!(hp.m_permutations.len(), 1);
    }

    #[test]
    pub fn test_premutations_2() {
        let hp = PermutationHandler::new(
            &vec!["a".to_string(), "b".to_string()],
            1,
            true,
            &vec![],
            &vec![]);
        assert_eq!(hp.m_permutations.len(), 2);
    }

    #[test]
    pub fn test_premutations_3() {
        {
            let hp = PermutationHandler::new(
                &vec!["a".to_string(), "b".to_string(), "c".to_string()],
                2,
                true,
                &vec![],
                &vec![]);
            assert_eq!(hp.m_permutations.len(), 3);
            // hp.testAnalyze(&vec![]);
        }
    }
}
