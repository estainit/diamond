/*

QString DAG::stbl_blocks = "c_blocks";
QStringList DAG::stbl_blocks_fields = {"b_id", "b_hash", "b_type", "b_cycle", "b_confidence", "b_ext_root_hash", "b_docs_root_hash", "b_signals", "b_trxs_count", "b_docs_count", "b_ancestors_count", "b_ancestors", "b_descendents", "b_body", "b_creation_date", "b_receive_date", "b_confirm_date", "b_backer", "b_utxo_imported"};

QString DAG::stbl_docs_blocks_map = "c_docs_blocks_map";

void DAG::appendDescendents(const QStringList& block_hashes, const QStringList& new_descendents)
{
  if (new_descendents.size()>0)
  {
    for (QString a_block_hash : block_hashes)
    {
      QueryRes a_block_record = DbModel::select(
        stbl_blocks,
        {"b_hash", "b_descendents"},
        {{"b_hash", a_block_hash}},
        {},
        1,
        false,
        false);

      if (a_block_record.records.size() == 1)
      {
         QStringList current_descendents = a_block_record.records[0].value("b_descendents").toString().split(",");
         QStringList final_descendents = CUtils::arrayUnique(CUtils::arrayAdd(new_descendents, current_descendents));
         DbModel::update(
          stbl_blocks,
          {{"b_descendents", final_descendents.join(",")}},
          {{"b_hash", a_block_hash}},
          true,
          false);
      }
    }
  }
}

/**
 * @brief DAG::getBlockHashesByDocHashes
 * @param doc_hashes
 * @param hashes_type
 * @return {block_hashes, map_doc_to_block}
 */
std::tuple<QStringList, GRecordsT> DAG::getBlockHashesByDocHashes(
  const QStringList& doc_hashes,
  const QString& hashes_type)
{
  ClausesT clauses {};
  if (hashes_type == CConsts::SHORT)
  {
    QStringList tmp{};
    for(QString a_hash: doc_hashes)
      tmp.append(a_hash + "%");
    clauses.push_back({"dbm_doc_hash", tmp, "LIKE:OR"});
  } else {
    clauses.push_back({"dbm_doc_hash", doc_hashes, "IN"});
  }

  QueryRes res = DbModel::select(
    stbl_docs_blocks_map,
    {"dbm_block_hash", "dbm_doc_hash", "dbm_doc_index"},
    clauses);
  QStringList block_hashes{};
  GRecordsT map_doc_to_block{};
  for (QVDicT element: res.records)
  {
    block_hashes.append(element.value("dbm_block_hash").toString());
    // since we can have more than 1 coinbase block for each cycle, so mapping a document to its container block could be 1 to n mapping
    // also in lone transactions we have same psituation in which a certain transaction can take place in different blocks by different backers
    QString dbm_doc_hash = element.value("dbm_doc_hash").toString();
    if (!map_doc_to_block.keys().contains(dbm_doc_hash))
      map_doc_to_block[dbm_doc_hash] = QVDRecordsT{};
    map_doc_to_block[dbm_doc_hash].push_back(QVDicT{
      {"block_hash", element.value("dbm_block_hash")},
      {"doc_index", element.value("dbm_doc_index")}
    });
  }
  return {block_hashes, map_doc_to_block};
}

QVDRecordsT DAG::searchInDAG(
  const ClausesT& clauses,
  const QStringList& fields,
  const OrderT& order,
  const int& limit,
  const bool& is_transactional,
  const bool& do_log)
{
  QueryRes res = DbModel::select(
    stbl_blocks,
    fields,
    clauses,
    order,
    limit,
    is_transactional,
    do_log);
  return res.records;
}

std::tuple<QVDRecordsT, GRecordsT> DAG::getWBlocksByDocHash(
  const QStringList& doc_hashes,
  const QString& hashes_type)
{
  auto[block_hashes, map_doc_to_block] = getBlockHashesByDocHashes(doc_hashes, hashes_type);
    if (block_hashes.size() == 0)
      return {QVDRecordsT {}, map_doc_to_block};

  QVDRecordsT block_records = searchInDAG({{"b_hash", block_hashes, "IN"}});
  return {block_records, map_doc_to_block};
}

