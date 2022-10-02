use std::collections::HashMap;
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use crate::{CMachine, constants, dlog};
use crate::cutils::{padding_left};
use crate::lib::custom_types::{CDateT, TimeByMinutesT, TimeBySecT};

pub struct AppParams {
    m_app_machine_id: String,
    m_app_config_file: String,
    m_app_is_develop_mod: bool,
    m_app_clone_id: i8,
    m_app_should_loop_threads: bool,

    m_app_is_db_connected: bool,
    m_app_is_db_initialized: bool,

    m_app_email_is_active: bool,
    m_app_use_hard_disk_as_a_buffer: bool,
    m_app_should_run_web_server: bool,
    m_app_web_server_address: String,
    #[allow(dead_code, unused)]
    m_app_config_source: String,
    m_app_hard_root_path: String,

    m_app_forced_launch_date:bool,
    m_app_launch_date: String,

    m_app_db_host: String,
    m_app_db_name: String,
    m_app_db_user: String,
    m_app_db_pass: String,

}

impl AppParams {
    pub fn dummy_init(&self) {
        // eprintln!("dummy_init in AppParams was called.");
    }

    pub fn new() -> Self {
        eprintln!("New AppParams was create.");

        AppParams {
            m_app_machine_id: "".to_string(),
            m_app_config_file: "".to_string(),
            m_app_is_develop_mod: false,
            m_app_clone_id: 0,
            m_app_should_loop_threads: false,
            m_app_is_db_connected: false,
            m_app_is_db_initialized: false,
            m_app_email_is_active: false,
            m_app_use_hard_disk_as_a_buffer: false,
            m_app_should_run_web_server: false,
            m_app_web_server_address: "".to_string(),
            m_app_config_source: "".to_string(),
            m_app_hard_root_path: "".to_string(),

            m_app_forced_launch_date: false,
            m_app_launch_date: "".to_string(),

            m_app_db_host: "".to_string(),
            m_app_db_name: "".to_string(),
            m_app_db_user: "".to_string(),
            m_app_db_pass: "".to_string(),
        }
    }

    pub fn setup_app(&mut self, machine: &CMachine) -> bool {
        self.m_app_clone_id = machine.get_app_clone_id();
        self.m_app_machine_id = machine.get_app_machine_id();
        self.m_app_is_develop_mod = machine.is_develop_mod();

        self.m_app_forced_launch_date = machine.does_launch_date();
        self.m_app_launch_date = machine.get_launch_date();

        self.m_app_db_host = machine.m_db_host.clone();
        self.m_app_db_name = machine.m_db_name.clone();
        self.m_app_db_user = machine.m_db_user.clone();
        self.m_app_db_pass = machine.m_db_pass.clone();

        self.m_app_config_file = machine.m_config_file.clone();
        self.m_app_should_loop_threads = machine.m_should_loop_threads;
        self.m_app_email_is_active = machine.m_email_is_active;
        self.m_app_use_hard_disk_as_a_buffer = machine.m_use_hard_disk_as_a_buffer.clone();
        self.m_app_should_run_web_server = machine.m_should_run_web_server.clone();
        self.m_app_web_server_address = machine.m_web_server_address.clone();
        self.m_app_hard_root_path = machine.m_hard_root_path.clone();

        true
    }

    pub fn cycle_length(&self) -> u32 {
        10
        // in test environment it would be 10, for 10 minutes
    }

    pub fn id(&self) -> i8 {
        self.m_app_clone_id
    }

    pub fn machine_id(&self) -> String {
        self.m_app_machine_id.clone()
    }

    pub fn root_path(&self) -> String {
        self.m_app_hard_root_path.clone()
    }

    pub fn db_host(&self) -> String
    {
        self.m_app_db_host.clone()
    }

    pub fn db_name(&self) -> String
    {
        self.m_app_db_name.clone()
    }

    pub fn db_user(&self) -> String
    {
        self.m_app_db_user.clone()
    }

    pub fn db_pass(&self) -> String
    {
        self.m_app_db_pass.clone()
    }

    pub fn email_is_active(&self) -> bool
    {
        self.m_app_email_is_active
    }

    pub fn is_develop_mod(&self) -> bool
    {
        self.m_app_is_develop_mod
    }

    pub fn set_db_connected(&mut self, status: bool)
    {
        self.m_app_is_db_connected = status;
    }

    #[allow(dead_code, unused)]
    pub fn is_db_connected(&self) -> bool
    {
        self.m_app_is_db_connected
    }

