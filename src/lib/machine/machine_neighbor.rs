use std::collections::HashMap;
use std::fmt::format;
use serde::{Serialize, Deserialize};
use crate::{CMachine, constants, cutils, dlog};
use crate::lib::custom_types::{CDateT, ClausesT};
use crate::lib::database::abs_psql::{ModelClause, q_insert, q_select, q_update};
use crate::lib::database::tables::STBL_MACHINE_NEIGHBORS;
use crate::lib::utils::dumper::dump_hashmap_of_str;

#[derive(Serialize, Deserialize)]
pub struct NeighborInfo {
    name: String,
}

impl NeighborInfo {
    pub fn new() -> NeighborInfo {
        NeighborInfo {
            name: "".to_string()
        }
    }
}

impl<'m> CMachine<'m> {
    //old_name_was addANewNeighbor
    pub fn add_a_new_neighbor(
        &self,
        neighbor_email: String,
        connection_type: String,
        neighbor_public_key: String,
        mp_code: String,
        is_active: String,
        neighbor_info: NeighborInfo,
        c_date: CDateT) -> (bool, String)
    {
        dlog(
            &format!("add new Neighbor email({neighbor_email}) connection_type({connection_type}) "),
            constants::Modules::App,
            constants::SecLevel::Info);

        let mut creation_date = c_date.clone();
        if (neighbor_email == "") || (connection_type == "")
        {
            return (false, "The neighbor email or connection type is missed".to_string());
        }

        let (status, records) = q_select(
            STBL_MACHINE_NEIGHBORS,
            &vec!["n_mp_code", "n_email"],
            &vec![
                &ModelClause {
                    m_field_name: "n_mp_code",
                    m_field_single_str_value: &*mp_code,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                },
                &ModelClause {
                    m_field_name: "n_connection_type",
                    m_field_single_str_value: &*connection_type,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                },
                &ModelClause {
                    m_field_name: "n_email",
                    m_field_single_str_value: &*neighbor_email,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                },
            ],
            &vec![],
            0,
            true, );

        if records.len() > 0
        {
            if neighbor_public_key != ""
            {
                //update pgp key
                let now = cutils::get_now();
                let values: HashMap<&str, &str> = HashMap::from([
                    ("n_pgp_public_key", &*neighbor_public_key),
                    ("n_last_modified", &*now),
                ]);
                let c1 = ModelClause {
                    m_field_name: "n_mp_code",
                    m_field_single_str_value: &*mp_code,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                };
                let c2 = ModelClause {
                    m_field_name: "n_connection_type",
                    m_field_single_str_value: &*connection_type,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                };
                let c3 = ModelClause {
                    m_field_name: "n_email",
                    m_field_single_str_value: &*neighbor_email,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                };
                let clauses: ClausesT = vec![&c1, &c2, &c3];

                q_update(
                    STBL_MACHINE_NEIGHBORS,
                    &values,
                    &clauses,
                    true);
                return (true, format!("The iPGP key for email({neighbor_email}) connection({connection_type}) profile({mp_code}) updated"));
            } else {
                return (false, format!("The iPGP key for email({neighbor_email}) connection({connection_type}) profile({mp_code}) was missed"));
            }
        }


        if creation_date == "" {
            creation_date = cutils::get_now();
        }

        let (status, serialized_neighbor_info) = match serde_json::to_string(&neighbor_info) {
            Ok(ser) => { (true, ser) }
            Err(e) => {
                dlog(
                    &format!("Failed in serialization neighbor_info {:?}", e),
                    constants::Modules::App,
                    constants::SecLevel::Error);
                (false, "".to_string())
            }
        };
        if !status
        { return (false, "Failed in serialization neighbor_info".to_string()); }

        let now = cutils::get_now();
        let values: HashMap<&str, &str> = HashMap::from([
            ("n_mp_code", &*mp_code),
            ("n_email", &*neighbor_email),
            ("n_pgp_public_key", &*neighbor_public_key),
            ("n_is_active", &*is_active),
            ("n_connection_type", &*connection_type),
            ("n_creation_date", &*creation_date),
            ("n_info", &*serialized_neighbor_info),
            ("n_last_modified", &*now)
        ]);
        dlog(
            &format!("goint to add new Neighbor: {:?}", dump_hashmap_of_str(&values)),
            constants::Modules::App,
            constants::SecLevel::Info);

        q_insert(
            STBL_MACHINE_NEIGHBORS,
            &values,
            true);

        return (
            true,
            "new Neighbor(".to_owned() + &neighbor_email + ") connection(" + &connection_type + ") profile(" + &mp_code + &") was added".to_string()
        );
    }
}
/*
      static std::tuple<bool, QString> add_a_new_neighbor(
        const QString& email,
        const QString& connection_type,
        const QString& public_key = "",
        const QString& mp_code = getSelectedMProfile(),
        const QString& is_active = constants::YES,
        const QJsonObject& neighbor_info = QJsonObject(),
        QString creation_date = "");

        #include "stable.h"

#include "lib/ccrypto.h"
#include "lib/messaging_protocol/message_handler.h"
#include "lib/messaging_protocol/graphql_handler.h"
#include "lib/sending_q_handler/sending_q_handler.h"
#include "lib/network/network_handler.h"
#include "gui/c_gui.h"

#include "machine_handler.h"



QVDRecordsT CMachine::getActiveNeighbors(const QString& mp_code)
{

  QueryRes neighbors = DbModel::select(
    stbl_machine_neighbors,
    {"n_email", "n_pgp_public_key", "n_connection_type"},
    ClausesT{
      {"n_is_active", CConsts::YES},
      {"n_mp_code", mp_code}
    },
    {{"n_connection_type", "DESC"}}
  );
  return neighbors.records;
}

QVDRecordsT CMachine::IgetNeighbors(
  const QString& neighbor_type,
  const QString& connection_status,
  const QString& mp_code,
  const QString& n_id,
  const QString& n_email)
{
  ClausesT clauses{};
  if (connection_status != "")
    clauses.append({"n_is_active", connection_status});

  if (neighbor_type != "")
    clauses.append({"n_connection_type", neighbor_type});

  if (mp_code != "")
    clauses.append({"n_mp_code", mp_code});

  if (n_id != "")
    clauses.append({"n_id", n_id});

  if (n_email != "")
    clauses.append({"n_email", n_email});

  QueryRes neighbors = DbModel::select(
    stbl_machine_neighbors,
    {"n_id", "n_email", "n_pgp_public_key", "n_connection_type"},
    clauses,
    {{"n_connection_type", "DESC"}}
  );
  return neighbors.records;
}

*/

