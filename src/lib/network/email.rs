use crate::{constants, dlog, machine};
use crate::lib::machine::machine_profile::EmailSettings;

use lettre::{transport::smtp::{
    authentication::{Credentials, Mechanism},
    PoolConfig,
}, Message, SmtpTransport, Transport, Address};
use lettre::message::Mailbox;
use lettre::transport::smtp::response::{Category, Code, Detail, Response, Severity};
use serde::de::Unexpected::Option;

/*
// js name was fetchPrvEmailAndWriteOnHardDisk
bool EmailHandler::popPrivateEmail()
{
//  let machineInfo = machine.getMProfileSettingsSync();
//  // console.log('machineInfo', machineInfo);
//  let prvEmail = machineInfo.prvEmail;

//  // fetch private inbox
//  let params = {
//    emailAddress: prvEmail.address,
//    password: prvEmail.pwd,
//    host: prvEmail.incomingMailServer,
//    port: prvEmail.incomeIMAP,
//    funcMode: 'readUNSEENs'
//  }
//  if (
//    utils._nilEmptyFalse(params.emailAddress) ||
//    utils._nilEmptyFalse(params.password) ||
//    utils._nilEmptyFalse(params.host) ||
//    utils._nilEmptyFalse(params.port)
//  ) {
//    msg = `missed some parameter of Private IMAP fetching ${params}`;
//    clog.app.info(`${msg} `);
//    return { err: true, msg }
//  } else {
//    let popRes = await emailHandler.IMAPFetcher.fetchInbox(params);
//    clog.app.info(`${popCounter}. incomeIMAP prv mailbox ${popRes}`);
//    return popRes;
//  }
  return true;
}

// js name was fetchPubEmailAndWriteOnHardDisk
bool EmailHandler::popPublicEmail()
{
//  clog.app.info(`fetch Pub Email AndWriteOnHardDisk`);
//  let msg;
//  popCounter += 1;
//  let machineInfo = machine.getMProfileSettingsSync();
//  // console.log('machineInfo', machineInfo);
//  let pubEmail = machineInfo.pubEmail;
//  setTimeout(NetListener.fetchPubEmailAndWriteOnHardDisk, 60000 * pubEmail.fetchingIntervalByMinute);

//  // fetch private inbox
//  let params = {
//    emailAddress: pubEmail.address,
//    password: pubEmail.pwd,
//    host: pubEmail.incomingMailServer,
//    port: pubEmail.incomeIMAP,
//    funcMode: 'readUNSEENs'
//  }
//  if (
//    utils._nilEmptyFalse(params.emailAddress) ||
//    utils._nilEmptyFalse(params.password) ||
//    utils._nilEmptyFalse(params.host) ||
//    utils._nilEmptyFalse(params.port)
//  ) {
//    msg = `missed some parameter of Public IMAP fetching`;
//    console.log(`msg`, msg, params);
//    clog.app.info(`msg ${msg} ${params}`);
//    return { err: true, msg }
//  }
//  let popRes = await emailHandler.IMAPFetcher.fetchInbox(params);

//  clog.app.info(`${popCounter}. incomeIMAP pub mailbox ${popRes}`);

  return true;
}


void EmailHandler::loopEmailPoper()
{
  String thread_prefix = "email_poper_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    popPrivateEmail();
    popPublicEmail();

    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getPopEmailGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Email Poper");
}
*/


//old_name_was sendPrivateEmail
pub fn send_private_email() -> bool
{
    return true;
}

//old_name_was sendPublicEmail
pub fn send_public_email() -> bool
{
    return true;
}

/*
void EmailHandler::loopEmailSender()
{
  String thread_prefix = "email_sender_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    sendPrivateEmail();
    sendPublicEmail();

    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getSendEmailGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Email Sender");
}

*/
pub fn sendEmailWrapper(
    sender_: &String,
    title: &String,
    message: String,
    receiver: &String) -> bool
{
    dlog(
        &format!("send EmailWrapper args: sender({sender_}) receiver({receiver}) title({title})"),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let machine_public_email: EmailSettings = machine().getPubEmailInfo().clone();
    let machine_private_email: EmailSettings = machine().getPrivEmailInfo().clone();

    let mut sender: String;
    let mut pass: String;
    let mut host: String;
    let mut port: u16;

    if machine_private_email.m_address == sender_.to_string()
    {
        sender = machine_private_email.m_address.clone();
        pass = machine_private_email.m_password.clone();
        host = machine_private_email.m_outgoing_mail_server.clone();
        port = machine_private_email.m_outgoing_smtp.parse::<u16>().unwrap();
    } else {
        sender = machine_public_email.m_address.clone();
        pass = machine_public_email.m_password.clone();
        host = machine_public_email.m_outgoing_mail_server.clone();
        port = machine_public_email.m_outgoing_smtp.parse::<u16>().unwrap();
    }
    return sendMail(&host, &sender, &pass, title, message, receiver, port);
}

pub fn sendMail(
    host: &String,
    sender: &String,
    password: &String,
    subject: &String,
    message: String,
    recipient: &String,
    port: u16) -> bool
{
    let mut subject: String = subject.clone();
    if machine().is_develop_mod() {
        subject = "test".to_string();     //remove beforerelease
    }

    let sender_details = sender.split("@").collect::<Vec<&str>>();
    let (status, sender_address) = match Address::new(sender_details[0].to_string(), sender_details[1].to_string()) {
        Ok(a) => {
            (true, a)
        }
        Err(e) => {
            dlog(
                &format!("Failed in prepare sender address: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, Address::new("dummy", "dummy.com").unwrap())
        }
    };
    if !status {
        return false;
    }
    let sender_mailbox = Mailbox::new(None, sender_address);

    let recipient_details = recipient.split("@").collect::<Vec<&str>>();
    let (status, recipient_address) = match Address::new(recipient_details[0].to_string(), recipient_details[1].to_string()) {
        Ok(a) => {
            (true, a)
        }
        Err(e) => {
            dlog(
                &format!("Failed in prepare recipient address: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            (false, Address::new("dummy", "dummy.com").unwrap())
        }
    };
    if !status {
        return false;
    }
    let recipient_mailbox = Mailbox::new(None, recipient_address);


    let email = match Message::builder()
        .from(sender_mailbox)
        .to(recipient_mailbox)
        .subject(subject)
        .body(message) {
        Ok(m) => {
            dlog(
                &format!("Email was prepared: {:?}", m),
                constants::Modules::App,
                constants::SecLevel::Trace);
            m
        }
        Err(e) => {
            dlog(
                &format!("Failed in Email preparing: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }
    };

    // Create TLS transport on port 587 with STARTTLS
    let transporter = match SmtpTransport::starttls_relay(host) {
        Ok(t) => t,
        Err(e) => {
            dlog(
                &format!("Failed in SMTP preparing: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }
    };

    // Add credentials for authentication
    let password = "";
    let sender = transporter.credentials(Credentials::new(
        "username".to_string(),
        password.to_string(),
    ))
        // Configure expected authentication mechanism
        .authentication(vec![Mechanism::Plain])
        // Connection pool settings
        .pool_config(PoolConfig::new().max_size(20))
        .build();

    // Send the email via remote relay
    let result: Response = match sender.send(&email) {
        Ok(r) => {
            dlog(
                &format!("Email was sent: {:?}", r),
                constants::Modules::App,
                constants::SecLevel::Trace);
            r
        }
        Err(e) => {
            dlog(
                &format!("Failed in Email sending: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }
    };

    return true;
}

