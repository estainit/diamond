#[allow(unused_imports)]
use std::fmt::format;
use chrono::Utc;
use chrono::{DateTime, TimeZone};
use substring::Substring;

use crate::lib::constants as CConsts;
use crate::lib::custom_types::{CDateT, VString, VVString};

#[allow(dead_code)]
pub fn right_padding(inp_str: String, length: u8) -> String {
    let mut str = inp_str.clone();
    if str.len() >= length as usize {
        return str;
    }

    for _i in 0..length - (str.len() as u8) {
        str = str + "0";
    }
    return str;
}

pub fn left_padding(inp_str: String, length: u8) -> String {
    let mut str = inp_str.clone();
    if str.len() >= length as usize {
        return str;
    }

    for _i in 0..length - (str.len() as u8) {
        str = "0".to_string() + &str;
    }
    return str;
}

//old_name_was convertFloatToString
#[allow(dead_code)]
pub fn convert_float_to_string(num: f64, precision: u32) -> String {
    let mut num_per_10 = num.clone();
    for _i in 0..precision {
        num_per_10 = num_per_10 * 10.0;
    }

    let num2: u64 = num_per_10 as u64;
    let mut out = format!("{}", num2);
    let precision_as_usize = precision as usize;
    if out.len() <= precision_as_usize {
        for _i in 0..precision_as_usize - out.len() {
            out = "0".to_string() + &out;
        }
        out = "0.".to_string() + &out;
    } else {
        let mut int_part = "".to_string();
        if out.len() > precision_as_usize
        {
            int_part = out[0..out.len() - precision_as_usize].to_string();
        }
        let mut float_part: String;
        if out.len() >= precision_as_usize
        {
            float_part = out[out.len() - precision_as_usize..out.len()].to_string();
        } else {
            float_part = out[0..out.len()].to_string();
            for _i in 0..precision_as_usize - out.len() {
                float_part = "0".to_string() + &float_part;
            }
        }
        out = int_part + "." + &float_part;
    }

    // in order to replace 0.0 by 0
    let mut non_zero: String = out.clone();
    non_zero = non_zero.replace("0", "");

    if non_zero == "." {
        out = "0".to_string();
    }

    if out == "0".to_string() {
        return out;
    }

    // in order to replace 100.0000 by 100
    let mut segments: Vec<&str> = out.split(".").collect();

    if segments[0] == "100".to_string() {
        out = "100".to_string();
        return out;
    }

    if (segments.len() == 2) && (segments[1].chars().last().unwrap() == '0') {
        // try to remove 0 from right side of floating part (if exist) e.g. 99.96353346750 => 99.9635334675
        while segments[1].chars().last().unwrap() == '0' {
            segments[1] = &segments[1][0..segments[1].len() - 1];
        }
        out = segments[0].to_string() + "." + segments[1];
    }

    return out;
}

// - - - - - - time functions - - - - -

//old name was isAmOrPm
#[allow(dead_code)]
pub fn is_am_or_pm(minutes: u32) -> String {
    if minutes >= 720
    { return "12:00:00".to_string(); }
    return "00:00:00".to_string();
}

//old_name_was getCoinbaseCycleNumber
#[allow(dead_code)]
pub fn get_coinbase_cycle_number(c_date: CDateT) -> String {
    let minutes: u32;
    if c_date == "".to_string() {
        minutes = get_now_by_minutes();
    } else {
        let minutes_dtl1: Vec<&str> = c_date.split(" ").collect();
        let minutes_dtl2: String = minutes_dtl1[1].to_string().clone();
        let minutes_dtl3: Vec<&str> = minutes_dtl2.split(":").collect();
        minutes = (minutes_dtl3[0].to_string().parse::<u32>().unwrap() * 60) + minutes_dtl3[1].to_string().parse::<u32>().unwrap();
    }

    let cycle_number: String;
    if CConsts::TIME_GAIN == 1 {
        cycle_number = is_am_or_pm(minutes);
    } else {
        cycle_number = format!("{}", minutes / get_cycle_by_minutes());
    }
    return cycle_number;
}

