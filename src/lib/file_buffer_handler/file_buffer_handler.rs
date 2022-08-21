use crate::{constants, cutils, dlog, machine};
use crate::lib::file_handler::file_handler::{delete_exact_file, file_read, list_exact_files, read_exact_file};
use crate::lib::messaging_protocol::dispatcher::parse_a_packet;
use crate::lib::network::cpacket_handler::decrypt_and_parse_packet;


//old_name_was maybeBootDAGFromBundle
pub fn maybe_boot_dag_from_bundle() -> bool {
    let clone_id: i8 = machine().get_app_clone_id();
    // let mut bundle = String::from("");
    let (status, bundle) = read_dag_bundle_if_exist(clone_id);

    if !status || (bundle == "") { return false; };

    /*
        JSonObject DAGBundle = cutils::parseToJsonObj(bundle);
        JSonArray blocks = DAGBundle.value("blocks").toArray();
        JSonObject ballots = DAGBundle.value("ballots").toObject();

        CLog::log("Read & Dispatching (" + String::number(blocks.len()) + ")blocks and (" + String::number(ballots.keys().len()) + ")Ballots from DAGBundle");
        // normalizing/sanitize Ballots Receive Dates and upsert into kv
        try {
        JSonObject sanBallots {};
        for (String aBlt: ballots.keys())
        {
        JSonObject a_ballot = ballots[aBlt].toObject();
        sanBallots[aBlt] = JSonObject {
        {"baReceiveDate", cutils::stripNonInDateString(a_ballot.value("baReceiveDate").to_string())},
        {"baVoteRDiff", a_ballot.value("baVoteRDiff").toDouble()}};
        }
        KVHandler::upsertKValue("ballotsReceiveDates", cutils::serializeJson(sanBallots));
        } catch (std::exception) {
        CLog::log("exception in reading DAGBundle", "sec", "error");

        }

        // dispatching blocks to sending q
        for(auto aBlock: blocks)
        {
        Dispatcher::dispatchMessage(
        "DAGBundle",
        aBlock.toObject(),
        constants::PRIVATE);
        }

        // Archive DAGBundle file in tmp folder
        FullDAGHandler::archiveDAGBundle();

        */
    return true;
}

//old_name_was readDAGBundleIfExist
pub fn read_dag_bundle_if_exist(clone_id: i8) -> (bool, String)
{
    return file_read(
        constants::HD_ROOT_FILES.to_string(),
        format!("DAGBundle.txt"),
        clone_id);
}

/*

StringList FileBufferHandler::listHardDiskInbox()
{
  String inbox = CMachine::getInboxPath();

  CLog::log("reading inbox(" + inbox + ")", "app", "trace");

  QDir directory(inbox);
  StringList files = directory.entryList({"*.txt"}, QDir::Files);  //FIXME: maybe read files ordered by reverse modify date!

    if (files.len() == 0)
    return {};

  StringList outs {};
  for(auto a_file: files)
    outs.push(files[0]);

  return outs;
}

StringList FileBufferHandler::listHardDiskOutbox()
{
  String outbox = CMachine::getOutboxPath();

  CLog::log("reading outbox(" + outbox + ")", "app", "trace");

  QDir directory(outbox);
  StringList files = directory.entryList({"*.txt"}, QDir::Files);  //FIXME: maybe read files ordered by reverse modify date!

  // the live system never delet outbox, instead can delete outbox after parsing
  if (files.len() == 0)
    return {};

  StringList outs {};
  for(auto a_file: files)
    outs.push(files[0]);

  return outs;
}

void FileBufferHandler::loopReadAndParseHardDiskInbox()
{
  String thread_prefix = "read_and_parse_hard_disk_inbox_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    doReadAndParseHardDiskInbox();

    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getHardDiskReadingGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loopReadAndParseHardDiskInbox");
}

 */

pub fn received_email_checks() {
    // auto[status, sender, receiver, file_name, message] = readEmails();
    // if (!status)
    // return status;
}

