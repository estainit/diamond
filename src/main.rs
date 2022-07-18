use std::env;



// use std::thread;
// use std::thread::sleep;
// use std::time::Duration;

// use lib::c_log::log;

mod config;
mod lib;
mod constants;

use std::thread::sleep;
use std::time::Duration;
use lib::threads_handler::launch_threads;
use lib::machine::machine_handler as machine_handler;


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

    config::print_config();

    // use Merkle crate, if exist

    let mut manual_clone_id: i8 = 0;
    // CMachine::onAboutToQuit(&w);

    machine_handler::CMachine::init();
    machine_handler::CMachine::parse_args(env::args().collect(), manual_clone_id);
    println!("machine_handler::CMachine::should_loop_threads HHHH: {}", machine_handler::CMachine::should_loop_threads());

    /*

      InitCCrypto::init();

      CMachine::setLaunchDateAndCloneId("2021-03-02 00:20:00", manual_clone_id);

      w.initMachineEnvironment();

      if (true)
      {
        dummyTestsHandler();
      }
         */

    launch_threads();

    sleep(Duration::from_secs(7));
}