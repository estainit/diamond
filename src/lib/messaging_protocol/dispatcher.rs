use serde_json::{json};
use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::{CDateT, JSonObject};
use crate::lib::machine::machine_neighbor::{parse_handshake, parse_nice_to_meet_you};
use crate::lib::messaging_protocol::dag_message_handler::{extract_leaves_and_push_in_sending_q, handle_received_leave_info};
use crate::lib::parsing_q_handler::parsing_q_handler::push_to_parsing_q;
use crate::lib::utils::version_handler::is_valid_version_number;

/*

void invokeDescendents_()
{
  std::this_thread::sleep_for (std::chrono::seconds(50));
  DAGMessageHandler::invokeDescendents();
}


*/

//old_name_was dispatchMessage
pub fn parse_a_packet(
    sender: &String,
    packet: &JSonObject,
    connection_type: &String) -> (bool, bool)
{

    // * pType (Packet type) is more recent than old one bType(Block type) which was created to support block exchanging,
    // * wherease pType is proposed to support cpacket exchange
    // * which is more comprehensive(and expanded concept) than block.
    // * each package can contain one or more blocks and misc requests
    // * it is a-kind-of graphGL implementation.
    // * each packet contains one or more cards, and each card represents a single query or single query result
    let mut packet_type: String = "".to_string();
    if !packet["pType"].is_null()
    {
        packet_type = remove_quotes(&packet["pType"].to_string());

        if packet_type != constants::DEFAULT_PACKET_TYPE.to_string()
        {
            dlog(
                &format!("Undefined packet in packet parsing: {}!={}", packet_type, constants::DEFAULT_PACKET_TYPE.to_string()),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, true);
        }
    } else {
        dlog(
            &format!("Unknown packet in packet parsing"),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, true);
    }

    let mut packet_version: String = "".to_string();
    if !packet["pVer"].is_null()
    {
        packet_version = remove_quotes(&packet["pVer"].to_string());
    }

    dlog(
        &format!("Parsing a packet from sender({}) connection type({}) packet type({}) packet version({})", sender, connection_type, packet_type, packet_version.clone()),
        constants::Modules::App,
        constants::SecLevel::Info);

    if (sender == "") || (connection_type == "")
    {
        dlog(
            &format!("No sender or connection_type to dispatch sender({})", sender),
            constants::Modules::App,
            constants::SecLevel::Error);

        return (false, true);
    }

    let mut c_date: String = "".to_string();
    if !packet["pDate"].is_null()
    {
        c_date = remove_quotes(&packet["pDate"].to_string());
    }

    println!("packet[cards]: {}", packet["cards"]);
    let (status, cards) = match packet["cards"].as_array() {
        Some(r) => (true, r.clone()),
        _ => {
            dlog(
                &format!("Failed in serialising cards: {}", packet["cards"]),
                constants::Modules::Sec,
                constants::SecLevel::Error);

            (false, vec![])
        }
    };
    if !status
    { return (false, false); }

    let mut status: bool = true;
    let mut should_purge_file: bool = false;
    for a_card in cards
    {
        let (status_, should_purge_file_) = dispatch_a_card(
            sender,
            connection_type,
            &c_date,
            &a_card,
            &remove_quotes(&a_card["cdType"].to_string()),
            &remove_quotes(&a_card["cdVer"].to_string()),
            &packet_version.clone(),
        );

        status &= status_;
        should_purge_file |= should_purge_file_;

        dlog(
            &format!("Dispatch a card response card type ({}) status({}) should purge file({})",
                     remove_quotes(&a_card["cdType"].to_string()), status_, should_purge_file_),
            constants::Modules::App,
            constants::SecLevel::Debug);
    }

    dlog(
        &format!("Dispatch all cards response status({}) should purge file({})", status, should_purge_file),
        constants::Modules::App,
        constants::SecLevel::Debug);

    return (status, should_purge_file);

    // }
    // else
    // {
    // // it is old packet style which is one block per cpacket
    // String type_ = keys.contains("bType") ? message["bType"].to_string() : "";
    // if (type_ == "")
    //   type_ = keys.contains("mType") ? message["mType"].to_string() : "";
    // if (type_ == "")
    //     return {false, true};
    //
    // String bVer = keys.contains("bVer") ? message["bVer"].to_string() : "";
    //
    // /**
    //  * @brief pVer
    //  * message version.
    //  * indeed messages are messages between nodes, only in order to synch nodes togethere.
    //  * they are some kind of internal commands, and have no effect on DAG it self.
    //  * messages are about asking a particular block information, or the leave information or handshake with other nodes, etc
    //  */
    // String pVer = keys.contains("pVer") ? message["pVer"].to_string() : "";
    //
    // if ((bVer == "") && (pVer == ""))
    // {
    //   CLog::log("No bVer or pVer stated", "app" "error");
    //   return { false, true};
    // }
    //
    // return innerDispatchMessage(
    //     sender,
    //     connection_type,
    //     creation_date,
    //     message,
    //     type_,
    //     bVer,
    //     pVer);
    // }
}

