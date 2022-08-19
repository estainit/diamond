use serde_json::json;
use crate::{constants, cutils, dlog, machine};
use crate::lib::custom_types::{QVDicT, QVDRecordsT};
use crate::lib::pgp::cpgp::{pgp_encrypt, wrap_pgp_envelope};
use crate::lib::ccrypto;
use crate::lib::machine::machine_neighbor::getNeighbors;

pub fn createHandshakeRequest(
    connection_type: &String,
    receiver_id: &String) -> (bool, String, String, String, String)
{
    dlog(
        &format!("generate Handshake packet for: {} {}", connection_type, receiver_id),
        constants::Modules::App,
        constants::SecLevel::Info);


    if connection_type == ""
    {
        dlog(
            &format!("The connection_type can not be empty"),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }

    if receiver_id == ""
    {
        dlog(
            &format!("The receiver_id can not be empty!"),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }

    let receivers_info: QVDRecordsT = getNeighbors(
        "",
        "",
        "",
        receiver_id,
        ""
    );
    if receivers_info.len() != 1
    {
        dlog(
            &format!("Wrong receiver id! {:?}", receivers_info),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }

    let receiver_info: QVDicT = receivers_info[0].clone();
    let machine_settings = machine().get_profile();
    let mut email: String;
    if connection_type == constants::PRIVATE
    {
        email = machine_settings.m_mp_settings.m_private_email.m_address
    } else {
        email = machine_settings.m_mp_settings.m_public_email.m_address;
    }

    let pgp_public_key: String;
    if connection_type == constants::PRIVATE
    {
        pgp_public_key = machine_settings.m_mp_settings.m_private_email.m_pgp_public_key
    } else {
        pgp_public_key = machine_settings.m_mp_settings.m_public_email.m_pgp_public_key;
    }


    let email_body: String = cutils::serializeJson(&json!({
         "type": constants::message_types::HANDSHAKE,
         "mVer": "0.0.0",
         "connectionType": connection_type,
         "email": email, // sender email
         // the node public key is used to secure comiunication between nodes
         "PGPPubKey": ccrypto::b64_encode(&pgp_public_key)  //sender's iPGP public key
    }));

    dlog(
        &format!("write Handshake email_body: {}", email_body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (pgp_status, pgp_msg) = pgp_encrypt(
        &email_body,
        &"".to_string(), // sender private key
        &"".to_string(), // receiver Public Key
        &"".to_string(), // secert key
        &"".to_string(), // iv
        false, // should compress
        false, // should sign
    );
    if !pgp_status
    {
        dlog(
            &format!("PGP encryption failed"),
            constants::Modules::App,
            constants::SecLevel::Fatal);
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }

    let mut final_message: String = cutils::breakByBR(&pgp_msg, 128);
    final_message = wrap_pgp_envelope(&final_message);

    return (
        true,
        "Handshake from a new neighbor".to_string(),
        email,  //sender
        receiver_info["n_email"].to_string(), //target
        final_message
    );
}

pub fn createNiceToMeetYou(
    connection_type: &String,
    receiver_email: &String,
    receiver_pgp_public_key: &String) -> (bool, String, String, String, String)
{
    dlog(
        &format!("packet Generators .write NiceToMeetYou args: connection_type({}) receiver_email({})", connection_type, receiver_email),
        constants::Modules::App,
        constants::SecLevel::Info);

    if (receiver_email == "") || (receiver_pgp_public_key == "")
    {
        dlog(
            &format!("niceToMeetYou missed receiver_email receiver_email({}) or PGP key", receiver_email),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }

    if connection_type == ""
    {
        dlog(
            &format!("In niceToMeetYou, the connection_type can not be ({})", connection_type),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }
    let default_message_version = "0.0.0";
    let json_email_body = json!({
        "type": constants::message_types::NICETOMEETYOU,
        "connectionType": connection_type ,
        "mVer": default_message_version ,
        "email": machine().getPubEmailInfo().m_address ,
        "PGPPubKey": ccrypto::b64_encode(&machine().getPubEmailInfo().m_pgp_public_key)});
    let email_body: String = cutils::serializeJson(&json_email_body);
    dlog(
        &format!("write NiceToMeetYou email_body1: {}", email_body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (_pgp_status, encrypted_message) = pgp_encrypt(
        &email_body,
        &"".to_string(),
        receiver_pgp_public_key,
        &"".to_string(),
        &"".to_string(),
        false,
        false);

    let mut email_body = cutils::breakByBR(&encrypted_message, 128);
    email_body = wrap_pgp_envelope(&email_body);
    dlog(
        &format!("packetGenerators.write NiceToMeetYou email_body2: {}", email_body),
        constants::Modules::App,
        constants::SecLevel::Info);

    return (
        true,
        "niceToMeetYou".to_string(), // title
        machine().getPubEmailInfo().m_address.clone(), // sender
        receiver_email.to_string(),
        email_body);

    //TODO after successfull sending must save some part the result and change the email to confirmed
}

pub fn createHereIsNewNeighbor(
    connection_type: &String,
    machine_email: &String,
    machine_PGP_private_key: &String,
    receiver_email: &String,
    receiver_pgp_public_key: &String,
    new_neighbor_email: &String,
    new_neighbor_pgp_public_key: &String) -> (bool, String, String, String, String)
{
    dlog(
        &format!("create Here Is New Neighbor args: connection_type({})", connection_type),
        constants::Modules::App,
        constants::SecLevel::Info);
    if
    (new_neighbor_email == "") ||
        (new_neighbor_pgp_public_key == "") ||
        (receiver_email == "") ||
        (receiver_pgp_public_key == "")
    {
        return (false, "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }

    let default_mVer = "0.0.0";
    let json_email_body = json!({
        "type": constants::message_types::HEREISNEWNEIGHBOR,
        "mVer": default_mVer ,
        "connectionType": connection_type ,
        "newNeighborEmail": new_neighbor_email ,
        "newNeighborPGPPubKey": ccrypto::b64_encode(new_neighbor_pgp_public_key)});  //sender's iPGP public key

    let email_body: String = cutils::serializeJson(&json_email_body);


//  let params = {
//   shouldSign: true, // in real world you can not sign an email in negotiation step in which the receiver has not your pgp public key
//   shouldCompress: true,
//   message: email_body,
//   sendererPrvKey: machine_PGP_private_key,
//   receiverPubKey: receiver_pgp_public_key
//  }

    dlog(
        &format!("write Here Is New Neighbor going to pgp mVer({}) connection_type({}) new_neighbor_email({})", default_mVer, connection_type, new_neighbor_email),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (_pgp_status, message) = pgp_encrypt(
        &email_body,
        machine_PGP_private_key,
        receiver_pgp_public_key,
        &"".to_string(),
        &"".to_string(),
        true,
        true);
    let mut message = cutils::breakByBR(&message, 128);
    message = wrap_pgp_envelope(&message);

    return (
        true,
        "hereIsNewNeighbor".to_string(),  // title
        machine_email.to_string(),  // sender
        receiver_email.to_string(),
        email_body  // message
    );
    // TODO after successfull sending must save some part the result and change the email
    // to confirmed
}

