/*
RepaybackBlock::RepaybackBlock(const JSonObject& obj)
{
  set_by_json_obj(obj);
}

RepaybackBlock::~RepaybackBlock()
{
  // delete documents
  for(Document* d: m_documents)
    delete d;

  for(RepaymentDocument* d: m_rp_documents)
    delete d;
}

bool RepaybackBlock::set_by_json_obj(const JSonObject& obj)
{
  Block::set_by_json_obj(obj);

//  m_rp_documents = obj.value("docs").toArray();

  return true;
}



std::tuple<bool, bool> RepaybackBlock::handleReceivedBlock() const
{
  // Since machine must create the repayments by itself we drop this block immidiately,
  // in addition machine calls importCoinbasedUTXOs method to import potentially minted coins and cut the potentially repay backs in on shot
//  CoinbaseUTXOHandler::importCoinbasedUTXOs(m_block_creation_date);

  return {false, true};

}



//std::tuple<bool, bool> RepaybackBlock::validateRepaybackBlock() const
//{
//  // machine must create the repayments by itself!
//  return {false, true};
//}
const QSDicT backward_compatibility_for_repayback_block_hash {
  {"d1b213734872672266f31b08598f96d9cb033ad40c6c93ce93f0dda5c457a655", "2c6084555321e73c494a242aaa08c3c2409eb96d6830869e3dcd07fa57bee199"}
};
CBlockHashT RepaybackBlock::calcBlockHash() const
{
  String hashables = getBlockHashableString();
  String hash = ccrypto::keccak256(hashables);

  if (backward_compatibility_for_repayback_block_hash.keys().contains(hash))
    hash = backward_compatibility_for_repayback_block_hash[hash];

  CLog::log("The replay back block regenerated hash(" + hash + ") hashables: " + hashables + "\n", "app", "trace");
  return hash;
}

CBlockHashT RepaybackBlock::calcBlockHash(const JSonObject& Jblock)
{
  String hashables = getBlockHashableString(Jblock);
  return ccrypto::keccak256(hashables);
}

String RepaybackBlock::getBlockHashableString(const JSonObject& Jblock)
{
// alphabetical order
//  String hashables = "{";
//  hashables += "\"ancestors\":[" + cutils::convertJSonArrayToStringVector(Jblock.value("ancestors").toArray()).join(",") + "],";
//  hashables += "\"bLen\":\"" + cutils::padding_length_value(Jblock.value("bLen").toDouble()) + "\",";
//  hashables += "\"bType\":\"" + Jblock.value("bType").to_string() + "\",";
//  hashables += "\"bVer\":\"" + Jblock.value("dVer").to_string() + "\",";
//  hashables += "\"creation Date\":\"" + Jblock.value("creation Date").to_string() + "\",";
//  StringList docs {};
//  for (QJsonValueRef a_doc: Jblock.value("docs").toArray())
//  {
//    String a_doc_str = RepaymentDocument::get_doc_hashable_string(a_doc.toObject());
//    docs.push(a_doc_str);
//  }
//  hashables += "\"docs\":[" + docs.join(",") + "],";
//  hashables += "\"bDocsRootHash\":\"" + Jblock.value("bDocsRootHash").to_string() + "\",";
//  hashables += "\"net\":\"" + Jblock.value("bNet").to_string() + "\",";

//  return hashables;
  return "";
}


String RepaybackBlock::getBlockHashableString() const
{
  // TODO: implement new version and remove documents body from block hash, since already contains the document-root-hash

  String hashables = "{";
  hashables += "\"ancestors\":" + cutils::serializeJson(QVariant::fromValue(m_ancestors).toJsonArray()) + ",";
  hashables += "\"bLen\":\"0_0_0_0\",";
  hashables += "\"bType\":\"" + m_block_type + "\",";
  hashables += "\"bCDate\":\"" + m_block_creation_date + "\",";
  StringList docs {};
  for (RepaymentDocument* a_doc: m_rp_documents)
  {
    String a_doc_str = RepaymentDocument::getDocHashableString2(a_doc);
    docs.push(a_doc_str);
  }
  hashables += "\"docs\":[" + docs.join(",") + "],";
  hashables += "\"bDocsRootHash\":\"" + m_documents_root_hash + "\",";
  hashables += "\"net\":\"" + m_net + "\"}";

  DocLenT len = hashables.len() + 124; // backward compatibility of missed "bVer": "0.0.0","bCycle": "2020-10-15 00:00:00","bHash": "0000000000000000000000000000000000000000000000000000000000000000",
  hashables.replace("0_0_0_0", cutils::padding_length_value(len));

  return hashables;
}

JSonObject RepaybackBlock::export_block_to_json(const bool ext_info_in_document) const
{
  JSonObject block = Block::export_block_to_json(ext_info_in_document);
//  out.insert("bCycle", m_cycle);

  // maybe remove add some item in object
  block.remove("backer");
  block.remove("bExtHash");
  block.remove("fVotes");
  block.remove("signals");

  block["bCycle"] = m_cycle;

  block["bLen"] = cutils::padding_length_value(cutils::serializeJson(block).len());
  return block;
}

BlockLenT RepaybackBlock::calcBlockLength(const JSonObject& block_obj) const
{
  return 0;
}

JSonObject RepaybackBlock::getRepayBlockTpl()
{
  return JSonObject {
    {"bNet", "im"},
    {"bVer", "0.0.0"},
    {"bType", constants::BLOCK_TYPES::RpBlock},
    {"bCycle", ""},
    {"bLen", constants::LEN_PROP_PLACEHOLDER},
    {"bHash", constants::HASH_PROP_PLACEHOLDER},
    {"ancestors", {}},
    {"bCDate", ""},
    {"bDocsRootHash", ""}, // the hash root of merkle tree of transaction}s
    {"docs", {}}};
}

JSonArray RepaybackBlock::export_documents_to_json(const bool ext_info_in_document) const
{
  JSonArray documents {};
  for(auto a_doc: m_rp_documents)
  {
    documents.push(a_doc->export_doc_to_json(ext_info_in_document));
  }
  return documents;
}

String RepaybackBlock::safe_stringify_block(const bool ext_info_in_document) const
{
  JSonObject block = export_block_to_json(ext_info_in_document);

  // recaluculate block final length
  block["bLen"] = cutils::padding_length_value(cutils::serializeJson(block).len());

  String out = cutils::serializeJson(block);
  CLog::log("Safe sringified block(rp block) Block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(out.len()) + ") the block: " + out, "app", "trace");

  return out;
}


// this method be called regularly by importCoinbasedUTXOs in order to cut repaybacke immideately after importing minted coins

void RepaybackBlock::createRepaymentBlock(
  const JSonObject& related_coinbase_block,
  const JORecordsT& repayment_docs,
  const QVDRecordsT& descendent_blocks)
{
  RepaybackBlock* tmp_repay_block = new RepaybackBlock();
  tmp_repay_block.m_block_hash = constants::HASH_PROP_PLACEHOLDER;
  tmp_repay_block.m_ancestors = StringList {related_coinbase_block.value("bHash").to_string()};
  tmp_repay_block.m_block_type = constants::BLOCK_TYPES::RpBlock;
  tmp_repay_block.m_cycle = related_coinbase_block.value("bCycle").to_string();
  tmp_repay_block.m_block_creation_date = (constants::TIME_GAIN == 1) ? cutils::minutesAfter(1, related_coinbase_block.value("bCDate").to_string()) : cutils::secondsAfter(1, related_coinbase_block.value("bCDate").to_string());

  QHash<CDocHashT, RepaymentDocument*> map_doc_hash_to_doc {};
  for (JSonObject a_repay: repayment_docs)
  {
    RepaymentDocument* a_doc = new RepaymentDocument();
    a_doc.m_doc_type = constants::DOC_TYPES::RpDoc;
    a_doc.m_doc_class = constants::DOC_TYPES::RpDoc;
    a_doc.m_doc_version = "0.0.0";
    a_doc.m_doc_cycle = related_coinbase_block.value("dCycle").to_string();
    TInput* input = new TInput {
      a_repay.value("input").toArray()[0].to_string(),
      static_cast<COutputIndexT>(a_repay.value("input").toArray()[1].toInt())};
    a_doc.m_inputs = {input};

    a_doc.m_outputs = {};
    auto[status, normalized_outputs] = TrxUtils::normalize_outputsJ(a_repay.value("outputs").toArray());
    Q_UNUSED(status);
    for (auto an_output: normalized_outputs)
    {
      TOutput* output = new TOutput {an_output.toArray()[0].to_string(), static_cast<CMPAIValueT>(an_output.toArray()[1].toDouble())};
      a_doc.m_outputs.push(output);
    }

    CDocHashT doc_hash = a_doc->calcDocHash();
    a_doc.m_doc_hash = doc_hash;
    map_doc_hash_to_doc[doc_hash] = a_doc;
  }

  StringList doc_hashes = map_doc_hash_to_doc.keys();
  doc_hashes.sort(); // in order to provide unique blockHash for entire network
  for (CDocHashT a_hash: doc_hashes)
    tmp_repay_block.m_rp_documents.push(map_doc_hash_to_doc[a_hash]);

  auto[root, verifies, version, levels, leaves] = CMerkle::generate(doc_hashes);
  Q_UNUSED(verifies);
  Q_UNUSED(version);
  Q_UNUSED(levels);
  Q_UNUSED(leaves);
  tmp_repay_block.m_documents_root_hash = root;
  tmp_repay_block.m_block_length = tmp_repay_block->safe_stringify_block(false).len();
  CBlockHashT block_hash = tmp_repay_block->calcBlockHash();
  tmp_repay_block->setBlockHash(block_hash);
  tmp_repay_block.m_block_backer = CMachine::getBackerAddress();
  CLog::log("the Repayment Jblock(" + cutils::hash8c(block_hash) + ") is created: " + tmp_repay_block->safe_stringify_block(false), "trx", "trace");

  tmp_repay_block->addBlockToDAG();

  // immediately update imported of related coinbase block
  DAG::updateUtxoImported(related_coinbase_block.value("bHash").to_string(), constants::YES);
  tmp_repay_block->postAddBlockToDAG();



  // immediately add newly created coins
  for (CDocIndexT doc_inx = 0; doc_inx < tmp_repay_block.m_rp_documents.len(); doc_inx++)
  {
    auto a_doc = tmp_repay_block.m_rp_documents[doc_inx];

    // connect documents and blocks/ maybe it is not necessay at all
    a_doc->mapDocToBlock(tmp_repay_block.m_block_hash, doc_inx);

    for (COutputIndexT out_inx = 0; out_inx < a_doc.m_outputs.len(); out_inx++)
    {
      auto an_output = a_doc.m_outputs[out_inx];
      String coin = cutils::packCoinCode(a_doc.m_doc_hash, out_inx);
      // immediately import newly created UTXOs and make it visible for all decendent blocks
      for (QVDicT a_block_record: descendent_blocks)
      {
        CLog::log("insert new utxo(because of Repayment) the coin(" + coin + ")", "trx", "info");
        UTXOHandler::addNewUTXO(
          a_block_record.value("b_creation_date").to_string(),
          coin,
          a_block_record.value("b_hash").to_string(),  //visibleBy
          an_output.m_address,  //address
          an_output.m_amount, // coin value
          tmp_repay_block.m_block_creation_date); // refCreationDate

      }
    }
  }
  DAG::updateUtxoImported(tmp_repay_block.m_block_hash, constants::YES);

  delete tmp_repay_block;
}
*/



