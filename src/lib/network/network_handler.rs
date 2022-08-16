use std::collections::HashMap;
use serde_json::json;
use crate::{ccrypto, constants, cutils, dlog, get_value};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::JSonObject;
use crate::lib::file_handler::file_handler::writeEmailAsFile;
use crate::lib::k_v_handler::upsert_kvalue;
use crate::lib::network::email::sendEmailWrapper;

pub fn iPush(
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
        to_send_message = constants::message_tags::customStartEnvelope.to_string() + &to_send_message + &constants::message_tags::customEndEnvelope;
    }

    let mut email_body: String = cutils::get_now() + constants::NL;
    email_body += &*("time: ".to_owned() + &cutils::get_now_sss() + &constants::NL);
    email_body += &(constants::message_tags::senderStartTag.to_owned() + sender + constants::message_tags::senderEndTag + constants::NL);
    email_body += &(constants::message_tags::receiverStartTag.to_owned() + receiver + constants::message_tags::receiverEndTag + constants::NL);
    email_body += &(to_send_message.clone() + &constants::NL);
    let mut email_hash: String = cutils::hash16c(&ccrypto::keccak256(&(sender.to_owned() + receiver + &to_send_message)));
    email_body += &(constants::message_tags::hashStartTag.to_owned() + &email_hash.clone() + constants::message_tags::hashEndTag + constants::NL);

    if constants::DO_HARDCOPY_OUTPUT_EMAILS
    {
        // create emails and write it on local hard drive
        // create an email copy in local hard drive
        // in such a way user can send email manually
        // she can sign some transactions and create a proper block, but not send it immideately
        // keeps it in safe place and when it need just send it manually to one or more backers
        let (status) = writeEmailAsFile(
            title,
            sender,
            receiver,
            &email_body);
        dlog(
            &format!("write on HD res: {}", status),
            constants::Modules::App,
            constants::SecLevel::Trace);
    }

    if !constants::EMAIL_IS_ACTIVE
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
        sent_emails_obj.insert(email_hash, cutils::get_now());
        upsert_kvalue(
            "SENT_EMAILS",
            &serde_json::to_string(&sent_emails_obj).unwrap(),
            false);
    }

    let mut refresh_sents: HashMap<String, String> = HashMap::new();
    let c_date: String = cutils::minutes_before(
        cutils::get_cycle_by_minutes(),
        &cutils::get_now());
    for aHash in sent_emails_obj.keys()
        .cloned()
        .collect::<Vec<String>>() {
        if sent_emails_obj[&aHash].to_string() > c_date {
            refresh_sents.insert(aHash.clone(), sent_emails_obj[&aHash].clone());
        }
    }
    upsert_kvalue("SENT_EMAILS", &serde_json::to_string(&refresh_sents).unwrap(), false);

    // send email to via SMTP server
    let email_status: bool = sendEmailWrapper(
        sender,
        title,
        &email_body,
        receiver,
    );
    return email_status;
}

