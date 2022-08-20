use std::collections::HashMap;
use actix_web::cookie::time::format_description::parse;
use postgres::types::ToSql;
use serde_json::json;
use crate::{constants, cutils, dlog, machine};
use crate::lib::custom_types::{JSonObject, QVDRecordsT};
use crate::lib::database::abs_psql::{q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_CPACKET_TICKETING, C_CPACKET_TICKETING_FIELDS};
use crate::lib::machine::machine_neighbor::{add_a_new_neighbor, get_neighbors, handshake_neighbor, NeighborInfo};
use crate::lib::machine::machine_profile::EmailSettings;
use crate::lib::pgp::cpgp::{CPGPMessage};
use crate::lib::pgp::cpgp_decrypt::pgp_decrypt;

//old_name_was decryptAndParsePacketSync
pub fn decrypt_and_parse_packet(
    sender: &String,
    receiver: &String,
    file_name: &String,
    message: &String,
) -> (bool, String, JSonObject)
{
    let mut connection_type: String = "".to_string();
    let mut message_obj: JSonObject = json!({});

    if message == ""
    {
        dlog(
            &format!("CPacket has empty message! sender({sender}) receiver({receiver}) file_name({file_name})"),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, connection_type, message_obj);
    }

    // later again try this message or purge it after a certain try and fail. but not blocked on this message.
    t_log(file_name);

    // retrieve sender's info
    let sender_info: QVDRecordsT = get_neighbors(
        "",
        "",
        "",
        0,
        sender);
    let mut sender_public_key: String = "".to_string();

    if sender_info.len() > 0
    {
        sender_public_key = sender_info[0]["n_pgp_public_key"].to_string();
    } else {
        //     * sender is not in my neighbors, so i add it as a new neighbor
        // * TODO: probably security issue! machine must not add all new random emails.
        // * instead must list them and user decides about that
        // * so it must be implemented ASAP

        dlog(
            &format!("Unknown email addresse sent msg, so add it automatically as a new neighbor({})", sender),
            constants::Modules::Sec,
            constants::SecLevel::Info);

        add_a_new_neighbor(
            sender.clone(),
            constants::PUBLIC.to_string(),
            "".to_string(),
            machine().get_selected_m_profile(),
            constants::YES.to_string(),
            NeighborInfo::new(),
            cutils::get_now(),
        );

// retrieve id of newly inserted email
        let new_neighbor_info: QVDRecordsT = get_neighbors(
            "",
            "",
            "",
            0,
            sender);
        if new_neighbor_info.len() == 0
        {
            dlog(
                &format!("Couldn't insert unknown email as a new neighbor({})", sender),
                constants::Modules::App,
                constants::SecLevel::Info);

            return (false, connection_type, message_obj);
        }

        // and now do handshake (possible in async mode)
        let (status, msg) = handshake_neighbor(new_neighbor_info[0]["n_id"].parse::<i64>().unwrap_or(0), constants::PUBLIC);
        if status
        {
            dlog(
                &format!("Handshake Done neighbor({}/{}): {}", sender, receiver, msg),
                constants::Modules::App,
                constants::SecLevel::Info);
        } else {
            dlog(
                &format!("Failed Handshake neighbor({}/{}): {}", sender, receiver, msg),
                constants::Modules::App,
                constants::SecLevel::Error);
        }
    }
// if (sender_public_Key == '')
//     sender_public_Key = null;    // for new neighbors

    let machine_private_pgp_key: String;
    let machine_profile = machine().get_profile();


    if receiver.to_string() == machine_profile.m_mp_settings.m_private_email.m_address
    {
        machine_private_pgp_key = machine_profile.m_mp_settings.m_private_email.m_pgp_private_key.clone();
        connection_type = constants::PRIVATE.to_string();
    } else if receiver.to_string() == machine_profile.m_mp_settings.m_public_email.m_address
    {
        machine_private_pgp_key = machine_profile.m_mp_settings.m_public_email.m_pgp_private_key.clone();
        connection_type = constants::PUBLIC.to_string();
    } else {
        dlog(
            &format!("in parse PacketSync unknown email cpacket: {}", receiver),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, connection_type, message_obj);
    }


    let decrypt_res: CPGPMessage;
    decrypt_res = pgp_decrypt(message, &machine_private_pgp_key, &sender_public_key);

    dlog(
        &format!(
            "decrypt PGP res: m_decryption_status({}) \
                 m_decryption_status({}) \
                 m_is_authenticated({}) \
                 m_is_signed({}) \
                 m_is_verified({}) \
                 m_is_compressed({}) \
                ", decrypt_res.m_decryption_status, decrypt_res.m_decryption_status, decrypt_res.m_is_authenticated,
            decrypt_res.m_is_signed, decrypt_res.m_is_verified, decrypt_res.m_is_compressed
        ),
        constants::Modules::App,
        constants::SecLevel::Trace);

    if !decrypt_res.m_decryption_status
    {
        dlog(
            &format!("Failed decrypt c packet msg1 !"),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, connection_type, message_obj);
    }

    if decrypt_res.m_is_signed && !decrypt_res.m_is_verified
    {
        dlog(
            &format!("Dropped Invalid Or Insecure Message (Wrong signature)."),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, connection_type, message_obj);
    }

    if (decrypt_res.m_message == "")
    {
        dlog(
            &format!("Dropped Invalid Or Insecure Message (empty message body)."),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, connection_type, message_obj);
    }

    let (status, message_obj) = cutils::controlled_str_to_json(&decrypt_res.m_message);
    if !status {
        dlog(
            &format!("Failed parse msg {}", decrypt_res.m_message),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, connection_type, message_obj);
    }

    return (true, connection_type, message_obj);
}