/**
 * @brief DAG::retrieveDocByDocHash
 * @param doc_hash
 * @param need_doc_ext_info
 * @param need_doc_merkle_proof
 * @return [status, document, doc_index, merkle_verify, doc_ext_info]
 */
std::tuple<bool, QJsonObject, CDocIndexT, MerkleNodeData, QJsonArray> DAG::retrieveDocByDocHash(
  const CDocHashT& doc_hash,
  const bool need_doc_ext_info,
  const bool need_doc_merkle_proof)
{
  QString msg;
  auto[block_records, map_] = getWBlocksByDocHash({doc_hash});
  if (block_records.size() == 0)
    return {false, {}, {}, {}, {}};

  QJsonObject block = CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(block_records[0].value("b_body").toString()).content);

  QJsonObject document {};
  CDocIndexT documentIndex = 0;
  QJsonArray docs = block["docs"].toArray();
  for (CDocIndexT docInx = 0; docInx < docs.size(); docInx++)
  {
    QJsonObject aDoc = docs[docInx].toObject();
    if ((document.keys().size() == 0) && (aDoc.value("dHash").toString() == doc_hash))
    {
      documentIndex = docInx;
      document = aDoc;
    }
  }

  if (document.keys().size() == 0)
    return {false, {}, {}, {}, {}};

  QJsonArray the_doc_ext_info {};
  if (need_doc_ext_info)
  {
    auto[status, extExist, block_ext_info] = Block::getBlockExtInfo(block.value("bHash").toString());
    if (!extExist)
    {
      msg = "missed bExtInfo3 (" + CUtils::hash8c(block.value("bHash").toString()) + ")";
      CLog::log(msg, "sec", "error");
      return {false, {}, {}, {}, {}};
    }
    block["bExtInfo"] = block_ext_info;
    the_doc_ext_info = block_ext_info[documentIndex].toArray();
  }

  MerkleNodeData proofs;
  if (need_doc_merkle_proof)
  {
    proofs = BlockUtils::getDocumentMerkleProof(
      block,
      doc_hash);
  }

  return {true, document, documentIndex, proofs, the_doc_ext_info};
}

/**
*
* @param {*} coins
* finding the blocks in which were created given coins
*/
QV2DicT DAG::getCoinsGenerationInfoViaSQL(const QStringList& coins)
{
  QStringList docsHashes {};
  for(QString a_coin: coins)
  {
    auto[doc_hash_, output_index_] = CUtils::unpackCoinCode(a_coin);
    Q_UNUSED(output_index_);
    docsHashes.append(doc_hash_);
  }
  CLog::log("find Output Info By RefLocViaSQL for docs: " + docsHashes.join(", "), "trx", "trace");

  auto[block_records, map_] = getWBlocksByDocHash(docsHashes);
  Q_UNUSED(map_);
  // console.log('find Output Info By RefLocViaSQL', block_records);
  QV2DicT outputsDict {};
  for (QVDicT wBlock: block_records)
  {
    QJsonObject block = CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock.value("b_body").toString()).content);
    if (block.keys().contains("docs"))
    {
      for (QJsonValueRef doc_: block.value("docs").toArray())
      {
        QJsonObject doc = doc_.toObject();
        if (doc.keys().contains("outputs"))
        {
          QJsonArray outputs = doc.value("outputs").toArray();
          for (COutputIndexT output_index = 0; output_index < static_cast<COutputIndexT>(outputs.size()); output_index++)
          {
            auto output = outputs[output_index].toArray();
            QString aCoin = CUtils::packCoinCode(doc.value("dHash").toString(), output_index);
            outputsDict[aCoin] = QVDicT {
              {"coinGenCycle", wBlock.value("b_cycle")},
              {"coinGenBlockHash", block.value("bHash")},
              {"coinGenBType", block.value("bType")},
              {"coinGenCreationDate", block.value("bCDate")}, // coin creation date is the BLOCK creation date and not the transaction date
              {"coinGenDocType", doc.value("dType")},
              {"coinGenRefLoc", aCoin},
              {"coinGenOutputAddress", output[0].toString()},
              {"coinGenOutputValue", output[1].toDouble()}};
          }
        }
      }
    }
  }

  QV2DicT res {};
  for (auto aCoin: coins)
  {
    if (outputsDict.keys().contains(aCoin))
      res[aCoin] = outputsDict[aCoin];
  }
  return res;
}

