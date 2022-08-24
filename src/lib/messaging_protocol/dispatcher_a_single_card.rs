use crate::{constants, dlog};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::JSonObject;
use crate::lib::machine::machine_neighbor::{parse_handshake, parse_nice_to_meet_you};
use crate::lib::messaging_protocol::dag_message_handler::{extract_leaves_and_push_in_sending_q, handle_received_leave_info};
use crate::lib::messaging_protocol::dispatcher::PacketParsingResult;
use crate::lib::parsing_q_handler::parsing_q_handler::push_to_parsing_q;
use crate::lib::utils::version_handler::is_valid_version_number;

//old_name_was handleSingleMessages
pub fn handle_a_single_card(
    sender: &String,
    connection_type: &String,
    creation_date: &String,
    card_body: &mut JSonObject,
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