/*

/**
 * @brief NormalBlcokHandler::createANormalBlock
 * @param creation_date
 * @return {creating block status, should empty buffer, msg}
 */
std::tuple<bool, Block*, bool, String> NormalBlcokHandler::createANormalBlock(
  StringList ancestors,
  CDateT creation_date,
  const bool allowed_to_double_spend) // test purpose
{
  String msg;
  bool should_reset_block_buffer = true;
  CLog::log("create ANormalBlock create A NormalBlock create A NormalBlock");

  if (creation_date == "")
    creation_date = CUtils::getNow();

  if (!LeavesHandler::hasFreshLeaves())
  {
    msg = "Machine hasn't fresh leaves, so it can not broadcas new block(Normal block)";
    CLog::log(msg, "app", "warning");
    DAGMessageHandler::setMaybeAskForLatestBlocksFlag(constants::YES);
    return {false, nullptr, false, msg};
  }

  Block* block = BlockFactory::create(QJsonObject{
    {"bType", constants::BLOCK_TYPES::Normal},
    {"net", constants::SOCIETY_NAME}});

  block->m_block_creation_date = creation_date;
  block->m_signals = NodeSignalsHandler::getMachineSignals();
  block->m_documents = {};
  block->m_block_ext_root_hash = "";  // bExtHash
  StringList externalInfoHashes {};
  block->m_block_ext_info = QJsonArray {};

  block->m_block_backer = CMachine::getBackerAddress();

  /**
   * the first step of creating a block is appending the transactions
   * each block MUST have at least one transaction
   */
  TransientBlockInfo transient_block_info {};
  auto[append_res, should_reset_block_buffer1, append_res_msg] = TransactionsInRelatedBlock::appendTransactions(block, transient_block_info);
  should_reset_block_buffer &= should_reset_block_buffer1;
  if (!append_res)
    return {append_res, block, should_reset_block_buffer, append_res_msg};

  auto[groupping_res, should_reset_block_buffer2, groupping_res_msg] = CMachine::retrieveAndGroupBufferedDocuments(block, transient_block_info);
  should_reset_block_buffer &= should_reset_block_buffer2;
  if (!groupping_res)
    return {groupping_res, block, should_reset_block_buffer, groupping_res_msg};


  // control if each trx is referenced to only one Document?
  StringList tmpTrxs {};
  for(CDocHashT a_trx_ref: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
    tmpTrxs.append(transient_block_info.m_map_trx_ref_to_trx_hash[a_trx_ref]);
  if (tmpTrxs.size()!= CUtils::arrayUnique(tmpTrxs).size())
  {
    msg = "Creating new block, same transaction is used as a ref for different docs! " + CUtils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash);
    CLog::log(msg, "app", "error");
    return {false, block, false, msg};
  }

  // TODO: important! currently the order of adding documents to block is important(e.g. polling must be added before proposalsand pledges)
  // improve the code and remove this dependency

  /**
  * add free Documents(if exist)
  * since block size controlling is not implemented completaly, it is better to put this part at the begening of appending,
  * just in order to be sure block has enough capacity to include entire docs in buffer
  */
  auto[free_append_res, free_should_reset_block_buffer, free_append_res_msg] = FreeDocumentsInRelatedBlock::appendFreeDocsToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= free_should_reset_block_buffer;
  if (!free_append_res)
    return {false, block, should_reset_block_buffer, free_append_res_msg};

  /**
   * add vote-ballots(if exist)
   */
  auto[ballot_append_res, ballot_should_reset_block_buffer, ballot_append_res_msg] = BallotsInRelatedBlock::appendBallotsToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= ballot_should_reset_block_buffer;
  if (!ballot_append_res)
    return {false, block, should_reset_block_buffer, ballot_append_res_msg};


  /**
   * add iName-reg-req(if exist)
   */
  auto[iname_append_res, iname_should_reset_block_buffer, iname_append_res_msg] = INamesInRelatedBlock::appendINamesToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= iname_should_reset_block_buffer;
  if (!iname_append_res)
    return {false, block, should_reset_block_buffer, iname_append_res_msg};


  /**
   * add bind iName(if exist)
   */
  auto[iname_pgp_bind_append_res, iname_pgp_bind_should_reset_block_buffer, iname_pgp_bind_append_res_msg] = INamesBindsInRelatedBlock::appendINameBindsToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= iname_pgp_bind_should_reset_block_buffer;
  if (!iname_pgp_bind_append_res)
    return {false, block, should_reset_block_buffer, iname_pgp_bind_append_res_msg};


//  /**
//   * add msg to iName(if exist)
//   */
//  let addInameMsgRes = iNameMsgsInRelatedBlock.appendINameMsgsToBlock(appendArgs);
//  clog.app.info(`addInameMsgRes: ${utils.stringify(addInameMsgRes)}`);
//  if (addInameMsgRes.err != false) {
//      clog.app.error(`addInameMsgRes ${addInameMsgRes.msg}`);
//      return addInameMsgRes;
//  }
//  block = addInameMsgRes.block;
//  transient_block_info.m_block_documents_hashes = addInameMsgRes.docsHashes;
//  externalInfoHashes = addInameMsgRes.externalInfoHashes;
//  if (addInameMsgRes.addedDocs > 0)
//      console.log(`\n\nblockAfterAdding iName-register: ${utils.stringify(block)}`);

  /**
   * add admPolling(if exist)
   */
  auto[adm_polling_append_res, adm_polling_should_reset_block_buffer, adm_polling_append_res_msg] = AdministrativePollingsInRelatedBlock::appendAdmPollingsToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= adm_polling_should_reset_block_buffer;
  if (!adm_polling_append_res)
    return {false, block, should_reset_block_buffer, adm_polling_append_res_msg};



//  /**
//   * add ReqForRelRes(if exist)
//   * TODO: move it to appendAdmPollingsToBlock
//   */
//  let addRelCoinsRes = reqRelRessInRelatedBlock.appendReqRelResToBlock(appendArgs);
//  if (addRelCoinsRes.err != false) {
//      clog.app.error(`addRelCoinsRes ${addRelCoinsRes.msg}`);
//      return addRelCoinsRes;
//  }
//  block = addRelCoinsRes.block;
//  transient_block_info.m_block_documents_hashes = addRelCoinsRes.docsHashes;
//  externalInfoHashes = addRelCoinsRes.externalInfoHashes;
//  if (addRelCoinsRes.addedDocs > 0)
//      console.log(`\n\nblockAfterAdding ReqRelRes: ${utils.stringify(block)}`);


  /**
   * add polling(if exist) except pollings for proposal which are generating automatically
   */
  auto[polling_append_res, polling_should_reset_block_buffer, polling_append_res_msg] = PollingsInRelatedBlock::appendPollingsToBlock(
    block,
    transient_block_info);
    should_reset_block_buffer &= polling_should_reset_block_buffer;
    if (!polling_append_res)
      return {false, block, should_reset_block_buffer, polling_append_res_msg};

  /**
   * add proposals(if exist)
   */
  auto[proposal_append_res, proposal_should_reset_block_buffer, proposal_append_res_msg] = ProposalsInRelatedBlock::appendProposalsToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= proposal_should_reset_block_buffer;
  if (!proposal_append_res)
    return {false, block, should_reset_block_buffer, proposal_append_res_msg};



  /**
   * add pledges(if exist)
   */
  auto[pledge_append_res, pledge_should_reset_block_buffer, pledge_append_res_msg] = PledgesInRelatedBlock::appendPledgesToBlock(
    block,
    transient_block_info);
  should_reset_block_buffer &= pledge_should_reset_block_buffer;
  if (!pledge_append_res)
    return {false, block, should_reset_block_buffer, pledge_append_res_msg};


//  /**
//   * add redeem pledges(if exist)
//   */
//  let addClosePledgesRes = closePledgeInRelatedBlock.appendClosePledgesToBlock(appendArgs);
//  clog.app.info(`addClosePledgesRes: ${utils.stringify(addClosePledgesRes)}`);
//  if (addClosePledgesRes.err != false) {
//      console.log(`addClosePledgesRes ${addClosePledgesRes.msg}`);
//      return addClosePledgesRes;
//  }
//  block = addClosePledgesRes.block;
//  transient_block_info.m_block_documents_hashes = addClosePledgesRes.docsHashes;
//  externalInfoHashes = addClosePledgesRes.externalInfoHashes;
//  if (addClosePledgesRes.addedDocs > 0)
//      console.log(`\n\nblockAfterAdding close-pledges: ${utils.stringify(block)}`);









  // retrieve wiki page
  // retrieve demos text
  // retrieve ...




  CLog::log("Creating the NORMAL block which has " + String::number(transient_block_info.m_block_documents_hashes.size()) + " document(s)");

  auto[doc_status, doc_root_hash] = block->calcDocumentsRootHash();
  if (!doc_status)
    return {false, block, false, "Failed in creation documents root hash"};
  block->m_documents_root_hash = doc_root_hash;


  auto[ext_status, ext_root_hash] = block->calcBlockExtRootHash();
  if (!ext_status)
    return {false, block, false, "Failed in creation documents ext root hash"};
  block->m_block_ext_root_hash = ext_root_hash;

  if (ancestors.size() > 0)
  {
    block->m_ancestors = ancestors;

  } else {
    block->m_ancestors = CUtils::arrayAdd(LeavesHandler::getLeaveBlocks().keys(), block->m_ancestors);

  }
  block->m_ancestors = BlockUtils::normalizeAncestors(block->m_ancestors);
  if (transient_block_info.m_pre_requisities_ancestors.size() > 0)
  {
    CLog::log("The outgoing block has to has some ancestors because of related polling creation block(s): " + transient_block_info.m_pre_requisities_ancestors.join(", "), "app", "info");
    block->m_ancestors = CUtils::arrayUnique(CUtils::arrayAdd(block->m_ancestors, transient_block_info.m_pre_requisities_ancestors));
  }
  block->m_ancestors.sort();

  CLog::log("The NORMAL block will be descendent of these ancestors: " + CUtils::dumpIt(block->m_ancestors));

  // fill in the bloc.m_block_ext_info
  block->fillInBlockExtInfo();

  block->calcAndSetBlockLength();
  block->setBlockHash(block->calcBlockHash());

  TransientBlockInfo transient_block_info2 = block->groupDocsOfBlock(constants::STAGES::Creating);
  if (!transient_block_info2.m_status)
    return {false, block, false, "Failed in group Docs Of Block"};

  CLog::log("Final block, before transactions validate: " + block->safeStringifyBlock(false), "app", "trace");

  // re-validate block transactions
  if (!allowed_to_double_spend)
  {

    auto[status, is_sus_block, validate_msg, double_spends] = TransactionsInRelatedBlock::validateTransactions(block, constants::STAGES::Creating);
    Q_UNUSED(is_sus_block);
    Q_UNUSED(double_spends);
    if (!status)
      return {false, block, false, "Failed in validate transactions. " + validate_msg};
  }


  return {
    true,
    block,
    should_reset_block_buffer,
    "Normal block created. block(" + block->getBlockHash() + ")"};
}



*/