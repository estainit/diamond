#[cfg(test)]
pub mod tests_cycle_times {
    use crate::{application, constants};
    use crate::lib::utils::cutils as cutils;
    use crate::lib::utils::version_handler;
    use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::{calc_coinbased_output_maturation_date};

    #[test]
    pub fn test_time_func_1() {
        if application().cycle_length() == 1
        {
            let tmp = application().make_date_from_str(&"2020-01-01 00:00:00".to_string());
            let tmp = application().make_str_date_from_date_object(tmp);
            assert_eq!(tmp, "2020-01-01 00:00:00");
        }
    }

    #[test]
    pub fn tests_convert_float_to_string1() {
        assert_eq!(cutils::convert_float_to_string(99.999999999989996, constants::FLOAT_LENGTH), "99.99999999999");
    }

    #[test]
    pub fn tests_convert_float_to_string2() {
        assert_eq!(cutils::convert_float_to_string(0.003_599_632_829_999_999_9, constants::FLOAT_LENGTH), "0.00359963283");
    }

    #[test]
    pub fn tests_convert_float_to_string3() {
        assert_eq!(cutils::convert_float_to_string(0.0, constants::FLOAT_LENGTH), "0.0");
    }

    #[test]
    pub fn tests_convert_float_to_string4() {
        assert_eq!(cutils::convert_float_to_string(0.00, constants::FLOAT_LENGTH), "0.0");
    }

    #[test]
    pub fn tests_version_controls() {
        assert_eq!(version_handler::is_older_than("0.0.8".to_string(), "0.0.8".to_string()), 0);
    }

    #[test]
    pub fn tests_1() {
        if application().cycle_length() == 1
        {
            let tmp = application().get_coinbase_cycle_number(&"2012-11-05 00:00:01".to_string());
            assert_eq!(tmp, "00:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 01:08:00".to_string()), "00:00:00".to_string());
            //TODO: Note: this test works only when cycle_length=1 in config file
            //Should control coinbase date range is valid (12 hour per cycle)

            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 07:08:00".to_string()), "00:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 11:59:59".to_string()), "00:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 12:00:00".to_string()), "12:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 12:00:01".to_string()), "12:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 13:08:00".to_string()), "12:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_number(&"2012-11-05 23:59:59".to_string()), "12:00:00".to_string());

            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 00:00:00".to_string()), 0);
            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 00:01:01".to_string()), 1);
            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 07:00:01".to_string()), 420);
            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 12:00:01".to_string()), 0);
            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 13:00:01".to_string()), 60);
            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 11:59:59".to_string()), 719);
            assert_eq!(application().get_cycle_elapsed_by_minutes("2012-11-05 23:59:59".to_string()), 719);

            assert_eq!(application().get_coinbase_range(&"2012-11-05 00:00:01".to_string()).from, "2012-11-05 00:00:00".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 00:00:01".to_string()).to, "2012-11-05 11:59:59".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 11:59:59".to_string()).from, "2012-11-05 00:00:00".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 11:59:59".to_string()).to, "2012-11-05 11:59:59".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 12:00:00".to_string()).from, "2012-11-05 12:00:00".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 12:00:00".to_string()).to, "2012-11-05 23:59:59".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 23:59:59".to_string()).from, "2012-11-05 12:00:00".to_string());
            assert_eq!(application().get_coinbase_range(&"2012-11-05 23:59:59".to_string()).to, "2012-11-05 23:59:59".to_string());


            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 00:00:00".to_string()), "2012-11-05 00:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 00:00:01".to_string()), "2012-11-05 00:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 11:59:59".to_string()), "2012-11-05 00:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 12:00:00".to_string()), "2012-11-05 12:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 12:00:01".to_string()), "2012-11-05 12:00:00".to_string());
            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 23:59:59".to_string()), "2012-11-05 12:00:00".to_string());
        }
        else if application().cycle_length() == 10
        {
            assert_eq!(application().get_coinbase_cycle_stamp(&"2012-11-05 00:00:00".to_string()), "2012-11-05 000".to_string());

        }
    }

    #[test]
    pub fn test_ature_times()
    {

        // coinbase_plan
        if application().get_cycle_by_minutes() == 5 {
            // if (CoinbaseUTXOHandler.calc_coinbased_output_maturation_date("&2019-05-10 00:00:00") != "2019-05-10 00:15:00");
            // if (CoinbaseUTXOHandler.calc_coinbased_output_maturation_date("&2019-05-10 00:02:00") != "2019-05-10 00:15:00");
            // if (CoinbaseUTXOHandler.calc_coinbased_output_maturation_date("&2019-05-10 00:04:59") != "2019-05-10 00:15:00");
        }

        if application().get_cycle_by_minutes() == 720 {
            let tmp = calc_coinbased_output_maturation_date(&"2019-05-10 00:00:00".to_string());
            assert_eq!(tmp, "2019-05-11 00:00:00".to_string());
            // assert_eq!(calc_coinbased_output_maturation_date(&"2019-05-10 11:59:59".to_string()), "2019-05-11 00:00:00".to_string());
            // assert_eq!(calc_coinbased_output_maturation_date(&"2019-05-10 12:00:00".to_string()), "2019-05-11 12:00:00".to_string());
            // assert_eq!(calc_coinbased_output_maturation_date(&"2019-05-10 23:59:59".to_string()), "2019-05-11 12:00:00".to_string());
        }
    }

    #[test]
    pub fn test_date_range()
    {
        if application().cycle_length() == 1
        {
            //TODO: Note: this test works only when cycle_length=1 in config file
            let tmp = application().get_cb_coins_date_range(&"2017-07-22 00:00:00".to_string()).from;
            assert_eq!(tmp, "2017-07-21 00:00:00".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 00:00:00".to_string()).to, "2017-07-21 11:59:59".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 11:59:59".to_string()).from, "2017-07-21 00:00:00".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 11:59:59".to_string()).to, "2017-07-21 11:59:59".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 12:00:00".to_string()).from, "2017-07-21 12:00:00".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 12:00:00".to_string()).to, "2017-07-21 23:59:59".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 23:59:00".to_string()).from, "2017-07-21 12:00:00".to_string());
            assert_eq!(application().get_cb_coins_date_range(&"2017-07-22 23:59:00".to_string()).to, "2017-07-21 23:59:59".to_string());
        }
    }

    #[test]
    pub fn test_cb_info()
    {

        // getCoinbaseInfo
        // Should getCoinbaseInfo
        {
            if application().cycle_length() == 1
            {
                let (cycle_stamp, from_, to_, from_hour, to_hour) = application().get_coinbase_info(&"".to_string(), &"2016-01-01 00:00:00".to_string());
                assert_eq!(cycle_stamp, "2016-01-01 00:00:00".to_string());
                assert_eq!(from_, "2016-01-01 00:00:00".to_string());
                assert_eq!(from_hour, "00:00:00".to_string());
                assert_eq!(to_, "2016-01-01 11:59:59".to_string());
                assert_eq!(to_hour, "11:59:59".to_string());
            }
        }

        {
            let (cycle_stamp, from_, to_, from_hour, to_hour) = application().get_coinbase_info(&"".to_string(), &"2016-01-01 12:00:00".to_string());
            assert_eq!(cycle_stamp, "2016-01-01 12:00:00".to_string());
            assert_eq!(from_, "2016-01-01 12:00:00".to_string());
            assert_eq!(from_hour, "12:00:00".to_string());
            assert_eq!(to_, "2016-01-01 23:59:59".to_string());
            assert_eq!(to_hour, "23:59:59".to_string());
        }
    }
}
