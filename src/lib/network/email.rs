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

use crate::{constants, dlog, machine};
use crate::lib::machine::machine_profile::EmailSettings;

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
    message: &String,
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
    host_: &String,
    sender_: &String,
    password_: &String,
    subject_: &String,
    message_: &String,
    recipient_: &String,
    port: u16) -> bool
{
    let mut subject: String = subject_.clone();
    if machine().is_develop_mod() {
        subject = "test".to_string();     //remove beforerelease
    }

    /*

      // connect to poco;
      std::string sender = sender_.toStdString();
      std::string host = host_.toStdString();
      std::string password = password_.toStdString();
      std::string message = message_.toStdString();
      std::string recipient = recipient_.toStdString();

      port = static_cast<Poco::UInt16>(port);

      try
      {
        SharedPtr<InvalidCertificateHandler> pCert = new ConsoleCertificateHandler(false);
        Context::Ptr pContext = new Context(Context::CLIENT_USE, "", "", "", Context::VERIFY_RELAXED, 9, true, "ALL:!ADH:!LOW:!EXP:!MD5:@STRENGTH");

        SSLManager::instance().initializeClient(0, pCert, pContext);

        SecureSMTPClientSession session(host, port);
        session.login();
        session.startTLS();
        if (!sender.empty())
          session.login(SMTPClientSession::AUTH_LOGIN, sender, password);

        MailMessage msg;
        msg.setSender(sender);
        msg.addRecipient(MailRecipient(MailRecipient::PRIMARY_RECIPIENT, recipient));
        msg.setSubject(subject);
        msg.setContent(message);
        session.sendMessage(msg);
        session.close();
      }
      catch (Exception& e)
      {
        std::cerr << e.message() << std::endl;
        CLog::log("Unable to send email! " + e.message(), "app", "fatal");
        return false;
      }

     */
    return true;
}