void DAG::recursive_backwardInTime(
  const QStringList& block_hashes,
  const QString& date,
  QStringList& ancestors)
{
  // console.log(`::::::::::: recursive_backwardInTime args: ${utils.stringify(args)}`);
  if (block_hashes.size()== 0)
    return;

  QVDRecordsT res = searchInDAG(
    {{"b_hash", block_hashes, "IN"},
    {"b_creation_date", date, ">"}},
    {"b_ancestors"});

  if (res.size()== 0)
    return;

  QStringList out {};
  for (QVDicT aRes: res)
    out = CUtils::arrayAdd(out, aRes.value("b_ancestors").toString().split(","));

  out = CUtils::arrayUnique(out);
  ancestors = CUtils::arrayAdd(ancestors, out);

  if (out.size()> 0)
    recursive_backwardInTime(
      out,
      date,
      ancestors);
}

/**
 *
 * @param {*} args
 * returns all ancestors of given block(s), which are youngre than a certain age (byMinutes or cycle)
 */
QStringList DAG::returnAncestorsYoungerThan(
  const QStringList& block_hashes,
  const QString& byDate,
  const int32_t& byMinutes,
  const int32_t& cycle)
{
//  CLog::log("return AncestorsYoungerThan: block_hashes: " + block_hashes.join(",") + "\nbyDate(" + byDate +")", "app", "trace");
  if (block_hashes.size() == 0)
    return {};

  QString date_;
  if (byDate != "")
  {
    date_ = byDate;
  } else if (byMinutes != -1)
  {
    date_ = CUtils::minutesBefore(byMinutes);
  } else if (cycle != -1)
  {
    date_ = CUtils::minutesBefore(CMachine::getCycleByMinutes() * cycle);
  }

  QStringList ancestors;
  recursive_backwardInTime(
    block_hashes,
    date_,
    ancestors
  );

  return ancestors;
}


QSDRecordsT DAG::analyzeDAGHealth(const bool shouldUpdateDescendants)
{
  // check if there are abb,andoned leaves
  QVDRecordsT wBlocks = searchInDAG(
    {},
    {"b_hash", "b_type", "b_ancestors", "b_descendents", "b_creation_date"},
    {{"b_creation_date", "DESC"}});



  QSDicT childByDad {};
  QV2DicT blocksInfo {};
  for (QVDicT wBlock: wBlocks)
  {
    blocksInfo[CUtils::hash8c(wBlock.value("b_hash").toString())] = QVDicT {
      {"b_type", wBlock.value("b_type").toString()},
      {"b_creation_date", wBlock.value("b_creation_date").toString()}};

    QStringList ancestors = wBlock.value("b_ancestors").toString().split(",");
    for (QString dadHash: ancestors)
    {
      childByDad[CUtils::hash8c(dadHash)] = CUtils::hash8c(wBlock.value("b_hash").toString());
    };
  };
  // console.log(blocksInfo);
  QSDRecordsT leaves {};
  QStringList tmp {};
  for (QString hash: blocksInfo.keys())
  {
    if (!childByDad.keys().contains(hash))
      tmp.append(hash);
  }

  tmp.sort();
  for (QString hash: tmp)
    leaves.push_back(QSDicT {
      {"b_ash", hash},
      {"b_type", blocksInfo[hash].value("b_type").toString()},
      {"b_creation_date", blocksInfo[hash].value("b_creation_date").toString().split(" ")[1]}});

  return leaves;
}


