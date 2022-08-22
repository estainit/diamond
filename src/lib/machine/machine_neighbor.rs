use std::collections::HashMap;
use postgres::types::ToSql;
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::{application, ccrypto, CMachine, constants, cutils, dlog, machine};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::{CDateT, ClausesT, JSonObject, QVDicT, QVDRecordsT};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_insert, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_NEIGHBORS, C_MACHINE_NEIGHBORS_FIELDS};
use crate::lib::messaging_protocol::greeting::{create_handshake_request, create_here_is_new_neighbor, create_nice_to_meet_you};
use crate::lib::network::network_handler::i_push;

#[derive(Clone, Serialize, Deserialize, Debug)]
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
    machine_PGP_private_key = get_pub_email_info().m_PGP_private_key;
    machine_email = get_pub_email_info().m_address;
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

*/
}

pub fn add_a_new_neighbor_by_email(neighbor_email: String) -> (bool, String, i64)
{
    // the node only has an email address of new neighbor
    // so inserts it as a new neighbor
    let mp_code = machine().m_profile.m_mp_code.clone();
    let neighbor_name = neighbor_email.split("@").collect::<Vec<&str>>()[0].to_string();
    let neighbor_name = format!("{}({})", neighbor_name, mp_code);
    let neighbor_info = NeighborInfo { m_name: neighbor_name };
    let (status, msg) = add_a_new_neighbor(
        neighbor_email.clone(),
        constants::PUBLIC.to_string(),
        "".to_string(),
        mp_code.clone(),
        constants::YES.to_string(),
        neighbor_info,
        application().get_now());

    if !status {
        return (status, msg, 0);
    }

    // retrieve newly inserted neighbor
    let neighbors = get_neighbors(
        constants::PUBLIC,
        "",
        mp_code.as_str(),
        0,
        neighbor_email.as_str());

    if neighbors.len() == 0
    {
        return (false, "Failed in insert new neighbor in db!".to_string(), 0);
    }

    let neighbor_id = neighbors[0]["n_id"].parse::<i64>().unwrap();
    return (true, format!("New neighbor({}) was added to your network.", neighbor_email), neighbor_id);
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
    let mp_code = mp_code.to_string();
    let connection_type = connection_type.to_string();
    let neighbor_email = neighbor_email.to_string();

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
        C_MACHINE_NEIGHBORS,
        vec!["n_mp_code", "n_email"],
        vec![
            simple_eq_clause("n_mp_code", &mp_code.to_string()),
            simple_eq_clause("n_connection_type", &connection_type.to_string()),
            simple_eq_clause("n_email", &neighbor_email.to_string()),
        ],
        vec![],
        0,
        true, );

    if records.len() > 0
    {
        return if neighbor_public_key != ""
        {
            //update pgp key
            let now_ = application().get_now();
            let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
                ("n_pgp_public_key", &neighbor_public_key as &(dyn ToSql + Sync)),
                ("n_last_modified", &now_ as &(dyn ToSql + Sync)),
            ]);
            let clauses: ClausesT = vec![
                simple_eq_clause("n_mp_code", &mp_code),
                simple_eq_clause("n_connection_type", &connection_type),
                simple_eq_clause("n_email", &neighbor_email),
            ];

            q_update(
                C_MACHINE_NEIGHBORS,
                &values,
                clauses,
                true);
            (true, format!("The iPGP key for email({neighbor_email}) connection({connection_type}) profile({mp_code}) updated"))
        } else {
            (false, format!("The iPGP key for email({neighbor_email}) connection({connection_type}) profile({mp_code}) was missed"))
        };
    }

    if creation_date == "" {
        creation_date = application().get_now();
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

    let now_ = application().get_now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("n_mp_code", &mp_code as &(dyn ToSql + Sync)),
        ("n_email", &neighbor_email as &(dyn ToSql + Sync)),
        ("n_pgp_public_key", &neighbor_public_key as &(dyn ToSql + Sync)),
        ("n_is_active", &is_active as &(dyn ToSql + Sync)),
        ("n_connection_type", &connection_type as &(dyn ToSql + Sync)),
        ("n_creation_date", &creation_date as &(dyn ToSql + Sync)),
        ("n_info", &serialized_neighbor_info as &(dyn ToSql + Sync)),
        ("n_last_modified", &now_ as &(dyn ToSql + Sync))
    ]);
    dlog(
        &format!("going to add new Neighbor: {:?}", &values),
        constants::Modules::App,
        constants::SecLevel::Info);

    q_insert(
        C_MACHINE_NEIGHBORS,
        &values,
        true);

    return (
        true,
        "new Neighbor(".to_owned() + &neighbor_email + ") connection(" + &connection_type + ") profile(" + &mp_code + &") was added".to_string()
    );
}

