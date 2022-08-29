use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block_utils::if_ancestors_are_valid;
use crate::lib::custom_types::{VString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::dag::missed_blocks_handler::add_missed_blocks_to_invoke;
use crate::lib::database::abs_psql::{ModelClause, simple_eq_clause};
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::parsing_q_handler::queue_utils::{append_prerequisites, search_parsing_q};

//old_name_was ancestorsConroll
pub fn ancestors_controls(pq_type: &String, block: &Block) -> EntryParsingResult
{
    let error_message: String;
    let block_identifier = block.get_block_identifier();

    if block.m_block_ancestors.len() == 0
    {
        error_message = format!("The {} {} MUST have ancestor!"
                                , pq_type, block_identifier);
        dlog(
            &error_message,
            constants::Modules::Sec,
            constants::SecLevel::Error);

        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: error_message,
        };
    }

    if !if_ancestors_are_valid(&block.m_block_ancestors)
    {
        error_message = format!(
            "invalid ancestosr for ({}) ancestors: {:?}"
            , block_identifier, block.m_block_ancestors);
        dlog(
            &error_message,
            constants::Modules::Sec,
            constants::SecLevel::Fatal);

        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: error_message,
        };
    }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &block.m_block_ancestors {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let existed_record_blocks = search_in_dag(
        vec![c1],
        vec!["b_hash", "b_creation_date", "b_type", "b_coins_imported"],
        vec![],
        0,
        false,
    );

    let mut existed_hashes: VString = vec![];
    for a_block_record in &existed_record_blocks
    {
        existed_hashes.push(a_block_record["b_hash"].to_string());
    }

    let missed_blocks = cutils::array_diff(
        &block.m_block_ancestors,
        &existed_hashes);
    if missed_blocks.len() > 0
    {
        dlog(
            &format!("In order to parsing {} machine needs these missed blocks({:?})",
                     block_identifier, missed_blocks),
            constants::Modules::App,
            constants::SecLevel::Info);

        append_prerequisites(
            &block.get_block_hash(),
            &missed_blocks, pq_type);

        // check if the block already is in parsing queue? if not add it to missed blocks to invoke
        let mut missed_hashes_in_parsing_queue: VString = vec![];
        for a_hash in &missed_blocks
        {
            let exists = search_parsing_q(
                vec![simple_eq_clause("pq_code", a_hash)],
                vec!["pq_code"],
                vec![],
                0,
            );

            if exists.len() == 0
            {
                missed_hashes_in_parsing_queue.push(a_hash.clone());
            }
        }
        if missed_hashes_in_parsing_queue.len() > 0
        {
            dlog(
                &format!("{}, Really missed Blocks, so push to invoking: {:?}",
                         block_identifier, missed_hashes_in_parsing_queue),
                constants::Modules::App,
                constants::SecLevel::Info);

            add_missed_blocks_to_invoke(&missed_hashes_in_parsing_queue);
        }
        error_message = format!(
            "--- Break parsing {} because of missed prerequisites blocks ({:?})",
            block_identifier, missed_blocks);
        dlog(
            &error_message,
            constants::Modules::App,
            constants::SecLevel::Error);

        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: false,
            m_message: error_message,
        };
        // must not purge the block until receiving prerquisities blocks
    }


    let mut all_ancestors_are_imported = true;
    let mut not_imported_ancs: VString = vec![];
    let mut oldest_ancestor_creation_date = application().now();
    println!("@@@@@@ existed_record_blocks {:?}", existed_record_blocks);
    for a_blk in &existed_record_blocks
    {
        // controll ancestors creation date
        if a_blk["b_creation_date"].to_string() > block.m_block_creation_date
        {
            error_message = format!(
                "{} pq_type({}) creation date({}) is before it's ancestors({}) The creation Date({})",
                block_identifier, pq_type,
                block.m_block_creation_date,
                cutils::hash6c(&a_blk["b_hash"].to_string()),
                a_blk["creation_date"].to_string());
            dlog(
                &error_message,
                constants::Modules::App,
                constants::SecLevel::Error);

            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: error_message,
            };
        }

        // control import new coins
        if [
            constants::block_types::NORMAL,
            constants::block_types::COINBASE,
            constants::block_types::REPAYMENT_BLOCK
        ].contains(&a_blk["b_type"].as_str()) &&
            (a_blk["b_coins_imported"].to_string() != constants::YES)
        {
            all_ancestors_are_imported = false;
            not_imported_ancs.push(a_blk["b_hash"].to_string());
            if oldest_ancestor_creation_date > a_blk["b_creation_date"].to_string()
            {
                oldest_ancestor_creation_date = a_blk["b_creation_date"].to_string();
            }
        }
    }

    // if is in sync mode, control if ancestors's coins(if exist) are imported
    if machine().is_in_sync_process(false)
        && [constants::block_types::NORMAL].contains(&block.m_block_type.as_str())     // in order to let adding FVote blocks to DAG, before importing uplinked Normal block
        && !all_ancestors_are_imported
    {
        let now_ = application().now();
        let block_age = application().time_diff(block.m_block_creation_date.clone(), now_.clone()).as_minutes;
        if block_age < application().get_cycle_by_minutes() / 6
        {
            // if block is enough new, probably machine is not in sync mode more
            machine().set_is_in_sync_process(true, &now_);
        }

        // run this controll if the block creation date is not in current sycle
        // infact by passing lastSyncStatus when machine reached to almost leaves in real time
        dlog(
            &format!(
                "--- Break parsing {}, because of not imported coins of ancestors > Ancestors: {:?}",
                block_identifier, not_imported_ancs),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        //    // manually calling import threads to import ancestors coins (if they are eligible)
        //    do_import_coins(block.m_block_creation_date);
        //    import_minted_coins(oldestAncestorCreationDate);

        return EntryParsingResult {
            m_status: true,
            m_should_purge_record: false,
            m_message: format!("Why shouldn't delete this block record in parsing q table? {}", block_identifier),
        };
    }

    return EntryParsingResult {
        m_status: true,
        m_should_purge_record: true,
        m_message: "The ancestor controls done".to_string(),
    };
}