QVDRecordsT DAG::excludeFloatingBlocks(
  const QStringList& block_hashes,
  const QStringList& fields)
{
  // exclude floating signature blocks
  if (block_hashes.size()== 0)
    return {};

  QVDRecordsT block_records = searchInDAG(
    {{"b_hash", block_hashes, "IN"},
    {"b_type", {CConsts::BLOCK_TYPES::FSign, CConsts::BLOCK_TYPES::FVote}, "NOT IN"}},
    fields);
  return block_records;
}

void DAG::updateUtxoImported(
  const QString& block_hash,
  const QString& status)
{
  CLog::log("update Utxo is Imported Block(" + CUtils::hash8c(block_hash) + ") to imported(" + status + ")", "trx", "info");
  DbModel::update(
    stbl_blocks,
    {{"b_utxo_imported", status}},
    {{"b_hash", block_hash}});

  // update also cached blocks
  CMachine::cachedBlocks("update", {QVDicT{{"b_hash", block_hash}}}, status);

}

bool DAG::isDAGUptodated(QString cDate)
{
  if (cDate == "")
    cDate = CUtils::getNow();

  QVDRecordsT latestBlockDate = searchInDAG(
    {},
    {"b_creation_date"},
    {{"b_creation_date", "DESC"}},
    1);
  if (latestBlockDate.size() == 0)
      return false;

  QString latest_block_date = latestBlockDate[0].value("b_creation_date").toString();
  return CUtils::isYoungerThan2Cycle(latest_block_date);
  // FIXME: it must be more sophisticated such as missed blocks count, last received blocks which are in parsingQ...
}

bool DAG::loopPrerequisitiesRemover()
{
  QString thread_prefix = "prerequisities_remover_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    doPrerequisitiesRemover();

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getPrerequisitiesRemoverGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Prerequisities Remover");
  return true;
}
*/

//old_name_was doPrerequisitiesRemover
pub fn do_prerequisities_remover() -> bool
{
    /*

      // TODO: improve it to be more efficient
      // first loop on all blocks in Q
      QVDRecordsT queued_packets = ParsingQHandler::searchParsingQ({}, {"pq_prerequisites"});

      for (QVDicT a_cpack: queued_packets)
      {
        // control if already recorded in DAG
        QStringList pre = CUtils::unpackCommaSeperated(a_cpack.value("pq_prerequisites").toString());
        ClausesT clauses = {{"b_hash", pre, "IN"}};
        // if (machine.isInSyncProcess())
        //     query.push(['b_utxo_imported', 'Y'])    // to avoid removing prerequisities, before importing UTXOs
        if (pre.size() > 0)
        {
          QVDRecordsT existedBlocksInDAG = DAG::searchInDAG(clauses, {"b_hash"});
          if (existedBlocksInDAG.size() > 0)
          {
            for (QVDicT aBlock: existedBlocksInDAG)
            {
              CLog::log("Prerequisities Remover, removed dependencies to block(" + CUtils::hash8c(aBlock.value("b_hash").toString()) + ")", "app", "trace");
              // remove dependency to this block
              ParsingQHandler::removePrerequisites(aBlock.value("b_hash").toString());
            }
          }
        }
      }

      */
    return true;
}

