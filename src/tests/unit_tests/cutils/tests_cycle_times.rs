#[cfg(test)]
pub mod tests_cycle_times {
    use crate::lib::constants as CConsts;
    use crate::lib::utils::cutils as cutils;
    use crate::lib::utils::version_handler;
    use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::{calc_coinbased_output_maturation_date};

    #[test]
    pub fn test_ddd() {
        assert!(1 == 1);
    }

    #[test]
    pub fn tests_convert_float_to_string() {
        assert!(cutils::convert_float_to_string(99.999999999989996, 11) == "99.99999999999");
        assert!(cutils::convert_float_to_string(0.0035996328299999999, 11) == "0.00359963282");
        assert!(cutils::convert_float_to_string(0.0, 11) == "0");
        assert!(cutils::convert_float_to_string(0.00, 11) == "0");
    }

    #[test]
    pub fn tests_version_controls() {
        assert_eq!(version_handler::is_older_than("0.0.8".to_string(), "0.0.8".to_string()), 0);
    }

    #[test]
    pub fn tests_1() {
        //Should control coinbase date range is valid (12 hour per cycle)
        if CConsts::TIME_GAIN == 1 {
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 00:00:01".to_string()), "00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 01:08:00".to_string()), "00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 07:08:00".to_string()), "00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 11:59:59".to_string()), "00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 12:00:00".to_string()), "12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 12:00:01".to_string()), "12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 13:08:00".to_string()), "12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_number("2012-11-05 23:59:59".to_string()), "12:00:00".to_string());

            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 00:00:00".to_string()), 0);
            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 00:01:01".to_string()), 1);
            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 07:00:01".to_string()), 420);
            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 12:00:01".to_string()), 0);
            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 13:00:01".to_string()), 60);
            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 11:59:59".to_string()), 719);
            assert_eq!(cutils::get_cycle_elapsed_by_minutes("2012-11-05 23:59:59".to_string()), 719);

            assert_eq!(cutils::get_coinbase_range("2012-11-05 00:00:01".to_string()).from, "2012-11-05 00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 00:00:01".to_string()).to, "2012-11-05 11:59:59".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 11:59:59".to_string()).from, "2012-11-05 00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 11:59:59".to_string()).to, "2012-11-05 11:59:59".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 12:00:00".to_string()).from, "2012-11-05 12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 12:00:00".to_string()).to, "2012-11-05 23:59:59".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 23:59:59".to_string()).from, "2012-11-05 12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_range("2012-11-05 23:59:59".to_string()).to, "2012-11-05 23:59:59".to_string());


            assert_eq!(cutils::get_coinbase_cycle_stamp("2012-11-05 00:00:00".to_string()), "2012-11-05 00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_stamp("2012-11-05 00:00:01".to_string()), "2012-11-05 00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_stamp("2012-11-05 11:59:59".to_string()), "2012-11-05 00:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_stamp("2012-11-05 12:00:00".to_string()), "2012-11-05 12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_stamp("2012-11-05 12:00:01".to_string()), "2012-11-05 12:00:00".to_string());
            assert_eq!(cutils::get_coinbase_cycle_stamp("2012-11-05 23:59:59".to_string()), "2012-11-05 12:00:00".to_string());
        }
    }

    #[test]
    pub fn test_ature_times()
    {

        // coinbase_plan
        if cutils::get_cycle_by_minutes() == 5 {
            // if (CoinbaseUTXOHandler.calc_coinbased_output_maturation_date("2019-05-10 00:00:00") != "2019-05-10 00:15:00");
            // if (CoinbaseUTXOHandler.calc_coinbased_output_maturation_date("2019-05-10 00:02:00") != "2019-05-10 00:15:00");
            // if (CoinbaseUTXOHandler.calc_coinbased_output_maturation_date("2019-05-10 00:04:59") != "2019-05-10 00:15:00");
        }

        if cutils::get_cycle_by_minutes() == 720 {
            assert_eq!(calc_coinbased_output_maturation_date("2019-05-10 00:00:00".to_string()), "2019-05-11 00:00:00".to_string());
            assert_eq!(calc_coinbased_output_maturation_date("2019-05-10 11:59:59".to_string()), "2019-05-11 00:00:00".to_string());
            assert_eq!(calc_coinbased_output_maturation_date("2019-05-10 12:00:00".to_string()), "2019-05-11 12:00:00".to_string());
            assert_eq!(calc_coinbased_output_maturation_date("2019-05-10 23:59:59".to_string()), "2019-05-11 12:00:00".to_string());
        }
    }

    #[test]
    pub fn test_date_range()
    {
        if CConsts::TIME_GAIN == 1 {
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 00:00:00".to_string()).from, "2017-07-21 00:00:00".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 00:00:00".to_string()).to, "2017-07-21 11:59:59".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 11:59:59".to_string()).from, "2017-07-21 00:00:00".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 11:59:59".to_string()).to, "2017-07-21 11:59:59".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 12:00:00".to_string()).from, "2017-07-21 12:00:00".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 12:00:00".to_string()).to, "2017-07-21 23:59:59".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 23:59:00".to_string()).from, "2017-07-21 12:00:00".to_string());
            assert_eq!(cutils::get_cb_utxos_date_range("2017-07-22 23:59:00".to_string()).to, "2017-07-21 23:59:59".to_string());
        }
    }

    #[test]
    pub fn test_cb_info()
    {

        // getCoinbaseInfo
        // Should getCoinbaseInfo
        {
            let(cycle_stamp, from_, to_, from_hour, to_hour) = cutils::get_coinbase_info("".to_string(), "2016-01-01 00:00:00".to_string());
            assert_eq!(cycle_stamp,"2016-01-01 00:00:00".to_string());
            assert_eq!(from_,"2016-01-01 00:00:00".to_string());
            assert_eq!(from_hour,"00:00:00".to_string());
            assert_eq!(to_,"2016-01-01 11:59:59".to_string());
            assert_eq!(to_hour,"11:59:59".to_string());
        }

        {
            let(cycle_stamp, from_, to_, from_hour, to_hour) = cutils::get_coinbase_info("".to_string(), "2016-01-01 12:00:00".to_string());
            assert_eq!(cycle_stamp,"2016-01-01 12:00:00".to_string());
            assert_eq!(from_,"2016-01-01 12:00:00".to_string());
            assert_eq!(from_hour,"12:00:00".to_string());
            assert_eq!(to_,"2016-01-01 23:59:59".to_string());
            assert_eq!(to_hour,"23:59:59".to_string());
        }
    }
}
