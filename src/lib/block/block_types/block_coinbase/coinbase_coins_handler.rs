use std::thread;
use std::thread::ThreadId;

// coinbase_coins_handler


//func old name was loopImportCoinbaseUTXOs
pub fn loop_import_coinbase_coins()
{
    let mut thread_prefix = "import_coinbase_UTXOs_";
    let mut thread_code = thread::current().id();
    println!("thread id: {:?}", thread_code);
    /*

    while (CMachine::shouldLoopThreads())
    {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);


    importCoinbasedUTXOs(CUtils::getNow());

    // double checking repayblock importing
    RepaybackBlock::importDoubleCheck();

    if ( (CConsts::DATABASAE_AGENT == "sqlite") && (CMachine::shouldLoopThreads()) )
    {
    // FIXME: remove this lines, when problem of database lock for sqlite solved and we can have real multi thread solution
    NormalUTXOHandler::doImportUTXOs(CUtils::getNow());

    PollingHandler::doConcludeTreatment();

    ParsingQHandler::smartPullQ();

    }

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getCoinbaseImportGap()));
    }

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
    CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Import Coinbase UTXOs");
    */
}
