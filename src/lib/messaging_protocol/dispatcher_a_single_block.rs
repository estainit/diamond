use crate::{application, ccrypto, constants, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::custom_types::JSonObject;
use crate::lib::dag::dag::search_in_dag;
use crate::lib::dag::missed_blocks_handler::remove_from_missed_blocks;
use crate::lib::database::abs_psql::simple_eq_clause;
use crate::lib::messaging_protocol::dag_message_handler::{set_last_received_block_timestamp};
use crate::lib::messaging_protocol::dispatcher::PacketParsingResult;
use crate::lib::parsing_q_handler::queue_push::push_to_parsing_q;

//old_name_was innerDispatchMessage
pub fn handle_a_single_block(
    sender: &String,
    connection_type: &String,
    card_body: &mut JSonObject,
    card_type: &String) -> PacketParsingResult
{
    dlog(
        &format!("--- Handling the block({}) from({}) ", card_type, sender),
        constants::Modules::App,
        constants::SecLevel::Info);

    // FIXME: security issue. what happend if adversary creates million of blocks in minute and send the final descendente?
    // in this case all nodes have to download entire blocks all the way back to find ancestor
    // and start to validate from the oldest one and add it to DAG(if is VALID)
    // in this process nodes can not control if the blocks in between are valid or not?
    // so the bandwidth&  machine harddisk will be vasted
    // and network will be blocked!
    // here we need implement a system to control creation date of eache received block(profiled for each neighbor or backer address)
    // and limit creating block(e.g 10 bloocks per minute) in proportion to neighbor's reputation.


    dlog(
        &format!("--- considering a Block({}) from({}) ", card_type, sender),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    // the message is a whole block, so push it to table c_parsing_q
    let mut err_msg = "".to_string();
    let block_hash = remove_quotes(&card_body["bHash"]);

    let already_recorded_in_dag = search_in_dag(
        vec![simple_eq_clause("b_hash", &block_hash)],
        vec!["b_hash"],
        vec![],
        0,
        false);

    if already_recorded_in_dag.len() > 0
    {
        err_msg = format!("Duplicated card received block({}) type({}) from({})! ", block_hash, card_type, sender);
        dlog(
            &err_msg,
            constants::Modules::App,
            constants::SecLevel::Info);


        return PacketParsingResult {
            m_status: true,
            m_should_purge_file: true,
            m_message: err_msg,
        };
    }

    let block_body = remove_quotes(&card_body["block"]);
    let (status, block_body) = ccrypto::b64_decode(&block_body);
    if !status
    {
        err_msg = format!("Failed in block b64 decoding block({}) type({}) from({})", block_hash, card_type, sender);
        dlog(
            &err_msg,
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: err_msg,
        };
    }

    let mut is_valid_js = false;
    let block: Block = match serde_json::from_str(&block_body) {
        Ok(b) =>
            {
                is_valid_js = true;
                b
            }
        Err(e) =>
            {
                err_msg = format!("Failed in deserialized block ({}) type({}) from({}) {}", block_hash, card_type, sender, e);
                dlog(
                    &err_msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                Block::new()
            }
    };
    if !is_valid_js
    {
        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: err_msg,
        };
    }
    println!("deserialized block: {:?}", block);

    dlog(
        &format!("--- Pushing block({}) type({}) from({}) to 'c_parsing_q'",
                 block_hash, card_type, sender),
        constants::Modules::App,
        constants::SecLevel::Info);

    card_body["block_type"] = block.m_block_type.into();
    let pa_pa_res = push_to_parsing_q(
        card_body,
        &block.m_block_creation_date,
        &card_type.to_string(),
        &block_hash,
        sender,
        connection_type,
        block.m_block_ancestors);

    // if it is a valid block, update last received block info
    if pa_pa_res.m_status
    {
        let now_ = application().get_now();
        set_last_received_block_timestamp(
            card_type,
            &block.m_block_hash,
            &now_);
    }

    // remove from missed blocks (if exist)
    remove_from_missed_blocks(&block.m_block_hash);


    return PacketParsingResult {
        m_status: true,
        m_should_purge_file: pa_pa_res.m_should_purge_file,
        m_message: err_msg,
    };
}
