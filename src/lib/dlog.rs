use crate::lib::constants as CConsts;

pub fn dlog (msg: &String, module: CConsts::Modules, level: CConsts::SecLevel) {
    let module = match module {
        CConsts::Modules::App => "App",
        _ => "Gen"
    };
    let level = match level {
        CConsts::SecLevel::Info => "Info",
        CConsts::SecLevel::Warning => "Warning",
        CConsts::SecLevel::Fatal => "Fatal",
        _ => "Gen"
    };

    println!("{}({}): {}", module, level, msg);
}
