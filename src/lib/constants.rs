#[allow(dead_code)]
pub enum Modules {
    App,
    CB,
}

#[allow(dead_code)]
pub enum SecLevel {
    Trace,
    Info,
    Warning,
    Error,
    Fatal,
}


#[allow(dead_code)]
pub const SOCIETY_NAME: &str = "im";

// in live environment time gain must be 1, in develop mode it is equal one cycle by minutes e.g. 5
pub const TIME_GAIN: u8 = 1;
pub const STANDARD_CYCLE_BY_MINUTES: u32 = 720;

pub const HD_FILES: &str = "/Users/silver/Documents/Diamond_files/";

#[allow(dead_code)]
pub const COINBASE_MATURATION_CYCLES: u8 = 2;

//bech32 part
pub const BECH32_ADDRESS_VER: &str = "0";
pub const TRUNCATE_FOR_BECH32_ADDRESS: u8 = 32;

pub const SIGN_MSG_LENGTH: u8 = 32;