    pub fn set_db_initialized(&mut self, status: bool)
    {
        self.m_app_is_db_initialized = status;
    }

    pub fn is_db_initialized(&self) -> bool
    {
        self.m_app_is_db_initialized
    }

    pub fn should_loop_threads(&self) -> bool
    {
        self.m_app_should_loop_threads
    }

    pub fn use_hard_disk_as_a_buffer(&self) -> bool
    {
        self.m_app_use_hard_disk_as_a_buffer
    }

    pub fn should_run_web_server(&self) -> bool
    {
        self.m_app_should_run_web_server
    }

    pub fn web_server_address(&self) -> String
    {
        self.m_app_web_server_address.clone()
    }


    // time functionalities

    //old_name_was getCoinbaseCycleStamp
    pub fn get_coinbase_cycle_stamp(&self, c_date: &CDateT) -> String {
        if self.cycle_length() == 1
        {
            let range = self.get_a_cycle_range(c_date, 0, 0).from;
            return range;
        }

        let day: Vec<&str> = c_date.split(" ").collect();
        let cycle_number = self.get_coinbase_cycle_number(c_date);
        return day[0].to_string() + " " + &padding_left(&cycle_number, 3);
    }

    //old_name_was getACycleRange
    pub fn get_a_cycle_range(
        &self,
        c_date_: &CDateT,
        back_by_cycle: u8,
        forward_by_cycle: u8, ) -> TimeRange
    {
        let mut c_date: CDateT = c_date_.clone();
        if c_date == "".to_string() {
            c_date = self.now();
        }

        if self.cycle_length() == 1
        {
            // one extra step to resolve +- summer time
            let h_: Vec<&str> = c_date.split(" ").collect();
            let h_: String = h_[1].to_string();
            let h_: Vec<&str> = h_.split(":").collect();
            let h_: String = h_[0].to_string();
            let h_: u16 = h_.parse::<u16>().unwrap();
            let h: String;
            if h_ >= 12 {
                h = "18:00:00".to_string();
            } else {
                h = "06:00:00".to_string();
            }
            let date_dtl: Vec<&str> = c_date.split(" ").collect();
            c_date = date_dtl[0].to_string() + " " + &h;
        }
        // else {
        //     c_date
        // }

        let min_creation_date: String;
        if forward_by_cycle == 0 {
            let back_in_time = (back_by_cycle as TimeByMinutesT * self.get_cycle_by_minutes()) as TimeByMinutesT;
            min_creation_date = self.minutes_before(back_in_time, &c_date);
        } else {
            let back_in_time = (forward_by_cycle as TimeByMinutesT * self.get_cycle_by_minutes()) as TimeByMinutesT;
            min_creation_date = self.minutes_after(back_in_time, &c_date);
        }

        let day: Vec<&str> = min_creation_date.split(" ").collect();
        let day = day[0].to_string();
        let time_details: Vec<&str> = min_creation_date.split(" ").collect();
        let time_details: String = time_details[1].to_string();
        let time_details: Vec<&str> = time_details.split(":").collect();
        let minutes_h: String = time_details[0].to_string();
        let minutes_h: TimeByMinutesT = minutes_h.parse::<TimeByMinutesT>().unwrap() * 60;
        let minutes_m: String = time_details[1].to_string();
        let minutes_m: TimeByMinutesT = minutes_m.parse::<TimeByMinutesT>().unwrap();
        // let minutes: u32 = (time_details[0].to_string().parse::<u32>().unwrap() * 60) + time_details[1].to_string().parse::<u32>().unwrap();
        let minutes: TimeByMinutesT = minutes_h + minutes_m;
        let start_minute: TimeByMinutesT = (minutes / self.get_cycle_by_minutes()) as TimeByMinutesT * self.get_cycle_by_minutes();
        let end_minute: TimeByMinutesT = start_minute + self.get_cycle_by_minutes() - 1;
        let from_ = day.clone() + " " + &self.convert_minutes_to_hhmm(start_minute) + ":00";
        let to_ = day.clone() + " " + &self.convert_minutes_to_hhmm(end_minute) + ":59";

        return TimeRange {
            from: from_,
            to: to_,
        };
    }


    //old name was getCycleByMinutes
    pub fn get_cycle_by_minutes(&self) -> TimeByMinutesT
    {
        if self.cycle_length() == 1 {
            return constants::STANDARD_CYCLE_BY_MINUTES as TimeByMinutesT;
        }
        return self.cycle_length() as TimeByMinutesT;
    }


