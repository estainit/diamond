
// use crate::lib::utils::cutils;
// use crate::lib::utils::cmerkle as cmerkle;

#[cfg(test)]
pub mod do_hash_a_node {
    use crate::cmerkle;

    #[test]
    pub fn test_1() {
        assert_eq!(cmerkle::do_hash_a_node(&"a".to_string(), &"noHash".to_string()), "a");
        assert_eq!(cmerkle::do_hash_a_node(&"a".to_string(), &"aliasHash".to_string()), "h(a)");
        // assert_eq!(cmerkle::do_hash_a_node(&"a".to_string(), &"keccak256".to_string()), "80084bf2fba02475726feb2cab2d8215eab14bc6bdd8bfb2c8151257032ecd8b");
        // assert_eq!(cmerkle::do_hash_a_node(&"a".to_string(), &"".to_string()), "80084bf2fba02475726feb2cab2d8215eab14bc6bdd8bfb2c8151257032ecd8b");
    }
}