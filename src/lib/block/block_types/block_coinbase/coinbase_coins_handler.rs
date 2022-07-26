use std::thread;

use crate::lib::constants as CConsts;
use crate::cutils;
use crate::lib::custom_types::{CDateT};

// coinbase_coins_handler

use crate::lib::machine::machine_handler as machine_handler;

//func old name was loopImportCoinbaseUTXOs
pub async fn loop_import_coinbase_coins()
{
    println!("DDDDDDDD1: {}", machine_handler::CMachine::should_loop_threads());
    let _thread_prefix = "import_coinbase_UTXOs_";
    let thread_code = thread::current().id();
    println!("thread id: {:?}", thread_code);
    machine_handler::CMachine::should_loop_threads();
    println!("____________________machine_handler::CMachine::should_loop_threads(): {}", machine_handler::CMachine::should_loop_threads());

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


/*

void CoinbaseUTXOHandler::importCoinbasedUTXOs(const QString &c_date)
{
  CLog::log("import Coinbased UTXOs " + c_date);

  // find coinbase block with 2 cycle age old, and insert the outputs as a matured&  spendable outputs to table trx_utxos
  auto [minCreationDate, maxCreationDate] = CUtils::getCbUTXOsDateRange(c_date);
  Q_UNUSED(minCreationDate);
  CLog::log("extract maturated coinbase UTXOs created before(" + maxCreationDate + ")", "trx", "trace");

  QVDRecordsT coinbases = DAG::searchInDAG(
      {{"b_type", CConsts::BLOCK_TYPES::Coinbase},
       {"b_utxo_imported", CConsts::NO},
       {"b_creation_date", maxCreationDate, "<="}},
      {"b_hash", "b_body"},
      {{"b_creation_date", "ASC"}});

  GRecordsT pledged_accounts_info = GeneralPledgeHandler::getPledgedAccounts(
      c_date,
      true);
  //  CLog::log("pledged Accounts Info: " + CUtils::dumpIt(pledged_accounts_info), "app", "trace");

  for (QVDicT a_coinbase_record : coinbases)
  {
    // start transactional block of coinbase UTXO importing: FIXME: implement it ASAP
    auto unwrapRes = BlockUtils::unwrapSafeContentForDB(a_coinbase_record.value("b_body").toString());
    if (!unwrapRes.status)
    {
      CLog::log("maleformed recorded Coinbase unwrapping block(" + a_coinbase_record.value("b_hash").toString() + ")!", "app", "fatal");
      CUtils::exiter("maleformed recorded Coinbase block(" + a_coinbase_record.value("b_hash").toString() + ")!", 76);
    }
    QJsonObject block = CUtils::parseToJsonObj(unwrapRes.content); // do not need safe open check
    if (block.keys().size() == 0)
    {
      CLog::log("maleformed recorded Coinbase to json block(" + a_coinbase_record.value("b_hash").toString() + ")!", "app", "fatal");
      CUtils::exiter("maleformed recorded Coinbase block(" + a_coinbase_record.value("b_hash").toString() + ")!", 76);
    }

    // since we examinate Coinbases from 2 cycle past, then we must be sure the entire precedents has visibility of these UTXOs
    auto [status, descendent_blocks, validity_percentage] = DAG::getAllDescendents(block.value("bHash").toString());
    Q_UNUSED(status);
    Q_UNUSED(validity_percentage);
    CLog::log("visibleBys after exclude floating signature blocks(CoinBases): " + CUtils::dumpIt(descendent_blocks), "trx", "trace");

    JORecordsT repayment_docs{};
    // clog.app.info(`block.docs[0].outputs ${block.docs[0].outputs}`);
    auto the_only_doc = block.value("docs").toArray()[0].toObject();
    auto outputs = the_only_doc.value("outputs").toArray();
    for (COutputIndexT output_index = 0; output_index < outputs.size(); output_index++)
    {
      QJsonArray an_output = outputs[output_index].toArray();
      QString the_coin = CUtils::packCoinCode(the_only_doc.value("dHash").toString(), output_index);

      /**
       * if the account is pledged, so entire account incomes must be transferres to repayback transaction and
       * from that, cutting repayments and at the end if still remains some coins, return back to shareholder's account
       */
      if (pledged_accounts_info.keys().contains(an_output[0].toString()))
      {
        QJsonObject a_repayback_doc = RepaymentDocument::calcRepaymentDetails(
            the_only_doc.value("dHash").toString(),
            output_index,
            static_cast<CMPAIValueT>(an_output[1].toDouble()),
            pledged_accounts_info,
            an_output[0].toString());

        CLog::log("Repayment Doc: " + CUtils::serializeJson(a_repayback_doc), "trx", "trace");
        repayment_docs.push_back(a_repayback_doc);
      }
      else
      {
        for (QVDicT a_block_record : descendent_blocks)
        {
          CLog::log("Importing Coinbase block Coins Block(" + CUtils::hash8c(block.value("bHash").toString()) + ")", "trx", "info");
          UTXOHandler::addNewUTXO(
              a_block_record.value("b_creation_date").toString(),
              the_coin,
              a_block_record.value("b_hash").toString(),
              an_output[0].toString(),           // address
              an_output[1].toDouble(),           // coin_value
              block.value("bCDate").toString()); // refCreationDate:
        }
      }
    }

    // if there is some cutting from income, create a new block(RpBlock) and record
    if (repayment_docs.size() > 0)
    {
      RepaybackBlock::createRepaymentBlock(
          block,
          repayment_docs,
          descendent_blocks);
    }

    // update utxo_imported
    DAG::updateUtxoImported(block.value("bHash").toString(), CConsts::YES);

    // end of transactional block of coinbase UTXO importing: FIXME: implement it ASAP
  }
}

 */

// TODO some uintteasts need
//  every coinbased incomes will be spendable after 2 cycle and right after starting 3rd cycle

//old_name_was calcCoinbasedOutputMaturationDate
#[allow(dead_code)]
pub fn calc_coinbased_output_maturation_date(c_date_: CDateT) -> CDateT {
    let mut c_date = c_date_.clone();
    if c_date == "" {
        c_date = cutils::get_now();
    }

    let mature_date: String = cutils::minutes_after(CConsts::COINBASE_MATURATION_CYCLES as u64 * cutils::get_cycle_by_minutes() as u64, c_date);
    return cutils::get_coinbase_range(mature_date).from;
}

