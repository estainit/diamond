#[allow(unused_imports)]
use std::fmt::format;
use substring::Substring;
use lazy_static::lazy_static;
use regex::{Regex};
use serde_json::json;
use crate::{application, constants, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::custom_types::{CCoinCodeT, CDocHashT, CMPAISValueT, COutputIndexT, JSonArray, JSonObject, VString, VVString};

#[allow(unreachable_code)]
pub fn remove_quotes(input_value: &JSonObject) -> String
{
    if input_value.is_null() {
        dlog(
            &format!("JSon Object (to be removed quotes) is null {}", input_value),
            constants::Modules::App,
            constants::SecLevel::Error);
        panic!("JSon Object (to be removed quotes) is null {}", input_value);
        return "".to_string();
    }

    return match input_value.as_str() {
        Some(r) => r.to_string(),
        _ =>
            {
                dlog(
                    &format!("Failed in JSon Object (to be removed quotes) {}", input_value),
                    constants::Modules::App,
                    constants::SecLevel::Error);
                "".to_string()
            }
    };
}

#[allow(unused, dead_code)]
pub fn padding_right(inp_str: &String, length: u8) -> String {
    return padding_right_custom(inp_str, length, "0".to_string());
}

pub fn padding_right_custom(inp_str: &String, length: u8, placeholder: String) -> String {
    let mut str = inp_str.clone();
    if str.len() >= length as usize {
        return str;
    }

    for _i in 0..length - (str.len() as u8) {
        str = str + &placeholder;
    }
    return str;
}

pub fn padding_left(inp_str: &String, length: u8) -> String {
    return padding_left_custom(inp_str, length, "0".to_string());
}

pub fn padding_left_custom(inp_str: &String, length: u8, placeholder: String) -> String {
    let mut str = inp_str.clone();
    if str.len() >= length as usize {
        return str;
    }

    for _i in 0..length - (str.len() as u8) {
        str = placeholder.clone() + &str;
    }
    return str;
}

//old_name_was convertFloatToString
pub fn convert_float_to_string(num: f64, precision: u8) -> String {
    if num == 0.0 {
        return "0.0".to_string();
    }

    let mut num_per_10 = num.clone();
    num_per_10 = num_per_10 * 10_u64.pow(precision as u32) as f64;
    // for _i in 0..precision {
    //     num_per_10 = num_per_10 * 10.0;
    // }

    let num_per_10: u64 = num_per_10 as u64;
    let mut out = num_per_10.to_string();
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
        let mut should_loop = true;
        while should_loop {
            should_loop = match segments[1].chars().last() {
                Some(l) => {
                    if l == '0' {
                        true
                    } else {
                        dlog(
                            &format!("maybe issue in convert_ float_ to_ string1 = {:?}", segments),
                            constants::Modules::App,
                            constants::SecLevel::Info);
                        // panic!("maybe issue in convert_ float_ to_ string1 = {:?}", segments);
                        false
                    }
                }
                _ => {
                    dlog(
                        &format!("maybe issue in convert_ float_ to_ string2 = {:?}", segments),
                        constants::Modules::App,
                        constants::SecLevel::Info);
                    // panic!("maybe issue in convert_ float_ to_ string2 = {:?}", segments);
                    false
                }
            };
            if should_loop {
                segments[1] = &segments[1][0..segments[1].len() - 1];
            }
        }

        // while segments[1].chars().last().unwrap() == '0' {
        //     segments[1] = &segments[1][0..segments[1].len() - 1];
        // }
        out = segments[0].to_string() + "." + segments[1];
    }

    return out;
}

// - - - - - - time functions - - - - -


//old_name_was chunkString
pub fn chunk_string(str: &String, chunk_size: u16) -> Vec<String> {
    let mut out: Vec<String> = vec![];
    let mut i = 0;
    while i < str.len() {
        let s: String = str.substring(i, i + chunk_size as usize).to_string();
        out.push(s);
        i = i + chunk_size as usize;
    }
    return out;
}


