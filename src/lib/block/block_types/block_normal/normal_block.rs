/*


NormalBlock::NormalBlock(const QJsonObject& obj)
{
  setByJsonObj(obj);
}


bool NormalBlock::setByJsonObj(const QJsonObject& obj)
{
  Block::setByJsonObj(obj);


  // custom settings for Normal block
  auto[shares_, percentage] = DNAHandler::getAnAddressShares(m_block_backer, m_block_creation_date);
  Q_UNUSED(shares_);
  m_block_confidence = percentage;

  return true;
}

String NormalBlock::dumpBlock() const
{
  // firsdt call parent dump
  String out = Block::dumpBlock();

  // then child dumpping
  out += "\n in child";
  return out;
}


String NormalBlock::stringifyFloatingVotes() const
{
  // process m_floating_votes (if exist)
  QJsonArray fVotes{};  // legacy including unimplemented feaure in blocks in order to forward compatibility
  if (m_floating_votes.size() > 0)
    fVotes = m_floating_votes;
  return CUtils::serializeJson(fVotes);
}


String NormalBlock::getBlockHashableString() const
{
  // in order to have almost same hash! we sort the attribiutes alphabeticaly
  String hashable_block = "{";
  hashable_block += "\"ancestors\":" + CUtils::serializeJson(QVariant::fromValue(m_ancestors).toJsonArray()) + ",";
  hashable_block += "\"backer\":\"" + m_block_backer + "\",";
  hashable_block += "\"bCDate\":\"" + m_block_creation_date + "\",";
  hashable_block += "\"bExtHash\":\"" + m_block_ext_root_hash + "\",";  // note that we do not put the segwits directly in block hash, instead using segwits-merkle-root-hash
  hashable_block += "\"bLen\":\"" + CUtils::paddingLengthValue(m_block_length) + "\",";
  hashable_block += "\"bType\":\"" + m_block_type + "\",";
  hashable_block += "\"bVer\":\"" + m_block_version + "\",";
  hashable_block += "\"docsRootHash\":\"" + m_documents_root_hash + "\",";  // note that we do not put the docsHash directly in block hash, instead using docsHash-merkle-root-hash
  hashable_block += "\"fVotes\":" + stringifyFloatingVotes() + ",";
  hashable_block += "\"net\":\"" + m_net + "\",";
  hashable_block += "\"signals\":" + CUtils::serializeJson(m_signals) + "}";
  return hashable_block;
}

QJsonObject NormalBlock::exportBlockToJSon(const bool ext_info_in_document) const
{
  QJsonObject Jblock = Block::exportBlockToJSon(ext_info_in_document);

  Jblock["fVotes"] = QJsonArray{};  // legacy including unimplemented feaure in blocks in order to forward compatibility
  if (m_floating_votes.size() > 0)
    Jblock["fVotes"] = m_floating_votes;

  Jblock["bLen"] = CUtils::paddingLengthValue(calcBlockLength(Jblock));

  return Jblock;
}

BlockLenT NormalBlock::calcBlockLength(const QJsonObject& block_obj) const
{
  return Block::calcBlockLength(block_obj);
}


String NormalBlock::calcBlockHash() const
{
  String hashable_block = getBlockHashableString();

  // clonedTransactionsRootHash: block.clonedTransactionsRootHash, // note that we do not put the clonedTransactions directly in block hash, instead using clonedTransactions-merkle-root-hash

  CLog::log("The NORMAL! block hashable: " + hashable_block + "\n", "app", "trace");
  return CCrypto::keccak256(hashable_block);
}

std::tuple<bool, String> NormalBlock::calcBlockExtRootHash() const
{
  // for POW blocks the block has only one document and the dExtHash of doc and bExtHash of block are equal
  StringList doc_ext_hashes = {};
  for(Document* a_doc: m_documents)
    doc_ext_hashes.append(a_doc->m_doc_ext_hash);
  auto[documentsExtRootHash, final_verifies, version, levels, leaves] = CMerkle::generate(doc_ext_hashes);
  Q_UNUSED(final_verifies);
  Q_UNUSED(version);
  Q_UNUSED(levels);
  Q_UNUSED(leaves);
  return {true, documentsExtRootHash};
}

bool NormalBlock::controlBlockLength() const
{
  String stringyfied_block = safeStringifyBlock(false);
  if (static_cast<BlockLenT>(stringyfied_block.len()) != m_block_length)
  {
    CLog::log("Mismatch Normal block length Block(" + CUtils::hash8c(m_block_hash) + ") local length(" + String::number(stringyfied_block.len()) + ") remote length(" + String::number(m_block_length) + ") stringyfied_block:" + stringyfied_block, "sec", "error");
    return false;
  }
  return true;
}

//QJsonArray NormalBlock::getBlockExtInfoByDocIndex(const CDocIndexT& document_index) const
//{
//  return m_block_ext_info.toJsonArray()[document_index].toVariant();
//}

/**
 * @brief NormalBlock::validateNormalBlock
 * @param stage
 * @return {status, is_sus_block, double_spends}
 */
