use std::thread;
use postgres::types::ToSql;
use crate::lib::constants;
use crate::{application, machine};
use crate::lib::custom_types::{CDateT, ClausesT, JSonObject, QVDRecordsT};
use crate::lib::dag::dag::{dag_has_blocks_which_are_created_in_current_cycle, search_in_dag};
use crate::lib::dag::normal_block::import_coins::import_normal_block_coins::import_normal_block_coins;
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
        let now_ = application().now();
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
    { c_date = application().now(); }

    import_normal_block_coins(&c_date);

//  bool OUTPUT_TIMELOCK_IS_ENABLED = false;
//  if (OUTPUT_TIMELOCK_IS_ENABLED)
//      outputTimeLockHandler.importTimeLocked();
}

//old_name_was retrieveProperBlocks
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

    let now_ = application().now();
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
        let now_ = application().now();
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