//old_name_was chunkStringList
pub fn chunk_to_vvstring(values: Vec<String>, chunk_size: u64) -> VVString {
    let mut out: VVString = vec![];

    if (values.len() == 0) || (chunk_size == 0) {
        return out;
    }

    if values.len() <= chunk_size as usize {
        out.push(values.iter().map(|x| x.to_string()).collect::<Vec<String>>());
        return out;
    }

    let the_len = values.len() as u64;
    let mut chunks_count: u64 = (the_len / chunk_size) as u64;
    if (the_len % chunk_size) != 0 {
        chunks_count += 1;
    }

    for i in 0..chunks_count {
        let end_index: u64;
        if (i + 1) * chunk_size < values.len() as u64 {
            end_index = (i + 1) * chunk_size;
        } else {
            end_index = the_len as u64;
        }
        let a_chunk: Vec<String> = values[(i * chunk_size) as usize..end_index as usize]
            .to_vec()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>();
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

// TODO: unittests for packCommaSeperated, unpackCommaSeperated, normalizeCommaSeperatedStr, removeNullMembersFromCommaSeperated
//old_name_was normalizeCommaSeperatedStr
pub fn normalize_comma_seperated_string(str: &String) -> String
{
    if str == ""
    { return "".to_string(); }

    let elms = str.split(",")
        .collect::<Vec<&str>>()
        .iter()
        .map(|&x| x.to_string())
        .collect::<Vec<String>>();

    let mut new_elms: VString = vec![];
    for elm in elms
    {
        if elm != ""
        {
            new_elms.push(elm);
        }
    }
    return format!(",{}", new_elms.join(","));
}

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

//old_name_was convertCommaSeperatedToArray
pub fn convert_comma_separated_to_array(inp: &String, separator: &String) -> Vec<String>
{
    if inp == "" {
        return vec![];
    }

    let mut out: Vec<String> = vec![];
    let elements: Vec<String> = inp
        .split(separator)
        .collect::<Vec<&str>>()
        .iter().map(|x| x.to_string())
        .collect::<Vec<String>>();
    for elm in elements {
        if elm != "" {
            out.push(elm);
        }
    }
    out.sort();
    out.dedup();
    return out;
}

pub trait ExtendString {
    fn clone_me(&self) -> String;
}

impl ExtendString for String {
    fn clone_me(&self) -> String {
        self.to_string().clone()
    }
}

pub fn remove_dbl_spaces(s: &String) -> String
{
    lazy_static! {
        static ref ISO8601_DATE_REGEX : Regex = Regex::new(
            r"[' ']{2,}"
            ).unwrap();
    }
    return ISO8601_DATE_REGEX.replace_all(s, " ").to_string();
}

//old_name_was breakByBR
pub fn break_by_br(content: &String, chunk_size: u16) -> String
{
    let chunks = chunk_string(content, chunk_size);
    let line_br = constants::message_tags::I_PGP_END_LINEBREAK.to_owned() + constants::message_tags::I_PGP_START_LINEBREAK;
    let mut out = chunks.join(&*line_br);
    out = constants::message_tags::I_PGP_START_LINEBREAK.to_owned() + &out + constants::message_tags::I_PGP_END_LINEBREAK;
    return out;
}

//old_name_was stripBR
pub fn strip_parentheses_as_break_line(mut content: String) -> String
{
    if content.contains("(")
    {
        content = content.replace("\n", "");
        content = content.replace("\r", "");
        let mut outs: String = "".to_string();
        let chunks = content.split("<br>");
        for a_chunk in chunks
        {
            let (has_open, open_p) = match a_chunk.find("(") {
                Some(p) => { (true, p) }
                _ => (false, 0)
            };
            let (has_close, close_p) = match a_chunk.find(")") {
                Some(p) => { (true, p) }
                _ => (false, 0)
            };
            if has_open && has_close
            {
                let ach = a_chunk.substring(1 as usize + open_p, close_p);
                outs += ach;
            }
        }
        return outs.trim().to_string();
    } else {
        return "".to_string();
    }
}

//old_name_was paddingLengthValue
#[allow(unused, dead_code)]
pub fn padding_length_value(value: String, needed_len: u8) -> String
{
    return padding_left(&value.to_string(), needed_len);
}

#[allow(unused, dead_code)]
pub fn hash4c(s: &String) -> String
{
    s.substring(0, 4).to_string()
}

pub fn hash6c(s: &String) -> String
{
    s.substring(0, 6).to_string()
}

pub fn hash8c(s: &String) -> String
{
    s.substring(0, 8).to_string()
}

//old_name_was shortCoinRef
pub fn short_coin_code(coin: &CCoinCodeT) -> CCoinCodeT
{
    let coins_segments =
        coin
            .split(":")
            .collect::<Vec<&str>>()
            .iter()
            .map(|x| x.to_string())
            .collect::<VString>();
    return [hash16c(&coins_segments[0]), coins_segments[1].to_string()].join(":");
}

//old_name_was packCoinSpendLoc
pub fn pack_coin_spend_loc(
    block_hash: &String,
    transaction_hash: &String) -> String
{
    return vec![block_hash.clone(), transaction_hash.clone()].join(":").to_string();
}


//old_name_was unpackCoinSpendLoc
pub fn unpack_coin_spend_loc(spend_location: &String) -> (String, String)
{
    let segments: VString =
        spend_location
            .split(":")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.to_string())
            .collect::<Vec<String>>();
    return (segments[0].clone(), segments[1].clone());
}

pub fn hash16c(s: &String) -> String
{
    s.substring(0, 16).to_string()
}

pub fn hash32c(s: &String) -> String
{
    s.substring(0, 32).to_string()
}

pub fn hash64c(s: &String) -> String
{
    s.substring(0, 64).to_string()
}

//old_name_was shortBech16
pub fn short_bech16(s: &String) -> String
{
    return s.substring(0, 5).to_string() + &s.substring(48, s.len()).to_string();
}

//old_name_was sepNum
pub fn sep_num_3(number: i64) -> String
{
    let mut str_number: String = number.to_string();
    let mut sign = "";
    if str_number.substring(0, 1) == "-"
    {
        str_number = str_number.substring(1, str_number.len()).to_string();
        sign = "-";
    }

    let segments: Vec<String> = chunk_string(&padding_left(&str_number, 30), 3);

    str_number = segments.join(",");
    while (str_number.substring(0, 1) == "0") || (str_number.substring(0, 1) == ",")
    {
        str_number = str_number.substring(1, str_number.len()).to_string();
    }

    if str_number == ""
    {
        return "0".to_string();
    }

    return sign.to_owned() + &str_number;
}

//old_name_was CFloor
pub fn c_floor(v: f64) -> i64
{
    return v.floor() as i64;
}

//old_name_was customFloorFloat
pub fn custom_floor_float(number: f64, percision: u8) -> f64
{
    let the_gain: f64 = 10_i64.pow(percision as u32) as f64;
    return (number * the_gain) / the_gain;
}

//old_name_was iFloorFloat
pub fn i_floor_float(number: f64) -> f64
{
    return custom_floor_float(number, constants::FLOAT_LENGTH); // in order to keep maximum 11 digit after point
}
//
// //old_name_was convertJSonArrayToStringList
// //old_name_was convertJSonArrayToStringVector
// pub fn convert_json_array_to_string_vector(inp: &JSonArray) -> Vec<String> {
//     if !inp.is_array() {
//         return vec![];
//     }
//     let mut out: Vec<String> = vec![];
//     let mut inx: usize = 0;
//     while !inp[inx].is_null() {
//         out.push(inp[inx].to_string());
//         inx += 1;
//     }
//     return out;
// }

//old_name_was convertJSonArrayToStringList
pub fn convert_comma_separated_string_to_string_vector(s: &String) -> VString
{
    let vec_val = s
        .split(",")
        .collect::<Vec<&str>>()
        .iter()
        .filter(|&x| *x != "")
        .map(|x| x.to_string())
        .collect::<VString>();
    return vec_val;
}

pub fn json_array_push(inpup_array: &mut JSonArray, elm: &JSonObject)
{
    inpup_array.as_array_mut().unwrap().push(elm.clone());
}

//old_name_was arrayDiff
pub fn array_diff(superset: &Vec<String>, subset: &Vec<String>) -> Vec<String>
{
    let mut remined_values: Vec<String> = vec![];
    for element in superset {
        if !subset.contains(element) {
            remined_values.push(element.clone());
        }
    }
    return remined_values;
}

//old_name_was arrayAdd
pub fn array_add(arr1: &Vec<String>, arr2: &Vec<String>) -> Vec<String>
{
    let mut out: Vec<String> = arr1.clone();
    for elm in arr2
    { out.push(elm.clone()); }
    return out;
}

//old_name_was arrayUnique
pub fn array_unique(inp_arr: &Vec<String>) -> Vec<String>
{
    let mut out_arr: Vec<String> = vec![];
    for elm in inp_arr {
        if !out_arr.contains(elm) {
            out_arr.push(elm.clone());
        }
    }
    out_arr
}

//old_name_was shortStringsList
pub fn short_strings_list(items: &VString, length: usize) -> VString
{
    let mut shorts: VString = vec![];
    for item in items
    {
        shorts.push(item.substring(0, length).to_string());
    }
    shorts
}

//old_name_was removeEmptyElements
pub fn remove_empty_elements(elms: &VString) -> VString
{
    let mut out: VString = vec![];
    for elm in elms
    {
        if elm != ""
        {
            out.push(elm.clone());
        }
    }
    out
}

//old_name_was packCoinCode
#[allow(unused, dead_code)]
pub fn pack_coin_code(ref_trx_hash: &CDocHashT, output_index: COutputIndexT) -> CCoinCodeT
{
    return vec![ref_trx_hash.to_string(), output_index.to_string()].join(":");
}

//old_name_was unpackCoinCode
pub fn unpack_coin_code(coin: &CCoinCodeT) -> (String, COutputIndexT)
{
    let segments: Vec<&str> = coin.split(":").collect();
    return (segments[0].to_string(), segments[1].parse::<COutputIndexT>().unwrap());
}

//old_name_was isGreaterThanNow
pub fn is_greater_than_now(c_date: &String) -> bool
{
    *c_date > application().now()
}

//old_name_was stripNonHex
pub fn strip_non_hex_chars(s: &String) -> String
{
    let re = Regex::new(r"[^0-9a-fA-F]").unwrap();
    return re.replace(&s.as_str(), "").to_string();
}

pub fn strip_non_alphabet_and_numbers(s: &String) -> String
{
    let re = Regex::new(r"[^0-9a-zA-Z]").unwrap();
    return re.replace(&s.as_str(), "").to_string();
}

//old_name_was stripOutputAddress
pub fn strip_output_address(address: &String) -> String
{
    return strip_non_alphabet_and_numbers(address);
}

//old_name_was isValidDateForamt
pub fn is_a_valid_date_format(_c_date: &String) -> bool
{
    return true; // TOD: implement a control on format "yyyy-MM-dd HH:mm:ss"
}

//old_name_was isValidHash
pub fn is_valid_hash(s: &String) -> bool
{
    if &strip_non_hex_chars(s) != s
    { return false; }

    if s.len() != 64
    { return false; }

    // TODO add some more control such as length control,...
    return true;
}

pub fn has_only_hex_chars(s: &String) -> bool
{
    strip_non_hex_chars(s).len() == s.len()
}

pub fn controlled_json_stringify(inp: &JSonObject) -> String
{
    let (_status, str) = match serde_json::to_string(inp) {
        Ok(r) => (true, r),
        Err(e) => {
            dlog(
                &format!("json serde serializing failed for input: {} {}", e, inp),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, "".to_string())
        }
    };
    str
}

pub fn controlled_block_stringify(inp: &Block) -> String
{
    let (_status, str) = match serde_json::to_string(inp) {
        Ok(r) => (true, r),
        Err(e) => {
            dlog(
                &format!("block serde serializing failed for input: {} {:?}", e, inp),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, "".to_string())
        }
    };
    str
}

//old_name_was parseToJsonObxjContolled
//old_name_was parseToJsonArr
//old_name_was parseToJsonObj
pub fn controlled_str_to_json(serialized: &String) -> (bool, JSonObject)
{
    return match serde_json::from_str(serialized) {
        Ok(r) => { (true, r) }
        Err(e) => {
            dlog(
                &format!("Failed in deserializing json object: {} {}", serialized, e),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, json!({}))
        }
    };
}


pub fn calc_log(
    x_value: i64,
    range: i64,
    exp: u32) -> (i64, f64, f64, f64)
{
    let powered = range.pow(exp as u32) as f64;
    let hundred_percent = powered.ln();

    let y_value: f64;
    if x_value >= range
    {
        y_value = 0.0;
    } else {
        let powered = (range - x_value).pow(exp) as f64;
        y_value = powered.ln();
    }
    let gain = i_floor_float((y_value * 100.0) / hundred_percent);

//  if ((gain == Number.POSITIVE_INFINITY || gain == Number.NEGATIVE_INFINITY))
//  {gain = Number.MAX_VALUE;}

    let rev_gain = 100.0 - gain;
//  if ((rev_gain == Number.POSITIVE_INFINITY || rev_gain == Number.NEGATIVE_INFINITY))
//  {rev_gain = Number.MAX_VALUE;}

    (x_value, y_value, gain, rev_gain)
}

//old_name_was microPAIToPAI6
pub fn nano_pai_to_pai(int_amount: CMPAISValueT) -> String
{
    if int_amount == 0
    { return "0.0".to_string(); }

    let mut str_number = int_amount.to_string();
    let mut sign = "";
    if str_number.substring(0, 1).to_string() == "-"
    {
        str_number = str_number.substring(1, str_number.len() - 1).to_string();
        sign = "-";
    }

    let max_nano: CMPAISValueT = constants::ONE_BILLION as CMPAISValueT;
    if ((0 < int_amount) && (int_amount < max_nano)) ||
        (((-1 * max_nano) < int_amount) && (int_amount < 0))
    { return format!("{}0.{}", sign, padding_left(&str_number, 9)); }

    let ppp = str_number.substring(0, str_number.len() - 9).to_string();
    let mut pai_s = padding_left(&ppp, 27);
    let segments: VString = chunk_string(&pai_s, 3);
    pai_s = segments.join(",");
    while ["0", ","].contains(&pai_s.substring(0, 1))
    {
        pai_s = pai_s.substring(1, pai_s.len() - 1).to_string();
    }
    if pai_s.len() == 0
    {
        pai_s = "0".to_string();
    }
    let str_number = format!(
        "{}.{}",
        pai_s,
        str_number.substring(str_number.len() - 9, str_number.len()));
    return format!("{}{}", sign, str_number);
}

// pub fn ZZnano_pai_to_pai<'n, T>(int_amount: &'n T) -> String
//     where T: std::fmt::Display + std::cmp::PartialEq + std::cmp::PartialOrd<&'n T>
// {
//     let zero: T = 0 as T;
//     if *int_amount == zero
//     { return "0.0".to_string(); }
//
//     let mut str_number = int_amount.to_string();
//     let mut sign = "";
//     if str_number.substring(0, 1).to_string() == "-"
//     {
//         str_number = str_number.substring(1, str_number.len() - 1).to_string();
//         sign = "-";
//     }
//
//     let max_nano: T = constants::ONE_BILLION as T;
//     if ((zero < int_amount) && (int_amount < max_nano)) ||
//         (((-1 * max_nano) < int_amount) && (int_amount < zero))
//     { return sign + "0." + padding_left(&str_number, 9); }
//
//     let ppp = str_number.substring(0, str_number.len() - 9).to_string();
//     let mut PAIs = padding_left(&ppp, 27);
//     let mut segments: VString = chunk_string(&PAIs, 3);
//     PAIs = segments.join(",");
//     while ["0", ","].contains(&PAIs.substring(0, 1))
//     {
//         PAIs = PAIs.substring(1, PAIs.len() - 1).to_string();
//     }
//     if PAIs.len() == 0
//     {
//         PAIs = "0".to_string();
//     }
//     let str_number = format!(
//         "{}.{}",
//         PAIs,
//         str_number.substring(str_number.len() - 9, str_number.len()));
//     return sign + str_number;
// }
