#[cfg(test)]
pub mod tests_cycle_times {
    use crate::lib::constants as CConsts;
    use crate::lib::utils::cutils as cutils;
    use crate::lib::utils::version_handler;
    use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::{calc_coinbased_output_maturation_date};

    #[test]
    pub fn test_time_func_1() {
        assert_eq!(cutils::make_str_date_from_date_object(cutils::make_date_from_str(&"2020-01-01 00:00:00".to_string())), "2020-01-01 00:00:00");
    }

    #[test]
    pub fn tests_minutes_before() {
        assert_eq!(cutils::minutes_before(1, &"2000-01-01 12:00:00".to_string()), "2000-01-01 11:59:00");
        assert_eq!(cutils::minutes_before(1, &"2000-02-02 00:00:00".to_string()), "2000-02-01 23:59:00");
        assert_eq!(cutils::minutes_before(1, &"2000-02-01 00:00:00".to_string()), "2000-01-31 23:59:00");
        assert_eq!(cutils::minutes_before(1, &"2000-01-01 00:00:00".to_string()), "1999-12-31 23:59:00");
        assert_eq!(cutils::minutes_before(12 * 60, &"2000-01-01 12:00:01".to_string()), "2000-01-01 00:00:01");
        assert_eq!(cutils::minutes_before(12 * 60, &"2000-02-02 00:00:00".to_string()), "2000-02-01 12:00:00");
        assert_eq!(cutils::minutes_before(12 * 60, &"2000-02-01 11:00:00".to_string()), "2000-01-31 23:00:00");
        assert_eq!(cutils::minutes_before(12 * 60, &"2000-01-01 11:00:00".to_string()), "1999-12-31 23:00:00");
        assert_eq!(cutils::minutes_before(5 * 720, &"2020-02-02 00:00:00".to_string()), "2020-01-30 12:00:00");
    }

    #[test]
    pub fn tests_years_before() {
        assert_eq!(cutils::yearsBefore(1, &"2020-02-02 00:00:00".to_string()), "2019-02-02 00:00:00");
    }
}
