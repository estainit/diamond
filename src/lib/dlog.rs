use log::{debug, error, info, LevelFilter, trace, warn};
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;
use crate::{machine};
use crate::constants::{Modules, SecLevel};

pub fn initialize_log(){
    let window_size = 1; // log0, log1, log2
    let fixed_window_roller = FixedWindowRoller::builder().build("log{}", window_size).unwrap();
    let size_limit = 2000 * 1024; // 50 KB as max log file size to roll
    let size_trigger = SizeTrigger::new(size_limit);
    let compound_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(fixed_window_roller));
    let config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(LevelFilter::Debug)))
                .build(
                    "logfile",
                    Box::new(
                        RollingFileAppender::builder()
                            .encoder(Box::new(PatternEncoder::new("{d} {l}::{m}{n}")))
                            .build(machine().get_logs_path()+&"/dlog.log", Box::new(compound_policy)).unwrap(),
                    ),
                ),
        )
        .build(
            Root::builder()
                .appender("logfile")
                .build(LevelFilter::Debug),
        ).unwrap();
    let _handle = log4rs::init_config(config).unwrap();


}

pub fn dlog(msg: &String, module: Modules, level: SecLevel) {
    let module_ = match module {
        Modules::App => "App",
        Modules::CB => "CB",
        Modules::Sql => "Sql",
        Modules::Trx => "Trx",
        _ => "Gen"
    };

    // let level_ = match level {
    //     SecLevel::Debug => "Debug",
    //     SecLevel::Trace => "Trace",
    //     SecLevel::Info => "Info",
    //     SecLevel::Warning => "Warning",
    //     SecLevel::Error => "Error",
    //     SecLevel::Fatal => "Fatal",
    //     _ => "Gen"
    // };

    let log_msg = format!("({}): {}", module_, msg);

    match level {
        SecLevel::Debug => { debug!("{}", log_msg); }
        SecLevel::Trace => { trace!("{:#?}", log_msg); }
        SecLevel::Info => { info!("{:?}", log_msg); }
        SecLevel::Warning => { warn!("{:#?}", log_msg); }
        SecLevel::Error => { error!("{}", log_msg); }
        SecLevel::Fatal => { error!("{}", log_msg); }
        // _ => { error!("{}", log_msg); }
    }
}
