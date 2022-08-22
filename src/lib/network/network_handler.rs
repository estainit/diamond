use std::collections::HashMap;
use crate::{application, ccrypto, constants, cutils, dlog, get_value, machine};
use crate::lib::file_handler::file_handler::write_email_as_file;
use crate::lib::k_v_handler::upsert_kvalue;
use crate::lib::network::email::send_email_wrapper;

//old_name_was iPush
pub fn i_push(
    title: &String,
    message: &String,
    sender: &String,
    receiver: &String) -> bool
{
    dlog(
        &format!("iPush args: title({}) sender({}) receiver({}) message({})", title, sender, receiver, message),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let is_custom = false;
    let mut to_send_message: String = message.clone();
    if is_custom
    {
        to_send_message = constants::message_tags::ENVELOPE_CUSTOM_START.to_string() + &to_send_message + &constants::message_tags::ENVELOPE_CUSTOM_END;
    }

    let mut email_body: String = application().get_now() + constants::NL;
    email_body += &*("time: ".to_owned() + &application().get_now_sss() + &constants::NL);
    email_body += &(constants::message_tags::SENDER_START_TAG.to_owned() + sender + constants::message_tags::SENDER_END_TAG + constants::NL);
    email_body += &(constants::message_tags::RECEIVE_START_TAG.to_owned() + receiver + constants::message_tags::RECEIVE_END_TAG + constants::NL);
    email_body += &(to_send_message.clone() + &constants::NL);
    let mut email_hash: String = cutils::hash16c(&ccrypto::keccak256(&(sender.to_owned() + receiver + &to_send_message)));
    email_body += &(constants::message_tags::HASH_START_TAG.to_owned() + &email_hash.clone() + constants::message_tags::HASH_END_TAG + constants::NL);

    if machine().m_use_hard_disk_as_a_buffer
    {
        // create emails and write it on local hard drive
        // create an email copy in local hard drive
        // in such a way user can send email manually
        // she can sign some transactions and create a proper block, but not send it immideately
        // keeps it in safe place and when it need just send it manually to one or more backers
        let status = write_email_as_file(
            title,
            sender,
            receiver,
            &email_body);
        dlog(
            &format!("write on HD res: {}", status),
            constants::Modules::App,
            constants::SecLevel::Trace);
    }

    if !machine().m_email_is_active
    { return true; }

    // avoid duplicate sending
    let mut sent_emails_obj: HashMap<String, String>;
    let sent_emails = get_value("SENT_EMAILS"); //sentEmails -> sent_emails & SENT_EMAILS
    if sent_emails == ""
    {
        sent_emails_obj = HashMap::new();
        let serialized = serde_json::to_string(&sent_emails_obj).unwrap();
        upsert_kvalue("sent_emails", &serialized, false);
    } else {
        sent_emails_obj = serde_json::from_str(&sent_emails).unwrap();
    }

    let mut is_sent: bool = sent_emails_obj
        .keys()
        .cloned()
        .collect::<Vec<String>>()
        .contains(&email_hash);

    if is_sent
    {
        dlog(
            &format!("email already was sent TO: {} + ({}) hash({})", receiver, title, email_hash),
            constants::Modules::App,
            constants::SecLevel::Trace);
        return true;
    } else {
        sent_emails_obj.insert(email_hash, application().get_now());
        upsert_kvalue(
            "SENT_EMAILS",
            &serde_json::to_string(&sent_emails_obj).unwrap(),
            false);
    }

    let mut refresh_sents: HashMap<String, String> = HashMap::new();
    let back_in_time = application().get_cycle_by_minutes();
    let now_=application().get_now();
    let c_date: String = application().minutes_before(
        back_in_time,
        &now_);
    for a_hash in sent_emails_obj.keys()
        .cloned()
        .collect::<Vec<String>>() {
        if sent_emails_obj[&a_hash].to_string() > c_date {
            refresh_sents.insert(a_hash.clone(), sent_emails_obj[&a_hash].clone());
        }
    }
    upsert_kvalue("SENT_EMAILS", &serde_json::to_string(&refresh_sents).unwrap(), false);

    // send email to via SMTP server
    let email_status: bool = send_email_wrapper(
        sender,
        title,
        email_body,
        receiver,
    );
    return email_status;
}

