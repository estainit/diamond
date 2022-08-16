/*

void ParsingQHandler::loopSmartPullFromParsingQ()
{
  String thread_prefix = "pull_from_parsing_q_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    MissedBlocksHandler::refreshMissedBlock();

    smartPullQ();

    CLog::log("Smart Pull From Parsing Q, Every (" + String::number(CMachine::getParsingQGap()) + " seconds) ", "app", "trace");
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getParsingQGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Smart Pull From Parsing Q");
}

/**
 * @return std::tuple<bool status, bool should_purge_record>
 */
std::tuple<bool, bool> ParsingQHandler::handlePulledPacket(const QVDicT& packet)
{
//  listener.doCallAsync('APSH_before_handle_pulled_packet', args);

  CLog::log("handle Pulled Packet: " + cutils::dumpIt(packet), "app", "trace");

  String receive_date = packet.value("pq_receive_date", cutils::get_now()).to_string();
  String pq_type = packet.value("pq_type", "").to_string();
  String pq_code = packet.value("pq_code", "").to_string();
  String pq_sender = packet.value("pq_sender", "").to_string();
  String connection_type = packet.value("pq_connection_type", "").to_string();
  /**
  * payload could be a block, GQL or even old-style messages
  * TODO: optimizine to use heap allocation for bigger payloads
  */
  JSonObject payload = packet.value("pq_payload", JSonObject()).toJSonObject();

  if ((pq_sender == "") || (payload.keys().len() == 0))
  {
    CLog::log("missed sender or paylod to parse", "app", "error");
    return {false, true};
  }

  if (pq_type == "")
  {
    CLog::log("missed pq_type " + cutils::dumpIt(packet), "app", "error");
    return {false, true};
  }
  if (connection_type == "")
  {
    CLog::log("missed connection_type in parsing " + cutils::dumpIt(packet), "app", "error");
    return {false, true};
  }

  if(payload.value("bType").to_string() == constants::BLOCK_TYPES::RpBlock)
  {
    CLog::log("A repay Block received block(" + cutils::hash8c(payload.value("bHash").to_string()) + ")", "trx", "info");
    // Since machine must create the repayments by itself we drop this block immidiately,
    // in addition machine calls importCoinbasedUTXOs method to import potentially minted coins and cut the potentially repay backs in on shot
    return {true, true};
  }



  if (StringList {constants::BLOCK_TYPES::Normal,
  constants::BLOCK_TYPES::Coinbase,
  constants::BLOCK_TYPES::FSign,
  constants::BLOCK_TYPES::SusBlock,
  constants::BLOCK_TYPES::FVote,
  constants::BLOCK_TYPES::POW}.contains(pq_type))
  {
    payload["local_receive_date"] = receive_date;
    Block* block = BlockFactory::create(payload);

    if (!block->objectAssignmentsControlls())
    {
      CLog::log("Maleformed JSon block couldn't be parsed! block(" + cutils::hash8c(payload.value("bHash").to_string()) + ")", "trx", "error");
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

    if (!CMachine::is_in_sync_process())
      CGUI::refreshMonitor();

    return {status2, should_purge_record2};

  }

  CLog::log(
    "\n\n--- parsing CPacket type(" + pq_type + ") Block/Message \nfrom Q.sender(" + pq_sender + ") ", "app", "trace");

  // GQL part
  if (pq_type == constants::CARD_TYPES::ProposalLoanRequest)
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
  else if (pq_type == constants::CARD_TYPES::FullDAGDownloadRequest)
  {
    auto[status, should_purge_record] = FullDAGHandler::prepareFullDAGDlResponse(
      pq_sender,
      payload,
      connection_type);
    return {status, should_purge_record};

  }
  else if (pq_type == constants::CARD_TYPES::pleaseRemoveMeFromYourNeighbors)
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
  else if (pq_type == constants::MESSAGE_TYPES::DAG_INVOKE_BLOCK)
  {
    //comunications
    auto[status, should_purge_record] = DAGMessageHandler::handleBlockInvokeReq(
      pq_sender,
      payload,
      connection_type);
    return {status, should_purge_record};

  }
  else if (pq_type == constants::MESSAGE_TYPES::DAG_INVOKE_DESCENDENTS)
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
  const String& sender,
  const String& pq_type,
  const Block* block,
  const String& connection_type,
  const String& receive_date
  )
{
  Q_UNUSED(sender);
  Q_UNUSED(connection_type);
  Q_UNUSED(receive_date);

  // DAG existance ancestors controlls
  StringList needed_blocks = cutils::arrayDiff(block.m_ancestors, DAG::getCachedBlocksHashes());
  if (needed_blocks.len() > 0)
  {
    CLog::log(
      "in order to parse 1block(" + cutils::hash6c(block->getBlockHash()) + ") machine needs blocks(" +
      cutils::dumpIt(needed_blocks) + ") exist in DAG"
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
  const JSonObject& message,
  const String& creation_date,
  const String& type,
  const String& code,
  const String& sender,
  const String& connection_type,
  StringList prerequisites)
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
    if (dbl.records.len() > 0)
      return { true, true };

//    listener.doCallSync('SPSH_before_insert_packet_in_q', args);


    // control if needs some initiative prerequisities
    StringList message_ancestors = {};
    if (message.keys().contains("ancestors") && (message.value("ancestors").toArray().len() > 0))
    {
      for(auto an_anc: message.value("ancestors").toArray())
      {
        message_ancestors.push(an_anc.to_string());
      }

//      // check if ancestores exist in parsing q
//      QueryRes queuedAncs = DbModel::select(
//        stbl_parsing_q,
//        {"pq_code"},
//        {{"pq_code", message_ancestors, "IN"}});

//      StringList missedAnc = {};
//      if (queuedAncs.records.len() == 0)
//      {
//        missedAnc = message_ancestors;
//        CLog::log("block(" + code + ") totaly missed ancestors (" + cutils::dumpIt(missedAnc) + ")", "app", "trace");
//      }
//      else if (queuedAncs.records.len() < message_ancestors.len())
//      {
//        StringList pq_codes = {};
//        for(QVDicT a_row: queuedAncs.records)
//          pq_codes.push(a_row.value("pq_code").to_string());
//        missedAnc = cutils::arrayDiff(message_ancestors, pq_codes);
//        CLog::log("block(" + code + ") partially missed ancestors (" + cutils::dumpIt(missedAnc) + ") ", "app", "trace");
//      }

      CLog::log("block(" + code + ") before + missed ancs (" + cutils::dumpIt(prerequisites) + "\n\n " + cutils::dumpIt(message_ancestors), "app", "trace");

      // control if missedAnc alredy exist in DAG?
      StringList exist_in_DAG;
      StringList existed_blocks_in_DAG = DAG::getCachedBlocksHashes();
      for (CBlockHashT an_ancestor: message_ancestors)
        if (existed_blocks_in_DAG.contains(an_ancestor))
          exist_in_DAG.push(an_ancestor);

//      QVDRecordsT DAGedAncs = DAG::searchInDAG(
//        {{"b_hash", message_ancestors, "IN"}},
//        {"b_hash"});

//      if (DAGedAncs.len() > 0)
//      {
//        StringList exist_in_DAG;
//        for (QVDicT x: DAGedAncs)
//        {
//          exist_in_DAG.push(x.value("b_hash").to_string());
//        }

        CLog::log("some likly missed blocks(" + message_ancestors.join(",") + ") already recorded in DAG(" + exist_in_DAG.join(",") + ")", "app", "trace");
        message_ancestors = cutils::arrayDiff(message_ancestors, exist_in_DAG);
//      }
      prerequisites = cutils::arrayAdd(prerequisites, message_ancestors);
    }

    /**
     * if blcok is FVote, maybe we need customized treatment, since generally in DAG later blocks are depend on
     * early blocks and it is one way graph.
     * but in case of vote blocks, they have effect on previous blocks (e.g accepting or rejecting a transaction of previously block)
     * so depends on voting type(bCat) for, we need proper treatment
     */
    if (message.value("bType").to_string() == constants::BLOCK_TYPES::FVote)
    {

      if (message.value("bCat").to_string() == constants::FLOAT_BLOCKS_CATEGORIES::Trx)
      {
        /**
        * if the machine get an FVote, so insert uplink block in SUS BLOCKS WHICH NEEDED VOTES TO BE IMPORTED AHAED(SusBlockWNVTBIA)
        * WNVTBIA: Wait becaue Needs Vote To Be Importable
        */
        String uplinkBlock = message.value("ancestors").toArray()[0].to_string();    // FVote blocks always have ONLY one ancestor for which Fvote is voting
        String currentWNVTBIA = KVHandler::getValue("SusBlockWNVTBIA");
        StringList currentWNVTBIA_arr = {};
        if (currentWNVTBIA == "")
        {
          currentWNVTBIA_arr.push(uplinkBlock);
        } else {
          auto tmp = cutils::parseToJsonArr(currentWNVTBIA);
          for(auto x: tmp)
            currentWNVTBIA_arr.push(x.to_string());
          currentWNVTBIA_arr.push(uplinkBlock);
          currentWNVTBIA_arr = cutils::arrayUnique(currentWNVTBIA_arr);
        }
        currentWNVTBIA = cutils::serializeJson(currentWNVTBIA_arr);
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
      {"pq_receive_date", cutils::get_now()},
      {"pq_payload", BlockUtils::wrapSafeContentForDB(cutils::serializeJson(message)).content},
      {"pq_prerequisites", "," + prerequisites.join(",")},  //"," prefix intentionally was added
      {"pq_parse_attempts", 0},
      {"pq_v_status", "new"},
      {"pq_creation_date", creation_date},
      {"pq_insert_date", cutils::get_now()},
      {"pq_last_modified", cutils::get_now()}
    };
    DbModel::insert(
      stbl_parsing_q,
      values,
      false,
      false);

//    listener.doCallSync('SPSH_after_insert_packet_in_q', args);

    if (CMachine::is_develop_mod())
      DbModel::insert(
        stbldev_parsing_q,
        values,
        false,
        false);


    rmoveFromParsingQ({
      {"pq_parse_attempts", constants::MAX_PARSE_ATTEMPS_COUNT, ">"},
      {"pq_creation_date", cutils::minutes_before(cutils::get_cycle_by_minutes()), "<"}
    });

    if (!CMachine::is_in_sync_process())
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