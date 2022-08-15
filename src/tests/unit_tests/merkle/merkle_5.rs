#[cfg(test)]
pub mod merkel_tests_3 {
    use substring::Substring;
    use crate::cmerkle;

    #[test]
    pub fn test_3_leaves_a() {
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(vec![
                    "a".to_string(),
                    "b".to_string(),
                    "c".to_string(),
                    "d".to_string(),
                    "e".to_string()],
                                    &"hashed".to_string(),
                                    &"noHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "abcdeleave_6leave_7leave_8".to_string());
            assert_eq!(proofs["a"].m_left_hash, "");
            assert_eq!(proofs["a"].m_merkle_proof[0].substring(0, 1), "r");
            assert_eq!(proofs["a"].m_merkle_proof[0].substring(2, proofs["a"].m_merkle_proof[0].len()), "b");
            assert_eq!(proofs["a"].m_merkle_proof[1].substring(0, 1), "r");
            assert_eq!(proofs["a"].m_merkle_proof[1].substring(2, proofs["a"].m_merkle_proof[1].len()), "cd");
            assert_eq!(proofs["b"].m_left_hash, "a");
            assert_eq!(proofs["c"].m_left_hash, "");
            assert_eq!(proofs["d"].m_left_hash, "c");
            assert_eq!(proofs["b"].m_merkle_proof[0].substring(2, proofs["b"].m_merkle_proof[0].len()), "cd");
            assert_eq!(proofs["e"].m_merkle_proof[0], "r.leave_6");
            assert_eq!(proofs["leave_6"].m_merkle_proof[0], "r.leave_7leave_8");
        }
    }
}