    //old name was minutesBefore
    pub fn minutes_before(&self, back_in_time_by_minutes: u64, c_date: &CDateT) -> String
    {
        let mut since_epoch: i64;
        if c_date == "" {
            // since_epoch = QDateTime::currentDateTimeUtc().toSecsSinceEpoch();
            since_epoch = Utc::now().timestamp();
        } else {
            let dt = self.make_date_from_str(&c_date);
            since_epoch = dt.timestamp();
        }
        since_epoch -= (back_in_time_by_minutes * 60) as i64;
        let dt = Utc.timestamp(since_epoch, 0);
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    //old_name_was minutesAfter
    pub fn minutes_after(&self, forward_in_time_by_minutes: TimeByMinutesT, c_date: &CDateT) -> CDateT
    {
        let mut since_epoch: i64;
        if c_date == "" {
            since_epoch = Utc::now().timestamp();
        } else {
            let _t_ = self.add_fff_zzzz_to_yyyymmdd(c_date.clone());
            let dt = self.make_date_from_str(&c_date);
            since_epoch = dt.timestamp();
        }
        since_epoch += (forward_in_time_by_minutes * 60) as i64;
        let dt = Utc.timestamp(since_epoch, 0);
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    //old_name_was secondsAfter
    pub fn seconds_after(&self, forward_in_time_by_seconds: TimeBySecT, c_date: &CDateT) -> CDateT
    {
        let mut since_epoch: i64;
        if c_date == ""
        {
            since_epoch = Utc::now().timestamp();
        } else {
            let dt = self.make_date_from_str(&c_date);
            since_epoch = dt.timestamp();
        }
        since_epoch += forward_in_time_by_seconds as i64;
        let dt = Utc.timestamp(since_epoch, 0);
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    //old_name_was getCoinbaseCycleNumber
    #[allow(dead_code, unused)]
    pub fn get_coinbase_cycle_number(&self, c_date: &CDateT) -> String {
        let minutes: u32;
        if c_date == ""
        {
            minutes = self.get_now_by_minutes();
        } else {
            let minutes_dtl1: Vec<&str> = c_date.split(" ").collect();
            let minutes_dtl2: String = minutes_dtl1[1].to_string().clone();
            let minutes_dtl3: Vec<&str> = minutes_dtl2.split(":").collect();
            minutes = (minutes_dtl3[0].to_string().parse::<u32>().unwrap() * 60) + minutes_dtl3[1].to_string().parse::<u32>().unwrap();
        }

        let cycle_number: String;
        if self.cycle_length() == 1 {
            cycle_number = self.is_am_or_pm(minutes);
        } else {
            cycle_number = (minutes / self.get_cycle_by_minutes() as u32).to_string();
        }
        return cycle_number;
    }

    pub fn does_forced_launch_date(&self) -> bool
    {
        self.m_app_forced_launch_date
    }

    pub fn launch_date(&self) -> CDateT
    {
        self.m_app_launch_date.clone()
    }

    //old_name_was getCycleBySeconds
    pub fn get_cycle_by_seconds(&self) -> TimeBySecT
    {
        return self.get_cycle_by_minutes() * 60;
    }

    //old_name_was getCoinbaseAgeByMinutes
    pub fn get_coinbase_age_by_minutes(&self, c_date: &CDateT) -> TimeByMinutesT
    {
        let cycle_range = self.get_a_cycle_range(c_date, 0, 0);
        return self.time_diff(
            cycle_range.from.clone(),
            c_date.clone()).as_minutes;
    }

    //old_name_was getCoinbaseAgeBySecond
    pub fn get_coinbase_age_by_seconds(&self, c_date: &CDateT) -> TimeBySecT
    {
        return self.get_coinbase_age_by_minutes(c_date) * 60;
    }

    //old_name_was getPrevCoinbaseInfo
    pub fn get_prev_coinbase_info(
        &self,
        c_date: &CDateT)
        -> (String, String, String, String, String)
    {
        let cycle_range = self.get_a_cycle_range(
            c_date,
            1,
            0);

        let info = self.get_coinbase_info(
            &cycle_range.from,
            "");
        info
    }

    //old_name_was timeDiff
    pub fn time_diff(&self, from_t_: CDateT, to_t_: CDateT) -> TimeDiff
    {
        let mut from_t = from_t_;
        if from_t == ""
        {
            from_t = self.now();
        }

        let mut to_t = to_t_;
        if to_t == ""
        {
            to_t = self.now();
        }

        let mut res: TimeDiff = TimeDiff::new();
        let start_t = self.make_date_from_str(&from_t);
        let end_t = self.make_date_from_str(&to_t);
        let gap_duration = end_t - start_t;

        res.as_seconds = gap_duration.num_seconds() as u64; // entire gap by seconds
        res.as_minutes = gap_duration.num_minutes() as u64;
        res.as_hours = gap_duration.num_hours() as u64;
        res.as_days = gap_duration.num_days() as u64;
        res.as_months = (res.as_days / 30) as u64;  // FIXME: more test
        res.as_years = (res.as_months / 12) as u64; // FIXME: more test

        res.days = res.as_days % 30;       // FIXME: more test
        res.hours = res.as_hours % 24;     // FIXME: more test
        res.minutes = res.as_minutes % 60; // FIXME: more test
        res.seconds = res.as_seconds % 60; // FIXME: more test
        // FIXME: implement the other missed properties
        return res;
    }


    //old_name_was getNow
    pub fn now(&self) -> String
    {
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
    }

    //old_name_was getNow
    pub fn get_now_compress(&self) -> String
    {
        Utc::now().format("%Y%m%d%H%M%S%3f").to_string()
    }

    pub fn get_now_sss(&self) -> String
    {
        Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }

    pub fn get_since_epoch(&self) -> i64
    {
        Utc::now().timestamp()
    }

    pub fn make_date_from_str(&self, yyyymmddhhmmss: &CDateT) -> DateTime<FixedOffset>
    {
        return match DateTime::parse_from_str(&self.add_fff_zzzz_to_yyyymmdd(yyyymmddhhmmss.clone()), "%Y-%m-%d %H:%M:%S%.3f %z") {
            Ok(dt) => { dt }
            Err(e) => {
                println!("Failed in time creating {} {}", yyyymmddhhmmss.clone(), e);
                panic!("Failed in time creating {} {}", yyyymmddhhmmss.clone(), e);
            }
        };
    }

    pub fn add_fff_zzzz_to_yyyymmdd(&self, c_date: CDateT) -> CDateT
    {
        let out: String = c_date + ".000 +0000";
        return out;
    }

    #[allow(dead_code, unused)]
    pub fn make_str_date_from_date_object(&self, dt: DateTime<FixedOffset>) -> String
    {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    //old name was isAmOrPm
    #[allow(dead_code, unused)]
    pub fn is_am_or_pm(&self, minutes: u32) -> String
    {
        if minutes >= 720
        { return "12:00:00".to_string(); }
        return "00:00:00".to_string();
    }


    //old_name_was isGreaterThanNow
    pub fn is_greater_than_now(&self, c_date: &CDateT) -> bool
    {
        if c_date.to_string() > self.now()
        {
            return true;
        }
        return false;
    }

    //old_name_was getCurrentYear
    pub fn get_current_year(&self) -> String
    {
        return self.now().split("-").collect::<Vec<&str>>()[0].to_string();
    }

    //old name was getNowByMinutes
    pub fn get_now_by_minutes(&self) -> u32
    {
        let minutes_dtl: String = self.now().clone();
        let minutes_dtl: Vec<&str> = minutes_dtl.split(" ").collect();
        let minutes_dtl: Vec<&str> = minutes_dtl[1].split(":").collect();
        let now_by_minutes: u32 = (minutes_dtl[0].to_string().parse::<u32>().unwrap() * 60) + minutes_dtl[1].to_string().parse::<u32>().unwrap();
        now_by_minutes
    }

    //old_name_was getCycleElapsedByMinutes
    #[allow(dead_code, unused)]
    pub fn get_cycle_elapsed_by_minutes(&self, c_date_: CDateT) -> u64
    {
        let mut c_date = c_date_;
        if c_date == "".to_string() {
            c_date = self.now();
        }

        let cycle_start_time: CDateT = self.get_a_cycle_range(&c_date.clone(), 0, 0).from;
        return self.time_diff(cycle_start_time, c_date).as_minutes;
    }

    //old_name_was isInCurrentCycle
    pub fn is_in_current_cycle(&self, c_date: &CDateT) -> bool
    {
        let now_ = self.now();
        c_date >= &self.get_coinbase_range(&now_).from
    }

    //old_name_was convertMinutesToHHMM
    pub fn convert_minutes_to_hhmm(&self, minutes: TimeByMinutesT) -> String
    {
        let h: TimeByMinutesT = minutes / 60;
        let mut h: String = h.to_string();
        h = padding_left(&h, 2);

        let m: TimeByMinutesT = minutes % 60;
        let mut m: String = m.to_string();
        m = padding_left(&m, 2);
        // return String::number((minutes / 60) as u8).rightJustified(2, '0') + ':' + String::number(minutes % 60).rightJustified(2, '0');
        return h + ":" + &m;
    }

    //old name was getCoinbaseRange
    pub fn get_coinbase_range(&self, c_date: &CDateT) -> TimeRange
    {
        return self.get_a_cycle_range(c_date, 0, 0);
    }

    //old_name_was getCbUTXOsDateRange
    #[allow(dead_code, unused)]
    pub fn get_cb_coins_date_range(&self, c_date: &CDateT) -> TimeRange
    {
        return self.get_a_cycle_range(c_date, constants::COINBASE_MATURATION_CYCLES, 0);
    }

    //old_name_was getCoinbaseInfo
    pub fn get_coinbase_info(&self, c_date: &CDateT, cycle_stamp_inp: &str) ->
    (String, String, String, String, String)
    {
        if c_date != ""
        {
            let the_range = self.get_coinbase_range(c_date);
            let from_hour: Vec<&str> = the_range.from.split(" ").collect::<Vec<&str>>();
            let from_hour: String = from_hour[1].to_string();
            let to_hour: Vec<&str> = the_range.to.split(" ").collect::<Vec<&str>>();
            let to_hour: String = to_hour[1].to_string();
            let cycle_stamp = self.get_coinbase_cycle_stamp(c_date);
            return (
                cycle_stamp.clone(),
                the_range.from.clone(),
                the_range.to.clone(),
                from_hour.clone(),
                to_hour.clone()
            );
        } else if cycle_stamp_inp != ""
        {
            let the_range = self.get_coinbase_range_by_cycle_stamp(cycle_stamp_inp);
            let from_hour: Vec<&str> = the_range.from.split(" ").collect();
            let from_hour: String = from_hour[1].to_string();
            let to_hour: Vec<&str> = the_range.to.split(" ").collect();
            let to_hour: String = to_hour[1].to_string();
            let cycle_stamp = cycle_stamp_inp.to_string().clone();
            println!("mmmmkm 9");
            let from_ = the_range.from.clone();
            let to_ = the_range.to.clone();
            println!("mmmmkm 9");
            return (
                cycle_stamp,
                from_,
                to_,
                from_hour,
                to_hour
            );
        }
        panic!("invalid input for get Coinbase Info");
    }

    //old_name_was getCoinbaseInfo
    #[allow(dead_code, unused)]
    pub fn get_coinbase_info_by_date(&self, c_date: &CDateT) ->
    (String, String, String, String, String)
    {
        println!("mmmmmy get_coinbase_info: c_date: {}, ", c_date);

        let the_range = self.get_coinbase_range(c_date);
        let from_hour: Vec<&str> = the_range.from.split(" ").collect::<Vec<&str>>();
        let from_hour: String = from_hour[1].to_string();
        let to_hour: Vec<&str> = the_range.to.split(" ").collect::<Vec<&str>>();
        let to_hour: String = to_hour[1].to_string();
        let cycle_stamp = self.get_coinbase_cycle_stamp(c_date);
        return (
            cycle_stamp.clone(),
            the_range.from.clone(),
            the_range.to.clone(),
            from_hour.clone(),
            to_hour.clone()
        );
    }

    //old_name_was getCoinbaseInfo
    #[allow(dead_code, unused)]
    pub fn get_coinbase_info_by_cycle_stamp(&self, cycle_stamp_inp: &str) ->
    (String, String, String, String, String)
    {
        println!("mmmmmZ get_coinbase_info: cycle_stamp_inp: {}", cycle_stamp_inp);
        let the_range = self.get_coinbase_range_by_cycle_stamp(cycle_stamp_inp);
        let from_hour: Vec<&str> = the_range.from.split(" ").collect();
        let from_hour: String = from_hour[1].to_string();
        let to_hour: Vec<&str> = the_range.to.split(" ").collect();
        let to_hour: String = to_hour[1].to_string();
        let cycle_stamp = cycle_stamp_inp.to_string().clone();
        println!("mmmmkmz 9");
        let from_ = the_range.from.clone();
        let to_ = the_range.to.clone();
        println!("mmmmkmz 9");
        return (
            cycle_stamp,
            from_,
            to_,
            from_hour,
            to_hour
        );
    }

    //old_name_was yearsBefore
    pub fn years_before(&self, back_in_time_by_years: u64, c_date: &CDateT) -> String
    {
        let mut since_epoch: i64;
        if c_date == ""
        {
            since_epoch = self.get_since_epoch();
        } else {
            let dt = self.make_date_from_str(c_date);
            since_epoch = dt.timestamp();
        }
        since_epoch -= (back_in_time_by_years * 31536000 as TimeBySecT) as i64; // 365Days * 24Hours * 60Minutes * 60Seconds
        let dt = Utc.timestamp(since_epoch, 0);
        return dt.format("%Y-%m-%d %H:%M:%S").to_string();
    }

    //old_name_was getCoinbaseRangeByCycleStamp
    #[allow(dead_code, unused)]
    pub fn get_coinbase_range_by_cycle_stamp(&self, cycle_stamp: &str) -> TimeRange
    {
        let mut res: TimeRange = TimeRange { from: "".to_string(), to: "".to_string() };
        let cycle_dtl = cycle_stamp
            .to_string()
            .clone()
            .split(" ")
            .collect::<Vec<&str>>()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        if cycle_dtl[1] == "00:00:00".to_string()
        {
            res.from = cycle_dtl[0].to_owned() + &" 00:00:00".to_string();
            res.to = cycle_dtl[0].to_owned() + &" 11:59:59".to_string();
            return res;
        } else if cycle_dtl[1] == "12:00:00"
        {
            return TimeRange {
                from: cycle_dtl[0].to_owned() + " 12:00:00",
                to: cycle_dtl[0].to_owned() + " 23:59:59",
            };
        } else {
            // develop mod
            let cycle_by_minutes = self.get_cycle_by_minutes();
            let c_date: CDateT = self.minutes_after(
                cycle_dtl[1].to_string().parse::<u64>().unwrap() * (cycle_by_minutes as u64),
                &(cycle_dtl[0].to_string() + &" 00:00:01"));
            return self.get_coinbase_range(&c_date);
        }
    }

    // old name was getMatureCyclesCount
    pub fn get_mature_cycles_count(&self, document_type: &String) -> u8
    {
        let cycle_map: HashMap<String, u8> = HashMap::from([
            (constants::document_types::BASIC_TX.to_string(), 1),
            (constants::document_types::DATA_AND_PROCESS_COST_PAYMENT.to_string(), 1),
            (constants::document_types::COINBASE.to_string(), 2),
            (constants::document_types::REPAYMENT_DOCUMENT.to_string(), 2),
        ]);

        if cycle_map.contains_key(document_type)
        { return cycle_map[document_type]; }

        dlog(
            &format!("Invalid document_type in 'get Mature Cycles Count'! {}", document_type),
            constants::Modules::App,
            constants::SecLevel::Error);
        return 0;
    }

    // old name was isMatured
    pub fn is_matured(
        &self,
        doc_type: &String,
        coin_creation_date: &String,
        c_date: &String) -> bool
    {
        let mature_cycles = self.get_mature_cycles_count(doc_type);
        if mature_cycles == 0
        { return false; }

        // control maturity
        if
        // (spendBlock.bType != iConsts.BLOCK_TYPES.RpBlock) &&
        self.time_diff(coin_creation_date.clone(), c_date.clone()).as_minutes < mature_cycles as u64 * self.get_cycle_by_minutes()
        {
            let msg = format!(
                "Is Matured: {} block({}) uses an output coin-creation-date{}) before being maturated by {} cycles",
                doc_type,
                c_date,
                coin_creation_date,
                mature_cycles
            );
            dlog(
                &msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }

        return true;
    }
}


pub struct TimeDiff
{
    pub as_years: u64,
    pub years: u64,
    pub as_months: u64,
    pub months: u64,
    pub as_days: u64,
    pub days: u64,
    pub as_hours: u64,
    pub hours: u64,
    pub as_minutes: u64,
    pub minutes: u64,
    pub as_seconds: u64,
    pub seconds: u64,

    // implement operator overloading
    // bool operator ==(const TimeDiff& obj);
    // bool operator !=(const TimeDiff& obj);
}


impl TimeDiff {
    fn new() -> TimeDiff {
        let o: TimeDiff = TimeDiff {
            as_years: 0,
            years: 0,
            as_months: 0,
            months: 0,
            as_days: 0,
            days: 0,
            as_hours: 0,
            hours: 0,
            as_minutes: 0,
            minutes: 0,
            as_seconds: 0,
            seconds: 0,
        };
        return o;
    }
}

#[derive(Debug)]
pub struct TimeRange
{
    pub from: CDateT,
    pub to: CDateT,
}
