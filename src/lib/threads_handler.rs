use tokio::task;
use tokio::time::{sleep, Duration};


// use std::thread;
// use std::thread::sleep;
// use std::time::Duration;

use crate::lib::constants as CConsts;
use crate::lib::dlog::dlog;
use crate::lib::file_buffer_handler::maybe_boot_dag_from_bundle;
use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::loop_import_coinbase_coins;

use crate::lib::machine::machine_handler as machine_handler;

// func name was launchImportUTXOsFromCoinbaseBlocks
async fn launch_import_utxos_from_coinbase_blocks()
{

    dlog(
        &"Launching import UTXOs From Coinbase Blocks...".to_string(),
        CConsts::Modules::App,
        CConsts::SecLevel::Info);
    loop_import_coinbase_coins().await;
}


// func name was launchThreadsBunch
// default was only_lazy_loadings=false;
pub async fn launch_threads_bunch(only_lazy_loadings: bool) {
    let mut asyncs = vec![];

    /*
    if only_lazy_loadings
    {
        let th = thread::spawn(move || {
            println!("new thread launchLazyLoadings");
            sleep(Duration::from_millis( 1000));
        });
        asyncs.push(th);

        // std::thread(launchLazyLoadings).detach();
        return;
    }
     */

    maybe_boot_dag_from_bundle();

    {
        // coin importing

        // import new minted coins
        sleep(Duration::from_secs(1)).await;
        let as_launch_import_utxos_from_coinbase_blocks = task::spawn(async move {
            launch_import_utxos_from_coinbase_blocks()
        });
        asyncs.push(as_launch_import_utxos_from_coinbase_blocks);

        /*
                // import UTXOs
                std::this_thread::sleep_for(std::chrono::seconds(2));
                std::thread(launchImportUTXOsFromNormalBlocks).detach();

                // remove unusefull visibility in order to lightening coins table
                std::this_thread::sleep_for(std::chrono::seconds(1));
                std::thread(launchCoinCleaner).detach();

                if (!CMachine::isInSyncProcess()){
                std::thread(launchCoinbaseIssuer).detach();
                }else{
                CLog::log("Since machine is in sync mode, so will not launch coinbase isuer thread");
                }
                    */
    }

    /*

        {
        // missed blocks
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchMissedBlocksInvoker).detach();

        // prerequisities cleaner
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchPrerequisitiesRemover).detach();

        }


        {
        // output cpackets

        // fetching sending queue
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchPullSendingQ).detach();

        // read from hard disk and send by email
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchEmailSender).detach();

        }


        {
        // control if should conclude pollings/proposal
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchConcludeTreatment).detach();

        // maybe iNames settelment
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchINamesSettlement).detach();

        }


        {
        // ingress cpackets and parsing
        if (CConsts::EMAIL_IS_ACTIVE)
        std::thread(launchEmailPoper).detach();

        // read messages from hard drive
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchHardCopyReading).detach();

        // read messages from database queue
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchSmartPullFromParsingQ).detach();

        }


        std::thread(launchMonitorRefresher).detach();

        // lazy loadings
        std::thread(launchLazyLoadings).detach();
        */

    // let (a) = tokio::join!(as_launch_import_utxos_from_coinbase_blocks);
    // use the results so the compiler doesn't complain
    // as_launch_import_utxos_from_coinbase_blocks.unwrap();
    // b.unwrap();
}

// launchThreads
pub async fn launch_threads() {
    println!("DDDDDDDD2: {}", machine_handler::CMachine::should_loop_threads());
    dlog(
        &String::from("launch threads bunch"),
        CConsts::Modules::App,
        CConsts::SecLevel::Info);
    launch_threads_bunch(false).await;

    // //CoinbaseIssuer::tryCreateCoinbaseBlock();
}
