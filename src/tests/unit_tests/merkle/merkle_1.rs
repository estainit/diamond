// use crate::lib::utils::cutils;
#[allow(unused_imports)]
use crate::lib::utils::cmerkle as cmerkle;

#[cfg(test)]
pub mod merkel_tests_1 {
    use crate::cmerkle;
    use crate::cmerkle::MerkleNodeDataTrait;
    use crate::lib::ccrypto;

    #[test]
    pub fn test_do_merkle() {
        let mrk = cmerkle::MerkleNodeData::new();
        assert_eq!(mrk.m_left_hash, "".to_string());
    }

    #[test]
    pub fn tests_1_leave() {
        {
            let (
                root,
                proofs,
                _version,
                _levels,
                _leaves) =
                cmerkle::generate(&vec!["a".to_string()],
                                  &"hashed".to_string(),
                                  &"noHash".to_string(),
                                  &"".to_string());
            assert_eq!(root, "a");
            assert_eq!(proofs.len(), 0);
        }


        {
            let (
                root,
                proofs,
                _version,
                _levels,
                _leaves) =
                cmerkle::generate(&vec!["1".to_string()],
                                  &"string".to_string(),
                                  &"".to_string(),
                                  &"".to_string());
            assert_eq!(root, ccrypto::keccak256(&"1".to_string()));
            assert_eq!(proofs.len(), 0);
        }

    }
}
