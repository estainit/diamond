use crate::{application, constants, cutils, dlog};
use crate::lib::block::block_types::block_factory::load_block;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::QVDicT;
use crate::lib::messaging_protocol::dag_message_handler::handle_block_invoke_request;
use crate::lib::parsing_q_handler::parsing_q_handler::parse_pure_block;

pub struct EntryParsingResult
{
    pub m_status: bool,
    pub m_should_purge_record: bool,
    pub m_message: String,
}


//old_name_was handlePulledPacket
pub fn handle_pulled_packet(pulled_record: &QVDicT) -> EntryParsingResult
{
    let err_msg;

    dlog(
        &format!("handle Pulled Packet: {:?}", pulled_record),
        constants::Modules::App,
        constants::SecLevel::Debug);

    let mut receive_date: String = application().get_now();
    if pulled_record["pq_receive_date"] != ""
    {
        receive_date = pulled_record["pq_receive_date"].clone();
    }
    let pq_type: String = pulled_record["pq_type"].to_string();
    let pq_code: String = pulled_record["pq_code"].to_string();
    let pq_sender: String = pulled_record["pq_sender"].to_string();
    let connection_type: String = pulled_record["pq_connection_type"].to_string();
    // * payload could be a block, GQL or even old-style messages
    // * TODO: optimizine to use heap allocation for bigger payloads

    if pq_type == ""
    {
        err_msg = format!("missed pq_type in handle pulled packet: {:?}", pulled_record);
        dlog(
            &err_msg,
            constants::Modules::App,
            constants::SecLevel::Error);
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: err_msg,
        };
    }

    if connection_type == ""
    {
        dlog(
            &format!("missed connection type in handle pulled packet: {:?}", pulled_record),
            constants::Modules::App,
            constants::SecLevel::Error);

        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: format!("missed connection type in handle pulled packet, {:?}", pulled_record),
        };
    }

    let (status, _sf_version, unwrapped_res) =
        unwrap_safed_content_for_db(&pulled_record["pq_payload"].to_string());
    if !status
    {
        // purge record
        // reputation report
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: format!("Failed in pq safe deser, {}", unwrapped_res),
        };
    }

    let (status, mut json_payload) = cutils::controlled_str_to_json(&unwrapped_res);
    if !status
    {
        // purge record
        // reputation report
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: format!("Failed in pq json deser, {}", unwrapped_res),
        };
    }

    if pq_sender == ""
    {
        dlog(
            &format!("missed sender or payload to parse: {:?}", pulled_record),
            constants::Modules::App,
            constants::SecLevel::Error);

        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: true,
            m_message: format!("Failed in pq json deser, {}", unwrapped_res),
        };
    }

    let mut block_type = "".to_string();
    if !json_payload["bType"].is_null()
    {
        block_type = json_payload["bType"].to_string();
    }

    if block_type == constants::block_types::REPAYMENT_BLOCK
    {
        dlog(
            &format!(
                "A repay Block received block({})!",
                cutils::hash8c(&json_payload["bHash"].to_string())),
            constants::Modules::Trx,
            constants::SecLevel::Warning);
        // Since machine must create the repayments by itself we drop this block immidiately,
        // in addition machine calls importCoinbasedUTXOs method to import potentially minted coins
        // and cut the potentially repay backs in on shot
        return EntryParsingResult {
            m_status: true,
            m_should_purge_record: true,
            m_message: format!("Normally we do not propagate repayment block!"),
        };
    }

    if [constants::block_types::NORMAL,
        constants::block_types::COINBASE,
        constants::block_types::FLOATING_SIGNATURE,
        constants::block_types::SUS_BLOCK,
        constants::block_types::FLOATING_VOTE,
        constants::block_types::POW].contains(&pq_type.as_str())
    {
        json_payload["local_receive_date"] = receive_date.clone().into();
        let (status, block) = load_block(&json_payload);

        if !status
        {
            err_msg = format!(
                "Failed in 'load block' block({})",
                cutils::hash8c(&json_payload["bHash"].to_string()));
            dlog(
                &err_msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: err_msg,
            };
        }

        if !block.object_assignments_controls()
        {
            err_msg = format!(
                "Malformed JSon block couldn't be parsed! block({})",
                cutils::hash8c(&json_payload["bHash"].to_string()));
            dlog(
                &err_msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: err_msg,
            };
        }

        return parse_pure_block(
            &pq_sender,
            &pq_type,
            &block,
            &connection_type,
            &receive_date,
        );
    }

    dlog(
        &format!("\n\n--- parsing CPacket type({}) Block/Message \nfrom Q.sender({}) ", pq_type, pq_sender),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    if pq_type == constants::card_types::DAG_INVOKE_BLOCK
    {
        //communications
        return handle_block_invoke_request(
            &pq_sender,
            &json_payload,
            &connection_type);
    }

    /*
    // GQL part
    if (pq_type == constants::CARD_TYPES::ProposalLoanRequest)
    {
        auto
        [status, should_purge_record] = GeneralPledgeHandler::handleReceivedProposalLoanRequest(
            pq_sender,
            payload,
            connection_type,
            receive_date);
        if (status)
        CGUI::signalUpdateReceivedLoanRequests();
        return { status, should_purge_record };
    } else if (pq_type == constants::CARD_TYPES::FullDAGDownloadRequest)
    {
        auto
        [status, should_purge_record] = FullDAGHandler::prepareFullDAGDlResponse(
            pq_sender,
            payload,
            connection_type);
        return { status, should_purge_record };
    } else if (pq_type == constants::CARD_TYPES::pleaseRemoveMeFromYourNeighbors)
    {
//    case GQLHandler.cardTypes.pleaseRemoveMeFromYourNeighbors:
//        res = require('../../machine/machine-handler').neighborHandler.doDeleteNeighbor({
//            sender,
//            payload,
//            connection_type,
//            receive_date
//        });
//        break;
*/
    /*
    } else else if (pq_type == constants::card_types::DAG_INVOKE_DESCENDENTS)
    {
//    case message_types.DAG_INVOKE_DESCENDENTS:
//        res = dagMsgHandler.handleDescendentsInvokeReq({
//            sender,
//            payload,
//            connection_type: connection_type
//        })
//        break;
    }
*/
    err_msg = format!(
        "Unknown record in parsing Q! {} {} from {}",
        pq_type, pq_code, pq_sender);
    dlog(
        &err_msg,
        constants::Modules::Sec,
        constants::SecLevel::Error);
    return EntryParsingResult {
        m_status: true,
        m_should_purge_record: true,
        m_message: err_msg,
    };
}