//old name was getCycleByMinutes
pub fn get_cycle_by_minutes() -> u32 {
    if CConsts::TIME_GAIN == 1 {
        return CConsts::STANDARD_CYCLE_BY_MINUTES;
    }
    return CConsts::TIME_GAIN as u32;
}

//old_name_was getNow
#[allow(dead_code)]
pub fn get_now() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

//old name was getNowByMinutes
pub fn get_now_by_minutes() -> u32 {
    let minutes_dtl: String = get_now().clone();
    let minutes_dtl: Vec<&str> = minutes_dtl.split(" ").collect();
    println!("minutes_dtl[0] {}", minutes_dtl[0]);
    println!("minutes_dtl[1] {}", minutes_dtl[1]);

    let minutes_dtl: Vec<&str> = minutes_dtl[1].split(":").collect();
    let now_by_minutes: u32 = (minutes_dtl[0].to_string().parse::<u32>().unwrap() * 60) + minutes_dtl[1].to_string().parse::<u32>().unwrap();
    now_by_minutes
}

//old_name_was getCycleElapsedByMinutes
pub fn get_cycle_elapsed_by_minutes(c_date_: CDateT) -> u64 {
    let mut c_date = c_date_;
    if c_date == "".to_string() {
        c_date = get_now();
    }

    let cycle_start_time: CDateT = get_a_cycle_range(&c_date.clone(), 0, 0).from;
    return time_diff(cycle_start_time, c_date).as_minutes;
}

