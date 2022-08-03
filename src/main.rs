extern crate core;

use std::env;

use std::thread;
use std::time::Duration;
use once_cell::sync::Lazy;
use std::sync::{LockResult, Mutex, MutexGuard};
use std::thread::sleep as std_sleep;
use lib::rest::apis;

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
use crate::lib::ccrypto;
use crate::lib::threads_handler::launch_giga_loop;
use crate::lib::utils::cmerkle as cmerkle;
use crate::lib::utils::permutation_handler::PermutationHandler;
use crate::machine_handler::CMachine;

static CMACHINE: Lazy<Mutex<CMachine>> = Lazy::new(|| Mutex::new(CMachine::new()));
fn machine() -> MutexGuard<'static, CMachine>{
    CMACHINE.lock().unwrap()
}

#[tokio::main]
async fn main() {
    

// config::print_config();

// use Merkle crate, if exist

    let manual_clone_id: i8 = 0;
// CMachine::onAboutToQuit(&w);

    machine().init();
    machine().parse_args(env::args().collect(), manual_clone_id);




    /*

      InitCCrypto::init();

      CMachine::setLaunchDateAndCloneId("2021-03-02 00:20:00", manual_clone_id);

      w.initMachineEnvironment();

      if (true)
      {
        dummyTestsHandler();
      }
         */
        tokio::join!(
            lib::rest::apis::run_web_server(),
            run_loops()
        );
    // launch_giga_loop(false);//    launch_threads();
    

}

async fn run_loops() {
    launch_giga_loop(false);
}