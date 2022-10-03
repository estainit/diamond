#[cfg(test)]
pub mod tests_time_funcs {
    use serde_json::Value::String;
    use crate::application;

    #[test]
    pub fn test_1() {
        let res = application().make_date_from_str(
            &"2022-10-03 09:25:12".to_string());
        let dt = format!("{}", res);
        assert_eq!(dt, "2022-10-03 09:25:12 +00:00");
    }
}