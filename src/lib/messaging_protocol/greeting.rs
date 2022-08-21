use serde_json::json;
use crate::{constants, cutils, dlog, machine};
use crate::lib::custom_types::{QVDicT, QVDRecordsT};
use crate::lib::pgp::cpgp::{wrap_pgp_envelope};
use crate::lib::ccrypto;
use crate::lib::machine::machine_neighbor::get_neighbors;
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::pgp::cpgp_encrypt::pgp_encrypt;

//old_name_was createHandshakeRequest
pub fn create_handshake_request(
    connection_type: &str,
    receiver_id: i64) -> (bool, String, String, String, String)
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

    if receiver_id == 0
    {
        dlog(
            &format!("The receiver_id can not be zero!"),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "".to_string(), "".to_string(), "".to_string(), "The receiver_id can not be zero!".to_string());
    }

    let receivers_info: QVDRecordsT = get_neighbors(
        "",
        "",
        "",
        receiver_id,
        "",
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
    let email: String;
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

    let (code, body) = make_a_packet(
        vec![
            json!({
                "cdType": constants::card_types::HANDSHAKE,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "connectionType": connection_type,
                "email": email, // sender email
                // the node public key is used to secure communication between nodes
                "PGPPubKey": ccrypto::b64_encode(&pgp_public_key)  //sender's iPGP public key
            }),
        ],
        constants::DEFAULT_PACKET_TYPE,
        constants::DEFAULT_PACKET_VERSION,
        cutils::get_now()
    );
    dlog(
        &format!("prepared handshake packet, before insert into DB code({}) to ({}): {}",code, email, body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (pgp_status, pgp_msg) = pgp_encrypt(
        &body,
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

    let mut final_message: String = cutils::break_by_br(&pgp_msg, 128);
    final_message = wrap_pgp_envelope(&final_message);

    return (
        true,
        "Handshake from a new neighbor".to_string(),
        email,  //sender
        receiver_info["n_email"].to_string(), //target
        final_message
    );
}

//old_name_was createNiceToMeetYou
pub fn create_nice_to_meet_you(
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

    let (code, body) = make_a_packet(
        vec![
            json!({
                "cdType": constants::card_types::NICE_TO_MEET_YOU,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "connectionType": connection_type,
                "email": machine().get_pub_email_info().m_address,
                // the node public key is used to secure communication between nodes
                "PGPPubKey": ccrypto::b64_encode(&machine().get_pub_email_info().m_pgp_public_key)
            }),
        ],
        constants::DEFAULT_PACKET_TYPE,
        constants::DEFAULT_PACKET_VERSION,
        cutils::get_now()
    );
    dlog(
        &format!("prepared Nice To Meet You packet, before insert into DB code({}) to ({}): {}",code, receiver_email, body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (_pgp_status, encrypted_message) = pgp_encrypt(
        &body,
        &"".to_string(),
        receiver_pgp_public_key,
        &"".to_string(),
        &"".to_string(),
        false,
        false);

    let mut email_body = cutils::break_by_br(&encrypted_message, 128);
    email_body = wrap_pgp_envelope(&email_body);
    dlog(
        &format!("packetGenerators.write NiceToMeetYou email_body2: {}", email_body),
        constants::Modules::App,
        constants::SecLevel::Info);

    return (
        true,
        "niceToMeetYou".to_string(), // title
        machine().get_pub_email_info().m_address.clone(), // sender
        receiver_email.to_string(),
        email_body);

    //TODO after successfull sending must save some part the result and change the email to confirmed
}

//old_name_was createHereIsNewNeighbor
pub fn create_here_is_new_neighbor(
    connection_type: &String,
    machine_email: &String,
    machine_pgp_private_key: &String,
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

    let (code, body) = make_a_packet(
        vec![
            json!({
                "cdType": constants::card_types::HERE_IS_NEW_NEIGHBOR,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "connectionType": connection_type,
                "newNeighborEmail": new_neighbor_email ,
                "newNeighborPGPPubKey": ccrypto::b64_encode(new_neighbor_pgp_public_key)
            }),
        ],
        constants::DEFAULT_PACKET_TYPE,
        constants::DEFAULT_PACKET_VERSION,
        cutils::get_now()
    );
    dlog(
        &format!("prepared here is a new neighbor packet, before insert into DB code({}) to ({}): {}",code, new_neighbor_email, body),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (_pgp_status, pgp_message) = pgp_encrypt(
        &body,
        machine_pgp_private_key,
        receiver_pgp_public_key,
        &"".to_string(),
        &"".to_string(),
        true,
        true);
    let mut final_message = cutils::break_by_br(&pgp_message, 128);
    final_message = wrap_pgp_envelope(&final_message);

    return (
        true,
        "hereIsNewNeighbor".to_string(),  // title
        machine_email.to_string(),  // sender
        receiver_email.to_string(),
        final_message  // message
    );
    // TODO after successfull sending must save some part the result and change the email
    // to confirmed
}

