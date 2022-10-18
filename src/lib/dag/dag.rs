use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::cutils::{array_add, array_unique, remove_quotes};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CDateT, ClausesT, COutputIndexT, GRecordsT, OrderT, QV2DicT, QVDicT, QVDRecordsT, VString};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::{C_BLOCKS, C_BLOCKS_FIELDS, C_DOCS_BLOCKS_MAP};
use crate::lib::transactions::basic_transactions::coins::coins_handler::{CoinDetails};

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

//old_name_was getBlockHashesByDocHashes
pub fn get_block_hashes_by_doc_hashes(
    doc_hashes: &VString,
    hashes_type: &String,
) ->
    (
        VString, // block_hashes
        GRecordsT // map_doc_to_block
    )
{
    let mut clauses: ClausesT = vec![];
    let empty_string = "".to_string();

    let mut place_holder_hashes: VString = vec![];
    if hashes_type == constants::SHORT
    {
        // let mut tmp: VString = vec![];
        // for a_hash in doc_hashes
        // { tmp.push(a_hash + "%"); }
        let mut c0 = ModelClause {
            m_field_name: "dbm_doc_hash",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "LIKE:OR",
            m_field_multi_values: vec![],
        };
        for a_hash in doc_hashes
        {
            place_holder_hashes.push(format!("{}%", a_hash));
        }
        for a_hash in &place_holder_hashes
        {
            c0.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
        }
        clauses.push(c0);
    } else {
        let mut c1 = ModelClause {
            m_field_name: "dbm_doc_hash",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        for a_hash in doc_hashes {
            c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
        }
        clauses.push(c1);
    }

    let (_status, records) = q_select(
        C_DOCS_BLOCKS_MAP,
        vec!["dbm_block_hash", "dbm_doc_hash", "dbm_doc_index"],
        clauses,
        vec![],
        0,
        false);
    let mut block_hashes: VString = vec![];
    let mut map_doc_to_block: GRecordsT = HashMap::new();
    for element in records
    {
        block_hashes.push(element["dbm_block_hash"].clone());
        // since we can have more than 1 coinbase block for each cycle, so mapping a document to its container block could be 1 to n mapping
        // also in lone transactions we have same psituation in which a certain transaction can take place in different blocks by different backers
        let dbm_doc_hash = element["dbm_doc_hash"].clone();
        if !map_doc_to_block.contains_key(&dbm_doc_hash)
        { map_doc_to_block.insert(dbm_doc_hash.clone(), vec![]); }

        let mut tmp_v = map_doc_to_block[&dbm_doc_hash].clone();

        let b_info: QVDicT = HashMap::from([
            ("block_hash".to_string(), element["dbm_block_hash"].clone()),
            ("doc_index".to_string(), element["dbm_doc_index"].clone())
        ]);
        tmp_v.push(b_info);
        map_doc_to_block.insert(dbm_doc_hash, tmp_v);
    }
    return (block_hashes, map_doc_to_block);
}

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

//old_name_was getWBlocksByDocHash
pub fn get_wrap_blocks_by_doc_hash(
    doc_hashes: &VString,
    hashes_type: &String) -> (QVDRecordsT, GRecordsT)
{
    let (block_hashes, map_doc_to_block) = get_block_hashes_by_doc_hashes(doc_hashes, hashes_type);
    if block_hashes.len() == 0
    { return (vec![], map_doc_to_block); }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &block_hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }

    let block_records = search_in_dag(
        vec![c1],
        Vec::from(C_BLOCKS_FIELDS),
        vec![],
        0,
        false,
    );
    return (block_records, map_doc_to_block);
}

/*

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
  auto[block_records, map_] = get_wrap_blocks_by_doc_hash({doc_hash});
  if (block_records.len() == 0)
    return {false, {}, {}, {}, {}};

  JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(block_records[0]["b_body"].to_string()).content);

  JSonObject document {};
  CDocIndexT documentIndex = 0;
  JSonArray docs = block["bDocs"].toArray();
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

*/
/**
 *
 * @param {*} coins
 * finding the blocks in which were created given coins
 */
