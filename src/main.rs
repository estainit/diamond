// use std::thread;
// use std::thread::sleep;
// use std::time::Duration;

// use lib::c_log::log;

mod config;
mod lib;
mod constants;

use lib::threads_handler::launch_threads;

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
    lib::clog::log(&String::from("yessss"), &String::from("Gen"), &String::from("info"));
    // use Merkle crate, if exist

    println!("Hello, Diamond!");

    launch_threads();

}