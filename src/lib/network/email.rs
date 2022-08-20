use crate::{constants, dlog, machine};
use crate::lib::machine::machine_profile::EmailSettings;

use lettre::message::Mailbox;

use lettre::{transport::smtp::{
    authentication::{Credentials, Mechanism},
    PoolConfig,
}, Message, SmtpTransport, Transport, Address};
use lettre::transport::smtp::response::Response;

extern crate imap;
extern crate native_tls;

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

//old_name_was sendEmailWrapper
pub fn send_email_wrapper(
    sender_: &String,
    title: &String,
    message: String,
    receiver: &String) -> bool
{
    dlog(
        &format!("send EmailWrapper args: sender({sender_}) receiver({receiver}) title({title})"),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let machine_public_email: EmailSettings = machine().get_pub_email_info().clone();
    let machine_private_email: EmailSettings = machine().get_priv_email_info().clone();

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
    return send_mail(&host, &sender, &pass, title, message, receiver, port);
}

pub fn send_mail(
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
    let sender = transporter.credentials(Credentials::new(
        sender_details[0].to_string().clone(),
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

//old_name_was popPublicEmail
pub fn pop_public_email() -> (bool, String) // imap::error::Result<Option<String>>
{
    let pub_email = machine().get_pub_email_info().clone();
    let domain = pub_email.m_income_imap.clone(); // "imap.example.com";

    return read_email(&domain, &pub_email.m_address.clone(), &pub_email.m_password.clone());
}

pub fn read_email(
    domain: &String,
    mail_address: &String,
    mail_password: &String,
) -> (bool, String) // imap::error::Result<Option<String>>
{
    dlog(
        &format!("fetch Pub Email And Write On Hard Disk"),
        constants::Modules::App,
        constants::SecLevel::Trace);

    // // fetch private inbox
    // let params = {
    //     emailAddress: pubEmail.address,
    //     password: pubEmail.pwd,
    //     host: pubEmail.incomingMailServer,
    //     port: pubEmail.incomeIMAP,
    //     funcMode: "readUNSEENs"
    // }


    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let client = imap::connect((domain.clone(), 993), domain, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = match client
        .login(mail_address, mail_password)
        .map_err(|e| e.0) {
        Ok(r) => { r }
        Err(e) => {
            dlog(
                &format!("Failed in prepare IMAP session: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, "Failed in prepare IMAP session".to_string());
        }
    };

    // we want to fetch the first email in the INBOX mailbox
    let dd = match imap_session.select("INBOX") {
        Ok(m) => m,
        Err(e) => {
            dlog(
                &format!("Failed in prepare IMAP inbox check: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, "Failed in prepare IMAP inbox check".to_string());
        }
    };

    // fetch message number 1 in this mailbox, along with its RFC822 field.
    // RFC 822 dictates the format of the body of e-mails
    let messages = match imap_session.fetch("1", "RFC822") {
        Ok(r) => r,
        Err(e) => {
            dlog(
                &format!("Failed in prepare IMAP fetch: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, "Failed in prepare IMAP fetch".to_string());
        }
    };
    let message = if let Some(m) = messages.iter().next() {
        m
    } else {
        return (true, "".to_string());//Ok(None);
    };

    // extract the message's body
    let body = message.body().expect("message did not have a body!");
    let body = std::str::from_utf8(body)
        .expect("message was not valid utf-8")
        .to_string();

    // be nice to the server and log out
    // imap_session.logout()?;

    (true, body)
}


