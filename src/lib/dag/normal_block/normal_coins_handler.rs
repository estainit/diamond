use std::thread;
use postgres::types::ToSql;
use crate::lib::constants;
use crate::{application, machine};
use crate::lib::custom_types::{CDateT, ClausesT, JSonObject, QVDRecordsT};
use crate::lib::dag::dag::{dag_has_blocks_which_are_created_in_current_cycle, search_in_dag};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, simple_eq_clause};
use crate::lib::dlog::dlog;
use crate::lib::utils::dumper::dump_it;

//old_name_was loopImportNormalUTXOs
#[allow(unused, dead_code)]
pub fn loop_import_normal_coins()
{
    let thread_prefix = "import_normal_coins_".to_string();
    let thread_code = format!("{:?}", thread::current().id());

    // dlog(
    //     &format!("Going to launch the import normal coins for {} seconds intervals. Thread({} {})",
    //              machine().get_nb_coins_import_gap(),
    //              &thread_prefix,
    //              &thread_code ),
    //     constants::Modules::App,
    //     constants::SecLevel::Info);

    while application().should_loop_threads()
    {
        machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::RUNNING.to_string());
        let now_ = application().get_now();
        do_import_coins(&now_);

        machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::SLEEPING.to_string());
        // sleep(Duration::from_secs(machine().get_nb_coins_import_gap()));
    }

    machine().report_thread_status(&thread_prefix, &thread_code, &constants::thread_state::STOPPED.to_string());
    dlog(
        &format!("Gracefully stopped thread({}) of loop Import Normal Coins", thread_prefix.clone() + &thread_code),
        constants::Modules::App,
        constants::SecLevel::Info);
}

//old_name_was doImportUTXOs
pub fn do_import_coins(c_date_: &CDateT)
{
    let mut c_date = c_date_.clone();
    if c_date == ""
    { c_date = application().get_now(); }

    import_normal_block_coins(&c_date);


//  bool OUTPUT_TIMELOCK_IS_ENABLED = false;
//  if (OUTPUT_TIMELOCK_IS_ENABLED)
//      outputTimeLockHandler.importTimeLocked();
}