std::tuple<bool, bool, String, SpendCoinsList*> NormalBlock::validateNormalBlock(
  const String& stage) const
{
  String msg = "";
//   let hookValidate = listener.doCallSync('SASH_before_validate_normal_block', args);
   CLog::log("xxxxxxxxxxxx validate Normal Block xxxxxxxxxxxxxxxxxxxx", "app", "trace");
   CLog::log("\n\n\n" + dumpBlock(), "app", "trace");

   auto[status, is_sus_block, validate_msg, double_spends] = TransactionsInRelatedBlock::validateTransactions(this, stage);
   if (!status)
    return {false, false, validate_msg, {}};


  TransientBlockInfo transient_block_info = groupDocsOfBlock(stage);
  if (!transient_block_info.m_status)
  {
    return {false, false, "Failed in group-Docs-Of-Block", {}};
//       grpdRes.shouldPurgeMessage = true;
  }

  StringList dTyps = transient_block_info.m_groupped_documents.keys();
  dTyps.sort();
  CLog::log("Block(" +CUtils::hash6c(m_block_hash) + ") docs types(" + CUtils::dumpIt(dTyps), "app", "info");

  // control if each trx is referenced to only one Document?
  StringList tmpTrxs;
  for(String  key: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
    tmpTrxs.append(transient_block_info.m_map_trx_ref_to_trx_hash[key]);

  if (tmpTrxs.size() != CUtils::arrayUnique(tmpTrxs).size())
  {
    msg = "invalid block! same transaction is used as a ref for different docs! Block(" +CUtils::hash6c(m_block_hash) + ") mapTrxRefToTrxHash(" + CUtils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash);
    CLog::log(msg, "sec", "error");
    return {false, false, msg, {}};
  }

  // TODO: important! currently the order of validating documents of block is important(e.g. polling must be validate before proposals and pledges)
  // improve the code and remove this dependency

  /**
   * validate polling request(if exist)
   */
  bool status_polling = PollingsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!status_polling)
    return {false, true, "Failed in validate-In-Block polling", {}};

  /**
   * validate requests for administrative polling(if exist)
   */
  bool adm_polling_validate_res = AdministrativePollingsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!adm_polling_validate_res)
    return {false, false, "Failed in validate-In-Block adm-polling", {}};

//  /**
//   * validate reqRelRes request(if exist)
//   * TODO: move it to validate
//   */
//  let reserveCoinsValidateRes = reqRelRessInRelatedBlock.validateReqRelRess(validateParams);
//  if (reserveCoinsValidateRes.err != false) {
//      return reserveCoinsValidateRes;
//  }

  /**
   * validate vote-ballots (if exist)
   */
  bool ballots_validate_res = BallotsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!ballots_validate_res)
    return {false, false, "Failed in validate-In-Block votes", {}};

  /**
   * validate proposals (if exist)
   */
  bool proposals_validate_res = ProposalsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!proposals_validate_res)
    return {false, false, "Failed in validate-In-Block proposals", {}};

  /**
   * validate pledges (if exist)
   */
  bool pledges_validate_res = PledgesInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!pledges_validate_res)
    return {false, false, "Failed in validate-In-Block pledges", {}};

  /**
   * validate close pledges (if exist)
   */
  bool close_pledges_validate_res = ClosePledgesInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!close_pledges_validate_res)
    return {false, false, "Failed in validate-In-Block close-pledges", {}};


  /**
   * validate iNames (if exist)
   */
  bool inames_validate_res = INamesInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!inames_validate_res)
    return {false, false, "Failed in validate-In-Block iNames", {}};

  /**
   * validate bind-iNames (if exist)
   */
  bool inames_bindings_validate_res = INamesBindsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!inames_bindings_validate_res)
    return {false, false, "Failed in validate-In-Block iName bindings", {}};