//old_name_was innerDispatchMessage
pub fn dispatch_a_card(
    sender: &String,
    connection_type: &String,
    c_date: &String,
    card_body: &JSonObject,
    card_type: &String,
    card_ver: &String,
    packet_ver: &str) -> (bool, bool)
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
        /*
    // the essage is a whole block, so push it to table c_parsing_q
    String code = CUtils::hash16c(message["bHash"].to_string());
    if (!CUtils::isValidVersionNumber(message["bVer"].to_string()))
    {
      CLog::log("invalid bVer(" + message["bVer"].to_string() + ") for block(" + code + ") in dispatcher! type(" + type + ")", "sec", "error");
      return {false, true};
    }
    CLog::log("--- pushing block(" + code + ") type(" + type + ") from(" + sender + ") to 'c_parsing_q'");

    QVDRecordsT alreadyRecordedInDAG = DAG::searchInDAG(
      {{"b_hash", message["bHash"]}},
      {"b_hash"});

    if (alreadyRecordedInDAG.size() > 0)
    {
      CLog::log("Duplicated packet received block(" + code + ") type(" + type + ") from(" + sender + ") ", "app", "trace");
      return { true, true};

    } else {

      auto[push_status, should_purge_file] = ParsingQHandler::push_to_parsing_q(
        message,
        message["bCDate"].to_string(),
        type,
        message["bHash"].to_string(),
        sender,
        connection_type);
        Q_UNUSED(should_purge_file);

      // if it is a valid block, update last received block info
      if (push_status)
        DAGMessageHandler::setLastReceivedBlockTimestamp(type, message["bHash"].to_string());
    }

    // remove from missed blocks (if exist)
    MissedBlocksHandler::removeFromMissedBlocks(message["bHash"].to_string());
*/
        return (true, true);
    } else if card_types.contains(card_type)
    {
        let (status, should_purge_file) = handle_a_single_card(
            sender,
            connection_type,
            c_date,
            card_body,
            card_type,
            card_ver,
            packet_ver);

        dlog(
            &format!("Handle a single card response status({}) should purge file({})", status, should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);

        return (status, should_purge_file);
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
    (false, false)
}

//old_name_was handleSingleMessages
pub fn handle_a_single_card(
    sender: &String,
    connection_type: &String,
    creation_date: &String,
    card_body: &JSonObject,
    card_type: &str,
    card_ver: &str,
    packet_ver: &str) -> (bool, bool)
{
    let mut card_code: String = format!("{}-{}-{}", packet_ver, card_type, card_ver);

    if !card_body["bHash"].is_null()
    {
        card_code = remove_quotes(&card_body["bHash"].to_string()).to_string();
    }

    if !is_valid_version_number(card_ver)
    {
        dlog(
            &format!("invalid card version for in dispatcher! card type({}) card version({})", card_type, card_ver),
            constants::Modules::Sec,
            constants::SecLevel::Error);

        return (false, true);
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
// auto[push_status, should_purge_file] = ParsingQHandler::push_to_parsing_q(
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

        return (true, true);//FIXME: implement it ASAP

    let(parse_status, should_purge_file) = handle_received_leave_info(
        sender,
        card_body,
        connection_type);
//    dspchRes = { err: false, shouldPurgeMessage: true }
    } else if card_type == constants::card_types::HANDSHAKE
    {
        // handshake
        // TODO: implement a switch to set off/on for no more new neighbor
        dlog(
            &format!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@ constants::card_types::HANDSHAKE sender: {sender} @@@@@@@@@@@@@@@@@@@@@@@@@@@"),
            constants::Modules::App,
            constants::SecLevel::Info);

        let (parse_status, should_purge_file) = parse_handshake(
            sender,
            card_body,
            connection_type);
        dlog(
            &format!("greeting Parsers parse Handshake res: parse_status ({}) should_purge_file({})", parse_status, should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);
        return (parse_status, should_purge_file);
    } else if card_type == constants::card_types::NICE_TO_MEET_YOU
    {
        let (parse_status, should_purge_file) = parse_nice_to_meet_you(
            sender,
            card_body,
            connection_type);
        // invokeDescendents_(); // FIXME: do it in Async mode
        dlog(
            &format!("greeting Parsers parse nice to meet you res: parse_status ({}) should_purge_file({})", parse_status, should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);
        return (parse_status, should_purge_file);
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
        return (false, true);
    }

    return (false, false);
}

/*

std::tuple<bool, bool> Dispatcher::handleGQLMessages(
const String& sender,
const String& connection_type,
const String& creation_date,
const JSonObject& message,
const String& type,
const String& ver)
{

if (StringList {
  constants::ProposalLoanRequest,
  constants::FullDAGDownloadRequest,
  constants::pleaseRemoveMeFromYourNeighbors}.contains(type))
{
String dummy_hash = CCrypto::keccak256(CUtils::getNowSSS());
if (!CUtils::isValidVersionNumber(message["cdVer"].to_string()))
{
  CLog::log("invalid cdVer for GQL(" + dummy_hash + ") in dispatcher! type(" + type + ")", "sec", "error");
  return {false, true};
}
auto[status, should_purge_file] = ParsingQHandler::push_to_parsing_q(
  message,
  CUtils::getNow(),
  type,
  dummy_hash,
  sender,
  connection_type);
return {status, should_purge_file};

}
else if (card_type == constants::FullDAGDownloadResponse)
{
JSonObject block = message["block"].toObject();
String block_hash = block["bHash"].to_string();
if (!CUtils::isValidVersionNumber(message["cdVer"].to_string()))
{
  CLog::log("invalid cdVer for GQL(" + block_hash + ") in dispatcher! type(" + type + ")", "sec", "error");
  return {false, true};
}

// update flag LastFullDAGDownloadResponse
KVHandler::upsertKValue("LastFullDAGDownloadResponse", CUtils::getNow());

// control if already exist in DAG
QVDRecordsT alreadyRecordedInDAG = DAG::searchInDAG({{"b_hash", block_hash}});
if (alreadyRecordedInDAG.size()> 0)
{
  CLog::log("Duplicated packet received " + type + "-" + block_hash, "app", "trace");
  return {true, true};

} else {
  // push to table _parsing_q
  auto[status, should_purge_file] = ParsingQHandler::push_to_parsing_q(
    block,
    block["creationDate"].to_string(),
    block["bType"].to_string(),
    block_hash,
    sender,
    connection_type);
  Q_UNUSED(should_purge_file);

  // if it was a valid message
  if (status)
  {
    DAGMessageHandler::setLastReceivedBlockTimestamp(block["bType"].to_string(), block_hash);
    if (!CMachine::isInSyncProcess())
      CGUI::signalUpdateParsingQ();
  }
}
return {true, true};

//  }
//  else if (card_type == constants::BallotsReceiveDates)
//  {
//    // recceived all ballotes received date via QGL
//          clog.app.info(`Ballots Receive date message: ${utils.stringify(message.ballotsReceiveDates)}`);
//          if (!iutils.isValidVersionNumber(message.cdVer)) {
//              msg = `invalid cdVer for GQL(${block.blockHash}) Ballots Receive Dates in dispatcher! ${type}`
//              clog.sec.error(msg);
//              return { err: true, msg }
//          }
//          try {
//              // normalizing/sanitize Ballots Receive Dates and upsert into kv
//              let sanBallots = {};
//              for (let aBlt of utils.objKeys(message.ballotsReceiveDates)) {
//                  sanBallots[utils.stripNonAlphaNumeric(aBlt)] = {
//                      baReceiveDate: utils.stripNonInDateString(message.ballotsReceiveDates[aBlt].baReceiveDate.to_string()),
//                      baVoteRDiff: utils.stripNonNumerics(message.ballotsReceiveDates[aBlt].baVoteRDiff.to_string()),
//                  }
//              }
//              kvHandler.upsertKValueSync('ballotsReceiveDates', utils.stringify(sanBallots));
//              dspchRes = { err: false, shouldPurgeMessage: true }
//          } catch (e) {
//              clog.sec.error(e);
//              return { err: true, msg: e }
//          }
//    }
//  }
//  else if (card_type == constants::NodeStatusScreenshot)
//  {
//    // recceived an screenshot of neighbor's Machine status
//    console.log(`screenshott message: ${message.creationDate}-${message.sender}`);
//    clog.app.info(`screenshott message: ${message.creationDate}-${message.sender}`);
//    // "cdType":"NodeStatusScreenshot","cdVer":"0.0.1","creationDate":"2020-03-08 13:34:10","":"abc@def.gh",

//    if (!iutils.isValidVersionNumber(message.cdVer)) {
//        msg = `invalid cdVer for GQL(${block.blockHash}) Node Status Screenshot in dispatcher! ${type}`
//        clog.sec.error(msg);
//        return { err: true, msg }
//    }
//    const nodeScHandler = require('../services/node-screen-shot/node-screen-shot-handler');
//    let saveRes = nodeScHandler.pushReportToDB({
//        scOwner: `${message.sender}:${message.creationDate}`,
//        content: message.finalReport
//    });
//    saveRes.shouldPurgeMessage = true;
//    return saveRes;

//  }
//  else if (card_type == constants::directMsgToNeighbor)
//  {
//    // recceived an screenshot of neighbor's Machine status
//    console.log(`direct Msg To Neighbor: ${utils.stringify(message)}`);
//    clog.app.info(`direct Msg To Neighbor: ${message.creationDate}-${message.sender}`);
//    // "cdType":"NodeStatusScreenshot","cdVer":"0.0.1","creationDate":"2020-03-08 13:34:10","":"abc@def.gh",

//    if (!iutils.isValidVersionNumber(message.cdVer)) {
//      msg = `invalid cdVer for GQL(${block.blockHash}) direct Msg To Neighbor in dispatcher! ${type}`
//      clog.sec.error(msg);
//      return { err: true, msg }
//    }
//    const directmsgHandler = require('./direct-msg-handler');
//    let msgRes = directmsgHandler.recordReceivedMsg({
//      sender: message.sender,
//      receiver: message.receiver,
//      directMsgBody: message.directMsgBody
//    });
//    return msgRes;

}
return {false, false};
}

*/

//old_name_was makeAPacket
pub fn make_a_packet(
    cards: Vec<JSonObject>,
    packet_type: &str,
    packet_version: &str,
    packet_creation_date: CDateT) -> (String, String)
{
//FIXME: after finish development, use "wrap_safe_content_for_db" before insert in database(for security reason)
    let body_json: JSonObject = json!({
        "cards": cards,
        "pType": packet_type,
        "pVer": packet_version,
        "pDate": packet_creation_date});
    let body: String = cutils::serialize_json(&body_json);
    let code: String = ccrypto::keccak256(&body);
    return (code, body);
}

