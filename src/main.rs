extern crate core;

mod lib;
mod tests;

use std::{env, time};
use std::thread;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::sync::{LockResult, Mutex, MutexGuard};
use std::thread::sleep as std_sleep;

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
use lib::rest::apis;
use crate::apis::{do_handshake, do_handshake_by_email};
use crate::cutils::strip_parentheses_as_break_line;
use crate::lib::machine::app_params::AppParams;
use crate::lib::machine::machine_neighbor::get_neighbors;

static APPGLOBAL: Lazy<Mutex<AppParams>> = Lazy::new(|| Mutex::new(AppParams::new()));
fn application() -> MutexGuard<'static, AppParams> { APPGLOBAL.lock().unwrap() }

static CMACHINE: Lazy<Mutex<CMachine>> = Lazy::new(|| Mutex::new(CMachine::new()));
fn machine() -> MutexGuard<'static, CMachine> {
    CMACHINE.lock().unwrap()
}

static DBHANDLER: Lazy<Mutex<DBHandler>> = Lazy::new(|| Mutex::new(DBHandler::new()));
fn dbhandler() -> MutexGuard<'static, DBHandler> { DBHANDLER.lock().unwrap() }

fn main() {
    //! # Diamond, The scalable Blockchain
    //! ```
    //! fn main()
    //! ```
    //! This starts whole game
    //!

    application().dummy_init();

    let force_clone_id: i8 = 0;
    let force_boot_in_dev_mod: bool = false;
    machine().parse_args(env::args().collect(), force_clone_id);
    initialize_log();
    machine().initialize_machine();
    machine().boot_machine();

    let mut web_server_msg: &str = "";
    let should_launch_web_server = false;
    if should_launch_web_server
    {
        web_server_msg = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(lib::rest::apis::run_web_server()) {
            Ok(r) => {
                ". Webserver Ready on http://localhost:8080"
            }
            Err(e) => {
                ". Webserver Failed!"
            }
        };
    }

    let msg = &format!(
        "Running Diamond Node (version {}). started at {} {}",
        constants::CLIENT_VERSION,
        application().get_now(),
        web_server_msg);
    dlog(
        msg,
        constants::Modules::App,
        constants::SecLevel::Info);
    println!("{}", msg);

    {
        // web api part
        if application().id() == 1
        {
            let (status, msg) = do_handshake_by_email("user@imagine.com".to_string());
            println!("do_handshake: {}, {}", status, msg);
        }
    }


    launch_giga_loop(false);//    launch_threads();
}

async fn run_loops() {
    launch_giga_loop(false);
}