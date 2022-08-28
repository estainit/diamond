use std::collections::HashMap;
use postgres::types::ToSql;
use crate::cutils::remove_quotes;
use crate::{application, constants, cutils, dlog};
use crate::lib::block_utils::wrap_safe_content_for_db;
use crate::lib::custom_types::{JSonObject, VString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::{ModelClause, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_PARSING_Q, CDEV_PARSING_Q};
use crate::lib::messaging_protocol::dispatcher::PacketParsingResult;
use crate::lib::parsing_q_handler::parsing_q_handler::remove_from_parsing_q;

//old_name_was pushToParsingQ
pub fn push_to_parsing_q(
    card_j_obj: &mut JSonObject,
    creation_date: &String,
    card_type: &String,
    card_code: &String,
    sender: &String,
    connection_type: &String,
    prerequisites: Vec<String>) -> PacketParsingResult
{
    let mut prerequisites = prerequisites;
    dlog(
        &format!("block({}) need these prerequisitie(s) {:?}",
                 card_code, prerequisites),
        constants::Modules::App,
        constants::SecLevel::Info);

    // check for duplicate entries
    let (_status, records) = q_select(
        C_PARSING_Q,
        vec!["pq_type"],
        vec![
            simple_eq_clause("pq_type", card_type),
            simple_eq_clause("pq_code", card_code),
        ],
        vec![],
        0,
        false,
    );
    if records.len() > 0
    {
        return PacketParsingResult {
            m_status: true,
            m_should_purge_file: true,
            m_message: "".to_string(),
        };
    }

    // control if this card needs some sepcial initiative prerequisites
    if !card_j_obj["ancestors"].is_null()
    {
        if !card_j_obj["ancestors"][0].is_null()
        {
            let mut i = 0;
            while !card_j_obj["ancestors"][i].is_null() {
                prerequisites.push(remove_quotes(&card_j_obj["ancestors"][i]));
                i += 1;
            }
        }
    }

    prerequisites = cutils::array_unique(&prerequisites);
    if prerequisites.len() > 0
    {
        // check if ancestors exist in parsing q
        let empty_string = "".to_string();
        let mut c1 = ModelClause {
            m_field_name: "pq_code",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        for a_hash in &prerequisites {
            c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
        }
        let (_status, queued_ancestors) = q_select(
            C_PARSING_Q,
            vec!["pq_code"],
            vec![c1],
            vec![],
            0,
            false);

        dlog(
            &format!("block({}) missed ancs ({:?}) VS qeued ancs {:?}",
                     card_code, prerequisites, queued_ancestors),
            constants::Modules::App,
            constants::SecLevel::Info);

        let queued_ancestors = queued_ancestors
            .iter()
            .map(|x| x["pq_code"].to_string())
            .collect::<VString>();
        prerequisites = cutils::array_unique(&prerequisites);
        prerequisites = cutils::array_diff(&prerequisites, &queued_ancestors);

        // // remove if missed anc already exist in cache?
        // let cached_blocks_hashes = &get_cached_blocks_hashes();
        // prerequisites = cutils::array_diff(&prerequisites, &cached_blocks_hashes);
    }

    if prerequisites.len() > 0
    {
        // remove if missed anc already exist in DAG?
        let empty_string = "".to_string();
        let mut c1 = ModelClause {
            m_field_name: "b_hash",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        for an_anc in &prerequisites {
            c1.m_field_multi_values.push(an_anc as &(dyn ToSql + Sync));
        }
        let daged_blocks = search_in_dag(
            vec![c1],
            vec!["b_hash"],
            vec![],
            0,
            false,
        );
        if daged_blocks.len() > 0
        {
            let daged_blocks = daged_blocks
                .iter()
                .map(|r, | r["b_hash"].to_string())
                .collect::<VString>();
            prerequisites = cutils::array_unique(&prerequisites);
            prerequisites = cutils::array_diff(&prerequisites, &daged_blocks);
        }
    }

    dlog(
        &format!("Some likely missed blocks({})", prerequisites.join(",")),
        constants::Modules::App,
        constants::SecLevel::Info);


    // * if blcok is FVote, maybe we need customized treatment, since generally in DAG later blocks are depend on
    // * early blocks and it is one way graph.
    // * but in case of vote blocks, they have effect on previous blocks (e.g accepting or rejecting a transaction of previously block)
    // * so depends on voting type(bCat) for, we need proper treatment

    if !card_j_obj["block_type"].is_null()
        && remove_quotes(&card_j_obj["block_type"]) == constants::block_types::FLOATING_VOTE
    {
        /*
        if (message["bCat"].to_string() == constants::FLOAT_BLOCKS_CATEGORIES::Trx)
        {
            /**
             * if the machine get an FVote, so insert uplink block in SUS BLOCKS WHICH NEEDED VOTES TO BE IMPORTED AHAED(SusBlockWNVTBIA)
             * WNVTBIA: Wait becaue Needs Vote To Be Importable
             */
            String
            uplinkBlock = message["ancestors"].toArray()[0].to_string();    // FVote blocks always have ONLY one ancestor for which Fvote is voting
            String
            currentWNVTBIA = KVHandler::getValue("SusBlockWNVTBIA");
            StringList
            currentWNVTBIA_arr = {};
            if (currentWNVTBIA == "")
            {
                currentWNVTBIA_arr.push(uplinkBlock);
            } else {
                auto
                tmp = cutils::parseToJsonArr(currentWNVTBIA);
                for (auto x: tmp)
                currentWNVTBIA_arr.push(x.to_string());
                currentWNVTBIA_arr.push(uplinkBlock);
                currentWNVTBIA_arr = cutils::arrayUnique(currentWNVTBIA_arr);
            }
            currentWNVTBIA = cutils::serializeJson(currentWNVTBIA_arr);
            KVHandler::upsertKValue("SusBlockWNVTBIA", currentWNVTBIA);
        }
        */
    }

    // TODO: security issue to control block (specially payload), before insert to db
    // potentially attacks: sql injection, corrupted JSON object ...

    let (status, _safe_version, pq_payload) = wrap_safe_content_for_db(
        &cutils::controlled_json_stringify(&card_j_obj), constants::DEFAULT_SAFE_VERSION);
    if !status
    {
        dlog(
            &format!("Failed inside push-to-parsing-q in wrap safe the card tpey({}) code ({})", card_type, card_code),
            constants::Modules::App,
            constants::SecLevel::Error);
        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: "".to_string(),
        };
    }
    let now_ = application().get_now();
    let pq_prerequisites = format!(",{}", prerequisites.join(","));
    let zero: i32 = 0;
    let pq_v_status = "new".to_string();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("pq_type", card_type as &(dyn ToSql + Sync)),
        ("pq_code", card_code as &(dyn ToSql + Sync)),
        ("pq_sender", sender as &(dyn ToSql + Sync)),
        ("pq_connection_type", connection_type as &(dyn ToSql + Sync)),
        ("pq_receive_date", &now_ as &(dyn ToSql + Sync)),
        ("pq_payload", &pq_payload as &(dyn ToSql + Sync)),
        ("pq_prerequisites", &pq_prerequisites as &(dyn ToSql + Sync)),
        ("pq_parse_attempts", &zero as &(dyn ToSql + Sync)),
        ("pq_v_status", &pq_v_status as &(dyn ToSql + Sync)),
        ("pq_creation_date", creation_date as &(dyn ToSql + Sync)),
        ("pq_insert_date", &now_ as &(dyn ToSql + Sync)),
        ("pq_last_modified", &now_ as &(dyn ToSql + Sync)),
    ]);

    q_insert(
        C_PARSING_Q,
        &values,
        false);

    if application().is_develop_mod()
    {
        q_insert(
            CDEV_PARSING_Q,
            &values,
            false);
    }

    clean_expired_entries();

    return PacketParsingResult {
        m_status: true,
        m_should_purge_file: true,
        m_message: "".to_string(),
    };
}

pub fn clean_expired_entries()
{
    let back_in_time = application().get_cycle_by_minutes();
    let now_ = application().get_now();
    remove_from_parsing_q(vec![
        ModelClause {
            m_field_name: "pq_parse_attempts",
            m_field_single_str_value: &constants::MAX_PARSE_ATTEMPTS_COUNT,
            m_clause_operand: ">",
            m_field_multi_values: vec![],
        },
        ModelClause {
            m_field_name: "pq_creation_date",
            m_field_single_str_value: &application().minutes_before(back_in_time, &now_),
            m_clause_operand: "<",
            m_field_multi_values: vec![],
        }]);
}