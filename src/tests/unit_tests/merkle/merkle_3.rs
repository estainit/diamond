#[cfg(test)]
pub mod merkel_tests_3 {
    use substring::Substring;
    use crate::cmerkle;
    use crate::lib::ccrypto;

    #[test]
    pub fn test_3_leaves_a() {
        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate(&vec!["1".to_string(), "2".to_string(), "3".to_string()],
                                  &"hashed".to_string(),
                                  &"noHash".to_string(),
                                  &"".to_string());
            assert_eq!(root, "123leave_4");
            assert_eq!(proofs["1"].m_left_hash, "");
            let proof_1 = proofs.get("1").unwrap();
            assert_eq!(proof_1.m_merkle_proof[0].to_string().substring(0, 1), "r".to_string());
            assert_eq!(proof_1.m_merkle_proof[0].to_string().substring(2, 3), "2".to_string());
            assert_eq!(proof_1.m_merkle_proof[1].to_string().substring(0, 1), "r".to_string());
            assert_eq!(proof_1.m_merkle_proof[1].to_string().substring(2, 10), "3leave_4".to_string());
            let proof_2 = proofs.get("2").unwrap();
            assert_eq!(proof_2.m_left_hash, "1".to_string());
            assert_eq!(proof_2.m_merkle_proof[0].to_string().substring(2, 10), "3leave_4".to_string());
            assert_eq!(proof_2.m_merkle_proof[0].to_string().substring(0, 1), "r".to_string());
            let proof_3 = proofs.get("3").unwrap();
            assert_eq!(proof_3.m_left_hash, "".to_string());
            assert_eq!(proof_3.m_merkle_proof[0].to_string().substring(0, 1), "r".to_string());
            assert_eq!(proof_3.m_merkle_proof[0].to_string().substring(2, 10), "leave_4".to_string());
            assert_eq!(proof_3.m_merkle_proof[1].to_string().substring(0, 1), "l".to_string());
            assert_eq!(proof_3.m_merkle_proof[1].to_string().substring(2, 4), "12".to_string());
            let proof_4 = proofs.get("leave_4").unwrap();
            assert_eq!(proof_4.m_left_hash, "3".to_string());
            assert_eq!(proof_4.m_merkle_proof[0].to_string().substring(0, 1), "l".to_string());
            assert_eq!(proof_4.m_merkle_proof[0].to_string().substring(2, 4), "12".to_string());

            assert_eq!(root,
                       cmerkle::get_root_by_a_prove(
                           &"1".to_string(),
                           &proof_1.m_merkle_proof,
                           &proof_1.m_left_hash,
                           &"hashed".to_string(),
                           &"noHash".to_string()));

            assert_eq!(root,
                       cmerkle::get_root_by_a_prove(
                           &"2".to_string(),
                           &proofs["2"].m_merkle_proof,
                           &proofs["2"].m_left_hash,
                           &"hashed".to_string(),
                           &"noHash".to_string()));

            assert_eq!(root,
                       cmerkle::get_root_by_a_prove(
                           &"3".to_string(),
                           &proofs["3"].m_merkle_proof,
                           &proofs["3"].m_left_hash,
                           &"hashed".to_string(),
                           &"noHash".to_string()));
        }

        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate(&vec!["1".to_string(), "2".to_string(), "3".to_string()],
                                  &"hashed".to_string(),
                                  &"aliasHash".to_string(),
                                  &"".to_string());