/*

bool CMachine::handshakeNeighbor(const QString& n_id, const QString& connection_type)
{
  CLog::log("handshake Neighbor: " + n_id + " " + connection_type);

  try
  {
    auto[status, title, sender_email, receiver_email, message] = MessageHandler::createHandshakeRequest(
      connection_type, n_id);
    CLog::log("packet Generators.write Handshake: sender_email(" + sender_email + ") title(" + title + ") sender_email(" + sender_email + ") receiver_email(" + receiver_email + ") message(" + message + ")");
    if (!status)
      return status;

    // the concept is the node public email is propagated to more neighbors in order to strength connectivity,
    // but the node private email will be used as a second plan to defence against the any kind of spaming/DOS Attacks ...
    return NetworkHandler::iPush(
        title,
        message,
        sender_email,
        receiver_email);

  } catch (std::exception) {
    CLog::log("Something went wrong in handshake", "app", "fatal");
    return false;
  }
}

struct TmpFloodData
{
  QString sender_email;
  QString PGP_public_key;
};

void floodEmailToNeighbors_(TmpFloodData info)
{
  std::this_thread::sleep_for (std::chrono::seconds(10));
  CMachine::floodEmailToNeighbors(info.sender_email, info.PGP_public_key);
}

std::tuple<bool, bool> CMachine::parseHandshake(
  const QString& sender_email,
  const QJsonObject& message,
  const QString& connection_type)
{

  CLog::log("parse Handshake args: sender_email(" + sender_email + ") connection_type(" + connection_type + ") message(" + cutils::dumpIt(message) + ")", "app", "trace");

  QString PGP_public_key = message.keys().contains("PGPPubKey") ? message.value("PGPPubKey").toString() : "";
  // QString backer_address = message.keys().contains("backerAddress") ? message.value("backerAddress").toString() : "";

  if (connection_type == "")
    return {false, true};

  // just to be sure handshake happends ONLY ONE TIME for each email at the start
  // if user needs to change publickkey or ... she can send alternate messages like changeMyPublicKey(which MUST be signed with current key)
  // retreive sender's info
  bool email_already_exist = false;
  QVDRecordsT sender_info = getNeighbors(connection_type, "", "", "", sender_email);

  if (sender_info.size() > 0)
  {
    // some security logs
    email_already_exist = true;
    CLog::log("!!! the email in parse Handshake (" + sender_email + ") already inserted", "sec", "error");
  }

  if (sender_email == "")
  {
    CLog::log("!!! invalid email received from neighbor via handshake", "sec", "error");
    return {false, true};
  }

  if (PGP_public_key == "")
  {
    CLog::log("!!! missed PGP_public_key received from neighbor via handshake", "sec", "error");
    return {false, true};
  }

  PGP_public_key = CCrypto::base64Decode(PGP_public_key);

  if (!email_already_exist)
  {
    add_a_new_neighbor(
      sender_email,
      connection_type,
      PGP_public_key,
      "",
      CConsts::YES);

  }
  else
  {
    if (sender_info[0].value("n_pgp_public_key").toString() == "")
      DbModel::update(
        stbl_machine_neighbors,
        {{"n_pgp_public_key", PGP_public_key}},
        {{"n_id", sender_info[0].value("n_id")}});

  }

  // send response niceToMeetYou
  auto[status, title, sender_email_, receiver_email, message_] = MessageHandler::createNiceToMeetYou(
    connection_type,
    sender_email,
    PGP_public_key);

  CLog::log("create NiceToMeetYou status(" + cutils::dumpIt(status) + ") message_: " + message_, "app", "trace");
  if (!status)
    return {false, true};

  bool sent = NetworkHandler::iPush(
    title,
    message_,
    sender_email_,
    receiver_email);
  Q_UNUSED(sent);

  // broadcast the email to other neighbors
  if (connection_type == CConsts::PUBLIC)
  {
    TmpFloodData data_{sender_email, PGP_public_key};
    std::thread(floodEmailToNeighbors_, data_).detach();
  }

  return {true, true};

}

bool CMachine::IfloodEmailToNeighbors(
  const QString& email,
  QString PGP_public_key)
{
  CLog::log("flood Email To Neighbors: " + email, "app", "trace");

  if (PGP_public_key == "")
  {
    QVDRecordsT email_info = getNeighbors(CConsts::PUBLIC, "", "", "", email);
    if (email_info.size() == 0)
    {
      CLog::log("email(" + email + ") doesn't exist as a neighbor!", "sec", "error");
      return false;
    }
    PGP_public_key = email_info[0].value("n_pgp_public_key").toString();
  }

  /**
  * avoiding duplicate sending email
  * [{vertice: "neighborEmail->targetEmail", date:"presenting date"}]
  */
  QJsonArray already_presented_neighbors = getAlreadyPresentedNeighbors();
  CLog::log("Already Presented to these Neighbors " + cutils::dumpIt(already_presented_neighbors), "app", "trace");

  QVDRecordsT active_neighbors = getNeighbors(
        CConsts::PUBLIC,
        CConsts::YES);

  CLog::log("Active Neighbors to flood email to neigbors: " + cutils::dumpIt(active_neighbors), "app", "trace");
  QString vertice = "";
  bool is_already_sent;
  for (QVDicT neighbor: active_neighbors)
  {
    QString n_email = neighbor.value("n_email").toString();
    if (n_email == email)
      continue;   // not presenting machine to itself

    is_already_sent = false;
    vertice = email + "__to__" + n_email;
    for(QJsonValue vert: already_presented_neighbors)
    {
      CLog::log("verts " + cutils::dumpIt(vert), "app", "trace");
      if (vert.toObject().value("vertice").toString() == vertice)
      {
        CLog::log("!!! the email already broadcasted " + vertice, "app", "trace");
        is_already_sent = true;
      }
    }
    if (!is_already_sent)
    {
      //TODO: adding some expiration control to have availabality to re-broadcast email
      already_presented_neighbors.push(QJsonObject{
        {"vertice", vertice},
        {"date", cutils::get_now()}});

      auto[status, title, sender_email, receiver_email, message] = MessageHandler::createHereIsNewNeighbor(
        CConsts::PUBLIC,
        getPubEmailInfo().m_address,
        getPubEmailInfo().m_PGP_private_key,

        n_email,
        neighbor.value("n_pgp_public_key").toString(),

        email,  //newNeighborEmail
        PGP_public_key);

      CLog::log("packet ready to flood email to neigbor: status(" + cutils::dumpIt(status) + ") title(" + title + ") sender_email(" + sender_email + ") receiver_email(" + receiver_email + ") message(" + message + ") " , "app", "trace");
      CLog::log("the machine presents(" + email + ") to (" + n_email + ")", "app", "trace");

      NetworkHandler::iPush(
        title,
        message,
        sender_email,
        receiver_email);
    }
  }

  // update machine settings
  setAlreadyPresentedNeighbors(already_presented_neighbors);
  saveSettings();


  return true;
}

