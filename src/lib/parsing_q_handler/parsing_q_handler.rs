/*
const QString ParsingQHandler::stbl_parsing_q = "c_parsing_q";
const QStringList ParsingQHandler::stbl_parsing_q_fields = {"pq_id", "pq_type", "pq_code", "pq_sender", "pq_connection_type", "pq_receive_date", "pq_payload", "pq_prerequisites", "pq_parse_attempts", "pq_v_status", "pq_creation_date", "pq_insert_date", "pq_last_modified"};
const QString ParsingQHandler::stbldev_parsing_q = "cdev_parsing_q";

void ParsingQHandler::loopSmartPullFromParsingQ()
{
  QString thread_prefix = "pull_from_parsing_q_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    MissedBlocksHandler::refreshMissedBlock();

    smartPullQ();

    CLog::log("Smart Pull From Parsing Q, Every (" + QString::number(CMachine::getParsingQGap()) + " seconds) ", "app", "trace");
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getParsingQGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Smart Pull From Parsing Q");
}

/**
 * @return std::tuple<bool status, bool should_purge_record>
 */
std::tuple<bool, bool> ParsingQHandler::handlePulledPacket(const QVDicT& packet)
{
//  listener.doCallAsync('APSH_before_handle_pulled_packet', args);

  CLog::log("handle Pulled Packet: " + CUtils::dumpIt(packet), "app", "trace");

  QString receive_date = packet.value("pq_receive_date", CUtils::getNow()).toString();
  QString pq_type = packet.value("pq_type", "").toString();
  QString pq_code = packet.value("pq_code", "").toString();
  QString pq_sender = packet.value("pq_sender", "").toString();
  QString connection_type = packet.value("pq_connection_type", "").toString();
  /**
  * payload could be a block, GQL or even old-style messages
  * TODO: optimizine to use heap allocation for bigger payloads
  */
  QJsonObject payload = packet.value("pq_payload", QJsonObject()).toJsonObject();

  if ((pq_sender == "") || (payload.keys().size() == 0))
  {
    CLog::log("missed sender or paylod to parse", "app", "error");
    return {false, true};
  }

  if (pq_type == "")
  {
    CLog::log("missed pq_type " + CUtils::dumpIt(packet), "app", "error");
    return {false, true};
  }
  if (connection_type == "")
  {
    CLog::log("missed connection_type in parsing " + CUtils::dumpIt(packet), "app", "error");
    return {false, true};
  }

  if(payload.value("bType").toString() == CConsts::BLOCK_TYPES::RpBlock)
  {
    CLog::log("A repay Block received block(" + CUtils::hash8c(payload.value("bHash").toString()) + ")", "trx", "info");
    // Since machine must create the repayments by itself we drop this block immidiately,
    // in addition machine calls importCoinbasedUTXOs method to import potentially minted coins and cut the potentially repay backs in on shot
    return {true, true};
  }



  if (QStringList {CConsts::BLOCK_TYPES::Normal,
  CConsts::BLOCK_TYPES::Coinbase,
  CConsts::BLOCK_TYPES::FSign,
  CConsts::BLOCK_TYPES::SusBlock,
  CConsts::BLOCK_TYPES::FVote,
  CConsts::BLOCK_TYPES::POW}.contains(pq_type))
  {
    payload["local_receive_date"] = receive_date;
    Block* block = BlockFactory::create(payload);

    if (!block->objectAssignmentsControlls())
    {
      CLog::log("Maleformed JSon block couldn't be parsed! block(" + CUtils::hash8c(payload.value("bHash").toString()) + ")", "trx", "error");
      return {false, true};
    }

    auto[status2, should_purge_record2] = parsePureBlock(
      pq_sender,
      pq_type,
      block,
      connection_type,
      receive_date
    );
    delete block;

    if (!CMachine::isInSyncProcess())
      CGUI::refreshMonitor();

    return {status2, should_purge_record2};

  }

  CLog::log(
    "\n\n--- parsing CPacket type(" + pq_type + ") Block/Message \nfrom Q.sender(" + pq_sender + ") ", "app", "trace");

  // GQL part
  if (pq_type == CConsts::CARD_TYPES::ProposalLoanRequest)
  {
    auto[status, should_purge_record] = GeneralPledgeHandler::handleReceivedProposalLoanRequest(
      pq_sender,
      payload,
      connection_type,
      receive_date);
    if (status)
      CGUI::signalUpdateReceivedLoanRequests();
    return {status, should_purge_record};

  }
  else if (pq_type == CConsts::CARD_TYPES::FullDAGDownloadRequest)
  {
    auto[status, should_purge_record] = FullDAGHandler::prepareFullDAGDlResponse(
      pq_sender,
      payload,
      connection_type);
    return {status, should_purge_record};

  }
  else if (pq_type == CConsts::CARD_TYPES::pleaseRemoveMeFromYourNeighbors)
  {
//    case GQLHandler.cardTypes.pleaseRemoveMeFromYourNeighbors:
//        res = require('../../machine/machine-handler').neighborHandler.doDeleteNeighbor({
//            sender,
//            payload,
//            connection_type,
//            receive_date
//        });
//        break;
  }
  else if (pq_type == CConsts::MESSAGE_TYPES::DAG_INVOKE_BLOCK)
  {
    //comunications
    auto[status, should_purge_record] = DAGMessageHandler::handleBlockInvokeReq(
      pq_sender,
      payload,
      connection_type);
    return {status, should_purge_record};

  }
  else if (pq_type == CConsts::MESSAGE_TYPES::DAG_INVOKE_DESCENDENTS)
  {
//    case MESSAGE_TYPES.DAG_INVOKE_DESCENDENTS:
//        res = dagMsgHandler.handleDescendentsInvokeReq({
//            sender,
//            payload,
//            connection_type: connection_type
//        })
//        break;

  }

  CLog::log("Unknown packet in parsing Q! " + pq_type + " " + pq_code + " from " + pq_sender, "sec", "error");
  return {false, true};

}






