use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, dlog};
use crate::cutils::{array_add, array_unique};
use crate::lib::custom_types::{CDateT, ClausesT, OrderT, QVDicT, QVDRecordsT};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::C_BLOCKS;

//old_name_was appendDescendents
pub fn append_descendants(block_hashes: &Vec<String>, new_descendents: &Vec<String>)
{
    if new_descendents.len() > 0
    {
        for a_block_hash in block_hashes
        {
            let c1 = simple_eq_clause("b_hash", &*a_block_hash);
            let (_status, records) = q_select(
                C_BLOCKS,
                vec!["b_hash", "b_descendants"],
                vec![c1],
                vec![],
                1,
                false);

            if records.len() == 1
            {
                let current_descendants: Vec<String> = records[0]["b_descendants"]
                    .split(",")
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|&x| x.to_string())
                    .collect::<Vec<String>>();

                let final_descendants: Vec<String> = array_unique(
                    &array_add(&new_descendents, &current_descendants)
                );

                let b_descendants = final_descendants.join(",");
                let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
                    ("b_descendants", &b_descendants as &(dyn ToSql + Sync))
                ]);
                let c1 = simple_eq_clause("b_hash", &*a_block_hash);
                q_update(
                    C_BLOCKS,
                    &update_values,
                    vec![c1],
                    true);
            }
        }
    }
}

/*

/**
 * @brief DAG::getBlockHashesByDocHashes
 * @param doc_hashes
 * @param hashes_type
 * @return {block_hashes, map_doc_to_block}
 */
std::tuple<StringList, GRecordsT> DAG::getBlockHashesByDocHashes(
  const StringList& doc_hashes,
  const String& hashes_type)
{
  ClausesT clauses {};
  if (hashes_type == constants::SHORT)
  {
    StringList tmp{};
    for(String a_hash: doc_hashes)
      tmp.push(a_hash + "%");
    clauses.push({"dbm_doc_hash", tmp, "LIKE:OR"});
  } else {
    clauses.push({"dbm_doc_hash", doc_hashes, "IN"});
  }

  QueryRes res = DbModel::select(
    stbl_docs_blocks_map,
    {"dbm_block_hash", "dbm_doc_hash", "dbm_doc_index"},
    clauses);
  StringList block_hashes{};
  GRecordsT map_doc_to_block{};
  for (QVDicT element: res.records)
  {
    block_hashes.push(element["dbm_block_hash"].to_string());
    // since we can have more than 1 coinbase block for each cycle, so mapping a document to its container block could be 1 to n mapping
    // also in lone transactions we have same psituation in which a certain transaction can take place in different blocks by different backers
    String dbm_doc_hash = element["dbm_doc_hash"].to_string();
    if (!map_doc_to_block.keys().contains(dbm_doc_hash))
      map_doc_to_block[dbm_doc_hash] = QVDRecordsT{};
    map_doc_to_block[dbm_doc_hash].push(QVDicT{
      {"block_hash", element["dbm_block_hash"]},
      {"doc_index", element["dbm_doc_index"]}
    });
  }
  return {block_hashes, map_doc_to_block};
}
*/

//old_name_was searchInDAG
pub fn search_in_dag(
    clauses: ClausesT,
    fields: Vec<&str>,
    order: OrderT,
    limit: u32,
    do_log: bool) -> QVDRecordsT
{
    let (status, records) = q_select(
        C_BLOCKS,
        fields,
        clauses,
        order,
        limit,
        do_log);
    if !status
    {
        dlog(
            &format!("Failed in search in DAG! "),
            constants::Modules::App,
            constants::SecLevel::Error);
    }

    return records;
}

