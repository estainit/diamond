use crate::lib::constants as cconsts;
use crate::lib::file_handler as file_handler;


//old_name_was maybeBootDAGFromBundle
pub fn maybe_boot_dag_from_bundle() -> bool {
    let clone_id: i8 = 1;
    // let mut bundle = String::from("");
    let (status, bundle) = read_dag_bundle_if_exist(clone_id);

    if !status || (bundle == "") { return false; };

    /*
        QJsonObject DAGBundle = CUtils::parseToJsonObj(bundle);
        QJsonArray blocks = DAGBundle.value("blocks").toArray();
        QJsonObject ballots = DAGBundle.value("ballots").toObject();

        CLog::log("Read & Dispatching (" + QString::number(blocks.size()) + ")blocks and (" + QString::number(ballots.keys().size()) + ")Ballots from DAGBundle");
        // normalizing/sanitize Ballots Receive Dates and upsert into kv
        try {
        QJsonObject sanBallots {};
        for (QString aBlt: ballots.keys())
        {
        QJsonObject a_ballot = ballots[aBlt].toObject();
        sanBallots[aBlt] = QJsonObject {
        {"baReceiveDate", CUtils::stripNonInDateString(a_ballot.value("baReceiveDate").toString())},
        {"baVoteRDiff", a_ballot.value("baVoteRDiff").toDouble()}};
        }
        KVHandler::upsertKValue("ballotsReceiveDates", CUtils::serializeJson(sanBallots));
        } catch (std::exception) {
        CLog::log("exception in reading DAGBundle", "sec", "error");

        }

        // dispatching blocks to sending q
        for(auto aBlock: blocks)
        {
        Dispatcher::dispatchMessage(
        "DAGBundle",
        aBlock.toObject(),
        cconsts::PRIVATE);
        }

        // Archive DAGBundle file in tmp folder
        FullDAGHandler::archiveDAGBundle();

        */
    return true;
}

//old_name_was readDAGBundleIfExist
pub fn read_dag_bundle_if_exist(clone_id: i8) -> (bool, String)
{
    return file_handler::read(
        &mut cconsts::HD_FILES.to_string(),
        &"DAGBundle.txt".to_string(),
        clone_id);
}

/*

QStringList FileBufferHandler::listHardDiskInbox()
{
  QString inbox = CMachine::getInboxPath();

  CLog::log("reading inbox(" + inbox + ")", "app", "trace");

  QDir directory(inbox);
  QStringList files = directory.entryList({"*.txt"}, QDir::Files);  //FIXME: maybe read files ordered by reverse modify date!

    if (files.size() == 0)
    return {};

  QStringList outs {};
  for(auto a_file: files)
    outs.append(files[0]);

  return outs;
}

QStringList FileBufferHandler::listHardDiskOutbox()
{
  QString outbox = CMachine::getOutboxPath();

  CLog::log("reading outbox(" + outbox + ")", "app", "trace");

  QDir directory(outbox);
  QStringList files = directory.entryList({"*.txt"}, QDir::Files);  //FIXME: maybe read files ordered by reverse modify date!

  // the live system never delet outbox, instead can delete outbox after parsing
  if (files.size() == 0)
    return {};

  QStringList outs {};
  for(auto a_file: files)
    outs.append(files[0]);

  return outs;
}

void FileBufferHandler::loopReadAndParseHardDiskInbox()
{
  QString thread_prefix = "read_and_parse_hard_disk_inbox_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    doReadAndParseHardDiskInbox();

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getHardDiskReadingGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
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
      if ((file_name!="") && (CMachine::isDevelopMod()))
        DbModel::insert(
          "cdev_inbox_logs",
          {
            {"il_creation_date", CUtils::getNow()},
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

      CLog::log("a cpacket received:" + CUtils::serializeJson(cpacket), "app", "trace");

      auto[dispatch_status, should_purge_file] = Dispatcher::dispatchMessage(
        sender,
        cpacket,
        connection_type);

      CLog::log("Dispatch Message res: dispatch_status(" + CUtils::dumpIt(dispatch_status) + ") should_purge_file(" + CUtils::dumpIt(should_purge_file) + ") ", "app", "trace");

      //should purge file?
      if (file_name != "")
      {
        if (should_purge_file == false) {
            CLog::log("why should_purge_file == false? " + CUtils::serializeJson(cpacket), "sec", "error");;
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
bool FileBufferHandler::maybePurgeMessage(const QString& full_path, const bool& should_purge_fessage)
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


std::tuple<bool, QString, QString, QString, QString> FileBufferHandler::readEmailFile()
{
  QString inbox = CMachine::getInboxPath();

  CLog::log("reading inbox(" + inbox + ")", "app", "trace");

  QDir directory(inbox);
  QStringList files = directory.entryList({"*.txt"}, QDir::Files);  //FIXME: maybe read files ordered by reverse modify date!

  // the live system never delet outbox, instead can delete inbox after parsing
  if (files.size() == 0)
    return {
      true,
      "", // sender: '',
      "", // receiver: '',
      "", // file_name: '',
      "" // message: ''
    };



  QString file_name = files[0];
  QString full_path = inbox + '/' + file_name;
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
    content.contains(CConsts::MESSAGE_TAGS::senderStartTag) &&
    content.contains(CConsts::MESSAGE_TAGS::senderEndTag) &&
    content.contains(CConsts::MESSAGE_TAGS::receiverStartTag) &&
    content.contains(CConsts::MESSAGE_TAGS::receiverEndTag) &&
    content.contains(CConsts::MESSAGE_TAGS::iPGPStartEnvelope) &&
    content.contains(CConsts::MESSAGE_TAGS::iPGPEndEnvelope)
  )
  {
    QString sender = content.split(CConsts::MESSAGE_TAGS::senderStartTag)[1].split(CConsts::MESSAGE_TAGS::senderEndTag)[0];
    QString receiver = content.split(CConsts::MESSAGE_TAGS::receiverStartTag)[1].split(CConsts::MESSAGE_TAGS::receiverEndTag)[0];
    QString pure_content = content.split(CConsts::MESSAGE_TAGS::iPGPStartEnvelope)[1].split(CConsts::MESSAGE_TAGS::iPGPEndEnvelope)[0];
    if (pure_content != "")
        pure_content = CUtils::stripBR(pure_content);

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