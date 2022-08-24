use std::thread;

use crate::lib::constants;
use crate::{application, machine};
use crate::lib::custom_types::{CDateT};
use crate::lib::dlog::dlog;

//func old name was loopImportCoinbaseUTXOs
pub fn loop_import_coinbase_coins()
{
    println!("DDDDDDDD1: {}", application().should_loop_threads());
    let thread_prefix = "import_coinbase_UTXOs_".to_string();
    let thread_code = format!("{:?}", thread::current().id());
    println!("thread id: {:?}", thread_code);
    // dlog(
    //     &format!("Going to launch the import normal coins for {} seconds intervals. Thread({} {})",
    //              application().nb_coins_import_gap(),
    //              &thread_prefix,
    //              &thread_code ),
    //     constants::Modules::App,
    //     constants::SecLevel::Info);
    println!("____________________should_loop_threads(): {}", application().should_loop_threads());


    while application().should_loop_threads()
    {
        machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::RUNNING.to_string());
        let now_ = application().get_now();
        import_coinbased_coins(&now_);
        /*

        // double checking repayblock importing
        RepaybackBlock::importDoubleCheck();

        if ( (constants::DATABASAE_AGENT == "sqlite") && (CMachine::shouldLoopThreads()) )
        {
        // FIXME: remove this lines, when problem of database lock for sqlite solved and we can have real multi thread solution
        NormalUTXOHandler::doImportUTXOs(application().get_now());

        PollingHandler::doConcludeTreatment();

        ParsingQHandler::smartPullQ();

        }

        */
        machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::SLEEPING.to_string());
        // sleep(Duration::from_secs(application().coinbase_import_gap()));
    }

    machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::STOPPED.to_string());
    dlog(
        &format!("Gracefully stopped thread({}) of loop Import Coinbase Coins", thread_prefix + &thread_code),
        constants::Modules::App,
        constants::SecLevel::Info);
}

//old_name_was importCoinbasedUTXOs
pub fn import_coinbased_coins(c_date: &CDateT)
{
    dlog(&format!("import Coinbased UTXOs {}", c_date.clone()), constants::Modules::App, constants::SecLevel::TmpDebug);

    // find coinbase block with 2 cycle age old, and insert the outputs as a matured&  spendable outputs to table trx_utxos
    let max_creation_date = application().get_cb_coins_date_range(&c_date).to;
    dlog(&format!("Extract maturated coinbase UTXOs created before({})", max_creation_date.clone()), constants::Modules::Trx, constants::SecLevel::TmpDebug);
    /*
      QVDRecordsT coinbases = DAG::searchInDAG(
          {{"b_type", constants::block_types::COINBASE},
           {"b_utxo_imported", constants::NO},
           {"b_creation_date", maxCreationDate, "<="}},
          {"b_hash", "b_body"},
          {{"b_creation_date", "ASC"}});

      GRecordsT pledged_accounts_info = GeneralPledgeHandler::getPledgedAccounts(
          c_date,
          true);
      //  CLog::log("pledged Accounts Info: " + cutils::dumpIt(pledged_accounts_info), "app", "trace");

      for (QVDicT a_coinbase_record : coinbases)
      {
        // start transactional block of coinbase UTXO importing: FIXME: implement it ASAP
        auto unwrapRes = BlockUtils::unwrapSafeContentForDB(a_coinbase_record.value("b_body").to_string());
        if (!unwrapRes.status)
        {
          CLog::log("maleformed recorded Coinbase unwrapping block(" + a_coinbase_record.value("b_hash").to_string() + ")!", "app", "fatal");
          cutils::exiter("maleformed recorded Coinbase block(" + a_coinbase_record.value("b_hash").to_string() + ")!", 76);
        }
        JSonObject block = cutils::parseToJsonObj(unwrapRes.content); // do not need safe open check
        if (block.keys().len() == 0)
        {
          CLog::log("maleformed recorded Coinbase to json block(" + a_coinbase_record.value("b_hash").to_string() + ")!", "app", "fatal");
          cutils::exiter("maleformed recorded Coinbase block(" + a_coinbase_record.value("b_hash").to_string() + ")!", 76);
        }

        // since we examinate Coinbases from 2 cycle past, then we must be sure the entire precedents has visibility of these UTXOs
        auto [status, descendent_blocks, validity_percentage] = DAG::getAllDescendents(block.value("bHash").to_string());
        Q_UNUSED(status);
        Q_UNUSED(validity_percentage);
        CLog::log("visibleBys after exclude floating signature blocks(CoinBases): " + cutils::dumpIt(descendent_blocks), "trx", "trace");

        JORecordsT repayment_docs{};
        // clog.app.info(`block.docs[0].outputs ${block.docs[0].outputs}`);
        auto the_only_doc = block.value("docs").toArray()[0].toObject();
        auto outputs = the_only_doc.value("outputs").toArray();
        for (COutputIndexT output_index = 0; output_index < outputs.len(); output_index++)
        {
          JSonArray an_output = outputs[output_index].toArray();
          String the_coin = cutils::packCoinCode(the_only_doc.value("dHash").to_string(), output_index);

          /**
           * if the account is pledged, so entire account incomes must be transferres to repayback transaction and
           * from that, cutting repayments and at the end if still remains some coins, return back to shareholder's account
           */
          if (pledged_accounts_info.keys().contains(an_output[0].to_string()))
          {
            JSonObject a_repayback_doc = RepaymentDocument::calcRepaymentDetails(
                the_only_doc.value("dHash").to_string(),
                output_index,
                static_cast<CMPAIValueT>(an_output[1].toDouble()),
                pledged_accounts_info,
                an_output[0].to_string());

            CLog::log("Repayment Doc: " + cutils::serializeJson(a_repayback_doc), "trx", "trace");
            repayment_docs.push(a_repayback_doc);
          }
          else
          {
            for (QVDicT a_block_record : descendent_blocks)
            {
              CLog::log("Importing Coinbase block Coins Block(" + cutils::hash8c(block.value("bHash").to_string()) + ")", "trx", "info");
              UTXOHandler::addNewUTXO(
                  a_block_record.value("b_creation_date").to_string(),
                  the_coin,
                  a_block_record.value("b_hash").to_string(),
                  an_output[0].to_string(),           // address
                  an_output[1].toDouble(),           // coin_value
                  block.value("bCDate").to_string()); // refCreationDate:
            }
          }
        }

        // if there is some cutting from income, create a new block(RpBlock) and record
        if (repayment_docs.len() > 0)
        {
          RepaybackBlock::createRepaymentBlock(
              block,
              repayment_docs,
              descendent_blocks);
        }

        // update utxo_imported
        DAG::updateUtxoImported(block.value("bHash").to_string(), constants::YES);

        // end of transactional block of coinbase UTXO importing: FIXME: implement it ASAP
      }
     */
}


// TODO some uintteasts need
//  every coinbased incomes will be spendable after 2 cycle and right after starting 3rd cycle

//old_name_was calcCoinbasedOutputMaturationDate
#[allow(dead_code, unused)]
pub fn calc_coinbased_output_maturation_date(c_date: &CDateT) -> CDateT {
    let cycle_by_minutes = application().get_cycle_by_minutes() as u64;
    let mature_date: String = application().minutes_after(
        constants::COINBASE_MATURATION_CYCLES as u64 * cycle_by_minutes,
        c_date);
    return application().get_coinbase_range(&mature_date).from;
}

