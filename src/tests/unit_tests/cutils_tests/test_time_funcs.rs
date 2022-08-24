#[cfg(test)]
pub mod tests_cycle_times {
    use crate::application;

    #[test]
    pub fn test_time_func_1() {
        let tmp = application().make_date_from_str(&"2020-01-01 00:00:00".to_string());
        let tmp =application().make_str_date_from_date_object(tmp);
        assert_eq!(tmp, "2020-01-01 00:00:00");
    }

    #[test]
    pub fn tests_minutes_before() {
        let tmp = application().minutes_before(1, &"2000-01-01 12:00:00".to_string());
        assert_eq!(tmp, "2000-01-01 11:59:00");
        assert_eq!(application().minutes_before(1, &"2000-02-02 00:00:00".to_string()), "2000-02-01 23:59:00");
        assert_eq!(application().minutes_before(1, &"2000-02-01 00:00:00".to_string()), "2000-01-31 23:59:00");
        assert_eq!(application().minutes_before(1, &"2000-01-01 00:00:00".to_string()), "1999-12-31 23:59:00");
        assert_eq!(application().minutes_before(12 * 60, &"2000-01-01 12:00:01".to_string()), "2000-01-01 00:00:01");
        assert_eq!(application().minutes_before(12 * 60, &"2000-02-02 00:00:00".to_string()), "2000-02-01 12:00:00");
        assert_eq!(application().minutes_before(12 * 60, &"2000-02-01 11:00:00".to_string()), "2000-01-31 23:00:00");
        assert_eq!(application().minutes_before(12 * 60, &"2000-01-01 11:00:00".to_string()), "1999-12-31 23:00:00");
        assert_eq!(application().minutes_before(5 * 720, &"2020-02-02 00:00:00".to_string()), "2020-01-30 12:00:00");
    }

    #[test]
    pub fn tests_years_before() {
        let tmp = application().years_before(1, &"2020-02-02 00:00:00".to_string());
        assert_eq!(tmp, "2019-02-02 00:00:00");
    }
}