//old_name_was getCoinsGenerationInfoViaSQL
pub fn get_coins_generation_info_via_sql(coins: &VString) -> QV2DicT
{
    let mut docs_hashes: VString = vec![];
    for a_coin in coins
    {
        let (doc_hash, _output_index) = cutils::unpack_coin_code(a_coin);
        docs_hashes.push(doc_hash);
    }
    dlog(
        &format!(
            "find Output Info By RefLocViaSQL for docs: {:?}", docs_hashes),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let (block_records, _map) = get_wrap_blocks_by_doc_hash(&docs_hashes, &constants::COMPLETE.to_string());
    // console.log('find Output Info By RefLocViaSQL', block_records);
    let mut outputs_dict: QV2DicT = HashMap::new();
    for w_block in block_records
    {
        let (_status, _sf_ver, content) = unwrap_safed_content_for_db(&w_block["b_body"]);
        let (_status, block) = cutils::controlled_str_to_json(&content);
        if !block["bDocs"].is_null()
        {
            for doc in block["bDocs"].as_array().unwrap()
            {
                // JSonObject doc = doc_.toObject();
                if !doc["outputs"].is_null()
                {
                    let outputs = doc["outputs"].as_array().unwrap();
                    for output_index in 0..outputs.len() as COutputIndexT
                    {
                        let output = &outputs[output_index as usize];
                        let a_coin: CCoinCodeT = cutils::pack_coin_code(
                            &remove_quotes(&doc["dHash"]),
                            output_index);
                        let tmp_dict: QVDicT = HashMap::from([
                            ("coinGenCycle".to_string(), w_block["b_cycle"].clone()),
                            ("coinGenBlockHash".to_string(), remove_quotes(&block["bHash"])),
                            ("coinGenBType".to_string(), remove_quotes(&block["bType"])),
                            ("coinGenCreationDate".to_string(), remove_quotes(&block["bCDate"])), // coin creation date is the BLOCK creation date and not the transaction date
                            ("coinGenDocType".to_string(), remove_quotes(&doc["dType"])),
                            ("coinGenRefLoc".to_string(), a_coin.clone()),
                            ("coinGenOutputAddress".to_string(), output[0].to_string()),
                            ("coinGenOutputValue".to_string(), output[1].to_string())
                        ]);
                        outputs_dict.insert(a_coin, tmp_dict);
                    }
                }
            }
        }
    }

    let mut res: QV2DicT = HashMap::new();
    for a_coin in coins
    {
        if outputs_dict.contains_key(a_coin)
        {
            res.insert(a_coin.clone(), outputs_dict[a_coin].clone());
        }
    }
    return res;
}

/*
void DAG::recursive_backwardInTime(
  const VString& block_hashes,
  const String& date,
  VString& ancestors)
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

  VString out {};
  for (QVDicT aRes: res)
    out = cutils::arrayAdd(out, aRes["b_ancestors"].to_string().split(","));

  out = cutils::array_unique(out);
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
VString DAG::returnAncestorsYoungerThan(
  const VString& block_hashes,
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

  VString ancestors;
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

    VString ancestors = wBlock["b_ancestors"].to_string().split(",");
    for (String dadHash: ancestors)
    {
      childByDad[cutils::hash8c(dadHash)] = cutils::hash8c(wBlock["b_hash"].to_string());
    };
  };
  // console.log(blocksInfo);
  QSDRecordsT leaves {};
  VString tmp {};
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

*/
//old_name_was excludeFloatingBlocks
pub fn exclude_floating_blocks(
    block_hashes: &VString,
    fields: Vec<&str>) -> QVDRecordsT
{
    // exclude floating signature blocks
    if block_hashes.len() == 0
    { return vec![]; }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in block_hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let mut c2 = ModelClause {
        m_field_name: "b_type",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "NOT IN",
        m_field_multi_values: vec![],
    };
    let excluded_block_types = vec![constants::block_types::FLOATING_SIGNATURE, constants::block_types::FLOATING_VOTE];
    for a_hash in &excluded_block_types {
        c2.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let block_records = search_in_dag(
        vec![
            c1,
            c2,
        ],
        fields,
        vec![],
        0,
        false);
    return block_records;
}

// old name was updateUtxoImported
pub fn set_coins_import_status(
    block_hash: &CBlockHashT,
    status: &String)
{
    dlog(
        &format!(
            "Update the coin is imported. Block({}) to imported({})",
            cutils::hash8c(block_hash),
            status),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("b_coins_imported", &status as &(dyn ToSql + Sync))
    ]);
    q_update(
        C_BLOCKS,
        &update_values,
        vec![simple_eq_clause("b_hash", block_hash)],
        false);

    // update also cached blocks
    // CMachine::cachedBlocks("update", {QVDicT{{"b_hash", block_hash}}}, status);
}

//old_name_was isDAGUptodated
pub fn is_dag_updated() -> bool
{
    let latest_block_date = search_in_dag(
        vec![],
        vec!["b_creation_date"],
        vec![
            &OrderModifier { m_field: "b_creation_date", m_order: "DESC" },
        ],
        1,
        true);
    if latest_block_date.len() == 0
    {
        return false;
    }
    let latest_block_date = latest_block_date[0]["b_creation_date"].to_string();
    return application().is_younger_than_2_cycle(&latest_block_date);
    // FIXME: it must be more sophisticated such as missed blocks count, last received blocks which are in parsingQ...
}

/*
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
pub fn do_prerequisites_remover() -> bool
{
    /*

      // TODO: improve it to be more efficient
      // first loop on all blocks in Q
      QVDRecordsT queued_packets = ParsingQHandler::searchParsingQ({}, {"pq_prerequisites"});

      for (QVDicT a_cpack: queued_packets)
      {
        // control if already recorded in DAG
        VString pre = cutils::unpackCommaSeperated(a_cpack["pq_prerequisites"].to_string());
        ClausesT clauses = {{"b_hash", pre, "IN"}};
        // if (machine.is_in_sync_process())
        //     query.push(['b_coins_imported', 'Y'])    // to avoid removing prerequisities, before importing UTXOs
        if (pre.len() > 0)
        {
          QVDRecordsT existedBlocksInDAG = DAG::searchInDAG(clauses, {"b_hash"});
          if (existedBlocksInDAG.len() > 0)
          {
            for (QVDicT aBlock: existedBlocksInDAG)
            {
              CLog::log("Prerequisities Remover, removed dependencies to block(" + cutils::hash8c(aBlock["b_hash"].to_string()) + ")", "app", "trace");
              // remove dependency to this block
              remove_prerequisites(aBlock["b_hash"].to_string());
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
    { c_date = application().now(); }

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


// need to be fixed because of array response of get BlockHashByDocHash
//old_name_was retrieveBlocksInWhichARefLocHaveBeenProduced
pub fn retrieve_blocks_in_which_a_coin_have_been_produced(the_coin: &CCoinCodeT) -> Vec<CoinDetails>
{
    let mut results: Vec<CoinDetails> = vec![];
    dlog(
        &format!("Retrieve Blocks In which the coin: ({}) is created", the_coin),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let (doc_hash, _output_index) = cutils::unpack_coin_code(&the_coin);

    let (block_hashes, _map_doc_to_block) =
        get_block_hashes_by_doc_hashes(&vec![doc_hash.clone()], &constants::COMPLETE.to_string());

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for an_hash in &block_hashes {
        c1.m_field_multi_values.push(an_hash as &(dyn ToSql + Sync));
    }
    let w_blocks = search_in_dag(
        vec![c1],
        vec!["b_hash"],
        vec![],
        0,
        true,
    );

    if w_blocks.len() == 0
    {
        dlog(
            &format!(
                "Retrieve Blocks In Which A Ref Loc Produced for the coin({}) not exist!",
                the_coin
            ),
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return results;
    }

    // it could be possible some docHash exactly exist in more than one block
    // e.g multiplication coinbase blocks which are created in same cycle
    // e.g cloned transactions(the same transaction which takes part in different blocks by different backers)
    // therefore we return list of blocks

    for w_block in w_blocks
    {
        let (_status, _sf_ver, content) = unwrap_safed_content_for_db(&w_block["b_body"]);
        let (_status, block) = Block::load_block_by_serialized_content(&content);

        for doc_index in 0..block.get_docs_count()
        {
            let doc: &Document = &block.get_documents()[doc_index as usize];
            if doc.doc_has_output()
            {
                for output_index in 0..doc.get_outputs().len()
                {
                    let tmp_coin = cutils::pack_coin_code(&doc.get_doc_hash(), output_index as COutputIndexT);
                    if tmp_coin == *the_coin
                    {
                        results.push(CoinDetails {
                            m_cd_code: the_coin.clone(),
                            m_cd_owner: doc.get_outputs()[output_index].m_address.clone(),
                            m_cd_amount: doc.get_outputs()[output_index].m_amount,
                            m_cd_creation_date: block.m_block_creation_date.clone(),
                            m_cd_block_hash: block.get_block_hash(),
                            m_cd_doc_index: doc_index,
                            m_cd_doc_hash: doc_hash.clone(),
                            m_cd_output_index: output_index as COutputIndexT,
                            m_cd_cycle: "".to_string(),
                        });
                    }
                }
            }
        }
    }

    return results;
}

/*

std::tuple<CMPAIValueT, QVDRecordsT, CMPAIValueT> DAG::getNotImportedCoinbaseBlocks()
{
  QVDRecordsT wBlocks = searchInDAG(
    {{"b_coins_imported", constants::NO},
    {"b_type", {constants::block_types::FSign, constants::block_types::FVote}, "NOT IN"},
    {"b_type", VString{constants::block_types::COINBASE}, "IN"}});

  CMPAIValueT sum = 0;
  QVDRecordsT processed_outputs {};
  VString calculated_coinbase {};
  for (QVDicT wBlock: wBlocks)
  {
    JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"].to_string()).content);

    if (calculated_coinbase.contains(block["bCycle"].to_string()))
      continue;

    if (block["bType"].to_string() == constants::block_types::COINBASE)
        calculated_coinbase.push(block["bCycle"].to_string());

    // analyze outputs
    JSonObject doc = block["bDocs"].toArray()[0].toObject();
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

  CMPAIValueT coinbase_value = (calculated_coinbase.len() * CoinbaseIssuer::calcPotentialMicroPaiPerOneCycle(application().now().split("-")[0]));

  return {sum, processed_outputs, coinbase_value};
}

std::tuple<CMPAIValueT, VString, String> DAG::getNotImportedNormalBlock()
{
  QVDRecordsT wBlocks = searchInDAG(
    {{"b_coins_imported", constants::NO},
    {"b_type", VString{constants::block_types::Normal, "IN"}}});
  CMPAIValueT sum = 0;
  HashMap<CDocHashT, int64_t> maybe_dbl_spends = {};
  VString processed_outputs {};

  for (QVDicT wBlock: wBlocks)
  {
    JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(wBlock["b_body"].to_string()).content);

    for (auto doc_: block["bDocs"].toArray())
    {
      auto doc = doc_.toObject();
      if (!VString{constants::document_types::BASIC_TX}.contains(doc["dType"].to_string()))
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
            processed_outputs.push(VString {
              block["bCDate"].to_string().split(" ")[1],
              cutils::hash6c(doc["dHash"].to_string()),
              "DPCost",
              cutils::microPAIToPAI6(output[1].toDouble())
            }.join("\t  "));
          } else {
            if (constants::TREASURY_PAYMENTS.contains(output[0].to_string()))
            {
              processed_outputs.push(VString {
                block["bCDate"].to_string().split(" ")[1],
                cutils::hash6c(doc["dHash"].to_string()),
                cutils::short_bech16(output[0].to_string()),
                cutils::microPAIToPAI6(output[1].toDouble())});

            } else {
                processed_outputs.push(VString {
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

  VString dbl_spends {};
  for (CDocHashT ref: maybe_dbl_spends.keys())
    if (maybe_dbl_spends[ref] > 1)
      dbl_spends.push(cutils::short_coin_code(ref) + "->" + String::number(maybe_dbl_spends[ref]));


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
std::tuple<CMPAIValueT, HashMap<CBlockHashT, CMPAIValueT>, CMPAIValueT, HashMap<CBlockHashT, CMPAIValueT>, CMPAIValueT> DAG::getCBBlocksStat()
{
  CMPAIValueT total_minted_coins = 0;
  HashMap<CBlockHashT, CMPAIValueT> outputs_by_block {};

  CMPAIValueT floorish_micro_PAIs = 0;    //missedMicroPAIs
  HashMap<CBlockHashT, CMPAIValueT> burned_by_block {}; //missedMicroPAIsBlocks

  CMPAIValueT waited_coinbases_to_be_spendable = 0;
  VString considered_cycles {};

  QVDRecordsT wBlocks = searchInDAG(
    {{"b_type", constants::block_types::COINBASE}},
    {"b_cycle", "b_body", "b_coins_imported"},
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
    if (block.keys().contains("bDocs"))
    {
      auto docs = block["bDocs"].toArray();
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

    if (wBlock["b_coins_imported"].to_string() == constants::NO)
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
    //   c_date = application().now();

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