/**
 * @return std::tuple<bool status, bool should_purge_record>
 */
std::tuple<bool, bool> ParsingQHandler::parsePureBlock(
  const QString& sender,
  const QString& pq_type,
  const Block* block,
  const QString& connection_type,
  const QString& receive_date
  )
{
  Q_UNUSED(sender);
  Q_UNUSED(connection_type);
  Q_UNUSED(receive_date);

  // DAG existance ancestors controlls
  QStringList needed_blocks = CUtils::arrayDiff(block->m_ancestors, DAG::getCachedBlocksHashes());
  if (needed_blocks.size() > 0)
  {
    CLog::log(
      "in order to parse 1block(" + CUtils::hash6c(block->getBlockHash()) + ") machine needs blocks(" +
      CUtils::dumpIt(needed_blocks) + ") exist in DAG"
      "app", "trace");

    // TODO: maybe some reputation system to report diorder of neighbor
    return {false, false};
  }

  auto[b_status, b_should_purge_record] = block->blockGeneralControls();
  if (!b_status)
    return {false, b_should_purge_record};

  // general ancestors controlls
  auto[status, should_purge_record] = ancestorsConroll(pq_type, block);
  if (!status)
    return {status, should_purge_record};


  return block->handleReceivedBlock();

//  switch (pq_type) {

//    case iConsts.BLOCK_TYPES.FVote:
//        res = require('../../dag/floating-vote/floating-vote-handler').handleReceivedFVoteBlock({
//            sender,
//            block,
//            connection_type,
//            receive_date
//        });
//        break;

//    case iConsts.BLOCK_TYPES.POW:
//        res = require('../../dag/pow-block/handle-received-block').handleReceivedPOWBlock({
//            sender,
//            block,
//            connection_type,
//            receive_date
//        });
//        break;
//  }

//  return res;

}

