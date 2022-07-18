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

use crate::constants as CConsts;

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
    lib::dlog::dlog(
        &String::from("yessss"),
        CConsts::Modules::App,
        CConsts::SecLevel::Info);
    // use Merkle crate, if exist

    println!("Hello, Diamond!");

    launch_threads();

    sleep(Duration::from_secs(7));
}