/*

bool DAG::DAGHasBlocksWhichAreCreatedInCurrrentCycle(QString cDate)
{
  if (cDate == "")
    cDate = CUtils::getNow();

  QVDRecordsT latest_blocks = DAG::searchInDAG(
    {},
    {"b_creation_date"},
    {{"b_creation_date", "DESC"}},
    1);
  if (latest_blocks.size() == 0)
    return false;
  QString latest_block_date = latest_blocks[0].value("b_creation_date").toString();
  return CUtils::timeDiff(latest_block_date, cDate).asMinutes < CMachine::getCycleByMinutes();
}

// need to be fixed because of array response of get BlockHashByDocHash
std::vector<CCoin> DAG::retrieveBlocksInWhichARefLocHaveBeenProduced(const CCoinCodeT& the_coin)
{
  std::vector<CCoin> results {};
  CLog::log("retrieve Blocks In which the coin: (" + the_coin + ") is created", "app", "trace");

  auto[doc_hash, output_index_] = CUtils::unpackCoinCode(the_coin);
  Q_UNUSED(output_index_);

  auto[block_hashes, map_doc_to_block_] = getBlockHashesByDocHashes({doc_hash});
  Q_UNUSED(map_doc_to_block_);

  QVDRecordsT wBlocks = DAG::searchInDAG(
    {{"b_hash", block_hashes, "IN"}},
    {"b_hash"});

  if (wBlocks.size() == 0)
  {
    CLog::log("Retrieve Blocks In Which A Ref Loc Produced for the coin(" + the_coin + ") not exist!", "trx", "error");
    return results;
  }

  // it could be possible some docHash exactly exist in more than one block
  // e.g multiplicated coinbase blocks which are created in same cycle
  // e.g cloned transactions(the same transaction which takes part in different blocks by diferent backers)
  // therfore we return list of blocks

  for (QVDicT wBlock: wBlocks)
  {
    Block* block = BlockFactory::create(CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock.value("b_body")).content));

    for (CDocIndexT doc_index = 0; doc_index < block->m_documents.size(); doc_index++)
    {
      Document* doc = block->m_documents[doc_index];
      if (doc->docHasOutput())
      {
        for (COutputIndexT output_index = 0; output_index < doc->getOutputs().size(); output_index++)
        {
          CCoinCodeT tmp_coin = CUtils::packCoinCode(doc->getDocHash(), output_index);
          if (tmp_coin == the_coin)
            results.emplace_back(CCoin(
              the_coin,
              doc->getOutputs()[output_index]->m_address,
              doc->getOutputs()[output_index]->m_amount,

              block->m_block_creation_date,

              block->getBlockHash(),
              doc_index,
              doc_hash,
              output_index));

        }
      }
    }
  }

  return results;
}

std::tuple<CMPAIValueT, QVDRecordsT, CMPAIValueT> DAG::getNotImportedCoinbaseBlocks()
{
  QVDRecordsT wBlocks = searchInDAG(
    {{"b_utxo_imported", CConsts::NO},
    {"b_type", {CConsts::BLOCK_TYPES::FSign, CConsts::BLOCK_TYPES::FVote}, "NOT IN"},
    {"b_type", QStringList{CConsts::BLOCK_TYPES::Coinbase}, "IN"}});

  CMPAIValueT sum = 0;
  QVDRecordsT processed_outputs {};
  QStringList calculated_coinbase {};
  for (QVDicT wBlock: wBlocks)
  {
    QJsonObject block = CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock.value("b_body").toString()).content);

    if (calculated_coinbase.contains(block.value("cycle").toString()))
      continue;

    if (block.value("bType").toString() == CConsts::BLOCK_TYPES::Coinbase)
        calculated_coinbase.append(block.value("cycle").toString());

    // analyze outputs
    QJsonObject doc = block.value("docs").toArray()[0].toObject();
    QJsonArray outputs = doc.value("outputs").toArray();
    for (auto output_inx = 0; output_inx < outputs.size(); output_inx++)
    {
      auto output = outputs[output_inx].toArray();

      sum += output[1].toDouble();

      processed_outputs.push_back(QVDicT {
        {"block_type", wBlock.value("bType").toString()},
        {"creation_date", block.value("bCDate").toString().split(" ")[1]},
        {"doc_hash", CUtils::hash8c(doc.value("dHash").toString())},
        {"owner", CUtils::shortBech16(output[0].toString())},
        {"value", CUtils::microPAIToPAI6(output[1].toDouble())}});
    }
  }

//  let dbl_spends = []
//  for (let ref: maybe_dbl_spends.keys())
//  {
//    if (maybe_dbl_spends[ref] > 1)
//    {
//      dbl_spends.push(`${iutils.shortCoinRef(ref)}->${maybe_dbl_spends[ref]}`)
//    }
//  }
//  processed_outputs = processed_outputs.map(x => x.join('\t   ')).join('\n');

  CMPAIValueT coinbase_value = (calculated_coinbase.size() * CoinbaseIssuer::calcPotentialMicroPaiPerOneCycle(CUtils::getNow().split("-")[0]));

  return {sum, processed_outputs, coinbase_value};
}

std::tuple<CMPAIValueT, QStringList, QString> DAG::getNotImportedNormalBlock()
{
  QVDRecordsT wBlocks = searchInDAG(
    {{"b_utxo_imported", CConsts::NO},
    {"b_type", QStringList{CConsts::BLOCK_TYPES::Normal, "IN"}}});
  CMPAIValueT sum = 0;
  QHash<CDocHashT, int64_t> maybe_dbl_spends = {};
  QStringList processed_outputs {};

  for (QVDicT wBlock: wBlocks)
  {
    QJsonObject block = CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock.value("b_body").toString()).content);

    for (auto doc_: block.value("docs").toArray())
    {
      auto doc = doc_.toObject();
      if (!QStringList{CConsts::DOC_TYPES::BasicTx}.contains(doc.value("dType").toString()))
        continue;

      // since DPCostPay docs alredy are in transactions , so we do not calculate it 2 times

      // analyze inputs
      for (auto input: doc.value("inputs").toArray())
      {
        CCoinCodeT the_coin = CUtils::packCoinCode(input.toArray()[0].toString(), input.toArray()[1].toDouble());
        if (!maybe_dbl_spends.contains(the_coin))
          maybe_dbl_spends[the_coin] = 0;
        maybe_dbl_spends[the_coin] += 1;
      }

      // analyze outputs
      // console.log(doc);
      auto outputs = doc.value("outputs").toArray();
      for (auto output_inx = 0; output_inx < outputs.size(); output_inx++)
      {
        auto output = outputs[output_inx].toArray();
        if (doc.keys().contains("dPIs"))
        {
          sum += output[1].toDouble();
          // cutting DPCosts to prevent double counting
          if (doc.value("dPIs").toArray().contains(output_inx))
          {
            processed_outputs.append(QStringList {
              block.value("bCDate").toString().split(" ")[1],
              CUtils::hash6c(doc.value("dHash").toString()),
              "DPCost",
              CUtils::microPAIToPAI6(output[1].toDouble())
            }.join("\t  "));
          } else {
            if (CConsts::TREASURY_PAYMENTS.contains(output[0].toString()))
            {
              processed_outputs.append(QStringList {
                block.value("bCDate").toString().split(" ")[1],
                CUtils::hash6c(doc.value("dHash").toString()),
                CUtils::shortBech16(output[0].toString()),
                CUtils::microPAIToPAI6(output[1].toDouble())});

            } else {
                processed_outputs.append(QStringList {
                  block.value("bCDate").toString().split(" ")[1],
                  CUtils::hash6c(doc.value("dHash").toString()),
                  CUtils::shortBech16(output[0].toString()),
                  CUtils::microPAIToPAI6(output[1].toDouble())});

            }
          }
        } else {
            // must not come in this part!
        }
      }
    }
  }

  QStringList dbl_spends {};
  for (CDocHashT ref: maybe_dbl_spends.keys())
    if (maybe_dbl_spends[ref] > 1)
      dbl_spends.append(CUtils::shortCoinRef(ref) + "->" + QString::number(maybe_dbl_spends[ref]));


  return {sum, dbl_spends, processed_outputs.join("\n")};
}

/**
 * @brief DAG::getCBBlocksStat
 * @return {
    floorish_micro_PAIs,
    burned_by_block,
    total_minted_coins,
    outputs_by_block,
    waited_coinbases_to_be_spendable};
 */