//old_name_was retrieveProperBlocks
#[allow(unused, dead_code)]
pub fn retrieve_proper_blocks(c_date: &CDateT) -> QVDRecordsT
{
    //find normal block with 12 hours age old, and insert the outputs as a matured & spendable outputs to table trx_coins
    let back_in_time = application().get_cycle_by_minutes() as u64;
    let min_creation_date = application().minutes_before(
        back_in_time,
        c_date);

    dlog(
        &format!("importing matured Coins(Normal Block) before({})", min_creation_date.clone()),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let b_type = constants::block_types::NORMAL.to_string();
    let b_coins_imported = constants::NO.to_string();
    let mut clauses: ClausesT = vec![
        // ModelClause {
        //     m_field_name: "b_type",
        //     m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        //     m_clause_operand: "IN",
        //     m_field_multi_values: vec![&constants::block_types::Normal.to_string() as &(dyn ToSql + Sync)],
        // },
        simple_eq_clause("b_type", &b_type),
        simple_eq_clause("b_coins_imported", &b_coins_imported),
        ModelClause {
            m_field_name: "b_creation_date",
            m_field_single_str_value: &min_creation_date as &(dyn ToSql + Sync),
            m_clause_operand: "<=",
            m_field_multi_values: vec![],
        },
    ];  // (12 hours * 60 minutes) from now

    let now_ = application().get_now();
    if dag_has_blocks_which_are_created_in_current_cycle(&now_)
    {
        //  * by (DAG-Has-Blocks-Which-Are-Created-In-Currrent-Cycle) clause we are almost sure the machine is synched
        //  * so must avoiding immidiately importing blocks with fake-old-creation Date
        //  * all above condition & clauses are valid for a normal working machine.
        //  * but if machine newly get synched, it has some blocks which are newly received but belongs to some old cycles
        //  * so we control if machine was in sync mode in last 12 hours? if no we add the b_receive_date condition
        let last_sync_status: JSonObject = machine().get_last_sync_status();
        dlog(
            &format!("last SyncStatus in import Normal Block coins: {}", dump_it(&last_sync_status)),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        let back_in_time = application().get_cycle_by_minutes();
        let now_=application().get_now();
        if last_sync_status["lastTimeMachineWasInSyncMode"].to_string() < application().minutes_before(back_in_time, &now_)
        {
            clauses.push(ModelClause {
                m_field_name: "b_receive_date",
                m_field_single_str_value: &min_creation_date as &(dyn ToSql + Sync),
                m_clause_operand: "<",
                m_field_multi_values: vec![],
            });
        }
    }
    let records: QVDRecordsT = search_in_dag(
        clauses,
        vec!["b_hash", "b_body"],
        vec![
            &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
        ],
        0,
        false);

    return records;
}

//old_name_was importNormalBlockUTXOs
pub fn import_normal_block_coins(c_date: &CDateT)
{
    dlog(
        &format!("Importing Normal block Coins at {}", c_date),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    /*
      QVDRecordsT wBlocks = retrieveProperBlocks(c_date);
      if (wBlocks.len() == 0)
      {
        CLog::log("There is no importable normal block for time(" + c_date + ")", "trx", "trace");
        return;
      }

      CoinImportDataContainer* block_inspect_container = new CoinImportDataContainer;
      Block* block = {};

      for (QVDicT wBlock: wBlocks)
      {
        block_inspect_container->reset();
        delete block;

        JSonObject blockJ = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock.value("b_body").to_string()).content);
        block = BlockFactory::create(blockJ);
        CLog::log("Extract matured coins(NormalBlock) on c_date(" + c_date + ") from block(" + cutils::hash8c(wBlock.value("b_hash").to_string()) + ") created on(" + block.m_block_creation_date + ")", "trx", "info");

        analyzeBlockUsedCoins(block_inspect_container, block);

        block_inspect_container.m_DPCost_coin_codes = {};
        for (CDocHashT a_key: block_inspect_container.m_a_single_trx_DPCost.keys())
          block_inspect_container.m_DPCost_coin_codes.push(block_inspect_container.m_a_single_trx_DPCost[a_key].m_coin);

        if (block_inspect_container.m_must_not_import_trx_outputs.len() > 0)
          std::sort(block_inspect_container.m_must_not_import_trx_outputs.begin(), block_inspect_container.m_must_not_import_trx_outputs.end());
        std::vector<CDocHashT>::iterator last = std::unique(block_inspect_container.m_must_not_import_trx_outputs.begin(), block_inspect_container.m_must_not_import_trx_outputs.end());
        block_inspect_container.m_must_not_import_trx_outputs.erase(last, block_inspect_container.m_must_not_import_trx_outputs.end());

        CLog::log("block_inspect_container(" + cutils::hash8c(block->getBlockHash()) + ") block_inspect_container: " + block_inspect_container->dumpMe(), "trx", "trace");


        if (CMachine::is_in_sync_process())
        {
          String currentWNVTBIA = KVHandler::getValue("SusBlockWNVTBIA");
          JSonArray JcurrentWNVTBIA {};
          if (currentWNVTBIA == "")
          {
            KVHandler::upsertKValue("SusBlockWNVTBIA", cutils::serializeJson(JSonArray{}));
          } else {
            JcurrentWNVTBIA = cutils::parseToJsonArr(currentWNVTBIA);
          }
          if (JcurrentWNVTBIA.contains(block->getBlockHash()))
          {
            if (!block_inspect_container.m_block_is_sus_case)
            {
              /**
               * during insert to parsing q, machine recognized the block is suspicious and must has some FVotes
               * and now machine recognized the sus votes still not being considered
               * so returns back inorder to giving more time to machine to insert upcoming sus votes in few later seconds
               */
              CLog::log("can not import block coins because parsingQ recognization was sus and now there is no vote! Block(" + cutils::hash8c(block->getBlockHash()) + ") blockIsSusCase(" + cutils::dumpIt(block_inspect_container.m_block_is_sus_case) + ") block_inspect_container: " + block_inspect_container->dumpMe(), "trx", "warning");
              continue;
            }
          }
        }

        if (block_inspect_container.m_does_enough_sus_votes_exist == "notEnoughSusVotesExist")
        {
          // log block import report
          //  logNormalBlockUTXOsImport.logImport({
          //  blockHash: block.blockHash,
          //  block_inspect_container: _.clone(block_inspect_container)
          //  });
          continue;
        }

        // find all descendent of current block(if exist)
        auto[status_, wBlocksDescendents, validityPercentage_] = DAG::getAllDescendents(block->getBlockHash());
        Q_UNUSED(status_);
        Q_UNUSED(validityPercentage_);

        // donate double spended funds(if exist)
        for (BlockTreasuryLog anEntry: block_inspect_container.m_block_treasury_logs)
        {
          TreasuryHandler::donateTransactionInput(
            anEntry.m_title,
            anEntry.m_cat,
            anEntry.m_descriptions,
            block.m_block_creation_date,
            anEntry.m_value,
            block->getBlockHash(),
            anEntry.m_coin
          );
        }


        // calculate if Block Trx Fee must be modified
        CMPAIValueT toCut = 0;
        CMPAIValueT toCutFromTreasuryFee = 0;
        CMPAIValueT toCutFromBackerFee = 0;
        for (CDocHashT docHash: block_inspect_container.m_must_not_import_trx_outputs)
          toCut += block_inspect_container.m_a_single_trx_DPCost[docHash].m_value; // cut the DPCost of rejected/donated transaction from block incomes

        if (toCut > 0)
        {
          toCutFromBackerFee = cutils::CFloor((toCut * constants::BACKER_PERCENT_OF_BLOCK_FEE) / 100);// - cnfHandler.getBlockFixCost();
          toCutFromTreasuryFee = cutils::CFloor(toCut - toCutFromBackerFee);
        }
        block_inspect_container.m_to_cut_from_backer_fee = toCutFromBackerFee;
        block_inspect_container.m_to_cut_from_treasury_fee = toCutFromTreasuryFee;

        if (block_inspect_container.m_rejected_transactions.len() > 0)
        {
          // listener.doCallSync('SPSH_block_has_double_spend_input', { block, block_inspect_container });
        }

        // import block DPCost Backer & Treasury
        block_inspect_container.m_block_DPCost_backer_final = block_inspect_container.m_block_DPCost_backer.m_value - block_inspect_container.m_to_cut_from_backer_fee;
        block_inspect_container.m_block_DPCost_treasury_final = block_inspect_container.m_block_DPCost_treasury.m_value - block_inspect_container.m_to_cut_from_treasury_fee;

        if (block_inspect_container.m_block_DPCost_backer_final < 0)
          block_inspect_container.m_block_DPCost_treasury_final += block_inspect_container.m_block_DPCost_backer_final; // to cover cnfHandler.getBlockFixCost()

        block_inspect_container.m_block_has_income = (
          (block_inspect_container.m_block_DPCost_backer_final > 0) &&
          (block_inspect_container.m_block_DPCost_treasury_final > 0));

        if (block_inspect_container.m_block_has_income)
        {

          // import backer's income
          CLog::log("Importing Normal block Coins(Backer) Block(" + cutils::hash8c(block->getBlockHash()) + ") ", "trx", "info");
          for (auto aWBlock: wBlocksDescendents)
            UTXOHandler::add_new_coin(
              aWBlock.value("b_creation_date").to_string(),
              block_inspect_container.m_block_DPCost_backer.m_coin,
              aWBlock.value("b_hash").to_string(),
              block_inspect_container.m_block_DPCost_backer.m_address,
              block_inspect_container.m_block_DPCost_backer_final,
              block.m_block_creation_date);


          // import blockDPCost_Treasury
          String title = block_inspect_container.m_block_DPCost_treasury.m_title;

          if (block_inspect_container.m_must_not_import_trx_outputs.len() > 0)
          {
            // cut fees because of rejected transactions or ...
            StringList tmp = {};
            for (auto elm: block_inspect_container.m_must_not_import_trx_outputs)
              tmp.push(cutils::hash8c(elm));
            title += " - rejected TRXs(" + tmp.join(", ") + ") = sum(" + cutils::microPAIToPAI6(block_inspect_container.m_to_cut_from_treasury_fee) + ") ";
          }

          TreasuryHandler::insertIncome(
            title,
            block_inspect_container.m_block_DPCost_treasury.m_cat,
            block_inspect_container.m_block_DPCost_treasury.m_descriptions,
            block.m_block_creation_date,
            block_inspect_container.m_block_DPCost_treasury_final,
            block->getBlockHash(),
            block_inspect_container.m_block_DPCost_treasury.m_coin);

          // import free-docs costs payments to treasury
          FreeDocument::importCostsToTreasury(block, block_inspect_container);

          // import Ballot costs payments to treasury
          BallotDocument::importCostsToTreasury(block, block_inspect_container);

          // import Polling costs payments to treasury
          PollingDocument::importCostsToTreasury(block, block_inspect_container);

          // import request for adm polling costs payments to treasury
          AdministrativePollingDocument::importCostsToTreasury(block, block_inspect_container);

    //      // TODO: remove to
    //      // // import request for relaese reserved coins costs payments to treasury
    //      // block_inspect_container.reqRelResCostStatus = reqRelResCostsHandler.importReqRelResCost({ block, block_inspect_container });

          // import proposal costs payments to treasury
          ProposalDocument::importCostsToTreasury(block, block_inspect_container);  //importProposalsCost

          // import FleNS costs(register, binding,...) payments to treasury
          INameRegDocument::importCostsToTreasury(block, block_inspect_container);  // importRegCost
          INameBindDocument::importCostsToTreasury(block, block_inspect_container);  // importBindingCost
          // IName Msg Document::importCostsToTreasury(block, block_inspect_container);  // importRegCost


          // import pledge costs payments to treasury
          PledgeDocument::importCostsToTreasury(block, block_inspect_container);

          // import close pledge costs payments to treasury
          ClosePledgeDocument::importCostsToTreasury(block, block_inspect_container);



          // import normal UTXOs
          for (CCoin aUTXO: block_inspect_container.m_importable_UTXOs)
          {
            // remove Ceased transaction's DPCost, if they are in a same block with related P4P transaction
            // or if the transaction is in some other block which is created by backers which are not listed in dPIs list of transaction
            if (cutils::contains_(block_inspect_container.m_cut_ceased_trx_from_UTXOs, aUTXO.m_coin))
              continue;

            if (cutils::contains_(block_inspect_container.m_DPCost_coin_codes, aUTXO.m_coin))
              continue;

            // looping on all descendents of current block, to be sure all desacendent can see thei utxo in their history
            CLog::log("Final Importing Normal block Coins" + CoinImportDataContainer::dumpCoinDetails(aUTXO) + " Block(" + cutils::hash8c(block->getBlockHash()) + ") ", "trx", "trace");
            for (QVDicT aWBlock: wBlocksDescendents)
            {
              UTXOHandler::add_new_coin(
                aWBlock.value("b_creation_date").to_string(),
                aUTXO.m_coin,
                aWBlock.value("b_hash").to_string(),
                aUTXO.m_owner,  // address
                aUTXO.m_amount, // coin_value
                aUTXO.m_creation_date);  // refCreationDate:
            }
          }

        }


        // restoring UTXOs of rejected transactions
        for (CCoin aUTXO: block_inspect_container.m_to_be_restored_coins)
        {
          CLog::log("a to Be Restored coin: " + CoinImportDataContainer::dumpCoinDetails(aUTXO), "trx", "warning");
          // looping on all descendents of current block, to be sure all desacendent can see thei utxo in their history
          CLog::log("Importing Normal block Coins(restored) Block(" + cutils::hash8c(block->getBlockHash()) + ")", "trx", "info");
          for (QVDicT aWBlock: wBlocksDescendents)
          {
            UTXOHandler::add_new_coin(
              aWBlock.value("b_creation_date").to_string(),
              aUTXO.m_coin,
              aWBlock.value("b_hash").to_string(),
              aUTXO.m_owner,  // address
              aUTXO.m_amount, // coin_value
              aUTXO.m_creation_date);  // refCreationDate:
          }
        }


        // log block import report
    //    logNormalBlockUTXOsImport.logImport({
    //      blockHash: block.blockHash,
    //      block_inspect_container: _.clone(block_inspect_container)
    //    });


        // update utxo_imported
        DAG::set_coins_import_status(block->getBlockHash(), constants::YES);

        if (!CMachine::is_in_sync_process())
          CGUI::signalUpdateBlocks();

      }

      CLog::log("block_inspect_container Final Result: " + block_inspect_container->dumpMe(), "trx", "trace");
      delete block_inspect_container;

      // finally refresh coins visibilities
      UTXOHandler::refreshVisibility();
    */
}