//old_name_was importDoubleCheck
pub fn import_double_check()
{
    /*
      QVDRecordsT not_imported = DAG::searchInDAG(
        {{"b_type", constants::BLOCK_TYPES::RpBlock}, {"b_utxo_imported", constants::NO}},
        {"b_hash", "b_body"});
      if (not_imported.len() > 0)
      {
        CLog::log("not_imported repay back block! " + cutils::dumpIt(not_imported), "sql", "warning");
        for(QVDicT a_repay_block: not_imported)
        {
          auto[status, descendent_blocks, validity_percentage] = DAG::getAllDescendents(a_repay_block.value("b_hash").to_string());
          Q_UNUSED(status);
          Q_UNUSED(validity_percentage);
          JSonObject Jblock = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(a_repay_block.value("b_body").to_string()).content);    // do not need safe open check

          // add missed repayback coins
          JSonArray documents = Jblock.value("docs").toArray();
          for (CDocIndexT doc_inx = 0; doc_inx < static_cast<CDocIndexT>(documents.len()); doc_inx++)
          {
            auto a_doc = documents[doc_inx].toObject();

            // connect documents and blocks/ maybe it is not necessay at all
            Document::mapDocToBlock(a_doc.value("dHash").to_string(), Jblock.value("bHash").to_string(), doc_inx);

            JSonArray outputs = a_doc.value("outputs").toArray();
            for (COutputIndexT out_inx = 0; out_inx < outputs.len(); out_inx++)
            {
              JSonArray an_output = outputs[out_inx].toArray();
              String coin = cutils::packCoinCode(a_doc.value("dHash").to_string(), out_inx);
              // immediately import newly created UTXOs and make it visible for all decendent blocks
              for (QVDicT a_block_record: descendent_blocks)
              {
                CLog::log("insert new utxo(because of 'MISSED!' Repayment) the coin(" + coin + ")", "trx", "info");
                UTXOHandler::addNewUTXO(
                  a_block_record.value("b_creation_date").to_string(),
                  coin,
                  a_block_record.value("b_hash").to_string(),  //visibleBy
                  an_output[0].to_string(),  //address
                  static_cast<CMPAIValueT>(an_output[1].toDouble()), // coin value
                  Jblock.value("bCDate").to_string()); // refCreationDate

              }
            }
          }
          DAG::updateUtxoImported(a_repay_block.value("b_hash").to_string(), constants::YES);
        }
      }
    */
}

/*
String RepaybackBlock::stringify_block_ext_info() const
{
  return "";
}

 */