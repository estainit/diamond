extern crate core;

use std::env;


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

// use crate::lib::constants as cconsts;
// use lib::threads_handler::launch_threads;
use lib::machine::machine_handler as machine_handler;
use lib::utils::cutils as cutils;
use crate::lib::ccrypto;
// use crate::lib::ccrypto;
// use crate::lib::custom_types::VString;
use crate::lib::utils::cmerkle as cmerkle;
use crate::lib::utils::permutation_handler::PermutationHandler;

// use crate::tests::unit_tests::cutils::test_chunk_qstring_list;

#[tokio::main]
async fn main() {
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
    {
        let mut hp = PermutationHandler::new(
            &vec!["a".to_string(), "b".to_string(), "c".to_string()],
            2,
            true,
            &vec![],
            &vec![]);
        println!("hp.m_elements.len() {:?}", hp.m_elements);
        println!("hp.m_permutations.len() {:?}", hp.m_permutations);
        assert_eq!(hp.m_permutations.len() , 3);
        hp.testAnalyze(&vec![]);
    }

// config::print_config();

// use Merkle crate, if exist

    let manual_clone_id: i8 = 0;
// CMachine::onAboutToQuit(&w);

    machine_handler::CMachine::init();
    machine_handler::CMachine::parse_args(env::args().collect(), manual_clone_id);

    /*

      InitCCrypto::init();

      CMachine::setLaunchDateAndCloneId("2021-03-02 00:20:00", manual_clone_id);

      w.initMachineEnvironment();

      if (true)
      {
        dummyTestsHandler();
      }
         */

// launch_threads().await;

// sleep(Duration::from_secs(5)).await;
}