//old_name_was getNeighbors
pub fn get_neighbors(
    neighbor_type: &str,
    connection_status: &str,
    mp_code: &str,
    n_id: i64,
    n_email: &str) -> QVDRecordsT
{
    let n_email = n_email.to_string();
    let mp_code = mp_code.to_string();
    let neighbor_type = neighbor_type.to_string();
    let connection_status = connection_status.to_string();

    let mut clauses: ClausesT = vec![];

    if connection_status != "" {
        clauses.push(simple_eq_clause("n_is_active", &connection_status));
    }

    if neighbor_type != "" {
        clauses.push(simple_eq_clause("n_connection_type", &neighbor_type));
    }

    if mp_code != "" {
        clauses.push(simple_eq_clause("n_mp_code", &mp_code));
    }

    if n_id != 0 {
        clauses.push(ModelClause {
            m_field_name: "n_id",
            m_field_single_str_value: &n_id as &(dyn ToSql + Sync),
            m_clause_operand: "=",
            m_field_multi_values: vec![],
        });
    }

    if n_email != "" {
        clauses.push(simple_eq_clause("n_email", &n_email));
    }

    let (_status, records) = q_select(
        C_MACHINE_NEIGHBORS,
        C_MACHINE_NEIGHBORS_FIELDS.iter().map(|&x| x).collect::<Vec<&str>>(),
        clauses,
        vec![
            &OrderModifier { m_field: "n_connection_type", m_order: "DESC" },
        ],
        0,
        true,
    );
    return records;
}

//old_name_was getActiveNeighbors
pub fn get_active_neighbors(mp_code: &str) -> QVDRecordsT
{
    let (_status, records) = q_select(
        C_MACHINE_NEIGHBORS,
        vec!["n_email", "n_pgp_public_key", "n_connection_type"],
        vec![
            simple_eq_clause("n_is_active", &constants::YES.to_string()),
            simple_eq_clause("n_mp_code", &mp_code.to_string())],
        vec![&OrderModifier { m_field: "n_connection_type", m_order: "DESC" }],
        0,
        true,
    );
    return records;
}

