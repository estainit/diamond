use crate::constants;
use crate::lib::file_handler::file_handler::read;


//old_name_was maybeBootDAGFromBundle
pub fn maybe_boot_dag_from_bundle() -> bool {
    let clone_id: i8 = 1;
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
    let mut file_path: String = constants::HD_ROOT_FILES.to_string() + &"/";
    return read(
        &mut file_path,
        &"DAGBundle.txt".to_string(),
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

//old_name_was doReadAndParseHardDiskInbox
pub fn do_read_and_parse_hard_disk_inbox() -> bool
{
    /*
    //  pullCounter += 1
      auto[status, sender, receiver, file_name, message] = readEmailFile();
      CLog::log("have read packet from HD sender(" + sender + ") receiver(" + receiver + ") file_name(" + file_name + ")", "app", "trace");
      if (!status)
        return status;


      //developer log
      if ((file_name!="") && (CMachine::is_develop_mod()))
        DbModel::insert(
          "cdev_inbox_logs",
          {
            {"il_creation_date", cutils::get_now()},
            {"il_title", file_name}
          });

      if (message == "")
        return false;

    //  listener.doCallSync('SPSH_before_parse_packet', packet);

      auto[dec_status, connection_type, cpacket] = CPacketHandler::decryptAndParsePacketSync(sender, receiver, file_name, message);
      if (!dec_status)
      {
        //TODO: implement a reputation system based on sender email address to avoid pottentially attacks (e.g DOS)
        maybePurgeMessage(file_name, true);
        return false;
      }

    //  listener.doCallSync('SPSH_after_parse_packet', { packet, parsePacketRes });

      CLog::log("a cpacket received:" + cutils::serializeJson(cpacket), "app", "trace");

      auto[dispatch_status, should_purge_file] = Dispatcher::dispatchMessage(
        sender,
        cpacket,
        connection_type);

      CLog::log("Dispatch Message res: dispatch_status(" + cutils::dumpIt(dispatch_status) + ") should_purge_file(" + cutils::dumpIt(should_purge_file) + ") ", "app", "trace");

      //should purge file?
      if (file_name != "")
      {
        if (should_purge_file == false) {
            CLog::log("why should_purge_file == false? " + cutils::serializeJson(cpacket), "sec", "error");;
        }
        maybePurgeMessage(file_name, should_purge_file);
      }

    //  let dispatchResErr = _.has(dispatchRes, 'error') ? dispatchRes.error : null;
    //  if (utils._notNil(dispatchResErr)) {
    //    //TODO:  some log to db denoting to "unable to parse a message"
    //    clog.app.error(dispatchRes)
    //  }

    //  return parsePacketRes
    */
    return true;
}

/*
bool FileBufferHandler::maybePurgeMessage(const String& full_path, const bool& should_purge_fessage)
{
  //should purge packet?
  bool is_expired = false;
//  if (_.has(file_name, 'creation_date'))
//  {
//    let creation_date = new Date(file_name.creation_date);
//    is_expired = utils.isItExpired(creation_date, 60); //after 60 minutes
//  }
  bool reachedTL = false; // this.richedTryLimitation(packet);
  if (should_purge_fessage || is_expired || reachedTL)
  {
    CLog::log("should-Purge-Message ${should_purge_fessage} is-expired ${expired}  reached-Try-Limitation ${reachedTL}");
    if (full_path == "")
    {
      CLog::log("maybe Purge Message, got empty fileName! ${utils.stringify(packet)}", "sec", "error");
      return false;
    }
    FileHandler::deleteFile(full_path);
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


std::tuple<bool, String, String, String, String> FileBufferHandler::readEmailFile()
{
  String inbox = CMachine::getInboxPath();

  CLog::log("reading inbox(" + inbox + ")", "app", "trace");

  QDir directory(inbox);
  StringList files = directory.entryList({"*.txt"}, QDir::Files);  //FIXME: maybe read files ordered by reverse modify date!

  // the live system never delet outbox, instead can delete inbox after parsing
  if (files.len() == 0)
    return {
      true,
      "", // sender: '',
      "", // receiver: '',
      "", // file_name: '',
      "" // message: ''
    };



  String file_name = files[0];
  String full_path = inbox + '/' + file_name;
  auto[status, content] = FileHandler::read(inbox, file_name);
  if (!status || (content == ""))
  {
    // delete curropted file
    FileHandler::deleteFile(full_path);
    return {
      false,
      "", // sender: '',
      "", // receiver: '',
      "", // file_name: '',
      "" // message: '',
    };
  }

  if (
    content.contains(constants::MESSAGE_TAGS::senderStartTag) &&
    content.contains(constants::MESSAGE_TAGS::senderEndTag) &&
    content.contains(constants::MESSAGE_TAGS::receiverStartTag) &&
    content.contains(constants::MESSAGE_TAGS::receiverEndTag) &&
    content.contains(constants::MESSAGE_TAGS::iPGPStartEnvelope) &&
    content.contains(constants::MESSAGE_TAGS::iPGPEndEnvelope)
  )
  {
    String sender = content.split(constants::MESSAGE_TAGS::senderStartTag)[1].split(constants::MESSAGE_TAGS::senderEndTag)[0];
    String receiver = content.split(constants::MESSAGE_TAGS::receiverStartTag)[1].split(constants::MESSAGE_TAGS::receiverEndTag)[0];
    String pure_content = content.split(constants::MESSAGE_TAGS::iPGPStartEnvelope)[1].split(constants::MESSAGE_TAGS::iPGPEndEnvelope)[0];
    if (pure_content != "")
        pure_content = cutils::stripBR(pure_content);

//    FileHandler::deleteFile(full_path);
    return {
      true,
      sender,
      receiver,
      full_path,
      pure_content};

  } else {
    // delete invalid message
    CLog::log("received invalid msg which missed either sender, receiver or iPGP tag", "app", "debug");
    FileHandler::deleteFile(full_path);
    return {
      false,
      "",
      "",
      full_path,
      ""
    };

  }
}



 */