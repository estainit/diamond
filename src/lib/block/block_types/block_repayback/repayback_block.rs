/*
RepaybackBlock::RepaybackBlock(const QJsonObject& obj)
{
  setByJsonObj(obj);
}

RepaybackBlock::~RepaybackBlock()
{
  // delete documents
  for(Document* d: m_documents)
    delete d;

  for(RepaymentDocument* d: m_rp_documents)
    delete d;
}

bool RepaybackBlock::setByJsonObj(const QJsonObject& obj)
{
  Block::setByJsonObj(obj);

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
  QString hashables = getBlockHashableString();
  QString hash = CCrypto::keccak256(hashables);

  if (backward_compatibility_for_repayback_block_hash.keys().contains(hash))
    hash = backward_compatibility_for_repayback_block_hash[hash];

  CLog::log("The replay back block regenerated hash(" + hash + ") hashables: " + hashables + "\n", "app", "trace");
  return hash;
}

CBlockHashT RepaybackBlock::calcBlockHash(const QJsonObject& Jblock)
{
  QString hashables = getBlockHashableString(Jblock);
  return CCrypto::keccak256(hashables);
}

QString RepaybackBlock::getBlockHashableString(const QJsonObject& Jblock)
{
// alphabetical order
//  QString hashables = "{";
//  hashables += "\"ancestors\":[" + CUtils::convertJSonArrayToQStringList(Jblock.value("ancestors").toArray()).join(",") + "],";
//  hashables += "\"bLen\":\"" + CUtils::paddingLengthValue(Jblock.value("bLen").toDouble()) + "\",";
//  hashables += "\"bType\":\"" + Jblock.value("bType").toString() + "\",";
//  hashables += "\"bVer\":\"" + Jblock.value("dVer").toString() + "\",";
//  hashables += "\"creation Date\":\"" + Jblock.value("creation Date").toString() + "\",";
//  QStringList docs {};
//  for (QJsonValueRef a_doc: Jblock.value("docs").toArray())
//  {
//    QString a_doc_str = RepaymentDocument::getDocHashableString(a_doc.toObject());
//    docs.append(a_doc_str);
//  }
//  hashables += "\"docs\":[" + docs.join(",") + "],";
//  hashables += "\"docsRootHash\":\"" + Jblock.value("docsRootHash").toString() + "\",";
//  hashables += "\"net\":\"" + Jblock.value("net").toString() + "\",";

//  return hashables;
  return "";
}


QString RepaybackBlock::getBlockHashableString() const
{
  // TODO: implement new version and remove documents body from block hash, since already contains the document-root-hash

  QString hashables = "{";
  hashables += "\"ancestors\":" + CUtils::serializeJson(QVariant::fromValue(m_ancestors).toJsonArray()) + ",";
  hashables += "\"bLen\":\"0_0_0_0\",";
  hashables += "\"bType\":\"" + m_block_type + "\",";
  hashables += "\"bCDate\":\"" + m_block_creation_date + "\",";
  QStringList docs {};
  for (RepaymentDocument* a_doc: m_rp_documents)
  {
    QString a_doc_str = RepaymentDocument::getDocHashableString2(a_doc);
    docs.append(a_doc_str);
  }
  hashables += "\"docs\":[" + docs.join(",") + "],";
  hashables += "\"docsRootHash\":\"" + m_documents_root_hash + "\",";
  hashables += "\"net\":\"" + m_net + "\"}";

  DocLenT len = hashables.length() + 124; // backward compatibility of missed "bVer": "0.0.0","cycle": "2020-10-15 00:00:00","bHash": "0000000000000000000000000000000000000000000000000000000000000000",
  hashables.replace("0_0_0_0", CUtils::paddingLengthValue(len));

  return hashables;
}

QJsonObject RepaybackBlock::exportBlockToJSon(const bool ext_info_in_document) const
{
  QJsonObject block = Block::exportBlockToJSon(ext_info_in_document);
//  out.insert("cycle", m_cycle);

  // maybe remove add some item in object
  block.remove("backer");
  block.remove("bExtHash");
  block.remove("fVotes");
  block.remove("signals");

  block["cycle"] = m_cycle;

  block["bLen"] = CUtils::paddingLengthValue(CUtils::serializeJson(block).length());
  return block;
}

BlockLenT RepaybackBlock::calcBlockLength(const QJsonObject& block_obj) const
{
  return 0;
}

QJsonObject RepaybackBlock::getRepayBlockTpl()
{
  return QJsonObject {
    {"net", "im"},
    {"bVer", "0.0.0"},
    {"bType", CConsts::BLOCK_TYPES::RpBlock},
    {"cycle", ""},
    {"bLen", "0000000"},
    {"bHash", "0000000000000000000000000000000000000000000000000000000000000000"},
    {"ancestors", {}},
    {"bCDate", ""},
    {"docsRootHash", ""}, // the hash root of merkle tree of transaction}s
    {"docs", {}}};
}

QJsonArray RepaybackBlock::exportDocumentsToJSon(const bool ext_info_in_document) const
{
  QJsonArray documents {};
  for(auto a_doc: m_rp_documents)
  {
    documents.push_back(a_doc->exportDocToJson(ext_info_in_document));
  }
  return documents;
}

QString RepaybackBlock::safeStringifyBlock(const bool ext_info_in_document) const
{
  QJsonObject block = exportBlockToJSon(ext_info_in_document);

  // recaluculate block final length
  block["bLen"] = CUtils::paddingLengthValue(CUtils::serializeJson(block).length());

  QString out = CUtils::serializeJson(block);
  CLog::log("Safe sringified block(rp block) Block(" + CUtils::hash8c(m_block_hash) + ") length(" + QString::number(out.length()) + ") the block: " + out, "app", "trace");

  return out;
}


// this method be called regularly by importCoinbasedUTXOs in order to cut repaybacke immideately after importing minted coins

void RepaybackBlock::createRepaymentBlock(
  const QJsonObject& related_coinbase_block,
  const JORecordsT& repayment_docs,
  const QVDRecordsT& descendent_blocks)
{
  RepaybackBlock* tmp_repay_block = new RepaybackBlock();
  tmp_repay_block->m_block_hash = "0000000000000000000000000000000000000000000000000000000000000000";
  tmp_repay_block->m_ancestors = QStringList {related_coinbase_block.value("bHash").toString()};
  tmp_repay_block->m_block_type = CConsts::BLOCK_TYPES::RpBlock;
  tmp_repay_block->m_cycle = related_coinbase_block.value("cycle").toString();
  tmp_repay_block->m_block_creation_date = (CConsts::TIME_GAIN == 1) ? CUtils::minutesAfter(1, related_coinbase_block.value("bCDate").toString()) : CUtils::secondsAfter(1, related_coinbase_block.value("bCDate").toString());

  QHash<CDocHashT, RepaymentDocument*> map_doc_hash_to_doc {};
  for (QJsonObject a_repay: repayment_docs)
  {
    RepaymentDocument* a_doc = new RepaymentDocument();
    a_doc->m_doc_type = CConsts::DOC_TYPES::RpDoc;
    a_doc->m_doc_class = CConsts::DOC_TYPES::RpDoc;
    a_doc->m_doc_version = "0.0.0";
    a_doc->m_doc_cycle = related_coinbase_block.value("cycle").toString();
    TInput* input = new TInput {
      a_repay.value("input").toArray()[0].toString(),
      static_cast<COutputIndexT>(a_repay.value("input").toArray()[1].toInt())};
    a_doc->m_inputs = {input};

    a_doc->m_outputs = {};
    auto[status, normalized_outputs] = TrxUtils::normalizeOutputsJ(a_repay.value("outputs").toArray());
    Q_UNUSED(status);
    for (auto an_output: normalized_outputs)
    {
      TOutput* output = new TOutput {an_output.toArray()[0].toString(), static_cast<CMPAIValueT>(an_output.toArray()[1].toDouble())};
      a_doc->m_outputs.push_back(output);
    }

    CDocHashT doc_hash = a_doc->calcDocHash();
    a_doc->m_doc_hash = doc_hash;
    map_doc_hash_to_doc[doc_hash] = a_doc;
  }

  QStringList doc_hashes = map_doc_hash_to_doc.keys();
  doc_hashes.sort(); // in order to provide unique blockHash for entire network
  for (CDocHashT a_hash: doc_hashes)
    tmp_repay_block->m_rp_documents.push_back(map_doc_hash_to_doc[a_hash]);

  auto[root, verifies, version, levels, leaves] = CMerkle::generate(doc_hashes);
  Q_UNUSED(verifies);
  Q_UNUSED(version);
  Q_UNUSED(levels);
  Q_UNUSED(leaves);
  tmp_repay_block->m_documents_root_hash = root;
  tmp_repay_block->m_block_length = tmp_repay_block->safeStringifyBlock(false).length();
  CBlockHashT block_hash = tmp_repay_block->calcBlockHash();
  tmp_repay_block->setBlockHash(block_hash);
  tmp_repay_block->m_block_backer = CMachine::getBackerAddress();
  CLog::log("the Repayment Jblock(" + CUtils::hash8c(block_hash) + ") is created: " + tmp_repay_block->safeStringifyBlock(false), "trx", "trace");

  tmp_repay_block->addBlockToDAG();

  // immediately update imported of related coinbase block
  DAG::updateUtxoImported(related_coinbase_block.value("bHash").toString(), CConsts::YES);
  tmp_repay_block->postAddBlockToDAG();



  // immediately add newly created coins
  for (CDocIndexT doc_inx = 0; doc_inx < tmp_repay_block->m_rp_documents.size(); doc_inx++)
  {
    auto a_doc = tmp_repay_block->m_rp_documents[doc_inx];

    // connect documents and blocks/ maybe it is not necessay at all
    a_doc->mapDocToBlock(tmp_repay_block->m_block_hash, doc_inx);

    for (COutputIndexT out_inx = 0; out_inx < a_doc->m_outputs.size(); out_inx++)
    {
      auto an_output = a_doc->m_outputs[out_inx];
      QString coin = CUtils::packCoinCode(a_doc->m_doc_hash, out_inx);
      // immediately import newly created UTXOs and make it visible for all decendent blocks
      for (QVDicT a_block_record: descendent_blocks)
      {
        CLog::log("insert new utxo(because of Repayment) the coin(" + coin + ")", "trx", "info");
        UTXOHandler::addNewUTXO(
          a_block_record.value("b_creation_date").toString(),
          coin,
          a_block_record.value("b_hash").toString(),  //visibleBy
          an_output->m_address,  //address
          an_output->m_amount, // coin value
          tmp_repay_block->m_block_creation_date); // refCreationDate

      }
    }
  }
  DAG::updateUtxoImported(tmp_repay_block->m_block_hash, CConsts::YES);

  delete tmp_repay_block;
}
*/