pub struct TimeRange
{
    pub from: CDateT,
    pub to: CDateT,
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

pub fn add_fff_zzzz_to_yyyymmdd(c_date: CDateT) -> CDateT {
    let out: String = c_date + ".000 +0000";
    return out;
}

//old_name_was timeDiff
pub fn time_diff(from_t_: CDateT, to_t_: CDateT) -> TimeDiff {
    let mut from_t = from_t_;
    if from_t == "" { from_t = get_now(); }
    let mut to_t = to_t_;
    if to_t == "" { to_t = get_now(); }

    let mut res: TimeDiff = TimeDiff::new();
    let start_t = DateTime::parse_from_str(&add_fff_zzzz_to_yyyymmdd(from_t), "%Y-%m-%d %H:%M:%S%.3f %z").unwrap();
    let end_t = DateTime::parse_from_str(&add_fff_zzzz_to_yyyymmdd(to_t), "%Y-%m-%d %H:%M:%S%.3f %z").unwrap();
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

//old_name_was getACycleRange
pub fn get_a_cycle_range(
    c_date_: &CDateT,
    back_by_cycle: u8,
    forward_by_cycle: u8, ) -> TimeRange
{
    let mut c_date: CDateT = c_date_.clone();
    if c_date == "".to_string() {
        c_date = get_now();
    }

    if CConsts::TIME_GAIN == 1
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
        let c_date_: Vec<&str> = c_date.split(" ").collect();
        c_date = c_date_[0].to_string() + " " + &h;
    }

    let min_creation_date: String;
    if forward_by_cycle == 0 {
        min_creation_date = minutes_before((back_by_cycle as u32 * get_cycle_by_minutes()) as u64, c_date);
    } else {
        min_creation_date = minutes_after((forward_by_cycle as u32 * get_cycle_by_minutes()) as u64, c_date);
    }

    let day: Vec<&str> = min_creation_date.split(" ").collect();
    let day = day[0].to_string(); // format("YYYY-MM-DD");
    let time_details: Vec<&str> = min_creation_date.split(" ").collect();
    let time_details: String = time_details[1].to_string();
    let time_details: Vec<&str> = time_details.split(":").collect();
    let minutes_h: String = time_details[0].to_string();
    let minutes_h: u32 = minutes_h.parse::<u32>().unwrap() * 60;
    let minutes_m: String = time_details[1].to_string();
    let minutes_m: u32 = minutes_m.parse::<u32>().unwrap();
    // let minutes: u32 = (time_details[0].to_string().parse::<u32>().unwrap() * 60) + time_details[1].to_string().parse::<u32>().unwrap();
    let minutes: u32 = minutes_h + minutes_m;
    let start_minute: u32 = (minutes / get_cycle_by_minutes()) as u32 * get_cycle_by_minutes();
    let end_minute: u32 = start_minute + get_cycle_by_minutes() - 1;
    return TimeRange {
        from: day.clone() + " " + &convert_minutes_to_hhmm(start_minute) + ":00",
        to: day.clone() + " " + &convert_minutes_to_hhmm(end_minute) + ":59",
    };
}


//old_name_was convertMinutesToHHMM
pub fn convert_minutes_to_hhmm(minutes: u32) -> String {
    let h: u32 = minutes / 60;
    let mut h: String = h.to_string();
    h = left_padding(h, 2);

    let m: u32 = minutes % 60;
    let mut m: String = m.to_string();
    m = left_padding(m, 2);
    // return QString::number((minutes / 60) as u8).rightJustified(2, '0') + ':' + QString::number(minutes % 60).rightJustified(2, '0');
    return h + ":" + &m;
}

//old name was minutesBefore
pub fn minutes_before(back_in_time_by_minutes: u64, c_date: CDateT) -> String {
    let mut since_epoch: i64;
    if c_date == "" {
        // since_epoch = QDateTime::currentDateTimeUtc().toSecsSinceEpoch();
        since_epoch = Utc::now().timestamp();
    } else {
        let dt = DateTime::parse_from_str(&add_fff_zzzz_to_yyyymmdd(c_date), "%Y-%m-%d %H:%M:%S%.3f %z");
        since_epoch = dt.unwrap().timestamp();
    }
    since_epoch -= (back_in_time_by_minutes * 60) as i64;
    let dt = Utc.timestamp(since_epoch, 0);
    return dt.format("%Y-%m-%d %H:%M:%S").to_string();
}

//old_name_was minutesAfter
pub fn minutes_after(forward_in_time_by_minutes: u64, c_date: CDateT) -> String {
    let mut since_epoch: i64;
    if c_date == "" {
        since_epoch = Utc::now().timestamp();
    } else {
        let dt = DateTime::parse_from_str(&add_fff_zzzz_to_yyyymmdd(c_date), "%Y-%m-%d %H:%M:%S%.3f %z");
        since_epoch = dt.unwrap().timestamp();
    }
    since_epoch += (forward_in_time_by_minutes * 60) as i64;
    let dt = Utc.timestamp(since_epoch, 0);
    return dt.format("%Y-%m-%d %H:%M:%S").to_string();
}

//old name was getCoinbaseRange
#[allow(dead_code)]
pub fn get_coinbase_range(c_date: CDateT) -> TimeRange {
    return get_a_cycle_range(&c_date, 0, 0);
}

//old_name_was getCoinbaseCycleStamp
pub fn get_coinbase_cycle_stamp(c_date_: CDateT) -> String {
    let mut c_date = c_date_.clone();
    if c_date == "" {
        c_date = get_now();
    }

    if CConsts::TIME_GAIN == 1 {
        return get_a_cycle_range(&c_date, 0, 0).from;
    }

    let day: Vec<&str> = c_date.split(" ").collect();
    return day[0].to_string() + " " + &right_padding(get_coinbase_cycle_number(c_date), 3);
}

//old_name_was getCbUTXOsDateRange
#[allow(dead_code)]
pub fn get_cb_coins_date_range(c_date: &CDateT) -> TimeRange {
    return get_a_cycle_range(c_date, CConsts::COINBASE_MATURATION_CYCLES, 0);
}

//old_name_was getCoinbaseInfo
#[allow(dead_code)]
pub fn get_coinbase_info(c_date: CDateT, cycle: String) -> (String, String, String, String, String) {
    if c_date != "" {
        let the_range = get_coinbase_range(c_date.clone());
        let from_hour: Vec<&str> = the_range.from.split(' ').collect();
        let from_hour: String = from_hour[1].to_string();
        let to_hour: Vec<&str> = the_range.to.split(' ').collect();
        let to_hour: String = to_hour[1].to_string();
        return (
            get_coinbase_cycle_stamp(c_date),
            the_range.from, the_range.to,
            from_hour, to_hour
        );
    } else if cycle != "" {
        let the_range = get_coinbase_range_by_cycle_stamp(cycle.clone());
        let from_hour: Vec<&str> = the_range.from.split(' ').collect();
        let from_hour: String = from_hour[1].to_string();
        let to_hour: Vec<&str> = the_range.to.split(' ').collect();
        let to_hour: String = to_hour[1].to_string();
        return (
            cycle.clone(),
            the_range.from, the_range.to,
            from_hour, to_hour
        );
    }
    panic!("invalid input for get Coinbase Info");
}

//old_name_was getCoinbaseRangeByCycleStamp
#[allow(dead_code)]
pub fn get_coinbase_range_by_cycle_stamp(cycle: String) -> TimeRange {
    let mut res: TimeRange = TimeRange { from: "".to_string(), to: "".to_string() };
    let cycle_dtl: Vec<&str> = cycle.split(" ").collect();
    if cycle_dtl[1].to_string() == "00:00:00" {
        res.from = cycle_dtl[0].to_string() + &" 00:00:00".to_string();
        res.to = cycle_dtl[0].to_string() + &" 11:59:59".to_string();
        return res;
    } else if cycle_dtl[1] == "12:00:00"
    {
        return TimeRange { from: cycle_dtl[0].to_owned() + " 12:00:00", to: cycle_dtl[0].to_owned() + " 23:59:59" };
    } else {
        // develop mod
        let c_date: CDateT = minutes_after(
            cycle_dtl[1].to_string().parse::<u64>().unwrap() * (get_cycle_by_minutes() as u64),
            cycle_dtl[0].to_string() + &" 00:00:01");
        return get_coinbase_range(c_date);
    }
}

//old_name_was chunkString
pub fn chunk_string(str: &String, chunck_size: u16) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let mut i = 0;
    while i < str.len() {
        let s: String = str.substring(i, i + chunck_size as usize).to_string();
        out.push(s);
        i = i + chunck_size as usize;
    }
    return out;
}


