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
    Fatal,
}


#[allow(dead_code)]
// in live environment time gain must be 1, in develop mode it is equal one cycle by minutes e.g. 5
pub const TIME_GAIN: u8 = 1;
pub const STANDARD_CYCLE_BY_MINUTES: u32 = 720;

pub const HD_FILES: &str = "/Users/silver/Documents/Diamond_files/";

#[allow(dead_code)]
pub const COINBASE_MATURATION_CYCLES: u8 = 2;