//old_name_was handshakeNeighbor
pub fn handshake_neighbor(n_id: i64, connection_type: &str) -> (bool, String)
{
    dlog(
        &format!("handshake Neighbor: id({}) connection({})", n_id, connection_type),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (
        status,
        title,
        sender_email,
        receiver_email,
        message) = create_handshake_request(connection_type, n_id);
    dlog(
        &format!("packet Generators.write Handshake: sender_email({}) receiver_email({}) title({}) message({})",
                 sender_email, receiver_email, title, message),
        constants::Modules::App,
        constants::SecLevel::Info);
    if !status
    { return (status, message); }

    // the concept is the node public email is propagated to more neighbors in order to strength connectivity,
    // but the node private email will be used as a second plan to defence against the any kind of spaming/DOS Attacks ...
    let status = i_push(
        &title,
        &message,
        &sender_email,
        &receiver_email);
    if status
    {
        return (true, "Done".to_string());
    } else {
        let msg = "Failed in push request to sending q! ".to_string();
        dlog(
            &format!("{}", msg),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, msg);
    }
}

//old_name_was parseHandshake
pub fn parse_handshake(
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
        pgp_public_key = remove_quotes(&message["PGPPubKey"].to_string());
    } else {
        pgp_public_key = "".to_string();
    }

    if connection_type == ""
    { return (false, true); }

    // just to be sure handshake happends ONLY ONE TIME for each email at the start
    // if user needs to change publickkey or ... she can send alternate messages like changeMyPublicKey(which MUST be signed with current key)
    // retreive sender's info
    let mut email_already_exist: bool = false;
    let sender_info: QVDRecordsT = get_neighbors(
        connection_type,
        "",
        "",
        0,
        sender_email);

    if sender_info.len() > 0
    {
        // some security logs
        email_already_exist = true;

        dlog(
            &format!("!!! the email in parse Handshake already exist ({})", sender_email),
            constants::Modules::Sec,
            constants::SecLevel::Warning);
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

    let (_status, pgp_public_key) = ccrypto::b64_decode(&pgp_public_key);

    if !email_already_exist
    {
        add_a_new_neighbor(
            sender_email.to_string(),
            connection_type.to_string(),
            pgp_public_key.clone(),
            "".to_string(),
            constants::YES.to_string(),
            NeighborInfo::new(),
            application().get_now());
    } else {
        if sender_info[0]["n_pgp_public_key"].to_string() == ""
        {
            let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
                ("n_pgp_public_key", &pgp_public_key as &(dyn ToSql + Sync)),
            ]);
            let n_id = sender_info[0]["n_id"].parse::<i64>().unwrap();
            q_update(
                C_MACHINE_NEIGHBORS,
                &update_values,
                vec![
                    ModelClause {
                        m_field_name: "n_id",
                        m_field_single_str_value: &n_id as &(dyn ToSql + Sync),
                        m_clause_operand: "=",
                        m_field_multi_values: vec![],
                    }
                ],
                false,
            );
        }
    }

    // FIXME: do it in Async mode
    // send response niceToMeetYou
    let (
        status,
        title,
        sender_email_,
        receiver_email,
        message_) = create_nice_to_meet_you(
        connection_type,
        sender_email,
        &pgp_public_key);

    dlog(
        &format!("create NiceToMeetYou status({}) message_: {}", status, message_),
        constants::Modules::App,
        constants::SecLevel::Info);

    if !status
    { return (false, true); }

    let _sent: bool = i_push(
        &title,
        &message_,
        &sender_email_,
        &receiver_email);

    // FIXME: do it in Async mode
    // broadcast the email to other neighbors
    if connection_type == constants::PUBLIC
    {
        flood_email_to_neighbors(sender_email, &pgp_public_key);
    }

    return (true, true);
}

