use std::thread;
use postgres::types::ToSql;

use crate::lib::constants;
use crate::{application, cutils, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_repayback::repayback_block::create_repayment_block;
use crate::lib::block::document_types::rp_document::{calc_repayment_details, RepaymentDocument};
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CCoinCodeT, CDateT, COutputIndexT, GRecordsT};
use crate::lib::dag::dag::{search_in_dag, set_coins_import_status};
use crate::lib::dag::dag_walk_through::get_all_descendants;
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, simple_eq_clause};
use crate::lib::dlog::dlog;
use crate::lib::services::contracts::pledge::general_pledge_handler::get_pledged_accounts;
use crate::lib::transactions::basic_transactions::coins::coins_handler::add_new_coin;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;

//func old name was loopImportCoinbaseUTXOs
#[allow(unused, dead_code)]
pub fn loop_import_coinbase_coins()
{
    println!("DDDDDDDD1: {}", application().should_loop_threads());
    let thread_prefix = "import_coinbase_coins_".to_string();
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
        let now_ = application().now();
        import_minted_coins(&now_);
        /*

        // double checking repayblock importing
        RepaybackBlock::importDoubleCheck();

        if ( (constants::DATABASAE_AGENT == "sqlite") && (CMachine::shouldLoopThreads()) )
        {
        // FIXME: remove this lines, when problem of database lock for sqlite solved and we can have real multi thread solution
        do_import_coins(application().now());

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
pub fn import_minted_coins(c_date: &CDateT)
{
    dlog(&format!("import Coinbased coins {}", c_date.clone()), constants::Modules::App, constants::SecLevel::TmpDebug);

    // find coinbase block with 2 cycle age old, and insert the outputs as a matured&  spendable outputs to table trx_coins
    let max_creation_date = application().get_cb_coins_date_range(&c_date).to;
    dlog(&format!("Extract maturated coinbase coins created before({})", max_creation_date.clone()), constants::Modules::Trx, constants::SecLevel::TmpDebug);
    let coinbases = search_in_dag(
        vec![
            simple_eq_clause("b_type", &constants::block_types::COINBASE.to_string()),
            simple_eq_clause("b_coins_imported", &constants::NO.to_string()),
            ModelClause {
                m_field_name: "b_creation_date",
                m_field_single_str_value: &max_creation_date as &(dyn ToSql + Sync),
                m_clause_operand: "<=",
                m_field_multi_values: vec![],
            },
        ],
        vec!["b_hash", "b_body"],
        vec![
            &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
        ],
        0,
        false);

    let pledged_accounts_info: GRecordsT = get_pledged_accounts(
        c_date,
        true);
    dlog(
        &format!("pledged Accounts Info: {:#?}", pledged_accounts_info),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);


    for a_coinbase_record in &coinbases
    {
        // start transactional block of coinbase coins importing: FIXME: implement it ASAP
        let (status, _sf_ver, content) = unwrap_safed_content_for_db(&a_coinbase_record["b_body"]);
        if !status
        {
            let msg: String = format!(
                "Malformed recorded Coinbase unwrapping block({})!",
                a_coinbase_record["b_hash"]);
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Fatal);
            panic!("{}", msg);
        }

        let (status, block) = Block::load_block_by_serialized_content(&content); // do not need safe open check
        if !status
        {
            let msg: String = format!(
                "Malformed recorded Coinbase to block({})!",
                a_coinbase_record["b_hash"]);
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Fatal);
            panic!("{}", msg);
        }

        // since we examine Coinbase's from 2 cycle past,
        // then we must be sure the entire precedents has visibility of these coins
        let (_status, descendent_blocks, _validity_percentage) =
            get_all_descendants(&block.get_block_hash(), false);
        dlog(
            &format!("visibleBys after exclude floating signature blocks(CoinBases): {:?}", descendent_blocks),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        let mut repayment_docs: Vec<RepaymentDocument> = vec![];
        // clog.app.info(`block.docs[0].outputs ${block.docs[0].outputs}`);
        let the_only_doc = &block.m_block_documents[0];
        let outputs = &the_only_doc.m_if_coinbase_doc.m_outputs;

        let mut output_index: COutputIndexT = 0;
        while output_index < outputs.len() as COutputIndexT
        {
            let an_output: &TOutput = &outputs[output_index as usize];
            let the_coin: CCoinCodeT = cutils::pack_coin_code(&the_only_doc.get_doc_hash(), output_index);

            // * if the account is pledged, so entire account incomes must be transferres to repayback transaction and
            // * from that, cutting repayments and at the end if still remains some coins, return back to shareholder's account
            if pledged_accounts_info.contains_key(&an_output.m_address)
            {
                let a_repayback_doc: RepaymentDocument = calc_repayment_details(
                    &the_only_doc.get_doc_hash(),
                    output_index,
                    an_output.m_amount,
                    &pledged_accounts_info,
                    &an_output.m_address);

                dlog(
                    &format!("Repayment Doc: {:?}", a_repayback_doc),
                    constants::Modules::Trx,
                    constants::SecLevel::TmpDebug);

                repayment_docs.push(a_repayback_doc);
            } else {
                for a_block_record in &descendent_blocks
                {
                    dlog(
                        &format!(
                            "Importing Coinbase block Coins {}",
                            block.get_block_identifier()),
                        constants::Modules::Trx,
                        constants::SecLevel::Info);

                    add_new_coin(
                        &a_block_record["b_creation_date"],
                        &the_coin,
                        &a_block_record["b_hash"],
                        &an_output.m_address,           // address
                        an_output.m_amount,           // coin_value
                        &block.get_creation_date()); // refCreationDate:
                }
            }
            output_index += 1;
        }

        // if there is some cutting from income, create a new block(RpBlock) and record in DAG
        if repayment_docs.len() > 0
        {
            create_repayment_block(
                &block,
                &repayment_docs,
                &descendent_blocks);
        }

        // update coins are imported
        set_coins_import_status(&block.get_block_hash(), &constants::YES.to_string());

        // end of transactional block of coinbase coins importing: FIXME: implement it ASAP
    }
}

// TODO some uint tests need
//  every coinbased incomes will be spendable after 2 cycle and right after starting 3rd cycle

//old_name_was calcCoinbasedOutputMaturationDate
pub fn calc_coinbased_output_maturation_date(c_date: &CDateT) -> CDateT {
    let cycle_by_minutes = application().get_cycle_by_minutes() as u64;
    let mature_date: String = application().minutes_after(
        constants::COINBASE_MATURATION_CYCLES as u64 * cycle_by_minutes,
        c_date);
    return application().get_coinbase_range(&mature_date).from;
}