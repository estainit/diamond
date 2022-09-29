use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NormalBlock
{}

impl NormalBlock
{
    pub fn new() -> Self
    {
        Self {}
    }
}

/*


::NormalBlock(const JSonObject& obj)
{
  setByJsonObj(obj);
}


bool NormalBlock::setByJsonObj(const JSonObject& obj)
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
  if (m_floating_votes.len() > 0)
    fVotes = m_floating_votes;
  return cutils::serializeJson(fVotes);
}


String NormalBlock::getBlockHashableString() const
{
  // in order to have almost same hash! we sort the attribiutes alphabeticaly
  String hashable_block = "{";
  hashable_block += "\"ancestors\":" + cutils::serializeJson(QVariant::fromValue(m_ancestors).toJsonArray()) + ",";
  hashable_block += "\"backer\":\"" + m_block_backer + "\",";
  hashable_block += "\"bCDate\":\"" + m_block_creation_date + "\",";
  hashable_block += "\"bExtHash\":\"" + m_block_ext_root_hash + "\",";  // note that we do not put the segwits directly in block hash, instead using segwits-merkle-root-hash
  hashable_block += "\"bLen\":\"" + cutils::paddingLengthValue(m_block_length) + "\",";
  hashable_block += "\"bType\":\"" + m_block_type + "\",";
  hashable_block += "\"bVer\":\"" + m_block_version + "\",";
  hashable_block += "\"docsRootHash\":\"" + m_documents_root_hash + "\",";  // note that we do not put the docsHash directly in block hash, instead using docsHash-merkle-root-hash
  hashable_block += "\"fVotes\":" + stringifyFloatingVotes() + ",";
  hashable_block += "\"net\":\"" + m_net + "\",";
  hashable_block += "\"signals\":" + cutils::serializeJson(m_signals) + "}";
  return hashable_block;
}

JSonObject NormalBlock::exportBlockToJSon(const bool ext_info_in_document) const
{
  JSonObject Jblock = Block::exportBlockToJSon(ext_info_in_document);

  Jblock["fVotes"] = QJsonArray{};  // legacy including unimplemented feaure in blocks in order to forward compatibility
  if (m_floating_votes.len() > 0)
    Jblock["fVotes"] = m_floating_votes;

  Jblock["bLen"] = cutils::paddingLengthValue(calcBlockLength(Jblock));

  return Jblock;
}

BlockLenT NormalBlock::calcBlockLength(const JSonObject& block_obj) const
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


bool NormalBlock::controlBlockLength() const
{
  String stringyfied_block = safeStringifyBlock(false);
  if (static_cast<BlockLenT>(stringyfied_block.len()) != m_block_length)
  {
    CLog::log("Mismatch Normal block length Block(" + cutils::hash8c(m_block_hash) + ") local length(" + String::number(stringyfied_block.len()) + ") remote length(" + String::number(m_block_length) + ") stringyfied_block:" + stringyfied_block, "sec", "error");
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

   auto[status, is_sus_block, validate_msg, double_spends] = validate_transactions(this, stage);
   if (!status)
    return {false, false, validate_msg, {}};


  TransientBlockInfo transient_block_info = group_docs_of_block(stage);
  if (!transient_block_info.m_status)
  {
    return {false, false, "Failed in group-Docs-Of-Block", {}};
//       grpdRes.shouldPurgeMessage = true;
  }

  VString dTyps = transient_block_info.m_grouped_documents.keys();
  dTyps.sort();
  CLog::log("Block(" +cutils::hash6c(m_block_hash) + ") docs types(" + cutils::dumpIt(dTyps), "app", "info");

  // control if each trx is referenced to only one Document?
  VString tmpTrxs;
  for(String  key: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
    tmpTrxs.push(transient_block_info.m_map_trx_ref_to_trx_hash[key]);

  if (tmpTrxs.len() != cutils::array_unique(tmpTrxs).len())
  {
    msg = "invalid block! same transaction is used as a ref for different docs! Block(" +cutils::hash6c(m_block_hash) + ") mapTrxRefToTrxHash(" + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash);
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

  CLog::log("--- confirmed normal block(" + cutils::hash8c(m_block_hash) + ")");

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
  CLog::log("******** handle Received Normal Block(" + cutils::hash8c(m_block_hash)+ ")", "app", "trace");

  auto[status, is_sus_block, validate_msg, double_spends] = validateNormalBlock(constants::stages::Validating);

  CLog::log("Received a block of type(" + m_block_type + ") block(" +cutils::hash8c(m_block_hash) + "), validation result: is_sus_block(" + cutils::dumpIt(is_sus_block) + ") double_spends(" +cutils::dumpDoubleSpends(double_spends) + ")", "app", "trace");
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
  String cDate = cutils::getNow();
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
      "Broadcasting confirmed normal block(" + cutils::hash8c(m_block_hash) + ")");

    CLog::log("Normal block pushRes(" + cutils::dumpIt(pushRes) + ")");


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
        CLog::log("\n\nFailed on generating floating vote(susVote) : of block uplink(" + cutils::hash8c(m_block_hash) + ") ", "app", "error");
        return {false, true};
      }
      String stringified_block = tmp_block->safeStringifyBlock();
      CLog::log(
        "\n\nBroadcasting floating vote(susVote) because of block uplink(" +
        cutils::hash8c(m_block_hash) + ") FVBlock(" + cutils::hash8c(tmp_block->getBlockHash()) +
        ") " + stringified_block, "app", "trace");

      bool pushRes = SendingQHandler::pushIntoSendingQ(
        tmp_block->m_block_type,
        tmp_block->getBlockHash(),
        stringified_block,
        "Broadcasting susVote block$(" + cutils::hash8c(tmp_block->getBlockHash()) + ")");
      CLog::log("Normal block pushRes(" + cutils::dumpIt(pushRes) + ")");

      delete tmp_block;
    }

    return {true, true};

  }
  else if (CMachine::isInSyncProcess())
  {
    if (is_sus_block)
    {
      CLog::log("machine in sync mode and found a sus block uplink(" + cutils::hash8c(m_block_hash) + ") ");
      delete double_spends;
    }
    return {true, true};
  }

  return {false, false};
}

*/