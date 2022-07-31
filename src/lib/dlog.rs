use crate::lib::constants as CConsts;

pub fn dlog(msg: &String, module: CConsts::Modules, level: CConsts::SecLevel) {
    let module = match module {
        CConsts::Modules::App => "App",
        CConsts::Modules::CB => "CB",
        CConsts::Modules::Sql => "Sql",
        CConsts::Modules::Trx => "Trx",
        _ => "Gen"
    };
    let level = match level {
        CConsts::SecLevel::Info => "Info",
        CConsts::SecLevel::Warning => "Warning",
        CConsts::SecLevel::Fatal => "Fatal",
        CConsts::SecLevel::Trace => "Trace",
        CConsts::SecLevel::Error => "Error",
        _ => "Gen"
    };

    println!("{}({}): {}", module, level, msg);
}