//old_name_was doReadAndParseHardDiskInbox
pub fn do_read_and_parse_hard_disk_inbox() -> bool
{
    //  pullCounter += 1
    let (
        status,
        sender,
        receiver,
        file_name,
        message) = read_email_file();
    if !status {
        dlog(
            &format!("Nothing in reading inbox files!"),
            constants::Modules::App,
            constants::SecLevel::Info);
        return false;
    }

    dlog(
        &format!("have read packet from HD sender({}) receiver({}) file_name({})", sender, receiver, file_name),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (
        dec_status,
        connection_type,
        cpacket) =
        decrypt_and_parse_packet(
            &sender,
            &receiver,
            &file_name,
            &message);

    if !dec_status
    {
        //TODO: implement a reputation system based on sender email address to avoid pottentially attacks (e.g DOS)
        maybe_purge_message(&file_name, true);
        return false;
    }

    println!(">>>>>>> cpacket: {}", cpacket);


    let (status, should_purge_file) = parse_a_packet(
        &sender,
        &cpacket,
        &connection_type);

    dlog(
        &format!("Parse packet response: status({}) should purge file({}) ", status, should_purge_file),
        constants::Modules::App,
        constants::SecLevel::Info);


    //should purge file?
    if file_name != ""
    {
        if should_purge_file == false
        {
            dlog(
                &format!("Why should not purge the file {}? {}", file_name, cutils::serialize_json(&cpacket)),
                constants::Modules::Sec,
                constants::SecLevel::Error);
        }
        maybe_purge_message(&file_name, should_purge_file);
    }

    return status;
}

//old_name_was maybePurgeMessage
pub fn maybe_purge_message(full_path: &String, should_purge_message: bool) -> bool
{
    //should purge packet?
    let mut is_expired: bool = false;
    let mut reached_tl: bool = false; // this.richedTryLimitation(packet);
    if should_purge_message || is_expired || reached_tl
    {
        dlog(
            &format!("should Purge Message: {}  is-expired: {}  reached-Try-Limitation: {}", should_purge_message, is_expired, reached_tl),
            constants::Modules::Sec,
            constants::SecLevel::Info);

        if full_path == ""
        {
            dlog(
                &format!("maybe Purge Message, got empty fileName! {}", full_path),
                constants::Modules::Sec,
                constants::SecLevel::Error);

            return false;
        }
        delete_exact_file(full_path);
    }
    return true;
}

//static richedTryLimitation(packet) {
//    try {
//        let tryCount = messageTicketing.getTry(packet.fileName)
//        if (tryCount > iConsts.MAX_PARSE_ATTEMPS_COUNT)
//            return true
//        return false
//    } catch (e) {
//        throw new Error(e)
//    }
//};

//old_name_was readEmailFile
pub fn read_email_file() -> (bool, String, String, String, String)
{
    let inbox: String = machine().get_inbox_path();

    dlog(
        &format!("Reading inbox({})", inbox),
        constants::Modules::App,
        constants::SecLevel::Info);

    let files = list_exact_files(&inbox, "txt");  //FIXME: maybe read files ordered by reverse modify date!
    println!("fileszzzzzzzzzzzzz {:?}", files);

    // the live system never delet outbox, instead can delete inbox after parsing
    if files.len() == 0
    {
        return (
            false,
            "".to_string(), // sender: "",
            "".to_string(), // receiver: "",
            "".to_string(), // file_name: "",
            "".to_string() // message: ""
        );
    }

    let full_path: &String = &files[0].clone();
    // let full_path: &String = &format!("{inbox}/{file_name}");
    let (status, content) = read_exact_file(full_path);

    if !status || (content == "")
    {
        // delete curropted file
        delete_exact_file(full_path);
        return (
            false,
            "".to_string(), // sender
            "".to_string(), // receiver
            "".to_string(), // file_name
            "".to_string() // content
        );
    }

    if
    content.contains(constants::message_tags::SENDER_START_TAG) &&
        content.contains(constants::message_tags::SENDER_END_TAG) &&
        content.contains(constants::message_tags::RECEIVE_START_TAG) &&
        content.contains(constants::message_tags::RECEIVE_END_TAG) &&
        content.contains(constants::message_tags::ENVELOPE_I_PGP_START) &&
        content.contains(constants::message_tags::ENVELOPE_I_PGP_END)
    {
        let sender: String = content.split(constants::message_tags::SENDER_START_TAG).collect::<Vec<&str>>()[1].to_string().split(constants::message_tags::SENDER_END_TAG).collect::<Vec<&str>>()[0].to_string();
        let receiver: String = content.split(constants::message_tags::RECEIVE_START_TAG).collect::<Vec<&str>>()[1].to_string().split(constants::message_tags::RECEIVE_END_TAG).collect::<Vec<&str>>()[0].to_string();
        let mut pure_content: String = content.split(constants::message_tags::ENVELOPE_I_PGP_START).collect::<Vec<&str>>()[1].to_string().split(constants::message_tags::ENVELOPE_I_PGP_END).collect::<Vec<&str>>()[0].to_string();
        if pure_content != ""
        {
            pure_content = cutils::strip_parentheses_as_break_line(pure_content);
        }

//    FileHandler::deleteFile(full_path);
        return (
            true,
            sender,
            receiver,
            full_path.clone(),
            pure_content
        );
    } else {
        // delete invalid message
        dlog(
            &format!("received invalid msg which missed either sender, receiver or iPGP tag. {}", full_path),
            constants::Modules::App,
            constants::SecLevel::Debug);

        delete_exact_file(full_path);
        return (
            false,
            "".to_string(),
            "".to_string(),
            full_path.clone(),
            "".to_string()
        );
    }
}
