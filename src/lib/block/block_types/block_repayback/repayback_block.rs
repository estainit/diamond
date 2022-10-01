use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::{application, constants, cutils, dlog, machine};
use crate::cmerkle::{generate_m, MERKLE_VERSION};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::block::document_types::rp_document::RepaymentDocument;
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CDocHashT, CDocIndexT, COutputIndexT, QVDRecordsT, VString};
use crate::lib::dag::dag::set_coins_import_status;
use crate::lib::transactions::basic_transactions::coins::coins_handler::add_new_coin;
use crate::lib::transactions::trx_utils::{normalize_rp_outputs};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepaybackBlock {
    pub m_block_cycle: String,
}

impl RepaybackBlock
{
    pub fn new() -> Self {
        Self {
            m_block_cycle: "".to_string()
        }
    }
}

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

//  m_rp_documents = obj["bDocs"].toArray();

  return true;
}



std::tuple<bool, bool> RepaybackBlock::handle_received_block() const
{
  // Since machine must create the repayments by itself we drop this block immidiately,
  // in addition machine calls importCoinbased coins method to import potentially minted coins and cut the potentially repay backs in on shot
//  import_minted_coins(m_block_creation_date);

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
//  hashables += "\"ancestors\":[" + cutils::convertJSonArrayToStringVector(Jblock["ancestors"].toArray()).join(",") + "],";
//  hashables += "\"bLen\":\"" + cutils::padding_length_value(Jblock["bLen"].toDouble()) + "\",";
//  hashables += "\"bType\":\"" + Jblock["bType"].to_string() + "\",";
//  hashables += "\"bVer\":\"" + Jblock["dVer"].to_string() + "\",";
//  hashables += "\"creation Date\":\"" + Jblock["creation ]ate").to_string() + "\",";
//  VString docs {};
//  for (QJsonValueRef a_doc: Jblock["bDocs"].toArray())
//  {
//    String a_doc_str = RepaymentDocument::get_doc_hashable_string(a_doc.toObject());
//    docs.push(a_doc_str);
//  }
//  hashables += "\"docs\":[" + docs.join(",") + "],";
//  hashables += "\"bDocsRootHash\":\"" + Jblock["bDocsRootHash"].to_string() + "\",";
//  hashables += "\"net\":\"" + Jblock["bNet"].to_string() + "\",";

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
  VString docs {};
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
    {"bType", constants::block_types::RpBlock},
    {"bCycle", ""},
    {"bLen", constants::LEN_PROP_PLACEHOLDER},
    {"bHash", constants::HASH_ZEROS_PLACEHOLDER.to_string()},
    {"ancestors", {}},
    {"bCDate", ""},
    {"bDocsRootHash", ""}, // the hash root of merkle tree of transaction}s
    {"bDocs", {}}};
}

