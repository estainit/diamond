use crate::lib::utils::cutils::cutils as cutils;

#[cfg(test)]
pub mod chunk_string_list {
    use crate::lib::utils::cutils;

    #[test]
    #[should_panic]
    pub fn test_do_panic() {
        panic!("oh NOOOOOOOO");
    }

    #[test]
    #[ignore]
    pub fn test_ignore() {
        assert!(1 == 11);
    }

    #[test]
    pub fn test1() {
        // panic!("oh NOOOOOOOO");
        assert!(1 == 1);
        assert_eq!(3, 1 + 1 + 1);
        assert_ne!(3, 1 + 1);

        // let res: vec! = {};
        let values: Vec<String> = vec!["a".to_string()];

        assert_eq!(cutils::cutils::chunk_to_vectors(vec![], 1).len(), 0);

        assert_eq!(cutils::cutils::chunk_to_vectors(vec!["a".to_string()], 1).len(), 1);
        assert_eq!(cutils::cutils::chunk_to_vectors(vec!["a".to_string()], 2).len(), 1);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string()], 1);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], vec!["a".to_string()]);
        assert_eq!(res[1], vec!["b".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string()], 2);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string()], 3);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string()], 4);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string(), "c".to_string()], 1);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], vec!["a".to_string()]);
        assert_eq!(res[1], vec!["b".to_string()]);
        assert_eq!(res[2], vec!["c".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string(), "c".to_string()], 2);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);
        assert_eq!(res[1], vec!["c".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string(), "c".to_string()], 3);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let mut res = cutils::cutils::chunk_to_vectors(vec!["a".to_string(), "b".to_string(), "c".to_string()], 4);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    }
}