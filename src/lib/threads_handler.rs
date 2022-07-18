
use std::thread;
use std::thread::sleep;
use std::time::Duration;


use crate::lib::clog::log;
use crate::lib::file_buffer_handler::maybe_boot_dag_from_bundle;

// func name was launchThreadsBunch
// default was only_lazy_loadings=false;
pub fn launch_threads_bunch(only_lazy_loadings: bool) {
    let mut threads = vec![];

    if only_lazy_loadings
    {
        let th = thread::spawn(move || {
            println!("new thread launchLazyLoadings");
            sleep(Duration::from_millis( 1000));
        });
        threads.push(th);

        // std::thread(launchLazyLoadings).detach();
        return;
    }

    maybe_boot_dag_from_bundle();

    /*

        {
        // coin importing

        // import new minted coins
        std::this_thread::sleep_for(std::chrono::seconds(1));
        std::thread(launchImportUTXOsFromCoinbaseBlocks).detach();

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
        }


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
}

// launchThreads
pub fn launch_threads() {
    log(&String::from("launch threads bunch"), &String::from("Gen"), &String::from("info"));
    launch_threads_bunch(false);

    // //CoinbaseIssuer::tryCreateCoinbaseBlock();
}