JSonArray RepaybackBlock::export_documents_to_json(const bool ext_info_in_document) const
{
  JSonArray documents {};
  for(auto a_doc: m_rp_documents)
  {
    documents.push(a_doc.export_doc_to_json(ext_info_in_document));
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


// this method be called regularly by import Coinbased  coins in order to cut repaybacke immideately after importing minted coins

*/

//old_name_was createRepaymentBlock
pub fn create_repayment_block(
    related_coinbase_block: &Block,
    repayment_docs: &Vec<RepaymentDocument>,
    descendent_blocks: &QVDRecordsT)
{
    let mut tmp_repay_block = Block::new();
    tmp_repay_block.m_block_hash = constants::HASH_ZEROS_PLACEHOLDER.to_string();
    tmp_repay_block.m_block_ancestors = vec![related_coinbase_block.get_block_hash().clone()];
    tmp_repay_block.m_block_type = constants::block_types::REPAYMENT_BLOCK.to_string();
    tmp_repay_block.m_if_repayback_block.m_block_cycle = related_coinbase_block.m_if_repayback_block.m_block_cycle.clone();
    if application().cycle_length() == 1
    {
        tmp_repay_block.m_block_creation_date = application().minutes_after(
            1,
            &related_coinbase_block.get_creation_date());
    } else {
        tmp_repay_block.m_block_creation_date = application().seconds_after(
            1,
            &related_coinbase_block.get_creation_date());
    }

    let mut map_doc_hash_to_doc: HashMap<CDocHashT, Document> = HashMap::new();
    for a_repay in repayment_docs
    {
        let mut a_doc: Document = Document::new();
        a_doc.m_doc_type = constants::document_types::REPAYMENT_DOCUMENT.to_string();
        a_doc.m_doc_class = constants::document_types::REPAYMENT_DOCUMENT.to_string();
        a_doc.m_doc_version = "0.0.0".to_string();
        a_doc.m_if_repayment_doc.m_doc_cycle = related_coinbase_block.m_if_coinbase_block.m_cycle.clone();
        a_doc.m_if_repayment_doc.m_input = a_repay.m_input.clone();

        a_doc.m_if_repayment_doc.m_outputs = vec![];
        let (_status, normalized_outputs) = normalize_rp_outputs(&a_repay.m_outputs, true);
        for an_output in normalized_outputs
        {
            a_doc.m_if_repayment_doc.m_outputs.push(an_output);
        }

        let doc_hash: CDocHashT = a_doc.calc_doc_hash();
        a_doc.m_doc_hash = doc_hash.clone();
        map_doc_hash_to_doc.insert(doc_hash, a_doc);
    }

    let mut doc_hashes: VString = map_doc_hash_to_doc
        .keys()
        .cloned()
        .collect::<VString>();
    doc_hashes.sort(); // in order to provide unique blockHash for entire network
    for a_hash in &doc_hashes
    {
        tmp_repay_block.m_block_documents.push(map_doc_hash_to_doc[a_hash].clone());
    }

    let (root, _verifies, _version, _levels, _leaves) = generate_m(
        doc_hashes,
        &"hashed".to_string(),
        &"keccak256".to_string(),
        &MERKLE_VERSION.to_string());

    tmp_repay_block.m_block_documents_root_hash = root;
    tmp_repay_block.m_block_length = tmp_repay_block.safe_stringify_block(false).len();
    let block_hash: CBlockHashT = tmp_repay_block.calc_block_hash();
    tmp_repay_block.set_block_hash(&block_hash);
    tmp_repay_block.m_block_backer = machine().get_backer_address();
    dlog(
        &format!("The repayment {} is created: {}" ,
    tmp_repay_block.get_block_identifier(),
    tmp_repay_block.safe_stringify_block(false)),
        constants::Modules::Trx,
        constants::SecLevel::Error);


    tmp_repay_block.add_block_to_dag();

    // immediately update imported of related coinbase block
    set_coins_import_status(&related_coinbase_block.get_block_hash(), &constants::YES.to_string());
    tmp_repay_block.post_add_block_to_dag();


    // immediately add newly created coins
    let mut doc_inx: CDocIndexT = 0;
    while doc_inx < tmp_repay_block.m_block_documents.len() as CDocIndexT
    {
        let a_doc: &Document = &tmp_repay_block.m_block_documents[doc_inx as usize];

        // connect documents and blocks/ maybe it is not necessay at all
        a_doc.map_doc_to_block(&tmp_repay_block.m_block_hash, doc_inx);

        let mut out_inx: COutputIndexT = 0;
        while out_inx < a_doc.m_if_repayment_doc.m_outputs.len() as COutputIndexT
        {
            let an_output = &a_doc.m_if_repayment_doc.m_outputs[out_inx as usize];
            let coin: CCoinCodeT = cutils::pack_coin_code(&a_doc.m_doc_hash, out_inx);
            // immediately import newly created  coins and make it visible for all decendent blocks
            for a_block_record in descendent_blocks
            {
                dlog(
                    &format!("insert new coin(because of Repayment) the coin({})", coin),
                    constants::Modules::Trx,
                    constants::SecLevel::Info);
                add_new_coin(
                    &a_block_record["b_creation_date"],
                    &coin,
                    &a_block_record["b_hash"],  //visibleBy
                    &an_output.m_address,  //address
                    an_output.m_amount, // coin value
                    &tmp_repay_block.m_block_creation_date); // refCreationDate
            }
            out_inx += 1;
        }
        doc_inx += 1;
    }

    set_coins_import_status(&tmp_repay_block.m_block_hash, &constants::YES.to_string());

    drop(tmp_repay_block);
}

//old_name_was importDoubleCheck
pub fn import_double_check()
{
    /*
      QVDRecordsT not_imported = DAG::searchInDAG(
        {{"b_type", constants::block_types::RpBlock}, {"b_coins_imported", constants::NO}},
        {"b_hash", "b_body"});
      if (not_imported.len() > 0)
      {
        CLog::log("not_imported repay back block! " + cutils::dumpIt(not_imported), "sql", "warning");
        for(QVDicT a_repay_block: not_imported)
        {
          auto[status, descendent_blocks, validity_percentage] = get_all_descendants(a_repay_block["b_hash"].to_string());
          Q_UNUSED(status);
          Q_UNUSED(validity_percentage);
          JSonObject Jblock = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(a_repay_block["b_body"].to_string()).content);    // do not need safe open check

          // add missed repayback coins
          JSonArray documents = Jblock["bDocs"].toArray();
          for (CDocIndexT doc_inx = 0; doc_inx < static_cast<CDocIndexT>(documents.len()); doc_inx++)
          {
            auto a_doc = documents[doc_inx].toObject();

            // connect documents and blocks/ maybe it is not necessay at all
            Document::mapDocToBlock(a_doc["dHash"].to_string(), Jblock["bHash"].to_string(), doc_inx);

            JSonArray outputs = a_doc["outputs"].toArray();
            for (COutputIndexT out_inx = 0; out_inx < outputs.len(); out_inx++)
            {
              JSonArray an_output = outputs[out_inx].toArray();
              String coin = cutils::packCoinCode(a_doc["dHash"].to_string(), out_inx);
              // immediately import newly created  coins and make it visible for all decendent blocks
              for (QVDicT a_block_record: descendent_blocks)
              {
                CLog::log("insert new coin(because of 'MISSED!' Repayment) the coin(" + coin + ")", "trx", "info");
                UTXOHandler::add_new_coin(
                  a_block_record["b_creation_date"].to_string(),
                  coin,
                  a_block_record["b_hash"].to_string(),  //visibleBy
                  an_output[0].to_string(),  //address
                  static_cast<CMPAIValueT>(an_output[1].toDouble()), // coin value
                  Jblock["bCDate"].to_string()); // refCreationDate

              }
            }
          }
           set_coins_import_status(a_repay_block["b_hash"].to_string(), constants::YES);
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