struct TmpData{
  QString machine_PGP_private_key;
  QString machine_email;
  QString neighbor_email_address;
};

void pleaseRemoveMeFromYourNeighbors_(TmpData tmpData)
{
  std::this_thread::sleep_for (std::chrono::seconds(5));

  QJsonObject card {
    {"cdType", CConsts::CARD_TYPES::pleaseRemoveMeFromYourNeighbors},
    {"cdVer", "0.0.1"},
    {"emailToBeRemoved", tmpData.machine_email}};

  QString sign_msg = CCrypto::keccak256(cutils::serializeJson(card));
  card.insert("signature", CCrypto::nativeSign(tmpData.machine_PGP_private_key, sign_msg));
  CLog::log("signed card to send remove " + cutils::dumpIt(card));

  auto[code, body] = GraphQLHandler::makeAPacket({card});

  SendingQHandler::pushIntoSendingQ(
    CConsts::GQL, //sqType
    code,
    body,
    "GQL PleaseRemoveMe packet(" + cutils::hash16c(code) + ")",
    {tmpData.neighbor_email_address});  //sqReceivers
}

bool CMachine::IdeleteNeighbors(
  const QString& n_id,
  const QString& connection_type,
  const QString& mp_code)
{
  QVDRecordsT neiInfo = getNeighbors(
    connection_type,
    "",
    mp_code,
    n_id);

  if (neiInfo.size() == 0)
  {
    CLog::log("Deleting neighbor connection_type(" + connection_type + ") mp_code(" + mp_code + ") id(" + n_id + ") not exist", "app", "error");
    return false;
  }

  QString machine_PGP_private_key, machine_email;

  if (neiInfo[0].value("n_connection_type").toString() == CConsts::PRIVATE)
  {
    machine_PGP_private_key = getPrivEmailInfo().m_PGP_private_key;
    machine_email = getPrivEmailInfo().m_address;
  }else{
    machine_PGP_private_key = getPubEmailInfo().m_PGP_private_key;
    machine_email = getPubEmailInfo().m_address;
  }
  QString neighbor_email_address = neiInfo[0].value("n_email").toString();

  if ((machine_PGP_private_key == "") || (machine_email == ""))
  {
    CLog::log("write Please Remove Me, missed parameters machine_PGP_private_key(" + machine_PGP_private_key + ") machine_email(" + machine_email + ")", "app", "error");
    return false;
  }

  if (neighbor_email_address != "")
  {
    TmpData tmpData{machine_PGP_private_key, machine_email, neighbor_email_address};
    std::thread(pleaseRemoveMeFromYourNeighbors_, tmpData).detach();

    if (n_id != "")
    {
      DbModel::dDelete(
        stbl_machine_neighbors,
        {{"n_id", n_id}});

      return true;
    }
  }
  return false;
}