//  -  -  -  -  -  CPacket ticketing
//old_name_was tLog
pub fn t_log(file_name: &String) -> bool
{
    let res: QVDRecordsT = i_read(file_name);
    if res.len() == 0
    {
        i_create(file_name);
    } else {
        iUpdate(file_name);
    }
    return true;
}

//old_name_was iRead
pub fn i_read(file_name: &String) -> QVDRecordsT
{
    let (status, records) = q_select(
        C_CPACKET_TICKETING,
        C_CPACKET_TICKETING_FIELDS.iter().map(|&x| x).collect::<Vec<&str>>(),
        vec![simple_eq_clause("msg_file_id", file_name)],
        vec![],
        0,
        false,
    );
    return records;
}
/*

int CPacketHandler::getTry(const String &file_name)
{
QueryRes res = DbModel::select(
stbl_cpacket_ticketing,
{"msg_try_count"},
{{"msg_file_id", file_name}}
);

//          let res = db.scustom('SELECT msg_try_count FROM i_message_ticketing WHERE =$1', [file_name]);
if (res.records.len() > 0)
return res.records[0]["msg_try_count"].toInt();

return 0;
}

*/
pub fn iUpdate(_file_name: &String) -> bool
{
// TODO: implement it ASAP
//  DbModel::customQuery("",
//    "UPDATE " + stbl_cpacket_ticketing + " SET msg_last_modified=:msg_last_modified, msg_try_count=msg_try_count+1 WHERE msg_file_id=:msg_file_id",
//  {{}}
//  );
    return true;
}

//old_name_was iCreate
pub fn i_create(file_id: &String) -> bool
{
    let zero: i32 = 0;
    let now_ = cutils::get_now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("msg_file_id", file_id as &(dyn ToSql + Sync)),
        ("msg_try_count", &zero as &(dyn ToSql + Sync)),
        ("msg_creation_date", &now_ as &(dyn ToSql + Sync)),
        ("msg_last_modified", &now_ as &(dyn ToSql + Sync)),
    ]);
    q_insert(
        C_CPACKET_TICKETING,
        &values,
        true);
    return true;
}


