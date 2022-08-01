#[cfg(test)]
pub mod merkel_tests_2 {
    use crate::cmerkle;
    use crate::lib::ccrypto;

    #[test]
    pub fn test_2_leaves_a() {
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"hashed".to_string(),
                                    &"noHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "12");
            assert_eq!(proofs.len(), 2);
            assert_eq!(levels, 2);
            assert_eq!(leaves, 2);
            let proof_1 = proofs.get("1").unwrap();
            let proof_2 = proofs.get("2").unwrap();
            assert_eq!(proof_1.m_left_hash, "");
            assert_eq!(proof_2.m_left_hash, "1");
            assert_eq!(proofs["1"].m_merkle_proof, vec!["r.2".to_string()]);
            assert_eq!(proofs["2"].m_merkle_proof.len(), 0);
        }


        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"hashed".to_string(),
                                    &"noHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "12");
            assert_eq!(proofs.len(), 2);
            assert_eq!(levels, 2);
            assert_eq!(leaves, 2);
            let proof_1 = proofs.get("1").unwrap();
            let proof_2 = proofs.get("2").unwrap();
            assert_eq!(proof_1.m_left_hash, "");
            assert_eq!(proof_2.m_left_hash, "1");
            assert_eq!(proofs["1"].m_merkle_proof, vec!["r.2".to_string()]);
            assert_eq!(proofs["2"].m_merkle_proof.len(), 0);
        }
    }

    #[test]
    pub fn test_2_leaves_b() {
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"hashed".to_string(),
                                    &"aliasHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "h(12)");
            assert_eq!(root, cmerkle::get_root_by_a_prove(
                &"1".to_string(),
                &proofs["1"].m_merkle_proof,
                &proofs["1"].m_left_hash,
                &"hashed".to_string(),
                &"aliasHash".to_string()));
            assert_eq!(root, cmerkle::get_root_by_a_prove(
                &"2".to_string(),
                &proofs["2"].m_merkle_proof,
                &proofs["2"].m_left_hash,
                &"hashed".to_string(),
                &"aliasHash".to_string()));
        }
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"string".to_string(),
                                    &"aliasHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "h(h(1)h(2))");
            assert_eq!(proofs["h(1)"].m_merkle_proof, vec!["r.h(2)"]);
            assert_eq!(proofs["h(2)"].m_merkle_proof.len(), 0);
            assert_eq!(proofs["h(2)"].m_left_hash, "h(1)");
        }
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"string".to_string(),
                                    &"".to_string(),
                                    &"".to_string());
            assert_eq!(root, ccrypto::keccak256(&(ccrypto::keccak256(&"1".to_string()) + &ccrypto::keccak256(&"2".to_string()))));
            assert_eq!(proofs[&ccrypto::keccak256(&"1".to_string())].m_merkle_proof, vec!["r.".to_owned() + &ccrypto::keccak256(&"2".to_string())]);
            assert_eq!(proofs[&ccrypto::keccak256(&"2".to_string())].m_merkle_proof.len(), 0);
        }
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"".to_string(),
                                    &"".to_string(),
                                    &"".to_string());
            assert_eq!(root, ccrypto::keccak256(&"12".to_string()));
            assert_eq!(root, "7f8b6b088b6d74c2852fc86c796dca07b44eed6fb3daf5e6b59f7c364db14528");
        }
    }

    #[test]
    pub fn test_2_leaves_c() {
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"hashed".to_string(),
                                    &"aliasHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "h(12)");
            assert_eq!(root, cmerkle::get_root_by_a_prove(
                &"1".to_string(),
                &proofs["1"].m_merkle_proof,
                &proofs["1"].m_left_hash,
                &"hashed".to_string(),
                &"aliasHash".to_string()));
        }
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate_m(&vec!["1".to_string(), "2".to_string()],
                                    &"string".to_string(),
                                    &"aliasHash".to_string(),
                                    &"".to_string());
            assert_eq!(root, "h(h(1)h(2))");
            assert_eq!(proofs["h(1)"].m_merkle_proof, vec!["r.h(2)"]);
            assert_eq!(proofs["h(2)"].m_merkle_proof.len(), 0);
            assert_eq!(proofs["h(2)"].m_left_hash, "h(1)");

        }
    }
}