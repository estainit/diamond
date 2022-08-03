extern crate core;

use std::env;

use std::thread;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::sync::{LockResult, Mutex, MutexGuard};
use std::thread::sleep as std_sleep;
use log4rs::append::console::ConsoleAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use serde_json::{json, Value};


// use substring::Substring;
// use der::Encode;
// use pkcs1::LineEnding;
// use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
// use std::fmt::Display;
// use tokio::task;
// use tokio::time::{sleep, Duration};


// use std::thread;
// use std::thread::sleep;
// use std::time::Duration;

// use lib::c_log::log;

mod config;
mod lib;
mod tests;

// use std::thread::sleep;
// use std::time::Duration;

use lib::machine::machine_handler as machine_handler;
use lib::utils::cutils as cutils;
use crate::lib::{ccrypto, constants};
use crate::lib::database::db_handler::DBHandler;
use crate::lib::dlog::{dlog, initialize_log};
use crate::lib::k_v_handler::get_value;
use crate::lib::threads_handler::launch_giga_loop;
use crate::lib::utils::cmerkle as cmerkle;
use crate::lib::utils::permutation_handler::PermutationHandler;
use crate::machine_handler::CMachine;

static CMACHINE: Lazy<Mutex<CMachine>> = Lazy::new(|| Mutex::new(CMachine::new()));

fn machine<'m>() -> MutexGuard<'static, CMachine<'static>> { CMACHINE.lock().unwrap() }

static DBHANDLER: Lazy<Mutex<DBHandler>> = Lazy::new(|| Mutex::new(DBHandler::new()));

fn dbhandler() -> MutexGuard<'static, DBHandler> { DBHANDLER.lock().unwrap() }


fn main() {
    //! # Diamond, the Community Maker Engine
    //! ```
    //! fn main()
    //! ```
    //!
    //! This starts whole game
    //!
    //!
    //!
    //!
    //!

    initialize_log();


    dlog(
        &format!("Running Diamond Node (version 0.0.0). started at {}", cutils::get_now()),
        constants::Modules::App,
        constants::SecLevel::Info);

    let john: Value = json!({
        "name": "John Doe",
        "age": 43,
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    });

    println!("first phone number: {}", john["phones"][0]);

    // Convert to a string of JSON and print it out
    println!("{}", john.to_string());


// config::print_config();

// use Merkle crate, if exist

    let manual_clone_id: i8 = 0;
// CMachine::onAboutToQuit(&w);
    machine().init();
    machine().parse_args(env::args().collect(), manual_clone_id);
    println!("uuuuuuuuu  uuuuuuu u uuu");
    machine().boot_machine();


    /*

      InitCCrypto::init();

      CMachine::setLaunchDateAndCloneId("2021-03-02 00:20:00", manual_clone_id);

      w.initMachineEnvironment();

      if (true)
      {
        dummyTestsHandler();
      }
         */

    let res = get_value(&"SELECTED_PROFILE".to_string());
println!("res res res: {}", res);

    launch_giga_loop(false);//    launch_threads();
}
