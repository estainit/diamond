use crate::{application, ccrypto, constants, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::custom_types::JSonObject;
use crate::lib::dag::dag::search_in_dag;
use crate::lib::dag::missed_blocks_handler::remove_from_missed_blocks;
use crate::lib::database::abs_psql::simple_eq_clause;
use crate::lib::machine::machine_neighbor::{parse_handshake, parse_nice_to_meet_you};
use crate::lib::messaging_protocol::dag_message_handler::{extract_leaves_and_push_in_sending_q, handle_received_leave_info, set_last_received_block_timestamp};
use crate::lib::messaging_protocol::dispatcher::PacketParsingResult;
use crate::lib::parsing_q_handler::parsing_q_handler::push_to_parsing_q;
use crate::lib::utils::version_handler::is_valid_version_number;

//old_name_was innerDispatchMessage
pub fn dispatch_a_card(
    sender: &String,
    connection_type: &String,
    c_date: &String,
    card_body: &JSonObject,
    card_type: &String,
    card_ver: &String,
    packet_ver: &str) -> PacketParsingResult
{
    dlog(
        &format!("--- dispatching card({}) from({}) ", card_type, sender),
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

    let block_types: Vec<String> = vec![
        constants::block_types::NORMAL.to_string(),
        constants::block_types::COINBASE.to_string(),
        constants::block_types::FLOATING_SIGNATURE.to_string(),
        constants::block_types::FLOATING_VOTE.to_string(),
        constants::block_types::POW.to_string(),
        constants::block_types::REPAYMENT_BLOCK.to_string()];

    let card_types: Vec<String> = vec![
        constants::card_types::DAG_INVOKE_BLOCK.to_string(),
        constants::card_types::DAG_INVOKE_DESCENDENTS.to_string(),
        constants::card_types::DAG_INVOKE_LEAVES.to_string(),
        constants::card_types::DAG_LEAVES_INFO.to_string(),
        constants::card_types::HANDSHAKE.to_string(),
        constants::card_types::NICE_TO_MEET_YOU.to_string(),
        constants::card_types::HERE_IS_NEW_NEIGHBOR.to_string(),
        constants::card_types::PROPOSAL_LOAN_REQUEST.to_string(),
        constants::card_types::FULL_DAG_DOWNLOAD_REQUEST.to_string(),
        constants::card_types::PLEASE_REMOVE_ME_FROM_YOUR_NEIGHBORS.to_string(),
        constants::card_types::FULL_DAG_DOWNLOAD_RESPONSE.to_string(),
        constants::card_types::BALLOTS_RECEIVE_DATES.to_string(),
        constants::card_types::NODE_STATUS_SNAPSHOT.to_string(),
        constants::card_types::NODE_STATUS_SCREENSHOT.to_string(),
        constants::card_types::DIRECT_MESSAGE_TO_NEIGHBOR.to_string(),
    ];


    let _gql_types: Vec<&str> = vec![];


    if block_types.contains(&card_type)
    {

        // the essage is a whole block, so push it to table c_parsing_q
        let block_hash = remove_quotes(&card_body["bHash"]);

        let already_recorded_in_dag = search_in_dag(
            vec![simple_eq_clause("b_hash", &block_hash)],
            vec!["b_hash"],
            vec![],
            0,
            false);

        if already_recorded_in_dag.len() > 0
        {
            dlog(
                &format!("Duplicated card received block({}) type({}) from({})! ", block_hash, card_type, sender),
                constants::Modules::App,
                constants::SecLevel::Info);


            return PacketParsingResult {
                m_status: true,
                m_should_purge_file: true,
                m_message: "".to_string(),
            };
        }

        let block_body = remove_quotes(&card_body["block"]);
        let (status, block_body) = ccrypto::b64_decode(&block_body);
        if !status
        {
            dlog(
                &format!("Failed in block b64 decoding block({}) type({}) from({})", block_hash, card_type, sender),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return PacketParsingResult {
                m_status: false,
                m_should_purge_file: true,
                m_message: "".to_string(),
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
                    dlog(
                        &format!("Failed in deserialized block ({}) type({}) from({}) {}", block_hash, card_type, sender, e),
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
                m_message: "".to_string(),
            };
        }
        println!("deserialized block: {:?}", block);

        dlog(
            &format!("--- Pushing block({}) type({}) from({}) to 'c_parsing_q'",
                     block_hash, card_type, sender),
            constants::Modules::App,
            constants::SecLevel::Info);
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
            m_message: "".to_string(),
        };
    } else if card_types.contains(card_type)
    {
        let pa_pa_res = handle_a_single_card(
            sender,
            connection_type,
            c_date,
            card_body,
            card_type,
            card_ver,
            packet_ver);

        dlog(
            &format!("Handle a single card response status({}) should purge file({})",
                     pa_pa_res.m_status, pa_pa_res.m_should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);

        return pa_pa_res;
    }

    /*
    else if (gql_types.contains(type))
    {
    return handleGQLMessages(
      sender,
      connection_type,
      creation_date,
      message,
      type,
      ver);

    }
    else if (type == constants::block_types::Genesis)
    {
    return {true, true};
    }
    else
    {
    String card_code = message.keys().contains("bHash") ? message["bHash"].to_string() : "";
    CLog::log("Unknown Message type(" + type + ") was received from (" + sender + ") HD in inbox (" + card_code + ")", "sec", "error");
    return {true, true};
    }
    */
    let msg = format!(
        "Reached end of func! sender: {}, connection type: {}, c_date: {}, card type: {}, card ver:{}, packet ver: {}, card body: {:?}",
        sender, connection_type, c_date, card_type, card_ver, packet_ver, card_body);
    return PacketParsingResult {
        m_status: false,
        m_should_purge_file: false,
        m_message: msg,
    };
}

//old_name_was handleSingleMessages
pub fn handle_a_single_card(
    sender: &String,
    connection_type: &String,
    creation_date: &String,
    card_body: &JSonObject,
    card_type: &str,
    card_ver: &str,
    packet_ver: &str) -> PacketParsingResult
{
    let mut card_code: String = format!("{}-{}-{}", packet_ver, card_type, card_ver);

    if !card_body["bHash"].is_null()
    {
        card_code = remove_quotes(&card_body["bHash"]).to_string();
    }

    if !is_valid_version_number(card_ver)
    {
        dlog(
            &format!("invalid card version for in dispatcher! card type({}) card version({})", card_type, card_ver),
            constants::Modules::Sec,
            constants::SecLevel::Error);

        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: "".to_string(),
        };
    }

    // DAG comunications
    if card_type == constants::card_types::DAG_INVOKE_BLOCK
    {
        dlog(
            &format!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@ constants::card_types::DAG_INVOKE_BLOCK @@@@@@@@@@@@@@@@@@@@@@@@@@@@@"),
            constants::Modules::App,
            constants::SecLevel::Info);

        return push_to_parsing_q(
            card_body,
            creation_date,
            &card_type.to_string(),
            &card_code,
            sender,
            connection_type,
            vec![]);

// }else if (card_type == constants::card_types::DAG_INVOKE_DESCENDENTS)
// {
//
// CLog::log("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ constants::card_types::DAG_INVOKE_DESCENDENTS @@@@@@@@@@@@@@@@@@@@@@@@@@@@", "app", "trace");
// pa_pa_res = ParsingQHandler::push_to_parsing_q(
//   message,
//   creation_date,
//   type,
//   card_code,
//   sender,
//   connection_type);
// return {push_status, should_purge_file};
//
    } else if card_type == constants::card_types::DAG_INVOKE_LEAVES
    {
        dlog(
            &format!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ constants::card_types::DAG_INVOKE_LEAVES sender: {sender} @@@@@@@@@@@@@@@@@@@@@@@@@@@"),
            constants::Modules::App,
            constants::SecLevel::Info);

        return extract_leaves_and_push_in_sending_q(sender);
    } else if card_type == constants::card_types::DAG_LEAVES_INFO
    {
        dlog(
            &format!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ constants::card_types::DAG_LEAVES_INFO sender: {sender} @@@@@@@@@@@@@@@@@@@@@@@@@@@"),
            constants::Modules::App,
            constants::SecLevel::Info);

        return handle_received_leave_info(
            sender,
            card_body,
            connection_type);
    } else if card_type == constants::card_types::HANDSHAKE
    {
        // handshake
        // TODO: implement a switch to set off/on for no more new neighbor
        dlog(
            &format!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ constants::card_types::HANDSHAKE sender: {sender} @@@@@@@@@@@@@@@@@@@@@@@@@@@"),
            constants::Modules::App,
            constants::SecLevel::Info);

        let pa_pa_res = parse_handshake(
            sender,
            card_body,
            connection_type);
        dlog(
            &format!("greeting Parsers parse Handshake res: parse_status ({}) should_purge_file({})", pa_pa_res.m_status, pa_pa_res.m_should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);
        return pa_pa_res;
    } else if card_type == constants::card_types::NICE_TO_MEET_YOU
    {
        let pa_pa_res = parse_nice_to_meet_you(
            sender,
            card_body,
            connection_type);
        // invokeDescendents_(); // FIXME: do it in Async mode
        dlog(
            &format!("greeting Parsers parse nice to meet you res: parse_status ({}) should_purge_file({})", pa_pa_res.m_status, pa_pa_res.m_should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);
        return pa_pa_res;
    } else if card_type == constants::card_types::HERE_IS_NEW_NEIGHBOR
    {
        // TODO: activate it after add some security and privacy care issues
        // parseHereIsNewNeighbor(
        //     sender,
        //     message,
        //     connection_type
        // );
    } else {
        dlog(
            &format!("Undefined card type in single card dispatching: {}", card_type),
            constants::Modules::App,
            constants::SecLevel::Error);
        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: "".to_string(),
        };
    }


    return PacketParsingResult {
        m_status: false,
        m_should_purge_file: false,
        m_message: "".to_string(),
    };
}