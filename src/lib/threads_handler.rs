use std::thread::sleep;
use std::time::Duration;
use crate::{application, machine};
use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::import_minted_coins;
use crate::lib::constants;
use crate::lib::dlog::dlog;
use crate::lib::block::block_types::block_coinbase::cb1_maybe_create_coinbase_block::maybe_create_coinbase_block;
use crate::lib::block::block_types::block_repayback::repayback_block::import_double_check;
use crate::lib::dag::dag::do_prerequisites_remover;
use crate::lib::dag::missed_blocks_handler::refresh_missed_block;
use crate::lib::dag::normal_block::normal_coins_handler::do_import_coins;
use crate::lib::file_buffer_handler::file_buffer_handler::{do_read_and_parse_hard_disk_inbox, maybe_boot_dag_from_bundle};
use crate::lib::messaging_protocol::dag_message_handler::do_missed_blocks_invoker;
use crate::lib::network::email::{received_email_checks, send_private_email, send_public_email};
use crate::lib::parsing_q_handler::queue_pick::smart_pull_q;
use crate::lib::sending_q_handler::sending_q_handler::send_out_the_packet;
use crate::lib::transactions::basic_transactions::coins::coins_handler::do_coin_clean;
use crate::lib::services::polling::polling_handler::do_conclude_treatment;

pub fn launch_giga_loop(only_lazy_loadings: bool) {
    maybe_boot_dag_from_bundle();

    let mut giga_loop_counter = 0;
    while application().should_loop_threads()
    {
        giga_loop_counter += 1;
        println!("{}. Loop Top", giga_loop_counter);
        dlog(
            &format!("Looping Giga loop"),
            constants::Modules::App,
            constants::SecLevel::Info);


        // pub fn launch_threads_bunch(_only_lazy_loadings: bool) {
        if only_lazy_loadings
        {
            // some GUI initializing
            //launchLazyLoadings();
            continue;
        }


        // coin importing
        {
            // import new minted coins
            let now_ = application().now();
            import_minted_coins(&now_);

            // double checking repayblock importing
            import_double_check();
            if constants::DATABASAE_AGENT == "sqlite"
            {
                // // FIXME: remove this lines, when problem of database lock for sqlite solved and we can have real multi thread solution
                // do_import_coins(application().now());
                // PollingHandler::doConcludeTreatment();
                // ParsingQHandler::smartPullQ();
            }

            // import Normal Coins
            let now_ = application().now();
            do_import_coins(&now_);

            // remove unusefull visibility in order to lightening coins table
            do_coin_clean(&"".to_string());

            if !machine().is_in_sync_process(false) {
                maybe_create_coinbase_block();
            } else {
                dlog(
                    &format!("Since machine is in sync mode, so will not launch coinbase isuer thread"),
                    constants::Modules::App,
                    constants::SecLevel::Info);
            }
        }

        {
            // pull from parsing_q
            smart_pull_q();
        }

        {
            // missed blocks
            do_missed_blocks_invoker();

            // prerequisities cleaner
            do_prerequisites_remover();
        }

        // output cpackets
        {
            // fetching sending queue
            send_out_the_packet();

            if application().email_is_active()
            {
                // read from hard disk and send by email
                send_private_email();
                send_public_email();
            }
        }

        {
            // control if should conclude pollings/proposal
            do_conclude_treatment(&"".to_string(), false);
        }

        {
//         // ingress cpackets and parsing
            if application().email_is_active()
            {
                // fetch mailboxes
                received_email_checks();
            }

            {
                if application().use_hard_disk_as_a_buffer()
                {
                    // read messages from hard drive
                    do_read_and_parse_hard_disk_inbox();
                }

                // read messages from database queue
                refresh_missed_block();
                smart_pull_q();
            }
        }

        sleep(Duration::from_secs(10));
    }
}