/*
std::tuple<QVDRecordsT, GRecordsT> DAG::getWBlocksByDocHash(
  const StringList& doc_hashes,
  const String& hashes_type)
{
  auto[block_hashes, map_doc_to_block] = getBlockHashesByDocHashes(doc_hashes, hashes_type);
    if (block_hashes.len() == 0)
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
std::tuple<bool, JSonObject, CDocIndexT, MerkleNodeData, JSonArray> DAG::retrieveDocByDocHash(
  const CDocHashT& doc_hash,
  const bool need_doc_ext_info,
  const bool need_doc_merkle_proof)
{
  String msg;
  auto[block_records, map_] = getWBlocksByDocHash({doc_hash});
  if (block_records.len() == 0)
    return {false, {}, {}, {}, {}};

  JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(block_records[0]["b_body"].to_string()).content);

  JSonObject document {};
  CDocIndexT documentIndex = 0;
  JSonArray docs = block["docs"].toArray();
  for (CDocIndexT docInx = 0; docInx < docs.len(); docInx++)
  {
    JSonObject aDoc = docs[docInx].toObject();
    if ((document.keys().len() == 0) && (aDoc["dHash"].to_string() == doc_hash))
    {
      documentIndex = docInx;
      document = aDoc;
    }
  }

  if (document.keys().len() == 0)
    return {false, {}, {}, {}, {}};

  JSonArray the_doc_ext_info {};
  if (need_doc_ext_info)
  {
    auto[status, extExist, block_ext_info] = Block::getBlockExtInfo(block["bHash"].to_string());
    if (!extExist)
    {
      msg = "missed bExtInfo3 (" + cutils::hash8c(block["bHash"].to_string()) + ")";
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
QV2DicT DAG::getCoinsGenerationInfoViaSQL(const StringList& coins)
{
  StringList docsHashes {};
  for(String a_coin: coins)
  {
    auto[doc_hash_, output_index_] = cutils::unpackCoinCode(a_coin);
    Q_UNUSED(output_index_);
    docsHashes.push(doc_hash_);
  }
  CLog::log("find Output Info By RefLocViaSQL for docs: " + docsHashes.join(", "), "trx", "trace");

  auto[block_records, map_] = getWBlocksByDocHash(docsHashes);
  Q_UNUSED(map_);
  // console.log('find Output Info By RefLocViaSQL', block_records);
  QV2DicT outputsDict {};
  for (QVDicT wBlock: block_records)
  {
    JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"].to_string()).content);
    if (block.keys().contains("docs"))
    {
      for (QJsonValueRef doc_: block["docs"].toArray())
      {
        JSonObject doc = doc_.toObject();
        if (doc.keys().contains("outputs"))
        {
          JSonArray outputs = doc["outputs"].toArray();
          for (COutputIndexT output_index = 0; output_index < static_cast<COutputIndexT>(outputs.len()); output_index++)
          {
            auto output = outputs[output_index].toArray();
            String aCoin = cutils::packCoinCode(doc["dHash"].to_string(), output_index);
            outputsDict[aCoin] = QVDicT {
              {"coinGenCycle", wBlock["b_cycle"]},
              {"coinGenBlockHash", block["bHash"]},
              {"coinGenBType", block["bType"]},
              {"coinGenCreationDate", block["bCDate"]}, // coin creation date is the BLOCK creation date and not the transaction date
              {"coinGenDocType", doc["dType"]},
              {"coinGenRefLoc", aCoin},
              {"coinGenOutputAddress", output[0].to_string()},
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
  const StringList& block_hashes,
  const String& date,
  StringList& ancestors)
{
  // console.log(`::::::::::: recursive_backwardInTime args: ${utils.stringify(args)}`);
  if (block_hashes.len()== 0)
    return;

  QVDRecordsT res = searchInDAG(
    {{"b_hash", block_hashes, "IN"},
    {"b_creation_date", date, ">"}},
    {"b_ancestors"});

  if (res.len()== 0)
    return;

  StringList out {};
  for (QVDicT aRes: res)
    out = cutils::arrayAdd(out, aRes["b_ancestors"].to_string().split(","));

  out = cutils::arrayUnique(out);
  ancestors = cutils::arrayAdd(ancestors, out);

  if (out.len()> 0)
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
StringList DAG::returnAncestorsYoungerThan(
  const StringList& block_hashes,
  const String& byDate,
  const int32_t& byMinutes,
  const int32_t& cycle)
{
//  CLog::log("return AncestorsYoungerThan: block_hashes: " + block_hashes.join(",") + "\nbyDate(" + byDate +")", "app", "trace");
  if (block_hashes.len() == 0)
    return {};

  String date_;
  if (byDate != "")
  {
    date_ = byDate;
  } else if (byMinutes != -1)
  {
    date_ = cutils::minutes_before(byMinutes);
  } else if (cycle != -1)
  {
    date_ = cutils::minutes_before(cutils::get_cycle_by_minutes() * cycle);
  }

  StringList ancestors;
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
    {"b_hash", "b_type", "b_ancestors", "b_descendants", "b_creation_date"},
    {{"b_creation_date", "DESC"}});



  QSDicT childByDad {};
  QV2DicT blocksInfo {};
  for (QVDicT wBlock: wBlocks)
  {
    blocksInfo[cutils::hash8c(wBlock["b_hash"].to_string())] = QVDicT {
      {"b_type", wBlock["b_type"].to_string()},
      {"b_creation_date", wBlock["b_creation_date"].to_string()}};

    StringList ancestors = wBlock["b_ancestors"].to_string().split(",");
    for (String dadHash: ancestors)
    {
      childByDad[cutils::hash8c(dadHash)] = cutils::hash8c(wBlock["b_hash"].to_string());
    };
  };
  // console.log(blocksInfo);
  QSDRecordsT leaves {};
  StringList tmp {};
  for (String hash: blocksInfo.keys())
  {
    if (!childByDad.keys().contains(hash))
      tmp.push(hash);
  }

  tmp.sort();
  for (String hash: tmp)
    leaves.push(QSDicT {
      {"b_ash", hash},
      {"b_type", blocksInfo[hash]["b_type"].to_string()},
      {"b_creation_date", blocksInfo[hash]["b_creation_date"].to_string().split(" ")[1]}});

  return leaves;
}


QVDRecordsT DAG::excludeFloatingBlocks(
  const StringList& block_hashes,
  const StringList& fields)
{
  // exclude floating signature blocks
  if (block_hashes.len()== 0)
    return {};

  QVDRecordsT block_records = searchInDAG(
    {{"b_hash", block_hashes, "IN"},
    {"b_type", {constants::BLOCK_TYPES::FSign, constants::BLOCK_TYPES::FVote}, "NOT IN"}},
    fields);
  return block_records;
}

void DAG::updateUtxoImported(
  const String& block_hash,
  const String& status)
{
  CLog::log("update Utxo is Imported Block(" + cutils::hash8c(block_hash) + ") to imported(" + status + ")", "trx", "info");
  DbModel::update(
    stbl_blocks,
    {{"b_utxo_imported", status}},
    {{"b_hash", block_hash}});

  // update also cached blocks
  CMachine::cachedBlocks("update", {QVDicT{{"b_hash", block_hash}}}, status);

}

bool DAG::isDAGUptodated(String c_date)
{
  if (c_date == "")
    c_date = application().get_now();

  QVDRecordsT latestBlockDate = searchInDAG(
    {},
    {"b_creation_date"},
    {{"b_creation_date", "DESC"}},
    1);
  if (latestBlockDate.len() == 0)
      return false;

  String latest_block_date = latestBlockDate[0]["b_creation_date"].to_string();
  return cutils::isYoungerThan2Cycle(latest_block_date);
  // FIXME: it must be more sophisticated such as missed blocks count, last received blocks which are in parsingQ...
}

bool DAG::loopPrerequisitiesRemover()
{
  String thread_prefix = "prerequisities_remover_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    doPrerequisitiesRemover();

    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getPrerequisitiesRemoverGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
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
        StringList pre = cutils::unpackCommaSeperated(a_cpack["pq_prerequisites"].to_string());
        ClausesT clauses = {{"b_hash", pre, "IN"}};
        // if (machine.is_in_sync_process())
        //     query.push(['b_utxo_imported', 'Y'])    // to avoid removing prerequisities, before importing UTXOs
        if (pre.len() > 0)
        {
          QVDRecordsT existedBlocksInDAG = DAG::searchInDAG(clauses, {"b_hash"});
          if (existedBlocksInDAG.len() > 0)
          {
            for (QVDicT aBlock: existedBlocksInDAG)
            {
              CLog::log("Prerequisities Remover, removed dependencies to block(" + cutils::hash8c(aBlock["b_hash"].to_string()) + ")", "app", "trace");
              // remove dependency to this block
              ParsingQHandler::removePrerequisites(aBlock["b_hash"].to_string());
            }
          }
        }
      }

      */
    return true;
}


