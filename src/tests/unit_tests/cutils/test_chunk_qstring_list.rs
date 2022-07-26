
#[cfg(test)]
pub mod chunk_string_list {
    use crate::lib::utils::cutils;

    #[test]
    #[should_panic]
    pub fn test_do_panic() {
        panic!("oh NOOOOOOOO");
    }

    #[test]
    pub fn test1() {
        // panic!("oh NOOOOOOOO");
        assert!(1 == 1);
        assert_eq!(3, 1 + 1 + 1);
        assert_ne!(3, 1 + 1);

        assert_eq!(cutils::chunk_to_vvstring(&vec![], 1).len(), 0);

        assert_eq!(cutils::chunk_to_vvstring(&vec!["a".to_string()], 1).len(), 1);
        assert_eq!(cutils::chunk_to_vvstring(&vec!["a".to_string()], 2).len(), 1);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string()], 1);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], vec!["a".to_string()]);
        assert_eq!(res[1], vec!["b".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string()], 2);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string()], 3);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string()], 4);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string(), "c".to_string()], 1);
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], vec!["a".to_string()]);
        assert_eq!(res[1], vec!["b".to_string()]);
        assert_eq!(res[2], vec!["c".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string(), "c".to_string()], 2);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string()]);
        assert_eq!(res[1], vec!["c".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string(), "c".to_string()], 3);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string(), "c".to_string()]);

        let res = cutils::chunk_to_vvstring(&vec!["a".to_string(), "b".to_string(), "c".to_string()], 4);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0], vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    }

    #[test]
    pub fn test_chunk_string_1_1(){
        let chunks = cutils::chunk_string(&"a1".to_string(), 16);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "a1");
    }

    #[test]
    pub fn test_chunk_string_1_2(){
        let chunks = cutils::chunk_string(&"a123456789bcdef0".to_string(), 16);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "a123456789bcdef0");
    }

    #[test]
    pub fn test_chunk_string_2(){
        let chunks = cutils::chunk_string(&"a123456789bcdef0a123456789bcdef0".to_string(), 16);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0], chunks[1]);
        assert_eq!(chunks[0], "a123456789bcdef0");
    }

    #[test]
    pub fn test_chunk_string_3(){
        let chunks = cutils::chunk_string(&"a123456789bcdef0a123456789bcdef0a".to_string(), 16);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], chunks[1]);
        assert_eq!(chunks[0], "a123456789bcdef0");
        assert_eq!(chunks[2], "a");
    }
}