// use std::thread;
// use std::thread::{JoinHandle, sleep};
// use std::time::Duration;
//
// use crate::lib::constants;
// use crate::lib::dlog::dlog;
// use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::loop_import_coinbase_coins;
// use crate::lib::dag::normal_block::normal_utxo_handler::loop_import_normal_coins;
//
// use crate::lib::machine::machine_handler as machine_handler;
// use crate::lib::transactions::basic_transactions::coins::coins_handler::loop_coin_cleaner;
// use crate::machine;
//
// //old_name_was launchImportUTXOsFromCoinbaseBlocks
// fn launch_import_coins_from_coinbase_blocks()
// {
//     dlog(
//         &"Launching import UTXOs From Coinbase Blocks...".to_string(),
//         constants::Modules::App,
//         constants::SecLevel::Info);
//     loop_import_coinbase_coins();
// }
//
// //old_name_was launchImportUTXOsFromNormalBlocks
// fn launch_import_coins_from_normal_blocks() {
//     dlog(
//         &"Launching import UTXOs From Normal Blocks...".to_string(),
//         constants::Modules::App,
//         constants::SecLevel::Info);
//     if constants::DATABASAE_AGENT != "sqlite" {
//         loop_import_normal_coins();
//     }
// }
//
// //old_name_was launchCoinCleaner
// pub fn launch_coin_cleaner()
// {
//     dlog(
//         &"Launching coins cleaner...".to_string(),
//         constants::Modules::App,
//         constants::SecLevel::Info);
//      loop_coin_cleaner(&"".to_string()); // FIXME: uncomment this line, when problem of database lock for sqlite solved and we can have real multi thread solution
// }
//
//
// // func name was launchThreadsBunch
// // default was only_lazy_loadings=false;
// pub fn launch_threads_bunch(_only_lazy_loadings: bool) {
//     let mut threads: Vec<JoinHandle<()>> = vec![];
//
//     /*
//     if only_lazy_loadings
//     {
//         let th = thread::spawn(move || {
//             println!("new thread launchLazyLoadings");
//             sleep(Duration::from_millis( 1000));
//         });
//         threads.push(th);
//
//         // std::thread(launchLazyLoadings).detach();
//         return;
//     }
//      */
//
//     maybe_boot_dag_from_bundle();
//
//     {
//         // coin importing
//
//         // import new minted coins
//         sleep(Duration::from_secs(1));
//         let thread_import_utxos_from_coinbase_blocks_handler = thread::spawn(|| {
//             // print i, then sleep thread for 2 milliseconds
//             println!("launch_import_utxos_from_coinbase_blocks going to launch");
//             launch_import_coins_from_coinbase_blocks();
//         });
//         threads.push(thread_import_utxos_from_coinbase_blocks_handler);
//
//         // // import Coins
//         // sleep(Duration::from_secs(2));
//         // let thread_import_coins_from_normal_blocks_handler = thread::spawn(|| {
//         //     // print i, then sleep thread for 2 milliseconds
//         //     println!("Import_ coins_ from_ normal_ blocks going to launch");
//         //     launch_import_coins_from_normal_blocks();
//         // });
//         // threads.push(thread_import_coins_from_normal_blocks_handler);
//         //
//         // // remove unusefull visibility in order to lightening coins table
//         // sleep(Duration::from_secs(2));
//         // let thread_coin_cleaner_handler = thread::spawn(|| {
//         //     // print i, then sleep thread for 2 milliseconds
//         //     println!("Import_ coin _cleaner going to launch");
//         //     launch_coin_cleaner();
//         // });
//         // threads.push(thread_coin_cleaner_handler);
//
//         /*
//
//                 if (!CMachine::is_in_sync_process()){
//                 std::thread(launchCoinbaseIssuer).detach();
//                 }else{
//                 CLog::log("Since machine is in sync mode, so will not launch coinbase isuer thread");
//                 }
//                     */
//     }
//
//     /*
//
//         {
//         // missed blocks
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchMissedBlocksInvoker).detach();
//
//         // prerequisities cleaner
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchPrerequisitiesRemover).detach();
//
//         }
//
//
//         {
//         // output cpackets
//
//         // fetching sending queue
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchPullSendingQ).detach();
//
//         // read from hard disk and send by email
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchEmailSender).detach();
//
//         }
//
//
//         {
//         // control if should conclude pollings/proposal
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchConcludeTreatment).detach();
//
//         // maybe iNames settelment
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchINamesSettlement).detach();
//
//         }
//
//
//         {
//         // ingress cpackets and parsing
//         if (application().email_is_active())
//         std::thread(launchEmailPoper).detach();
//
//         // read messages from hard drive
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchHardCopyReading).detach();
//
//         // read messages from database queue
//         std::this_thread::sleep_for(std::chrono::seconds(1));
//         std::thread(launchSmartPullFromParsingQ).detach();
//
//         }
//
//
//         std::thread(launchMonitorRefresher).detach();
//
//         // lazy loadings
//         std::thread(launchLazyLoadings).detach();
//         */
//
//
//     for th in threads {
//         th.join().unwrap();
//     }
// }
//
// // launchThreads
// pub fn launch_threads() {
//     println!("should_ loop_ threads in launch_ threads: {}", application().should_loop_threads());
//     dlog(
//         &String::from("launch threads bunch"),
//         constants::Modules::App,
//         constants::SecLevel::Info);
//
//     launch_threads_bunch(false);
//
//     // //CoinbaseIssuer::tryCreateCoinbaseBlock();
// }