//old_name_was floodEmailToNeighbors
pub fn flood_email_to_neighbors(
    the_new_neighbor_email: &String,
    the_new_neighbor_pgp_public_key: &String) -> bool
{
    let mut the_new_neighbor_pgp_public_key = the_new_neighbor_pgp_public_key.to_string();
    let the_new_neighbor_email = the_new_neighbor_email.to_string();
    dlog(
        &format!("flood this Email To Neighbors: {}", the_new_neighbor_email),
        constants::Modules::App,
        constants::SecLevel::Info);

    if the_new_neighbor_pgp_public_key == ""
    {
        let email_info: QVDRecordsT = get_neighbors(
            constants::PUBLIC,
            "",
            "",
            0,
            the_new_neighbor_email.as_str());
        if email_info.len() == 0
        {
            dlog(
                &format!("email({}) doesn't exist as a neighbor!", &the_new_neighbor_email),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }
        the_new_neighbor_pgp_public_key = email_info[0]["n_pgp_public_key"].to_string();
    }

    //  * avoiding duplicate sending email
    //  * [{vertice: "neighborEmail->targetEmail", date:"presenting date"}]

    let mut already_presented_neighbors: Vec<NeighborPresentation> = machine().m_profile.m_mp_settings.m_already_presented_neighbors.clone();
    let al_pr = &already_presented_neighbors.iter().map(|x| x.m_vertice.clone()).collect::<Vec<String>>().join(",");
    dlog(
        &format!("Already Presented to these Neighbors {:?}", al_pr),
        constants::Modules::App,
        constants::SecLevel::Info);

    let active_neighbors: QVDRecordsT = get_neighbors(
        constants::PUBLIC,
        constants::YES,
        &machine().get_selected_m_profile(),
        0,
        "");

    dlog(
        &format!("Active Neighbors to flood email to neighbors: {:?}", active_neighbors),
        constants::Modules::App,
        constants::SecLevel::Info);

    let mut vertice: String;
    let mut is_already_sent: bool;
    for neighbor in active_neighbors
    {
        let n_email: String = neighbor["n_email"].to_string();
        if n_email == the_new_neighbor_email
        {
            continue;   // not presenting machine to itself
        }

        is_already_sent = false;
        vertice = the_new_neighbor_email.to_owned() + "__to__" + &n_email;
        for vert in &already_presented_neighbors
        {
            dlog(
                &format!("vertices {}", vert.m_vertice),
                constants::Modules::App,
                constants::SecLevel::Info);

            if vert.m_vertice == vertice
            {
                dlog(
                    &format!("!!! the email already broadcast {}", vertice),
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
                m_date: application().get_now(),
            });

            let machine_address = machine().get_pub_email_info().m_address.clone();
            let machine_prv_key = machine().get_pub_email_info().m_pgp_private_key.clone();

            let (
                status,
                title,
                sender_email,
                receiver_email,
                message) = create_here_is_new_neighbor(
                &constants::PUBLIC.to_string(),
                &machine_address,
                &machine_prv_key,
                &n_email,
                &neighbor["n_pgp_public_key"].clone().to_string(),
                &the_new_neighbor_email,  //newNeighborEmail
                &the_new_neighbor_pgp_public_key);

            dlog(
                &format!("packet ready to flood email to neigbor: status({}) title({}) sender_email({}) receiver_email({}) message({}) ",
                         status, title, sender_email, receiver_email, message),
                constants::Modules::App,
                constants::SecLevel::Info);

            dlog(
                &format!("the machine presents({}) to ({})", &the_new_neighbor_email, n_email),
                constants::Modules::App,
                constants::SecLevel::Info);

            i_push(
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


//old_name_was parseNiceToMeetYou
pub fn parse_nice_to_meet_you(
    sender_email: &String,
    message: &JSonObject,
    connection_type: &String) -> (bool, bool)
{
    dlog(
        &format!("parse Nice To Meet You connection_type({}) sender({}) message: {}", connection_type, sender_email, message),
        constants::Modules::App,
        constants::SecLevel::Info);

    let mut email: String = "".to_string();
    if !message["email"].is_null()
    {
        email = remove_quotes(&message["email"].to_string());
    }

    let mut sender_pgp_public_key: String = "".to_string();
    if !message["PGPPubKey"].is_null()
    {
        sender_pgp_public_key = remove_quotes(&message["PGPPubKey"].to_string());
    }

    let mut sender_backer_address: String = "".to_string();
    if !message["backerAddress"].is_null()
    {
        sender_backer_address = remove_quotes(&message["backerAddress"].to_string());
    }

    // just to be sure handshake happends ONLY ONE TIME for each email at the start
    // if user needs to change publickkey or ...
    // she can send alternate messages like changeMyPublicKey(which MUST be signed with current key)
    // retrieve sender's info
    let sender_info: QVDRecordsT = get_neighbors(
        "",
        "",
        "",
        0,
        sender_email);

    if sender_info.len() == 0
    {
        // some security logs
        dlog(
            &format!("!!! Machine has not this sender_email({}) as a neighbor", sender_email),
            constants::Modules::Sec,
            constants::SecLevel::Warning);
        return (false, true);
    }

    if (sender_email == "") || (sender_pgp_public_key == "")
    {
        dlog(
            &format!("!!! invalid sender_email or PGPPubKey received from neighbor sender_email({}) sender_pgp_public_key({}) as a neighbor", sender_email, sender_pgp_public_key),
            constants::Modules::Sec,
            constants::SecLevel::Warning);
        return (false, true);
    }
    let (status, b64_dec_pgp_public_key) = ccrypto::b64_decode(&sender_pgp_public_key);
    if !status
    {
        dlog(
            &format!("Failed in pgp b64 decryption! sender({})", sender_email),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, true);
    }
    sender_pgp_public_key = b64_dec_pgp_public_key;

    let mut n_info = cutils::serialize_json(&json!({}));
    if sender_backer_address != ""
    {
        n_info = cutils::serialize_json(&json!({"backerAddress": sender_backer_address}));
    }

    let last_modified = application().get_now();
    let mut updates: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("n_info", &n_info as &(dyn ToSql + Sync)),
        ("n_pgp_public_key", &sender_pgp_public_key as &(dyn ToSql + Sync)),
        ("n_last_modified", &last_modified as &(dyn ToSql + Sync))
    ]);



    // update neighbor info's PGP public key
    let c1 = simple_eq_clause("n_email", sender_email);
    q_update(
        C_MACHINE_NEIGHBORS,
        &updates,
        vec![c1],
        false,
    );

    // TODO: publish this email to my neighbors

    return (true, true);
}