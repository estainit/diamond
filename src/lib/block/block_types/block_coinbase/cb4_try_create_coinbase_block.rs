use postgres::types::ToSql;
use serde_json::json;
use crate::{application, ccrypto, cutils, machine};
use crate::lib::block::block_types::block_coinbase::cb1_maybe_create_coinbase_block::if_passed_certain_time_of_cycle_to_record_in_dag;
use crate::lib::block::block_types::block_coinbase::cb5_do_generate_coinbase_block::do_generate_coinbase_block;
use crate::lib::block::block_types::block_coinbase::cb2_random_ordering_neighbors::if_i_have_first_hashed_email;
use crate::lib::constants;
use crate::lib::custom_types::{QVDRecordsT};
use crate::lib::dag::dag::{get_most_confidence_coinbase_block_from_dag, search_in_dag};
use crate::lib::dag::missed_blocks_handler::get_missed_blocks_to_invoke;
use crate::lib::database::abs_psql::{ModelClause, simple_eq_clause};
use crate::lib::dlog::dlog;
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;
use crate::lib::utils::dumper::{dump_it, dump_vec_of_str};


//old_name_was tryCreateCoinbaseBlock
pub fn try_create_coinbase_block() -> bool
{
    let now_ = application().now();
    let (
        _coinbase_cycle_stamp,
        coinbase_from,
        coinbase_to,
        _coinbase_from_hour,
        _coinbase_to_hour) = application().get_coinbase_info(&now_, "");

    dlog(
        &format!("Try to Create Coinbase for Range ({}, {})", coinbase_from, coinbase_to),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let cycle_stamp = application().get_coinbase_cycle_stamp(&now_);
    let (status, mut block) = do_generate_coinbase_block(
        &cycle_stamp,
        constants::stages::CREATING,
        "0.0.1");

    if !status {
        dlog(
            &format!("Due to an error, can not create a coinbase block for range ({}, {})", coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Fatal);
        return false;
    }

    dlog(
        &format!("Serialized locally created cb block. before objecting1 {}", cutils::controlled_block_stringify(&block)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);


    // calculating block length
    block.m_block_length = block.calc_block_length();

    let block_hash = block.calc_block_hash();
    block.set_block_hash(&block_hash);
    dlog(
        &format!("Serialized issued coin base block. {}", cutils::controlled_block_stringify(&block)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);
    dlog(
        &format!("Serialized issued coin base block json. {}", block.safe_stringify_block(true)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let tmp_local_confidence: f64 = block.m_block_confidence as f64;

    // if local machine can create a coinbase block with more confidence or ancestors, broadcast it
    let now_ = application().now();
    let (at_least_one_coinbase_block_exist, most_confidence_in_dag) = get_most_confidence_coinbase_block_from_dag(&now_);

    let mut tmp_dag_confidence: f64 = 0.0;
    let mut tmp_dag_ancestors: Vec<String> = vec![];
    if !at_least_one_coinbase_block_exist
    {
        dlog(
            &format!("DAG hasn't coinbase for cycle range ({}, {})", coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    } else {
        dlog(
            &format!("The most_confidence_in_DAG for cycle range ({}, {}) is: {}", coinbase_from, coinbase_to, dump_it(&most_confidence_in_dag)),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);

        tmp_dag_confidence = most_confidence_in_dag["b_confidence"].parse::<f64>().unwrap();
        tmp_dag_ancestors = cutils::convert_comma_separated_to_array(&most_confidence_in_dag["b_ancestors"].to_string(), &",".to_string());
    }

    let locally_created_coinbase_block_has_more_confidence_than_dag: bool = tmp_dag_confidence < tmp_local_confidence;
    // locally_created_coinbase_block_has_more_confidence_than_dag = false;// FIXME implement remote block confidence calcuilation
    if locally_created_coinbase_block_has_more_confidence_than_dag
    {
        dlog(
            &format!("More confidence: local coinbase({}) has more confidence({}) than DAG({}) in cycle range ({}, {})",
                     cutils::hash8c(&block.m_block_hash), tmp_local_confidence.to_string(), tmp_dag_confidence.to_string(), coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    }

    let mut ancestors_diff: Vec<String> = cutils::array_diff(
        &block.m_block_ancestors,
        &tmp_dag_ancestors);
    if ancestors_diff.len() > 0
    {
        // try to remove repayBack blocks
        let empty_string = "".to_string();
        let mut c1 = ModelClause {
            m_field_name: "b_hash",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        for an_anc in &ancestors_diff {
            c1.m_field_multi_values.push(an_anc as &(dyn ToSql + Sync));
        }
        let existed_repay_blocks: QVDRecordsT = search_in_dag(
            vec![
                simple_eq_clause("b_type", &constants::block_types::REPAYMENT_BLOCK.to_string()),
                c1,
            ],
            vec!["b_hash"],
            vec![],
            0,
            true);
        if existed_repay_blocks.len() > 0
        {
            let mut tmp: Vec<String> = vec![];
            for record in existed_repay_blocks
            {
                tmp.push(record["b_hash"].to_string());
            }
            ancestors_diff = cutils::array_diff(&ancestors_diff, &tmp);
        }
    }
    let locally_created_coinbase_block_has_more_ancestors_than_dag: bool = ancestors_diff.len() > 0;
    if locally_created_coinbase_block_has_more_ancestors_than_dag
    {
        dlog(
            &format!("More ancestors: local coinbase({}) has more ancestors({:?} than DAG({}) in cycle range ({}, {})",
                     cutils::hash8c(&block.m_block_hash.to_string()), block.m_block_ancestors, dump_vec_of_str(&tmp_dag_ancestors), coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    }

    dlog(
        &format!("Is about to issuing coinbase block in cycle range ({}, {}) the block: {}",
                 coinbase_from, coinbase_to, cutils::controlled_block_stringify(&block)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let missed_blocks: Vec<String> = get_missed_blocks_to_invoke(0);
    if missed_blocks.len() > 0
    {
        dlog(
            &format!("BTW machine has some missed blocks: {}", dump_it(&missed_blocks)),
            constants::Modules::CB,
            constants::SecLevel::Warning);
    }

    // FIXME: it is a way to avoid creating too many coinbases which have a little difference because of the ancestors.
    // could it be a security issue? when an adversory in last minutes(before midnight or mid-day) starts to spam network by blocks
    // and most of nodes can not be synched, so too many coinbase blocks creating
    //
    if (locally_created_coinbase_block_has_more_confidence_than_dag
        || locally_created_coinbase_block_has_more_ancestors_than_dag)
        && (get_missed_blocks_to_invoke(0).len() < 1)
    {

        // broadcast coin base
        if application().is_in_current_cycle(&block.m_block_creation_date.to_string())
        {
            dlog(
                &format!("pushing coinbase block to network (case A): {}", block.m_block_hash),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            let mut block_body = block.safe_stringify_block(true);

            dlog(
                &format!("The coinbase block safe stringifyed block, pushing to network (case A): {}", block_body),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            // serde_json::to_string(&block).unwrap();
            block_body = ccrypto::b64_encode(&block_body);
            // let _ancestors: Vec<String> = block.m_block_ancestors.clone();

            let (_code, body) = make_a_packet(
                vec![
                    json!({
                "cdType": constants::block_types::COINBASE,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "bHash": block.m_block_hash.clone(),
                // "ancestors": ancestors,
                "block": block_body,
            }),
                ],
                constants::DEFAULT_PACKET_TYPE,
                constants::DEFAULT_PACKET_VERSION,
                application().now(),
            );
            dlog(
                &format!(
                    "prepared coinbase packet, before insert into DB code({}) {}",
                    block.m_block_hash,
                    body),
                constants::Modules::App,
                constants::SecLevel::Info);

            let status = push_into_sending_q(
                constants::block_types::COINBASE,
                block.m_block_hash.as_str(),
                &body,
                &format!(
                    "{} {} case A issued by {}",
                    block.m_block_type,
                    cutils::hash8c(&block.m_block_hash),
                    application().machine_id()
                ),
                &vec![],
                &vec![],
                false,
            );

            dlog(
                &format!("coinbase push1 res({})", status),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            dlog(
                &format!("Coinbase issued because of clause 1 CB({}) issued by({} for cycle range({}, {})",
                         cutils::hash8c(&block.m_block_hash.to_string()), machine().get_pub_email_info().m_address, coinbase_from, coinbase_to),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            return status;
        }
    } else if if_passed_certain_time_of_cycle_to_record_in_dag(&now_)
        && !at_least_one_coinbase_block_exist
    {
        // another psudo random emulatore
        // if already passed more than 1/4 of cycle and still no coinbase block recorded in DAG,
        // so the machine has to create one
        if if_i_have_first_hashed_email("desc")
        {
            dlog(
                &format!("pushing coinbase block to network (case B): {}", block.m_block_hash),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            let mut block_body = block.safe_stringify_block(true);

            dlog(
                &format!("The coinbase block safe stringifyed block, pushing to network (case B): {}", block_body),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            // serde_json::to_string(&block).unwrap();
            block_body = ccrypto::b64_encode(&block_body);
            let _ancestors: Vec<String> = block.m_block_ancestors.clone();

            let (_code, body) = make_a_packet(
                vec![
                    json!({
                "cdType": constants::block_types::COINBASE,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "bHash": block.m_block_hash.clone(),
                "block": block_body,
            }),
                ],
                constants::DEFAULT_PACKET_TYPE,
                constants::DEFAULT_PACKET_VERSION,
                application().now(),
            );

            let status = push_into_sending_q(
                constants::block_types::COINBASE,
                block.m_block_hash.as_str(),
                &body,
                &format!(
                    "{} {} case B issued by {}",
                    block.m_block_type,
                    cutils::hash8c(&block.m_block_hash),
                    application().machine_id()
                ),
                &vec![],
                &vec![],
                false,
            );

            dlog(
                &format!("coinbase push (case B) res({})", status),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);
            dlog(
                &format!("Coinbase issued because of clause (case B) CB({}) issued by({} for cycle range({}, {})",
                         cutils::hash8c(&block.m_block_hash), &machine().get_pub_email_info().m_address, &coinbase_from, &coinbase_to),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);
            return true;
        }
    } else {
        dlog(
            &format!("Coinbase can be issued by clause (case C) but local hasn't neither more confidence nor more ancestors and still not riched to 1/4 of cycle time. CB({}) issued by({} for cycle range({}, {})",
                     cutils::hash8c(&block.m_block_hash),
                     &machine().get_pub_email_info().m_address,
                     coinbase_from,
                     coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        return true;
    }
    true
}