            assert_eq!(root, "h(h(12)h(3leave_4))");
        }

        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate(&vec!["1".to_string(), "2".to_string(), "3".to_string()],
                                  &"string".to_string(),
                                  &"aliasHash".to_string(),
                                  &"".to_string());

            assert_eq!(root, "h(h(h(1)h(2))h(h(3)h(leave_4)))");
        }

        {
            let (
                root,
                proofs,
                _version,
                levels,
                leaves) =
                cmerkle::generate(&vec!["1".to_string(), "2".to_string(), "3".to_string()],
                                  &"string".to_string(),
                                  &"".to_string(),
                                  &"".to_string());

            let l1 = &"1".to_string();
            let l2 = &"2".to_string();
            let l3 = &"3".to_string();
            let l4 = &"leave_4".to_string();
            assert_eq!(root,
                       ccrypto::keccak256(&(
                           ccrypto::keccak256(
                               &(ccrypto::keccak256(l1) + &ccrypto::keccak256(l2))
                           ) + &ccrypto::keccak256(
                               &(ccrypto::keccak256(l3) + &ccrypto::keccak256(l4))
                           )
                       )
                       )
            );

            let the_proof: Vec<String> = vec![
                "r.".to_owned() + &ccrypto::keccak256(&"2".to_string()),
                "r.".to_owned() + &ccrypto::keccak256(&(ccrypto::keccak256(&"3".to_string()) +
                    &ccrypto::keccak256(&"leave_4".to_string())))];
            assert_eq!(proofs[&ccrypto::keccak256(&"1".to_string())].m_merkle_proof, the_proof);


            assert_eq!(root, cmerkle::get_root_by_a_prove(
                &"1".to_string(),
                &proofs[&ccrypto::keccak256(&"1".to_string())].m_merkle_proof,
                &proofs[&ccrypto::keccak256(&"1".to_string())].m_left_hash,
                &"string".to_string(),
                &"keccak256".to_string()));

            assert_eq!(root, cmerkle::get_root_by_a_prove(
                &"2".to_string(),
                &proofs[&ccrypto::keccak256(&"2".to_string())].m_merkle_proof,
                &proofs[&ccrypto::keccak256(&"2".to_string())].m_left_hash,
                &"string".to_string(),
                &"keccak256".to_string()));

            assert_eq!(root, cmerkle::get_root_by_a_prove(
                &"3".to_string(),
                &proofs[&ccrypto::keccak256(&"3".to_string())].m_merkle_proof,
                &proofs[&ccrypto::keccak256(&"3".to_string())].m_left_hash,
                &"string".to_string(),
                &"keccak256".to_string()));
        }
    }

    #[test]
    pub fn test_3_leaves_b() {
        let (
            root,
            proofs,
            _version,
            levels,
            leaves) =
            cmerkle::generate(&vec!["1".to_string(), "2".to_string(), "3".to_string()],
                              &"".to_string(),
                              &"".to_string(),
                              &"".to_string());
        assert_eq!(root, "6c4915f1849b0171846ef4d6d2abab6eeb3548a6c89d63c660b49ed738d4736a".to_string());
    }

    #[test]
    pub fn test_3_leaves_c() {
        let (
            root,
            proofs,
            _version,
            levels,
            leaves) =
            cmerkle::generate(&vec!["1".to_string(), "2".to_string(), "3".to_string()],
                              &"hashed".to_string(),
                              &"noHash".to_string(),
                              &"".to_string());
        assert_eq!(root, "123leave_4".to_string());
        assert_eq!(proofs["1"].m_left_hash, "");
        assert_eq!(proofs["1"].m_merkle_proof[0].substring(0, 1), "r");
        assert_eq!(proofs["1"].m_merkle_proof[0].substring(2, 3), "2");
        assert_eq!(proofs["1"].m_merkle_proof[1].substring(0, 1), "r");
        assert_eq!(proofs["1"].m_merkle_proof[1].substring(2, proofs["1"].m_merkle_proof[1].len()), "3leave_4");
        assert_eq!(proofs["2"].m_left_hash, "1");
        assert_eq!(proofs["2"].m_merkle_proof[0].substring(2, proofs["2"].m_merkle_proof[0].len()), "3leave_4");
        assert_eq!(proofs["2"].m_merkle_proof[0].substring(0, 1), "r");
        assert_eq!(proofs["3"].m_left_hash, "");
        assert_eq!(proofs["3"].m_merkle_proof[0].substring(0, 1), "r");
        assert_eq!(proofs["3"].m_merkle_proof[0].substring(2, proofs["3"].m_merkle_proof[0].len()), "leave_4");
        assert_eq!(root, cmerkle::get_root_by_a_prove(
            &"1".to_string(),
            &proofs["1"].m_merkle_proof,
            &proofs["1"].m_left_hash,
            &"hashed".to_string(),
            &"noHash".to_string()));
        assert_eq!(root, cmerkle::get_root_by_a_prove(
            &"2".to_string(),
            &proofs["2"].m_merkle_proof,
            &proofs["2"].m_left_hash,
            &"hashed".to_string(),
            &"noHash".to_string()));
        assert_eq!(root, cmerkle::get_root_by_a_prove(
            &"3".to_string(),
            &proofs["3"].m_merkle_proof,
            &proofs["3"].m_left_hash,
            &"hashed".to_string(),
            &"noHash".to_string()));
    }

    #[test]
    pub fn test_3_leaves_d() {
        let (
            root,
            proofs,
            _version,
            levels,
            leaves) =
            cmerkle::generate(&vec![
                "98325468840887230d248330de2c99f76750d131aa6076dbd9e9a0ab20f09fd0".to_string(),
                "ff1da71d8a78d13fd280d29c3f124e6e97b78a5c8317a2a9ff3d6c5f7294143f".to_string(),
                "3b071f3d67e907ed5e2615ee904b9135e7ad4db666dad72aa63af1b04076eb9d".to_string()],
                              &"".to_string(),
                              &"".to_string(),
                              &"".to_string());
        assert_eq!(root , ccrypto::keccak256(&(
            ccrypto::keccak256(&"98325468840887230d248330de2c99f76750d131aa6076dbd9e9a0ab20f09fd0ff1da71d8a78d13fd280d29c3f124e6e97b78a5c8317a2a9ff3d6c5f7294143f".to_string()) +
                &ccrypto::keccak256(&"3b071f3d67e907ed5e2615ee904b9135e7ad4db666dad72aa63af1b04076eb9dleave_4".to_string())))
        )
    }
}