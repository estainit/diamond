use postgres::types::ToSql;
use serde_json::json;
use crate::{application, ccrypto, constants, dlog, get_value, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_normal::normal_block_handler::create_a_normal_block;
use crate::lib::custom_types::{CDateT, VString};
use crate::lib::database::abs_psql::{ModelClause, simple_eq_clause};
use crate::lib::machine::machine_buffer::block_buffer::remove_from_buffer;
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;

// old name was broadcastBlock
pub fn broadcast_block(
    cost_pay_mode: &String,
    create_date_type: &String) -> (bool, String)
{
    let msg: String = "".to_string();
    let mut block: Block = Block::new();
    let mut should_reset_block_buffer: bool = false;
    let mut cheating_creation_date: CDateT = "".to_string();
    let mut cheating_ancestors: VString = vec![];
    if create_date_type == "cheat"
    {
        cheating_creation_date = get_value("cheating_creation_date");
        // let tt = get_value("cheating_ancestors").split(",")
        cheating_ancestors = get_value("cheating_ancestors")
            .split(",")
            .collect::<Vec<&str>>()
            .iter()
            .map(|&x| x.to_string())
            .collect::<VString>()
    }


    if cost_pay_mode == "byPoW"
    {
        // TODO: implement it (if we really need POW payment block types)
        //res = await POWblockHandler.createAPOWBlock({
        //  creationDate: cheating_creation_date,
        //  ancestors: (!utils._nilEmptyFalse(cheating_ancestors)) ? utils.parse(cheating_ancestors) : null
        //});
    } else {
        let (status, block_, should_reset_block_buffer_, msg) = create_a_normal_block(
            &cheating_ancestors,
            &cheating_creation_date,
            cheating_ancestors.len() > 0);
        should_reset_block_buffer = should_reset_block_buffer_;
        if !status
        {
            dlog(
                &format!("Failed in generating normal block! {}", msg),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, format!("Failed in generating normal block! {}", msg));
        }
        block = block_;
    }


    // write file on hard output/send email
    let mut block_body = block.safe_stringify_block(true);
    dlog(
        &format!("About to sending a normal block to network block: {} {}", block.get_block_identifier(), block_body),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    // let push_res = push_into_sending_q(
    //   ,
    //   block.getBlockHash(),
    //   block_body,
    //   "Broadcasting the created normal block(" + cutils::hash8c(block.getBlockHash()) + ") " + cutils::getNow());
    block_body = ccrypto::b64_encode(&block_body);
    let (_code, body) = make_a_packet(
        vec![
            json!({
                "cdType": block.get_block_type(),
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
            "prepared Normal block packet, before insert into DB {}: {}",
            block.get_block_identifier(),
            body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let status = push_into_sending_q(
        &block.get_block_type(),
        &block.get_block_hash(),
        &body,
        &format!(
            "Broadcasting the created normal block {} {}",
            block.get_block_identifier(), application().now()
        ),
        &vec![],
        &vec![],
        false,
    );

    dlog(
        &format!(
            "Normal block generated & pushed to sending Q. push res({}) block {} {}",
            status,
            block.get_block_identifier(),
            application().now()
        ),
        constants::Modules::App,
        constants::SecLevel::Info);

    // remove from buffer
    if should_reset_block_buffer
    {
        let empty_string = "".to_string();
        let mut c1 = ModelClause {
            m_field_name: "bd_doc_hash",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        let hashes= block.get_documents_hashes();
        for a_hash in &hashes
        {
            c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
        }
        remove_from_buffer(
            vec![
                simple_eq_clause("bd_mp_code", &machine().get_selected_m_profile()),
                c1,
            ]);
    }
    drop(&block);

    return (true, msg);
}