std::tuple<bool, bool> ParsingQHandler::pushToParsingQ(
  const QJsonObject& message,
  const QString& creation_date,
  const QString& type,
  const QString& code,
  const QString& sender,
  const QString& connection_type,
  QStringList prerequisites)
{
  try {
    // check for duplicate entries
    QueryRes dbl = DbModel::select(
      stbl_parsing_q,
      {"pq_type"},
      {{"pq_type", type},
        {"pq_code", code}},
      {},
      0,
      false,
      false
    );
    if (dbl.records.size() > 0)
      return { true, true };

//    listener.doCallSync('SPSH_before_insert_packet_in_q', args);


    // control if needs some initiative prerequisities
    QStringList message_ancestors = {};
    if (message.keys().contains("ancestors") && (message.value("ancestors").toArray().size() > 0))
    {
      for(auto an_anc: message.value("ancestors").toArray())
      {
        message_ancestors.append(an_anc.toString());
      }

//      // check if ancestores exist in parsing q
//      QueryRes queuedAncs = DbModel::select(
//        stbl_parsing_q,
//        {"pq_code"},
//        {{"pq_code", message_ancestors, "IN"}});

//      QStringList missedAnc = {};
//      if (queuedAncs.records.size() == 0)
//      {
//        missedAnc = message_ancestors;
//        CLog::log("block(" + code + ") totaly missed ancestors (" + CUtils::dumpIt(missedAnc) + ")", "app", "trace");
//      }
//      else if (queuedAncs.records.size() < message_ancestors.size())
//      {
//        QStringList pq_codes = {};
//        for(QVDicT a_row: queuedAncs.records)
//          pq_codes.append(a_row.value("pq_code").toString());
//        missedAnc = CUtils::arrayDiff(message_ancestors, pq_codes);
//        CLog::log("block(" + code + ") partially missed ancestors (" + CUtils::dumpIt(missedAnc) + ") ", "app", "trace");
//      }

      CLog::log("block(" + code + ") before + missed ancs (" + CUtils::dumpIt(prerequisites) + "\n\n " + CUtils::dumpIt(message_ancestors), "app", "trace");

      // control if missedAnc alredy exist in DAG?
      QStringList exist_in_DAG;
      QStringList existed_blocks_in_DAG = DAG::getCachedBlocksHashes();
      for (CBlockHashT an_ancestor: message_ancestors)
        if (existed_blocks_in_DAG.contains(an_ancestor))
          exist_in_DAG.append(an_ancestor);

//      QVDRecordsT DAGedAncs = DAG::searchInDAG(
//        {{"b_hash", message_ancestors, "IN"}},
//        {"b_hash"});

//      if (DAGedAncs.size() > 0)
//      {
//        QStringList exist_in_DAG;
//        for (QVDicT x: DAGedAncs)
//        {
//          exist_in_DAG.append(x.value("b_hash").toString());
//        }

        CLog::log("some likly missed blocks(" + message_ancestors.join(",") + ") already recorded in DAG(" + exist_in_DAG.join(",") + ")", "app", "trace");
        message_ancestors = CUtils::arrayDiff(message_ancestors, exist_in_DAG);
//      }
      prerequisites = CUtils::arrayAdd(prerequisites, message_ancestors);
    }

    /**
     * if blcok is FVote, maybe we need customized treatment, since generally in DAG later blocks are depend on
     * early blocks and it is one way graph.
     * but in case of vote blocks, they have effect on previous blocks (e.g accepting or rejecting a transaction of previously block)
     * so depends on voting type(bCat) for, we need proper treatment
     */
    if (message.value("bType").toString() == CConsts::BLOCK_TYPES::FVote)
    {

      if (message.value("bCat").toString() == CConsts::FLOAT_BLOCKS_CATEGORIES::Trx)
      {
        /**
        * if the machine get an FVote, so insert uplink block in SUS BLOCKS WHICH NEEDED VOTES TO BE IMPORTED AHAED(SusBlockWNVTBIA)
        * WNVTBIA: Wait becaue Needs Vote To Be Importable
        */
        QString uplinkBlock = message.value("ancestors").toArray()[0].toString();    // FVote blocks always have ONLY one ancestor for which Fvote is voting
        QString currentWNVTBIA = KVHandler::getValue("SusBlockWNVTBIA");
        QStringList currentWNVTBIA_arr = {};
        if (currentWNVTBIA == "")
        {
          currentWNVTBIA_arr.append(uplinkBlock);
        } else {
          auto tmp = CUtils::parseToJsonArr(currentWNVTBIA);
          for(auto x: tmp)
            currentWNVTBIA_arr.append(x.toString());
          currentWNVTBIA_arr.append(uplinkBlock);
          currentWNVTBIA_arr = CUtils::arrayUnique(currentWNVTBIA_arr);
        }
        currentWNVTBIA = CUtils::serializeJson(currentWNVTBIA_arr);
        KVHandler::upsertKValue("SusBlockWNVTBIA", currentWNVTBIA);
      }
    }

    // TODO: security issue to control block (specially payload), before insert to db
    // potentially attacks: sql injection, corrupted JSON object ...

    QVDicT values {
      {"pq_type", type},
      {"pq_code", code},
      {"pq_sender", sender},
      {"pq_connection_type", connection_type},
      {"pq_receive_date", CUtils::getNow()},
      {"pq_payload", BlockUtils::wrapSafeContentForDB(CUtils::serializeJson(message)).content},
      {"pq_prerequisites", "," + prerequisites.join(",")},  //"," prefix intentionally was added
      {"pq_parse_attempts", 0},
      {"pq_v_status", "new"},
      {"pq_creation_date", creation_date},
      {"pq_insert_date", CUtils::getNow()},
      {"pq_last_modified", CUtils::getNow()}
    };
    DbModel::insert(
      stbl_parsing_q,
      values,
      false,
      false);

//    listener.doCallSync('SPSH_after_insert_packet_in_q', args);

    if (CMachine::isDevelopMod())
      DbModel::insert(
        stbldev_parsing_q,
        values,
        false,
        false);


    rmoveFromParsingQ({
      {"pq_parse_attempts", CConsts::MAX_PARSE_ATTEMPS_COUNT, ">"},
      {"pq_creation_date", CUtils::minutesBefore(CMachine::getCycleByMinutes()), "<"}
    });

    if (!CMachine::isInSyncProcess())
      CGUI::signalUpdateParsingQ();
    return { true, true};

  } catch (std::exception) {
    CLog::log("push To Parsing Q Sync was failed on block(" + code + ") type(" + type + ") from(" + sender + ")!", "app", "error");
    return {false, true};

  }
}


bool ParsingQHandler::rmoveFromParsingQ(const ClausesT& clauses)
{
  DbModel::dDelete(
    stbl_parsing_q,
    clauses
  );
  return true;
}

 */