#[allow(dead_code, unused)]
pub fn dag_has_blocks_which_are_created_in_current_cycle(c_date_: &CDateT) -> bool
{
    let mut c_date = c_date_.to_string();
    if c_date == ""
    { c_date = application().get_now(); }

    let latest_blocks: QVDRecordsT = search_in_dag(
        vec![],
        vec!["b_creation_date"],
        vec![
            &OrderModifier { m_field: "b_creation_date", m_order: "DESC" }],
        1,
        false);
    if latest_blocks.len() == 0
    { return false; }
    let latest_block_date = latest_blocks[0]["b_creation_date"].to_string();
    return application().time_diff(latest_block_date, c_date).as_minutes < application().get_cycle_by_minutes() as u64;
}

/*

// need to be fixed because of array response of get BlockHashByDocHash
std::vector<CCoin> DAG::retrieveBlocksInWhichARefLocHaveBeenProduced(const CCoinCodeT& the_coin)
{
  std::vector<CCoin> results {};
  CLog::log("retrieve Blocks In which the coin: (" + the_coin + ") is created", "app", "trace");

  auto[doc_hash, output_index_] = cutils::unpackCoinCode(the_coin);
  Q_UNUSED(output_index_);

  auto[block_hashes, map_doc_to_block_] = getBlockHashesByDocHashes({doc_hash});
  Q_UNUSED(map_doc_to_block_);

  QVDRecordsT wBlocks = DAG::searchInDAG(
    {{"b_hash", block_hashes, "IN"}},
    {"b_hash"});

  if (wBlocks.len() == 0)
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
    Block* block = BlockFactory::create(cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"]).content));

    for (CDocIndexT doc_index = 0; doc_index < block.m_documents.len(); doc_index++)
    {
      Document* doc = block.m_documents[doc_index];
      if (doc->docHasOutput())
      {
        for (COutputIndexT output_index = 0; output_index < doc->get_outputs().len(); output_index++)
        {
          CCoinCodeT tmp_coin = cutils::packCoinCode(doc->get_doc_hash(), output_index);
          if (tmp_coin == the_coin)
            results.emplace_back(CCoin(
              the_coin,
              doc->get_outputs()[output_index].m_address,
              doc->get_outputs()[output_index].m_amount,

              block.m_block_creation_date,

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
    {{"b_utxo_imported", constants::NO},
    {"b_type", {constants::BLOCK_TYPES::FSign, constants::BLOCK_TYPES::FVote}, "NOT IN"},
    {"b_type", StringList{constants::block_types::COINBASE}, "IN"}});

  CMPAIValueT sum = 0;
  QVDRecordsT processed_outputs {};
  StringList calculated_coinbase {};
  for (QVDicT wBlock: wBlocks)
  {
    JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"].to_string()).content);

    if (calculated_coinbase.contains(block["bCycle"].to_string()))
      continue;

    if (block["bType"].to_string() == constants::block_types::COINBASE)
        calculated_coinbase.push(block["bCycle"].to_string());

    // analyze outputs
    JSonObject doc = block["docs"].toArray()[0].toObject();
    JSonArray outputs = doc["outputs"].toArray();
    for (auto output_inx = 0; output_inx < outputs.len(); output_inx++)
    {
      auto output = outputs[output_inx].toArray();

      sum += output[1].toDouble();

      processed_outputs.push(QVDicT {
        {"block_type", wBlock["bType"].to_string()},
        {"creation_date", block["bCDate"].to_string().split(" ")[1]},
        {"doc_hash", cutils::hash8c(doc["dHash"].to_string())},
        {"owner", cutils::short_bech16(output[0].to_string())},
        {"value", cutils::microPAIToPAI6(output[1].toDouble())}});
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

  CMPAIValueT coinbase_value = (calculated_coinbase.len() * CoinbaseIssuer::calcPotentialMicroPaiPerOneCycle(application().get_now().split("-")[0]));

  return {sum, processed_outputs, coinbase_value};
}

std::tuple<CMPAIValueT, StringList, String> DAG::getNotImportedNormalBlock()
{
  QVDRecordsT wBlocks = searchInDAG(
    {{"b_utxo_imported", constants::NO},
    {"b_type", StringList{constants::BLOCK_TYPES::Normal, "IN"}}});
  CMPAIValueT sum = 0;
  QHash<CDocHashT, int64_t> maybe_dbl_spends = {};
  StringList processed_outputs {};

  for (QVDicT wBlock: wBlocks)
  {
    JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"].to_string()).content);

    for (auto doc_: block["docs"].toArray())
    {
      auto doc = doc_.toObject();
      if (!StringList{constants::document_types::BASIC_TX}.contains(doc["dType"].to_string()))
        continue;

      // since DPCostPay docs alredy are in transactions , so we do not calculate it 2 times

      // analyze inputs
      for (auto input: doc["inputs"].toArray())
      {
        CCoinCodeT the_coin = cutils::packCoinCode(input.toArray()[0].to_string(), input.toArray()[1].toDouble());
        if (!maybe_dbl_spends.contains(the_coin))
          maybe_dbl_spends[the_coin] = 0;
        maybe_dbl_spends[the_coin] += 1;
      }

      // analyze outputs
      // console.log(doc);
      auto outputs = doc["outputs"].toArray();
      for (auto output_inx = 0; output_inx < outputs.len(); output_inx++)
      {
        auto output = outputs[output_inx].toArray();
        if (doc.keys().contains("dPIs"))
        {
          sum += output[1].toDouble();
          // cutting DPCosts to prevent double counting
          if (doc["dPIs"].toArray().contains(output_inx))
          {
            processed_outputs.push(StringList {
              block["bCDate"].to_string().split(" ")[1],
              cutils::hash6c(doc["dHash"].to_string()),
              "DPCost",
              cutils::microPAIToPAI6(output[1].toDouble())
            }.join("\t  "));
          } else {
            if (constants::TREASURY_PAYMENTS.contains(output[0].to_string()))
            {
              processed_outputs.push(StringList {
                block["bCDate"].to_string().split(" ")[1],
                cutils::hash6c(doc["dHash"].to_string()),
                cutils::short_bech16(output[0].to_string()),
                cutils::microPAIToPAI6(output[1].toDouble())});

            } else {
                processed_outputs.push(StringList {
                  block["bCDate"].to_string().split(" ")[1],
                  cutils::hash6c(doc["dHash"].to_string()),
                  cutils::short_bech16(output[0].to_string()),
                  cutils::microPAIToPAI6(output[1].toDouble())});

            }
          }
        } else {
            // must not come in this part!
        }
      }
    }
  }

  StringList dbl_spends {};
  for (CDocHashT ref: maybe_dbl_spends.keys())
    if (maybe_dbl_spends[ref] > 1)
      dbl_spends.push(cutils::shortCoinRef(ref) + "->" + String::number(maybe_dbl_spends[ref]));


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
  StringList considered_cycles {};

  QVDRecordsT wBlocks = searchInDAG(
    {{"b_type", constants::block_types::COINBASE}},
    {"b_cycle", "b_body", "b_utxo_imported"},
    {{"b_creation_date", "ASC"}});

  for (QVDicT wBlock: wBlocks)
  {
    if (considered_cycles.contains(wBlock["b_cycle"].to_string()))
      continue;

    considered_cycles.push(wBlock["b_cycle"].to_string());


    JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"].to_string()).content);
    CBlockHashT block_hash = block["bHash"].to_string();

    CMPAIValueT block_outputs_sum = 0;
    CMPAIValueT blockMissedPAIs = 0;
    if (block.keys().contains("docs"))
    {
      auto docs = block["docs"].toArray();
      for (auto aDoc_: docs)
      {
        auto aDoc = aDoc_.toObject();
       if (aDoc.keys().contains("outputs"))
           for (auto output: aDoc["outputs"].toArray())
             block_outputs_sum += output.toArray()[1].toDouble();
      }

      blockMissedPAIs = docs[0].toObject()["treasuryIncomes"].toDouble() + docs[0].toObject()["mintedCoins"].toDouble() - block_outputs_sum;
      total_minted_coins += docs[0].toObject()["mintedCoins"].toDouble();
    }


    outputs_by_block[block_hash] = block_outputs_sum;

    floorish_micro_PAIs += blockMissedPAIs;
    if (blockMissedPAIs > 0)
       burned_by_block[block_hash] = blockMissedPAIs;

    if (wBlock["b_utxo_imported"].to_string() == constants::NO)
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
  String complete_query;
  if (constants::DATABASAE_AGENT == "psql")
  {
    String complete_query = "SELECT DISTINCT b_cycle cycle_ FROM " + DAG::stbl_blocks + " WHERE b_type='Coinbase' ";

  }else if (constants::DATABASAE_AGENT == "sqlite")
  {
    String complete_query = "SELECT DISTINCT b_cycle cycle_ FROM " + DAG::stbl_blocks + " WHERE b_type=\"Coinbase\" ";

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

  if (res.records.len() > 0)
    return res.records[0];

  return {};
}


/**
 * @brief DAG::getMostConfidenceCoinbaseBlockFromDAG
 * @param c_date
 * @return {atleast one CB exist, the max confidence}
 *
 */
 */
//old_name_was getMostConfidenceCoinbaseBlockFromDAG
pub fn get_most_confidence_coinbase_block_from_dag(c_date: &CDateT) -> (bool, QVDicT)
{
    // if (c_date == "")
    //   c_date = application().get_now();

    let (
        _coinbase_cycle_stamp,
        coinbase_from,
        _coinbase_to,
        _coinbase_from_hour,
        _coinbase_to_hour) =
        application().get_coinbase_info(c_date, "");


    let current_coinbases_in_dag: QVDRecordsT = search_in_dag(
        vec![
            simple_eq_clause("b_type", &constants::block_types::COINBASE.to_string()),
            ModelClause {
                m_field_name: "b_creation_date",
                m_field_single_str_value: &coinbase_from as &(dyn ToSql + Sync),
                m_clause_operand: ">=",
                m_field_multi_values: vec![],
            },
        ],
        vec!["b_hash", "b_confidence", "b_ancestors"],
        vec![&OrderModifier { m_field: "b_confidence", m_order: "DESC" }],
        0,
        true);

    if current_coinbases_in_dag.len() == 0 {
        return (false, HashMap::new());
    }

    return (true, current_coinbases_in_dag[0].clone());
}
