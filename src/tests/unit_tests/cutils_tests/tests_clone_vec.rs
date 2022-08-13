#[cfg(test)]
pub mod merkel_tests_1 {
    use crate::cutils::{clone_vec, clone_vvec};

    #[test]
    pub fn test_clone_vec() {
        let the_vec:Vec<String>= vec!["a".to_string(), "b".to_string()];
        let cloned_vec = clone_vec(&the_vec);
        assert_eq!(cloned_vec[0], "a".to_string());
        assert_eq!(cloned_vec[1], "b".to_string());
    }

    #[test]
    pub fn test_clone_vvec() {
        let the_vec_1:Vec<String>= vec!["a".to_string(), "b".to_string()];
        let the_vec_2:Vec<String>= vec!["c".to_string(), "d".to_string()];
        let the_vec:Vec<Vec<String>>= vec![the_vec_1, the_vec_2];
        let cloned_vvec = clone_vvec(&the_vec);
        assert_eq!(cloned_vvec[0][0], "a".to_string());
        assert_eq!(cloned_vvec[0][1], "b".to_string());
        assert_eq!(cloned_vvec[1][0], "c".to_string());
        assert_eq!(cloned_vvec[1][1], "d".to_string());
    }
}