//old_name_was importDoubleCheck
pub fn import_double_check()
{
    /*
      QVDRecordsT not_imported = DAG::searchInDAG(
        {{"b_type", CConsts::BLOCK_TYPES::RpBlock}, {"b_utxo_imported", CConsts::NO}},
        {"b_hash", "b_body"});
      if (not_imported.size() > 0)
      {
        CLog::log("not_imported repay back block! " + CUtils::dumpIt(not_imported), "sql", "warning");
        for(QVDicT a_repay_block: not_imported)
        {
          auto[status, descendent_blocks, validity_percentage] = DAG::getAllDescendents(a_repay_block.value("b_hash").toString());
          Q_UNUSED(status);
          Q_UNUSED(validity_percentage);
          QJsonObject Jblock = CUtils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(a_repay_block.value("b_body").toString()).content);    // do not need safe open check

          // add missed repayback coins
          QJsonArray documents = Jblock.value("docs").toArray();
          for (CDocIndexT doc_inx = 0; doc_inx < static_cast<CDocIndexT>(documents.size()); doc_inx++)
          {
            auto a_doc = documents[doc_inx].toObject();

            // connect documents and blocks/ maybe it is not necessay at all
            Document::mapDocToBlock(a_doc.value("dHash").toString(), Jblock.value("bHash").toString(), doc_inx);

            QJsonArray outputs = a_doc.value("outputs").toArray();
            for (COutputIndexT out_inx = 0; out_inx < outputs.size(); out_inx++)
            {
              QJsonArray an_output = outputs[out_inx].toArray();
              QString coin = CUtils::packCoinCode(a_doc.value("dHash").toString(), out_inx);
              // immediately import newly created UTXOs and make it visible for all decendent blocks
              for (QVDicT a_block_record: descendent_blocks)
              {
                CLog::log("insert new utxo(because of 'MISSED!' Repayment) the coin(" + coin + ")", "trx", "info");
                UTXOHandler::addNewUTXO(
                  a_block_record.value("b_creation_date").toString(),
                  coin,
                  a_block_record.value("b_hash").toString(),  //visibleBy
                  an_output[0].toString(),  //address
                  static_cast<CMPAIValueT>(an_output[1].toDouble()), // coin value
                  Jblock.value("bCDate").toString()); // refCreationDate

              }
            }
          }
          DAG::updateUtxoImported(a_repay_block.value("b_hash").toString(), CConsts::YES);
        }
      }
    */
}

/*
QString RepaybackBlock::stringifyBExtInfo() const
{
  return "";
}

 */