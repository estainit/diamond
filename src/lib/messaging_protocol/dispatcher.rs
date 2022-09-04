use serde_json::{json};
use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::{CDateT, JSonObject};
use crate::lib::messaging_protocol::dispatcher_switch::dispatch_a_card;
use crate::lib::utils::version_handler::is_valid_version_number;

pub struct PacketParsingResult
{
    pub m_status: bool,
    pub m_should_purge_file: bool,
    pub m_message: String,
}

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
    packet: &mut JSonObject,
    connection_type: &String) -> PacketParsingResult
{

    // * pType (Packet type) is more recent than old one bType(Block type) which was created to support block exchanging,
    // * wherease pType is proposed to support cpacket exchange
    // * which is more comprehensive(and expanded concept) than block.
    // * each package can contain one or more blocks and misc requests
    // * it is a-kind-of graphGL implementation.
    // * each packet contains one or more cards, and each card represents a single query or single query result
    let packet_type: String;
    let mut error_message: String = "".to_string();
    if packet["pType"].is_null()
    {
        error_message = format!("Unknown packet in packet parsing: missed pType: {}", packet);
        dlog(
            &error_message,
            constants::Modules::App,
            constants::SecLevel::Error);

        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: error_message,
        };
    }


    packet_type = remove_quotes(&packet["pType"]);
    if packet_type != constants::DEFAULT_PACKET_TYPE.to_string()
    {
        error_message = format!("Undefined packet in packet parsing: {}!={}", packet_type, constants::DEFAULT_PACKET_TYPE.to_string());
        dlog(
            &error_message,
            constants::Modules::App,
            constants::SecLevel::Error);

        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: error_message,
        };
    }

    let mut packet_version: String = "".to_string();
    if !packet["pVer"].is_null()
    {
        packet_version = remove_quotes(&packet["pVer"]);
    }

    dlog(
        &format!("Parsing a packet from sender({}) connection type({}) packet type({}) packet version({})", sender, connection_type, packet_type, packet_version.clone()),
        constants::Modules::App,
        constants::SecLevel::Info);

    if (sender == "") || (connection_type == "")
    {
        error_message = format!("No sender or connection_type to dispatch sender({})", sender);
        dlog(
            &error_message,
            constants::Modules::App,
            constants::SecLevel::Error);

        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: true,
            m_message: error_message,
        };
    }

    let mut c_date: String = "".to_string();
    if !packet["pDate"].is_null()
    {
        c_date = remove_quotes(&packet["pDate"]);
    }

    println!("packet[cards]: {}", packet["cards"]);
    let (status, cards) = match packet["cards"].as_array() {
        Some(r) => (true, r.clone()),
        _ => {
            error_message = format!("Failed in de-serialising cards array: {}", packet["cards"]);
            dlog(
                &error_message,
                constants::Modules::Sec,
                constants::SecLevel::Error);

            (false, vec![])
        }
    };
    if !status
    {
        return PacketParsingResult {
            m_status: false,
            m_should_purge_file: false,
            m_message: error_message,
        };
    }

    let mut status: bool = true;
    let mut should_purge_file: bool = false;
    let mut error_messages = "".to_string();
    for mut a_card in cards
    {
        if !is_valid_version_number(remove_quotes(&mut a_card["cdVer"]).as_str()) {
            dlog(
                &format!("Invalid card version ({}) for card type ({})",
                         &a_card["cdVer"], remove_quotes(&a_card["cdType"])),
                constants::Modules::App,
                constants::SecLevel::Error);
            continue;
        }

        let card_type_ = remove_quotes(&a_card["cdType"].clone());
        let card_ver_ = remove_quotes(&a_card["cdVer"].clone());
        let pa_pa_res = dispatch_a_card(
            sender,
            connection_type,
            &c_date,
            &mut a_card,
            &card_type_,
            &card_ver_,
            &packet_version.clone(),
        );

        status &= pa_pa_res.m_status;
        should_purge_file |= pa_pa_res.m_should_purge_file;
        error_messages += &*pa_pa_res.m_message;

        dlog(
            &format!("Dispatch a card response card type ({}) status({}) should purge file({}), {}",
                     remove_quotes(&a_card["cdType"]), pa_pa_res.m_status, pa_pa_res.m_should_purge_file, pa_pa_res.m_message),
            constants::Modules::App,
            constants::SecLevel::Debug);
    }

    dlog(
        &format!("Dispatch all cards response status({}) should purge file({})", status, should_purge_file),
        constants::Modules::App,
        constants::SecLevel::Debug);

    return PacketParsingResult {
        m_status: status,
        m_should_purge_file: should_purge_file,
        m_message: error_messages,
    };

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
String dummy_hash = CCrypto::keccak256(cutils::getNowSSS());
if (!cutils::isValidVersionNumber(message["cdVer"].to_string()))
{
  CLog::log("invalid cdVer for GQL(" + dummy_hash + ") in dispatcher! type(" + type + ")", "sec", "error");
  return {false, true};
}
auto[status, should_purge_file] = ParsingQHandler::push_to_parsing_q(
  message,
  cutils::getNow(),
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
if (!cutils::isValidVersionNumber(message["cdVer"].to_string()))
{
  CLog::log("invalid cdVer for GQL(" + block_hash + ") in dispatcher! type(" + type + ")", "sec", "error");
  return {false, true};
}

// update flag LastFullDAGDownloadResponse
KVHandler::upsertKValue("LastFullDAGDownloadResponse", cutils::getNow());

// control if already exist in DAG
QVDRecordsT alreadyRecordedInDAG = DAG::searchInDAG({{"b_hash", block_hash}});
if (alreadyRecordedInDAG.len()> 0)
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
    set_last_received_block_timestamp(block["bType"].to_string(), block_hash);
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
    let body: String = cutils::controlled_json_stringify(&body_json);
    let code: String = ccrypto::keccak256(&body);
    return (code, body);
}