//old_name_was chunkStringList
//old_name_was chunkQStringList

//old_name_was chunkStringList
pub fn chunk_to_vvstring(values: &VString, chunk_size: u64) -> VVString {
    let mut out: VVString = vec![];

    if (values.len() == 0) || (chunk_size == 0) {
        return out;
    }

    if values.len() <= chunk_size as usize {
        out.push(values.clone());
        return out;
    }

    let the_len = values.len() as u64;
    let mut chunks_count: u64 = (the_len / chunk_size) as u64;
    if (the_len % chunk_size) != 0 {
        chunks_count += 1;
    }

    for i in 0..chunks_count {
        let mut end_index: u64 = 0;
        if (i + 1) * chunk_size < values.len() as u64 {
            end_index = (i + 1) * chunk_size;
        } else {
            end_index = the_len as u64;
        }
        let a_chunk: Vec<String> = values[(i * chunk_size) as usize..end_index as usize].to_vec();
        if a_chunk.len() > 0 {
            out.push(a_chunk);
        }
    }

    return out;
    // fill
    //  std::size_t const half_size = lines.len() / 2;
    //  T split_lo(lines.begin(), lines.begin() + half_size);
    //  T split_hi(lines.begin() + half_size, lines.end());
}

// pub fn clone_vstring(inp: &VString) -> VString {
//     let mut out: VString = vec![];
//     for a_str in inp {
//         out.push(a_str.clone());
//     }
//     return out;
// }

// pub fn clone_T<T: Clone>(inp: &Vec<T>) -> Vec<T> {
//     let mut out: Vec<T> = vec![];
//     for a_str in inp {
//         out.push(a_str.clone());
//     }
//     return out;
// }

pub fn clone_vec<T: Clone>(vec: &Vec<T>) -> Vec<T> {
    let vec = vec[..].to_vec();
    vec
}

pub fn clone_vvec<T: Clone>(inp_vvec: &Vec<Vec<T>>) -> Vec<Vec<T>> {
    let mut out: Vec<Vec<T>> = vec![];
    for a_vec in inp_vvec {
        let new_vec: Vec<T> = clone_vec(a_vec);
        out.push(new_vec);
    }
    out
}


pub trait ExtendString {
    fn clone_me(&self) -> String;
}

impl ExtendString for String {
    fn clone_me(&self) -> String {
        self.to_string().clone()
    }
}