//  /**
//   * validate msg-to-iNames (if exist)
//   */
//  let iNameMsgsValidateRes = iNameMsgsInRelatedBlock.validateINameMsgs(validateParams);
//  if (iNameMsgsValidateRes.err != false) {
//      return iNameMsgsValidateRes;
//  }

  /**
   * validate free-docs (if exist)
   */
  bool free_documents_validate_res = FreeDocumentsInRelatedBlock::validateInBlock(this, transient_block_info, stage);
  if (!free_documents_validate_res)
    return {false, false, "Failed in validate-In-Block free-docs", {}};

  // validate...

  CLog::log("--- confirmed normal block(" + CUtils::hash8c(m_block_hash) + ")");

//  hookValidate = listener.doCallSync('SASH_validate_normal_block', block);
//  if (_.has(hookValidate, 'err')& & (hookValidate.err != false)) {
//      return hookValidate;
//  }

  return {
    true,
    is_sus_block,
    "valid",
    double_spends
  };
}

/**
* @brief NormalBlock::handleReceivedBlock
* @return <status, should_purge_record>
*/
// js name was handleReceivedNormalBlock
std::tuple<bool, bool> NormalBlock::handleReceivedBlock() const
{
  CLog::log("******** handle Received Normal Block(" + CUtils::hash8c(m_block_hash)+ ")", "app", "trace");

  auto[status, is_sus_block, validate_msg, double_spends] = validateNormalBlock(constants::STAGES::Validating);

  CLog::log("Received a block of type(" + m_block_type + ") block(" +CUtils::hash8c(m_block_hash) + "), validation result: is_sus_block(" + CUtils::dumpIt(is_sus_block) + ") double_spends(" +CUtils::dumpDoubleSpends(double_spends) + ")", "app", "trace");
  if (!status)
  {
    CLog::log(validate_msg, "app", "error");
    // maybe do something more! e.g. calling reputation system hooks via zmq
    return {false, true};
  }

  //TODO: prepare a mega query to run in atomic transactional mode
  addBlockToDAG();
  postAddBlockToDAG();

  // remove used UTXOs
  UTXOHandler::removeUsedCoinsByBlock(this);

  // log spend coins
  String cDate = CUtils::getNow();
  // if machine is in sync mode, we send half a cycle after creationdate to avoid deleting all spend records in table "trx_spend"
  if (CMachine::isInSyncProcess())
    cDate = m_block_creation_date;

  SpentCoinsHandler::markAsSpentAllBlockInputs(this, cDate);

  // broadcast block to neighbors
  if (DAG::isDAGUptodated())
  {
    bool pushRes = SendingQHandler::pushIntoSendingQ(
      m_block_type,
      m_block_hash,
      safeStringifyBlock(false),
      "Broadcasting confirmed normal block(" + CUtils::hash8c(m_block_hash) + ")");

    CLog::log("Normal block pushRes(" + CUtils::dumpIt(pushRes) + ")");


    if (is_sus_block)
    {
      auto[status_sus, tmp_block] = FloatingVoteBlock::createFVoteBlock(
        m_block_hash,  // uplink
        constants::FLOAT_BLOCKS_CATEGORIES::Trx,  // bCat
        SpentCoinsHandler::convertSpendsToJsonObject(double_spends), // voteData
        cDate);

      delete double_spends;

      if (!status_sus)
      {
        CLog::log("\n\nFailed on generating floating vote(susVote) : of block uplink(" + CUtils::hash8c(m_block_hash) + ") ", "app", "error");
        return {false, true};
      }
      String stringified_block = tmp_block->safeStringifyBlock();
      CLog::log(
        "\n\nBroadcasting floating vote(susVote) because of block uplink(" +
        CUtils::hash8c(m_block_hash) + ") FVBlock(" + CUtils::hash8c(tmp_block->getBlockHash()) +
        ") " + stringified_block, "app", "trace");

      bool pushRes = SendingQHandler::pushIntoSendingQ(
        tmp_block->m_block_type,
        tmp_block->getBlockHash(),
        stringified_block,
        "Broadcasting susVote block$(" + CUtils::hash8c(tmp_block->getBlockHash()) + ")");
      CLog::log("Normal block pushRes(" + CUtils::dumpIt(pushRes) + ")");

      delete tmp_block;
    }

    return {true, true};

  }
  else if (CMachine::isInSyncProcess())
  {
    if (is_sus_block)
    {
      CLog::log("machine in sync mode and found a sus block uplink(" + CUtils::hash8c(m_block_hash) + ") ");
      delete double_spends;
    }
    return {true, true};
  }

  return {false, false};
}

*/