std::tuple<CMPAIValueT, QHash<CBlockHashT, CMPAIValueT>, CMPAIValueT, QHash<CBlockHashT, CMPAIValueT>, CMPAIValueT> DAG::getCBBlocksStat()
{
  CMPAIValueT total_minted_coins = 0;
  QHash<CBlockHashT, CMPAIValueT> outputs_by_block {};

  CMPAIValueT floorish_micro_PAIs = 0;    //missedMicroPAIs
  QHash<CBlockHashT, CMPAIValueT> burned_by_block {}; //missedMicroPAIsBlocks

  CMPAIValueT waited_coinbases_to_be_spendable = 0;
  QStringList considered_cycles {};

  QVDRecordsT wBlocks = searchInDAG(
    {{"b_type", CConsts::BLOCK_TYPES::Coinbase}},
    {"b_cycle", "b_body", "b_utxo_imported"},
    {{"b_creation_date", "ASC"}});

  for (QVDicT wBlock: wBlocks)
  {
    if (considered_cycles.contains(wBlock.value("b_cycle").toString()))
      continue;

    considered_cycles.append(wBlock.value("b_cycle").toString());


    QJsonObject block = CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock.value("b_body").toString()).content);
    CBlockHashT block_hash = block.value("bHash").toString();

    CMPAIValueT block_outputs_sum = 0;
    CMPAIValueT blockMissedPAIs = 0;
    if (block.keys().contains("docs"))
    {
      auto docs = block.value("docs").toArray();
      for (auto aDoc_: docs)
      {
        auto aDoc = aDoc_.toObject();
       if (aDoc.keys().contains("outputs"))
           for (auto output: aDoc.value("outputs").toArray())
             block_outputs_sum += output.toArray()[1].toDouble();
      }

      blockMissedPAIs = docs[0].toObject().value("treasuryIncomes").toDouble() + docs[0].toObject().value("mintedCoins").toDouble() - block_outputs_sum;
      total_minted_coins += docs[0].toObject().value("mintedCoins").toDouble();
    }


    outputs_by_block[block_hash] = block_outputs_sum;

    floorish_micro_PAIs += blockMissedPAIs;
    if (blockMissedPAIs > 0)
       burned_by_block[block_hash] = blockMissedPAIs;

    if (wBlock.value("b_utxo_imported").toString() == CConsts::NO)
       waited_coinbases_to_be_spendable += block_outputs_sum;
  }

  return {
    floorish_micro_PAIs,
    burned_by_block,
    total_minted_coins,
    outputs_by_block,
    waited_coinbases_to_be_spendable};
}

