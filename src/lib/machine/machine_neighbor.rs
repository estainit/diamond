use std::collections::HashMap;
use postgres::types::ToSql;
use serde::{Serialize, Deserialize};
use crate::{ccrypto, CMachine, constants, cutils, dlog, machine};
use crate::lib::custom_types::{CDateT, ClausesT, JSonObject, QVDRecordsT};
use crate::lib::database::abs_psql::{OrderModifier, q_insert, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::STBL_MACHINE_NEIGHBORS;
use crate::lib::messaging_protocol::greeting::{createHandshakeRequest, createHereIsNewNeighbor, createNiceToMeetYou};
use crate::lib::network::network_handler::iPush;

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighborPresentation {
    m_vertice: String,
    m_date: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NeighborInfo {
    m_name: String,
}

impl NeighborInfo {
    pub fn new() -> NeighborInfo {
        NeighborInfo {
            m_name: "".to_string()
        }
    }
}

impl CMachine {
    /*
struct TmpData{
  String machine_PGP_private_key;
  String machine_email;
  String neighbor_email_address;
  };

  void pleaseRemoveMeFromYourNeighbors_(TmpData tmpData)
  {
  std::this_thread::sleep_for (std::chrono::seconds(5));

  JSonObject card {
    {"cdType", constants::CARD_TYPES::pleaseRemoveMeFromYourNeighbors},
    {"cdVer", "0.0.1"},
    {"emailToBeRemoved", tmpData.machine_email}};

  String sign_msg = ccrypto::keccak256(cutils::serializeJson(card));
  card.insert("signature", ccrypto::nativeSign(tmpData.machine_PGP_private_key, sign_msg));
  CLog::log("signed card to send remove " + cutils::dumpIt(card));

  auto[code, body] = GraphQLHandler::makeAPacket({card});

  SendingQHandler::pushIntoSendingQ(
    constants::GQL, //sqType
    code,
    body,
    "GQL PleaseRemoveMe packet(" + cutils::hash16c(code) + ")",
    {tmpData.neighbor_email_address});  //sqReceivers
  }

  bool CMachine::IdeleteNeighbors(
  const String& n_id,
  const String& connection_type,
  const String& mp_code)
  {
  QVDRecordsT neiInfo = getNeighbors(
    connection_type,
    "",
    mp_code,
    n_id);

  if (neiInfo.len() == 0)
  {
    CLog::log("Deleting neighbor connection_type(" + connection_type + ") mp_code(" + mp_code + ") id(" + n_id + ") not exist", "app", "error");
    return false;
  }

  String machine_PGP_private_key, machine_email;

  if (neiInfo[0]["n_connection_type"].to_string() == constants::PRIVATE)
  {
    machine_PGP_private_key = getPrivEmailInfo().m_PGP_private_key;
    machine_email = getPrivEmailInfo().m_address;
  }else{
    machine_PGP_private_key = getPubEmailInfo().m_PGP_private_key;
    machine_email = getPubEmailInfo().m_address;
  }
  String neighbor_email_address = neiInfo[0]["n_email"].to_string();

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
  const String& sender_email,
  const JSonObject& message,
  const String& connection_type)
  {
  CLog::log("parse Nice To Meet You connection_type(" + connection_type + ") sender(" + sender_email + ") message(" + cutils::dumpIt(message) + ")", "app", "trace");


  String email = message.keys().contains("email") ? message["email"].to_string() : "";
  String sender_pgp_public_key = message.keys().contains("PGPPubKey") ? message["PGPPubKey"].to_string() : "";
  String sender_backer_address = message.keys().contains("backerAddress") ? message["backerAddress"].to_string() : "";


  // just to be sure handshake happends ONLY ONE TIME for each email at the start
  // if user needs to change publickkey or ... she can send alternate messages like changeMyPublicKey(which MUST be signed with current key)
  // retreive sender's info
  QVDRecordsT sender_info = getNeighbors(
    "",
    "",
    "",
    "",
    sender_email);

  if (sender_info.len() == 0)
  {
    // some security logs
    CLog::log("!!! Machine has not this sender_email(" + sender_email + ") as a neighbor", "sec", "error");
    return {false, true};
  }

  // try {

  if ((sender_email == "") || (sender_pgp_public_key == ""))
  {
    CLog::log("!!! invalid sender_email or PGPPubKey received from neighbor sender_email(" + sender_email + ") sender_pgp_public_key(" + sender_pgp_public_key + ") as a neighbor", "sec", "error");
    return {false, true};
  }

  sender_pgp_public_key = ccrypto::base64Decode(sender_pgp_public_key);

  QVDicT updates {
    {"n_info", cutils::serializeJson(JSonObject{})},
    {"n_pgp_public_key", sender_pgp_public_key},
    {"n_last_modified", cutils::get_now()}};

  if (sender_backer_address != "")
    updates["n_info"] = cutils::serializeJson(JSonObject{{"backerAddress", sender_backer_address}});

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
}

//old_name_was addANewNeighbor
pub fn add_a_new_neighbor(
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

    let (_status, records) = q_select(
        STBL_MACHINE_NEIGHBORS,
        vec!["n_mp_code", "n_email"],
        vec![
            simple_eq_clause("n_mp_code", &*mp_code),
            simple_eq_clause("n_connection_type", &*connection_type),
            simple_eq_clause("n_email", &*neighbor_email),
        ],
        vec![],
        0,
        true, );

    if records.len() > 0
    {
        return if neighbor_public_key != ""
        {
            //update pgp key
            let now = cutils::get_now();
            let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
                ("n_pgp_public_key", &neighbor_public_key as &(dyn ToSql + Sync)),
                ("n_last_modified", &now as &(dyn ToSql + Sync)),
            ]);
            let clauses: ClausesT = vec![
                simple_eq_clause("n_mp_code", &*mp_code),
                simple_eq_clause("n_connection_type", &*connection_type),
                simple_eq_clause("n_email", &*neighbor_email),
            ];

            q_update(
                STBL_MACHINE_NEIGHBORS,
                &values,
                clauses,
                true);
            (true, format!("The iPGP key for email({neighbor_email}) connection({connection_type}) profile({mp_code}) updated"))
        } else {
            (false, format!("The iPGP key for email({neighbor_email}) connection({connection_type}) profile({mp_code}) was missed"))
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
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("n_mp_code", &mp_code as &(dyn ToSql + Sync)),
        ("n_email", &neighbor_email as &(dyn ToSql + Sync)),
        ("n_pgp_public_key", &neighbor_public_key as &(dyn ToSql + Sync)),
        ("n_is_active", &is_active as &(dyn ToSql + Sync)),
        ("n_connection_type", &connection_type as &(dyn ToSql + Sync)),
        ("n_creation_date", &creation_date as &(dyn ToSql + Sync)),
        ("n_info", &serialized_neighbor_info as &(dyn ToSql + Sync)),
        ("n_last_modified", &now as &(dyn ToSql + Sync))
    ]);
    dlog(
        &format!("goint to add new Neighbor: {:?}", &values),
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

pub fn getNeighbors(
    neighbor_type: &str,
    connection_status: &str,
    mp_code: &str,
    n_id: &str,
    n_email: &str) -> QVDRecordsT
{
    let mut clauses: ClausesT = vec![];

    if connection_status != "" {
        clauses.push(simple_eq_clause("n_is_active", connection_status));
    }

    if neighbor_type != "" {
        clauses.push(simple_eq_clause("n_connection_type", neighbor_type));
    }

    if mp_code != "" {
        clauses.push(simple_eq_clause("n_mp_code", mp_code));
    }

    if n_id != "" {
        clauses.push(simple_eq_clause("n_id", n_id));
    }

    if n_email != "" {
        clauses.push(simple_eq_clause("n_email", n_email));
    }

    let (_status, records) = q_select(
        STBL_MACHINE_NEIGHBORS,
        vec!["n_id", "n_email", "n_pgp_public_key", "n_connection_type"],
        clauses,
        vec![
            &OrderModifier { m_field: "n_connection_type", m_order: "DESC" },
        ],
        0,
        true,
    );
    return records;
}

pub fn getActiveNeighbors(mp_code: &str) -> QVDRecordsT
{
    let (_status, records) = q_select(
        STBL_MACHINE_NEIGHBORS,
        vec!["n_email", "n_pgp_public_key", "n_connection_type"],
        vec![
            simple_eq_clause("n_is_active", constants::YES),
            simple_eq_clause("n_mp_code", mp_code)],
        vec![&OrderModifier { m_field: "n_connection_type", m_order: "DESC" }],
        0,
        true,
    );
    return records;
}

pub fn handshakeNeighbor(n_id: &String, connection_type: &String) -> bool
{
    dlog(
        &format!("handshake Neighbor: {} {}", n_id, connection_type),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (
        status,
        title,
        sender_email,
        receiver_email,
        message) = createHandshakeRequest(connection_type, n_id);
    dlog(
        &format!("packet Generators.write Handshake: sender_email({}) title({}) sender_email({}) receiver_email({}) message({})",
                 sender_email, title, sender_email, receiver_email, message),
        constants::Modules::App,
        constants::SecLevel::Info);
    if !status
    { return status; }

    // the concept is the node public email is propagated to more neighbors in order to strength connectivity,
    // but the node private email will be used as a second plan to defence against the any kind of spaming/DOS Attacks ...
    return iPush(
        &title,
        &message,
        &sender_email,
        &receiver_email);
}


pub fn parseHandshake(
    sender_email: &String,
    message: &JSonObject,
    connection_type: &String) -> (bool, bool)
{
    dlog(
        &format!("parse Handshake args: sender_email({}) connection_type({}) message({})", sender_email, connection_type, message),
        constants::Modules::App,
        constants::SecLevel::Info);

    let pgp_public_key: String;
    if !message["PGPPubKey"].is_null() {
        pgp_public_key = message["PGPPubKey"].to_string();
    } else {
        pgp_public_key = "".to_string();
    }

    if connection_type == ""
    { return (false, true); }

    // just to be sure handshake happends ONLY ONE TIME for each email at the start
    // if user needs to change publickkey or ... she can send alternate messages like changeMyPublicKey(which MUST be signed with current key)
    // retreive sender's info
    let mut email_already_exist: bool = false;
    let sender_info: QVDRecordsT = getNeighbors(connection_type, "", "", "", sender_email);

    if sender_info.len() > 0
    {
        // some security logs
        email_already_exist = true;

        dlog(
            &format!("!!! the email in parse Handshake ({}) already inserted", sender_email),
            constants::Modules::Sec,
            constants::SecLevel::Error);
    }

    if sender_email == ""
    {
        dlog(
            &format!("!!! invalid email received from neighbor via handshake"),
            constants::Modules::Sec,
            constants::SecLevel::Error);

        return (false, true);
    }

    if pgp_public_key == ""
    {
        dlog(
            &format!("!!! missed PGP public key received from neighbor via handshake"),
            constants::Modules::Sec,
            constants::SecLevel::Error);

        return (false, true);
    }

    let (status, pgp_public_key) = ccrypto::b64_decode(&pgp_public_key);

    if !email_already_exist
    {
        add_a_new_neighbor(
            sender_email.to_string(),
            connection_type.to_string(),
            pgp_public_key.clone(),
            "".to_string(),
            constants::YES.to_string(),
            NeighborInfo::new(),
            cutils::get_now());
    } else {
        if sender_info[0]["n_pgp_public_key"].to_string() == ""
        {
            let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
                ("n_pgp_public_key", &pgp_public_key as &(dyn ToSql + Sync)),
            ]);
            q_update(
                STBL_MACHINE_NEIGHBORS,
                &update_values,
                vec![simple_eq_clause("n_id", &sender_info[0]["n_id"])],
                false,
            );
        }
    }

    // send response niceToMeetYou
    let (status, title, sender_email_, receiver_email, message_) = createNiceToMeetYou(
        connection_type,
        sender_email,
        &pgp_public_key);

    dlog(
        &format!("create NiceToMeetYou status({}) message_: {}", status, message_),
        constants::Modules::App,
        constants::SecLevel::Info);

    if !status
    { return (false, true); }

    let _sent: bool = iPush(
        &title,
        &message_,
        &sender_email_,
        &receiver_email);

    // broadcast the email to other neighbors
    if connection_type == constants::PUBLIC
    {
        floodEmailToNeighbors(sender_email, &pgp_public_key);
    }

    return (true, true);
}

pub fn floodEmailToNeighbors(
    email: &String,
    pgp_public_key: &String) -> bool
{
    let mut pgp_public_key = pgp_public_key.to_string();
    dlog(
        &format!("flood Email To Neighbors: {}", email),
        constants::Modules::App,
        constants::SecLevel::Info);

    if pgp_public_key == ""
    {
        let email_info: QVDRecordsT = getNeighbors(constants::PUBLIC, "", "", "", email);
        if email_info.len() == 0
        {
            dlog(
                &format!("email({}) doesn't exist as a neighbor!", email),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }
        pgp_public_key = email_info[0]["n_pgp_public_key"].to_string();
    }

    //  * avoiding duplicate sending email
    //  * [{vertice: "neighborEmail->targetEmail", date:"presenting date"}]

    let mut already_presented_neighbors: Vec<NeighborPresentation> = machine().m_profile.m_mp_settings.m_already_presented_neighbors.clone();
    let pr= &already_presented_neighbors.iter().map(|x| x.m_vertice.clone()).collect::<Vec<String>>().join(",");
    dlog(
        &format!("Already Presented to these Neighbors {:?}", pr),
        constants::Modules::App,
        constants::SecLevel::Info);

    let active_neighbors: QVDRecordsT = getNeighbors(
        constants::PUBLIC,
        constants::YES,
        &machine().getSelectedMProfile(),
        "",
        "");

    dlog(
        &format!("Active Neighbors to flood email to neigbors: {:?}", active_neighbors),
        constants::Modules::App,
        constants::SecLevel::Info);

    let mut vertice: String = "".to_string();
    let mut is_already_sent: bool;
    for neighbor in active_neighbors
    {
        let n_email: String = neighbor["n_email"].to_string();
        if &n_email == email
        {
            continue;   // not presenting machine to itself
        }

        is_already_sent = false;
        vertice = email.to_owned() + "__to__" + &n_email;
        for vert in &already_presented_neighbors
        {
            dlog(
                &format!("vertices {}", vert.m_vertice),
                constants::Modules::App,
                constants::SecLevel::Info);

            if vert.m_vertice == vertice
            {
                dlog(
                    &format!("!!! the email already broadcasted {}", vertice),
                    constants::Modules::App,
                    constants::SecLevel::Info);

                is_already_sent = true;
            }
        }
        if !is_already_sent
        {
            //TODO: adding some expiration control to have availabality to re-broadcast email
            already_presented_neighbors.push(NeighborPresentation {
                m_vertice: vertice,
                m_date: cutils::get_now(),
            });

            let (status, title, sender_email, receiver_email, message) = createHereIsNewNeighbor(
                &constants::PUBLIC.to_string(),
                &machine().getPubEmailInfo().m_address,
                &machine().getPubEmailInfo().m_pgp_private_key,
                &n_email,
                &neighbor["n_pgp_public_key"].to_string(),
                email,  //newNeighborEmail
                &pgp_public_key);

            dlog(
                &format!("packet ready to flood email to neigbor: status({}) title({}) sender_email({}) receiver_email({}) message({}) ",
                         status, title, sender_email, receiver_email, message),
                constants::Modules::App,
                constants::SecLevel::Info);

            dlog(
                &format!("the machine presents({}) to ({})", email, n_email),
                constants::Modules::App,
                constants::SecLevel::Info);

            iPush(
                &title,
                &message,
                &sender_email,
                &receiver_email);
        }
    }

    // update machine settings
    machine().m_profile.m_mp_settings.m_already_presented_neighbors = already_presented_neighbors;
    machine().save_settings();

    return true;
}