std::tuple<bool, bool> CMachine::parseNiceToMeetYou(
  const QString& sender_email,
  const QJsonObject& message,
  const QString& connection_type)
{
  CLog::log("parse Nice To Meet You connection_type(" + connection_type + ") sender(" + sender_email + ") message(" + cutils::dumpIt(message) + ")", "app", "trace");


  QString email = message.keys().contains("email") ? message.value("email").toString() : "";
  QString sender_PGP_public_key = message.keys().contains("PGPPubKey") ? message.value("PGPPubKey").toString() : "";
  QString sender_backer_address = message.keys().contains("backerAddress") ? message.value("backerAddress").toString() : "";


  // just to be sure handshake happends ONLY ONE TIME for each email at the start
  // if user needs to change publickkey or ... she can send alternate messages like changeMyPublicKey(which MUST be signed with current key)
  // retreive sender's info
  QVDRecordsT sender_info = getNeighbors(
    "",
    "",
    "",
    "",
    sender_email);

  if (sender_info.size() == 0)
  {
    // some security logs
    CLog::log("!!! Machine has not this sender_email(" + sender_email + ") as a neighbor", "sec", "error");
    return {false, true};
  }

  // try {

  if ((sender_email == "") || (sender_PGP_public_key == ""))
  {
    CLog::log("!!! invalid sender_email or PGPPubKey received from neighbor sender_email(" + sender_email + ") sender_PGP_public_key(" + sender_PGP_public_key + ") as a neighbor", "sec", "error");
    return {false, true};
  }

  sender_PGP_public_key = CCrypto::base64Decode(sender_PGP_public_key);

  QVDicT updates {
    {"n_info", cutils::serializeJson(QJsonObject{})},
    {"n_pgp_public_key", sender_PGP_public_key},
    {"n_last_modified", cutils::get_now()}};

  if (sender_backer_address != "")
    updates["n_info"] = cutils::serializeJson(QJsonObject{{"backerAddress", sender_backer_address}});

  // update neighbor info's PGP public key
  DbModel::update(
    stbl_machine_neighbors,
    updates,
    {{"n_email", sender_email}});

  // TODO: publish this email to my neighbors

  return {true, true};

  // } catch (err) {
  //     clog.app.error(err)
  //     return { err: true, msg: err, shouldPurgeMessage: null }
  // }

}

*/