QVDRecordsT DAG::getCyclesList()
{
  QString complete_query;
  if (CConsts::DATABASAE_AGENT == "psql")
  {
    QString complete_query = "SELECT DISTINCT b_cycle cycle_ FROM " + DAG::stbl_blocks + " WHERE b_type='Coinbase' ";

  }else if (CConsts::DATABASAE_AGENT == "sqlite")
  {
    QString complete_query = "SELECT DISTINCT b_cycle cycle_ FROM " + DAG::stbl_blocks + " WHERE b_type=\"Coinbase\" ";

  }
  QueryRes cycles = DbModel::customQuery(
    "db_comen_blocks",
    complete_query,
    {"cycle_"},
    0,
    {});

  return cycles.records;
}

QVDicT DAG::getLatestRecordedBlcok()
{
  QueryRes res = DbModel::select(
    stbl_blocks,
    stbl_blocks_fields,
    {},
    {{"b_creation_date", "DESC"},
      {"b_type", "ASC"},  // just to forcing a difinit order
      {"b_hash", "ASC"}}, // just to forcing a difinit order
    1);

  if (res.records.size() > 0)
    return res.records[0];

  return {};
}


/**
 * @brief DAG::getMostConfidenceCoinbaseBlockFromDAG
 * @param cDate
 * @return {atleast one CB exist, the max confidence}
 *
 */
std::tuple<bool, QVDicT> DAG::getMostConfidenceCoinbaseBlockFromDAG(CDateT cDate)
{
  if (cDate == "")
    cDate = CUtils::getNow();

  auto[coinbase_cycle_stamp, coinbase_from, coinbase_to, coinbase_from_hour, coinbase_to_hour] = CUtils::getCoinbaseInfo(cDate);

  QVDRecordsT current_coinbases_in_DAG = searchInDAG(
    {{"b_type", CConsts::BLOCK_TYPES::Coinbase},
    {"b_creation_date", coinbase_from, ">="}},
    {"b_hash", "b_confidence", "b_ancestors"},
    {{"b_confidence", "DESC"}},
    1);

  if (current_coinbases_in_DAG.size() == 0)
    return {false, QVDicT {}};

  return {true, current_coinbases_in_DAG[0]};
}

 */