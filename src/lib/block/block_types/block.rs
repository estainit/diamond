use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use crate::{application, CMachine, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block_coinbase::coinbase_block::CoinbaseBlock;
use crate::lib::block::block_types::block_genesis::genesis_block::b_genesis::{genesis_calc_block_hash, genesis_set_by_json_obj};
use crate::lib::block::document_types::document::Document;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::block::node_signals_handler::log_signals;
use crate::lib::block_utils::wrap_safe_content_for_db;
use crate::lib::custom_types::{BlockLenT, CBlockHashT, CDateT, CDocIndexT, ClausesT, JSonObject, JSonArray, OrderT, QVDRecordsT, QSDicT, CDocHashT, DocDicVecT, CMPAISValueT};
use crate::lib::dag::dag::append_descendants;
use crate::lib::dag::dag_walk_through::update_cached_blocks;
use crate::lib::dag::leaves_handler::{add_to_leave_blocks, remove_from_leave_blocks};
use crate::lib::dag::sceptical_dag_integrity_control::controls_of_new_block_insertion;
use crate::lib::database::abs_psql::{OrderModifier, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_BLOCK_EXTINFOS, C_BLOCKS};
use crate::lib::file_handler::file_handler::file_write;


// struct BlockRecord
// {
//     m_id: u64,
//     m_hash: String,
//     // block root hash
//     m_type: String,
//     // block type (genesis/coinbase/normal)
//     m_cycle: String,
//     // the coin base cycle
//     m_confidence: f64,
//     // if the block is coinbase it denots to percentage of share of signers
//     m_ext_root_hash: String,
//     // it was ext_infos_root_hash segwits/zippedInfo... root hashes
//     m_documents_root_hash: String,
//     // it was docs_root_hash documents root hash
//     m_signals: String,
//     // block signals
//     m_trxs_count: u64,
//     // transaction counts
//     m_docs_count: u64,
//     // documents counts
//     m_ancestors_count: u64,
//     // ancestors counts
//     m_ancestors: Vec<String>,
//     // comma seperated block ancestors
//     m_descendents: Vec<String>,
//     // comma seperated block descendents
//     m_body: String,
//     // stringified json block full body(except block ext info)
//     m_creation_date: String,
//     // the block creation date which stated by block creator
//     m_receive_date: String,
//     // the block receive date in local, only for statistics
//     m_confirm_date: String,
//     // the block confirmation date in local node
//     m_block_backer: String,
//     // the BECH32 address of who got paid because of creating this block
//     m_utxo_imported: String, // does UTXO imported to i_trx_utxo table?
//
//     /*
//
//       BlockRecord(const QVDicT& values = {})
//       {
//         if (values.keys().len() > 0)
//           setByRecordDict(values);
//       };
//       bool setByRecordDict(const QVDicT& values = {});
//
//     */
// }


pub struct TransientBlockInfo
{
    m_status: bool,
    m_block: Block,
    m_stage: String,

    m_map_trx_hash_to_trx_ref: QSDicT,
    m_map_trx_ref_to_trx_hash: QSDicT,
    m_map_referencer_to_referenced: QSDicT,
    m_map_referenced_to_referencer: QSDicT,

    m_doc_by_hash: HashMap<String, Document>,
    m_transactions_dict: HashMap<String, Document>,
    m_groupped_documents: DocDicVecT,
    m_doc_index_by_hash: HashMap<CDocHashT, CDocIndexT>,

    m_block_total_output: CMPAISValueT,
    m_block_documents_hashes: Vec<String>,
    m_block_ext_infos_hashes: Vec<String>,
    m_pre_requisities_ancestors: Vec<String>, // in case of creating a block which contains some ballots, the block explicitely includes the related polling blocks, in order to force and asure existance of polling recorded in DAG, befor applying the ballot(s)
}

impl TransientBlockInfo {
    pub fn new() -> TransientBlockInfo {
        TransientBlockInfo {
            m_status: false,
            m_block: Block::new(),
            m_stage: "".to_string(),
            m_map_trx_hash_to_trx_ref: Default::default(),
            m_map_trx_ref_to_trx_hash: Default::default(),
            m_map_referencer_to_referenced: Default::default(),
            m_map_referenced_to_referencer: Default::default(),
            m_doc_by_hash: HashMap::new(),
            m_transactions_dict: HashMap::new(),
            m_groupped_documents: HashMap::new(),
            m_doc_index_by_hash: HashMap::new(),
            m_block_total_output: 0,
            m_block_documents_hashes: vec![],
            m_block_ext_infos_hashes: vec![],
            m_pre_requisities_ancestors: vec![],
        }
    }
}

pub struct BlockApprovedDocument
{
    m_approved_doc: Document,
    m_approved_doc_hash: String,
    m_approved_doc_ext_info: Vec<DocExtInfo>,
    m_approved_doc_ext_hash: String,
}

impl BlockApprovedDocument {
    pub fn new() -> BlockApprovedDocument {
        BlockApprovedDocument {
            m_approved_doc: Document::new(),
            m_approved_doc_hash: "".to_string(),
            m_approved_doc_ext_info: vec![],
            m_approved_doc_ext_hash: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Block
{
    pub m_block_net: String,
    pub m_block_length: BlockLenT,
    pub m_block_hash: String,
    pub m_block_type: String,
    pub m_block_version: String,
    pub m_block_ancestors: Vec<String>,
    pub m_block_descendants: Vec<String>,
    pub m_block_signals: JSonObject,
    pub m_block_backer: String,
    pub m_block_confidence: f64,
    pub m_block_creation_date: String,
    pub m_block_receive_date: String,
    pub m_block_confirm_date: String,
    pub m_block_descriptions: String,
    pub m_block_documents_root_hash: String,
    pub m_block_ext_root_hash: String,
    pub m_block_documents: Vec<Document>,
    pub m_block_ext_info: Vec<Vec<JSonObject>>,
    pub m_block_floating_votes: JSonArray, // TODO: to be implemented later

    pub m_if_coinbase_block: CoinbaseBlock,
}

impl Block {
    pub fn new() -> Block {
        Block {
            m_block_net: constants::SOCIETY_NAME.to_string(),
            m_block_descriptions: "".to_string(),
            m_block_length: 0,
            m_block_hash: "".to_string(),
            m_block_type: "".to_string(),
            m_block_version: constants::DEFAULT_BLOCK_VERSION.to_string(),
            m_block_ancestors: vec![],
            m_block_descendants: vec![],
            m_block_signals: json!({}),
            m_block_backer: "".to_string(),
            m_block_confidence: 0.0,
            m_block_creation_date: "".to_string(),
            m_block_receive_date: "".to_string(),
            m_block_confirm_date: "".to_string(),
            m_block_documents: vec![],
            m_block_documents_root_hash: "".to_string(),
            m_block_ext_info: vec![],
            m_block_ext_root_hash: "".to_string(),
            m_block_floating_votes: json!([]),

            m_if_coinbase_block: CoinbaseBlock::new(),
        }
    }

    /*
    String TransientBlockInfo::dumpMe()
    {
      String out = "\n Block total outputs amount: " + String::number(m_block_total_output);
      out += "\n COMPLETE ME!";
      return out;
    }

    //  -  -  -  Block Record
    bool BlockRecord::setByRecordDict(const QVDicT& values)
    {
      if (values.value("b_id", "").to_string() != "")
        m_id = values.value("b_id", "").toUInt();
      if (values.value("b_hash", "").to_string() != "")
        m_hash = values.value("b_hash", "").to_string();
      if (values.value("b_type", "").to_string() != "")
        m_type = values.value("b_type", "").to_string();
      if (values.value("b_cycle", "").to_string() != "")
        m_cycle = values.value("b_cycle", "").to_string();

      if (values.value("b_confidence", "").to_string() != "")
        m_confidence = values.value("b_confidence", "").toFloat();

      if (values.value("b_ext_root_hash", "").to_string() != "")
        m_ext_root_hash = values.value("b_ext_root_hash", "").to_string();
      if (values.value("b_docs_root_hash", "").to_string() != "")
        m_documents_root_hash = values.value("b_docs_root_hash", "").to_string();
      if (values.value("b_signals", "").to_string() != "")
        m_signals = values.value("b_signals", "").to_string();
      if (values.value("b_trxs_count", "").to_string() != "")
        m_trxs_count = values.value("b_trxs_count", "").toUInt();
      if (values.value("b_docs_count", "").to_string() != "")
        m_docs_count = values.value("b_docs_count", "").toUInt();

      if (values.value("b_ancestors_count", "").to_string() != "")
        m_ancestors_count = values.value("b_ancestors_count", "").toUInt();

      if (values.value("b_ancestors", "").to_string() != "")
        m_ancestors = values.value("b_ancestors", "").to_string().split(",");

      if (values.value("b_descendants", "").to_string() != "")
        m_descendents = values.value("b_descendants", "").to_string().split(",");

      if (values.value("b_body", "").to_string() != "")
        m_body = values.value("b_body", "").to_string();
      if (values.value("b_creation_date", "").to_string() != "")
        m_creation_date = values.value("b_creation_date", "").to_string();
      if (values.value("b_receive_date", "").to_string() != "")
        m_receive_date = values.value("b_receive_date", "").to_string();
      if (values.value("b_confirm_date", "").to_string() != "")
        m_confirm_date = values.value("b_confirm_date", "").to_string();
      if (values.value("b_backer", "").to_string() != "")
        m_block_backer = values.value("b_backer", "").to_string();
      if (values.value("b_utxo_imported", "").to_string() != "")
        m_utxo_imported = values.value("b_utxo_imported", "").to_string();
      return true;
    }

    //  -  -  -  Block

    Block::~Block()
    {
      // delete documents
      for(Document* d: m_documents)
        delete d;
    }

    /**
     * @brief Block::setByReceivedJsonDoc
     * @param obj
     * @return
     * converts a JSon object(based on parsing text stream) to a standard c++ object
     */
     */

    pub fn set_by_json_obj(&mut self, obj: &JSonObject) -> bool
    {
        if !obj["local_receive_date"].is_null() {
            self.m_block_receive_date = remove_quotes(&obj["local_receive_date"]);
        }

        if !obj["bNet"].is_null() {
            self.m_block_net = remove_quotes(&obj["bNet"]);
        }
        if !obj["bVer"].is_null() {
            self.m_block_version = remove_quotes(&obj["bVer"]);
        }

        if !obj["bType"].is_null() {
            self.m_block_type = remove_quotes(&obj["bType"]);
        }

        if !obj["bDescriptions"].is_null() {
            self.m_block_descriptions = remove_quotes(&obj["bDescriptions"]);
        }

        // if obj["bConfidence"].to_string() != "" {
        //     println!("iiiiiiiiiiii {}", obj["bConfidence"]);
        //     self.m_block_confidence = remove_quotes(&obj["bConfidence"].to_string().parse4>().unwrap());
        // }

        if !obj["bLen"].is_null() {
            let b_len = obj["bLen"].to_string().parse::<BlockLenT>();
            let (status, b_len) = match obj["bLen"].to_string().parse::<BlockLenT>() {
                Ok(l) => { (true, l) }
                Err(e) => {
                    dlog(
                        &format!("Invalid bLen {:?} in received JSon Obj {:?}", obj["bLen"], e),
                        constants::Modules::App,
                        constants::SecLevel::Error);
                    (false, 0)
                }
            };
            if !status {
                return false;
            }
            self.m_block_length = b_len;
        }

        if !obj["bHash"].is_null() {
            self.m_block_hash = remove_quotes(&obj["bHash"]);
        }

        // if !obj["bAncestors"].toAis_null( > 0 {
        //     self.m_block_ancestors = cutils::convertJSonArrayToStringVector(obj["bAncestors"].toArray());
        // }

        // if !obj["signals"].toOis_null(len() > 0 {
        //     self.m_signals = remove_quotes(&obj["signals"].toObject());

        if !obj["bCDate"].is_null() {
            self.m_block_creation_date = remove_quotes(&obj["bCDate"]);
        }


        if !obj["bDocsRootHash"].is_null() {
            self.m_block_documents_root_hash = remove_quotes(&obj["bDocsRootHash"]);
        }

        if !obj["bExtHash"].is_null() {
            self.m_block_ext_root_hash = remove_quotes(&obj["bExtHash"]);
        }

        if !obj["bExtInfo"].is_null() {
            // self.m_block_ext_info = remove_quotes(&obj["bExtInfo"].to_);if !obj["bDocs"].is_null() {
            // createDocuments(obj["bDocs"]);
        }

        // if !obj["bCycle"].to_is_null( {
        //     self.m_block_cycle = remove_quotes(&obj["bCycle"].to_string());

        if !obj["bBacker"].is_null() {
            self.m_block_backer = remove_quotes(&obj["bBacker"]);
        }

        if !obj["bFVotes"].is_null() {
            // self.m_floating_votes = obj["bFVotes"].toArray();
        }


        let block_type = remove_quotes(&obj["bType"]);
        if block_type == constants::block_types::NORMAL {
            return true;
        } else if block_type == constants::block_types::COINBASE {
            return self.m_if_coinbase_block.set_by_json_obj(obj);
        } else if block_type == constants::block_types::REPAYMENT_BLOCK
        {} else if block_type == constants::block_types::FLOATING_SIGNATURE
        {} else if block_type == constants::block_types::FLOATING_VOTE
        {} else if block_type == constants::block_types::POW
        {} else if block_type == constants::block_types::GENESIS
        {
            return genesis_set_by_json_obj(self, obj);
        }

        println!("Invalid block type1 {:?} in received JSon Obj {:?}", block_type, serde_json::to_string(&obj).unwrap());
        println!("Invalid block type2 {} in received JSon Obj {}", block_type, serde_json::to_string(&obj).unwrap());
        dlog(
            &format!("Invalid block type {} in received JSon Obj {}", block_type, serde_json::to_string(&obj).unwrap()),
            constants::Modules::App,
            constants::SecLevel::Error);
        return false;
    }

    /*

    /**
     * @brief Block::objectAssignmentsControlls
     * @return
     *
     * the tests to avoid injection/maleformed data in received Jsons
     * FIXME: add more tests
     */
    bool Block::objectAssignmentsControlls()
    {
      if (!cutils::isValidHash(m_block_hash))
      {
        CLog::log("Invalid blockHash after js assignment", "sec", "error");
        return false;
      }

      if ((m_block_type != constants::block_types::COINBASE) && !ccrypto::isValidBech32(m_block_backer))
      {
        CLog::log("Invalid block backer after js assignment", "sec", "error");
        return false;
      }



      return true;

    }

*/
    //old_name_was stringifyBExtInfo
    pub fn stringify_block_ext_info(&self) -> String
    {
        if !vec![constants::block_types::COINBASE,
                 constants::block_types::NORMAL,
                 constants::block_types::POW].contains(&&*self.m_block_type)
        {
            dlog(
                &format!("DUMMY BREAKPOINT LOG :)"),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }
        let mut block_ext_info: Vec<Vec<JSonObject>> = vec![];
        for a_doc in &self.m_block_documents {
            block_ext_info.push(a_doc.m_doc_ext_info.clone());
        }
        let out = serde_json::to_string(&block_ext_info).unwrap();
        return out;
    }

    //old_name_was safeStringifyBlock
    pub fn safe_stringify_block(&self, ext_info_in_document: bool) -> String
    {
        let mut j_block: JSonObject = self.export_block_to_json(ext_info_in_document);

        // maybe remove add some item in object

        // recaluculate block final length
        j_block["bLen"] = constants::LEN_PROP_PLACEHOLDER.into();
        j_block["bLen"] = cutils::padding_length_value(
            cutils::serialize_json(&j_block).len().to_string(),
            constants::LEN_PROP_LENGTH).into();

        let out = cutils::serialize_json(&j_block);//serde_json::to_string
        dlog(
            &format!(
                "Safe stringified block(Base class) Block({}) length({}) the block: {}",
                cutils::hash8c(&self.m_block_hash), out.len(), out),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return out;
    }

    /*

    String Block::getBlockHashableString() const
    {
      return "";
    }

    StringList Block::getDocumentsHashes(const JSonObject& block)
    {
      StringList hashes {};
      JSonArray documents = block.value("docs").toArray();
      for (auto a_doc: documents)
        hashes.push(a_doc.toObject().value("dHash").to_string());
      return hashes;
    }

    StringList Block::getDocumentsHashes() const
    {
      StringList hashes {};
      for (Document* a_doc: m_documents)
        hashes.push(a_doc.m_doc_hash);
      return hashes;
    }

    std::tuple<bool, CDocHashT> Block::calcDocumentsRootHash() const
    {
      auto[root, verifies, merkle_version, levels, leaves] = CMerkle::generate(getDocumentsHashes());
      Q_UNUSED(verifies);
      Q_UNUSED(merkle_version);
      Q_UNUSED(levels);
      Q_UNUSED(leaves);
      return {true, root};
    }

    std::tuple<bool, CDocHashT> Block::calcBlockExtRootHash() const
    {
      return {true, ""};
    }

    bool Block::fillInBlockExtInfo()
    {
      m_block_ext_info = {};
      for (Document* doc: m_documents)
        m_block_ext_info.push(SignatureStructureHandler::compactUnlockersArray(doc->get_doc_ext_info()));
      return true;
    }

    */

    //old_name_was exportDocumentsToJSon
    pub fn export_documents_to_json(&self, ext_info_in_document: bool) -> Vec<JSonObject>
    {
        let mut documents: Vec<JSonObject> = vec![];
        for a_doc in &self.m_block_documents {
            documents.push(a_doc.export_doc_to_json(ext_info_in_document));
        }
        return documents;
    }

    //old_name_was exportBlockToJSon
    pub fn export_block_to_json(&self, ext_info_in_document: bool) -> JSonObject
    {
        let mut out: JSonObject = json!({
            "bAncestors": self.m_block_ancestors,
            "bCDate": self.m_block_creation_date,
            "bExtHash": self.m_block_ext_root_hash,
            "bExtInfo": self.m_block_ext_info,
            "bHash": self.m_block_hash,
            "bLen": cutils::padding_length_value(self.m_block_length.to_string(), constants::LEN_PROP_LENGTH),
            "bType": self.m_block_type,
            "bVer": self.m_block_version,
            "bDocs": self.export_documents_to_json(ext_info_in_document),
            "bDocsRootHash": self.m_block_documents_root_hash,
            "bFVotes": self.m_block_floating_votes,
            "bNet": self.m_block_net,
            "bSignals": self.m_block_signals});

        if self.m_block_backer != "" {
            out["bBacker"] = self.m_block_backer.clone().into();
        }

        return out;
    }

    /*

    void Block::calcAndSetBlockLength()
    {
      String stringyfied_block = safe_stringify_block(false);
      m_block_length = static_cast<BlockLenT>(stringyfied_block.len());
    }

    BlockLenT Block::calcBlockLength(const JSonObject& block_obj) const
    {
      return cutils::serializeJson(block_obj).len();
    }

    std::tuple<bool, bool> Block::handleReceivedBlock() const
    {
      return {false, false};
    }

    BlockLenT Block::getMaxBlockSize() const
    {
      return SocietyRules::getMaxBlockSize(m_block_type);
    }

    bool Block::controlBlockLength() const
    {
      String stringyfied_block = safe_stringify_block(false);
      if (static_cast<BlockLenT>(stringyfied_block.len()) != m_block_length)
      {
        CLog::log("Mismatch (Base class)block length Block(" + cutils::hash8c(m_block_hash) + ") local length(" + String::number(stringyfied_block.len()) + ") remote length(" + String::number(m_block_length) + ") stringyfied_block:" + stringyfied_block, "sec", "error");
        return false;
      }
      return true;
    }


    std::tuple<bool, bool> Block::blockGeneralControls() const
    {
      if (m_net != constants::SOCIETY_NAME)
      {
        CLog::log("Invalid society communication! Block(" + cutils::hash8c(m_block_hash) + ") Society(" + m_net + ")", "sec", "error");
        return {false, true};
      }

      // block create date control
      if (cutils::isGreaterThanNow(m_block_creation_date))
      {
        CLog::log("Invalid block creation date! Block(" + cutils::hash8c(m_block_hash) + ") creation date(" + m_block_creation_date + ")", "sec", "error");
        return {false, true};
      }

      if (m_block_length > getMaxBlockSize())
      {
        CLog::log("Invalid block length block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(m_block_length) + " > MAX: " + String::number(getMaxBlockSize()) + ")", "sec", "error");
        return {false, true};
      }

      // Block length control
      if (!controlBlockLength())
        return {false, true};


      if ((m_block_version == "") || !cutils::isValidVersionNumber(m_block_version))
      {
        CLog::log("Invalid bVer block(" + cutils::hash8c(m_block_hash) + ") block(" + m_block_version + ")", "sec", "error");
        return {false, true};
      }

      if (!cutils::isValidDateForamt(m_block_creation_date))
      {
        CLog::log("Invalid creation date block(" + cutils::hash8c(m_block_hash) + ") creation date(" + m_block_creation_date + ")", "sec", "error");
        return {false, true};
      }

      if (cutils::isGreaterThanNow(m_block_creation_date))
      {
        CLog::log("Block whith future creation date is not acceptable(" + m_block_creation_date + ") not Block(" + m_block_hash + ")!", "sec", "error");
        return {false, true};
      }

      // ancestors control
      if (m_ancestors.len() < 1)
      {
        CLog::log("Invalid ancestors for block(" + cutils::hash8c(m_block_hash) + ")", "sec", "error");
        return {false, true};
      }

      if (!cutils::isValidHash(m_block_hash))
      {
        CLog::log("Invalid block Hash(" + cutils::hash8c(m_block_hash) +")", "sec", "error");
        return {false, true};
      }

      // docRootHash control
      if (m_documents.len() > 0)
      {
        StringList doc_hashes {};
        for (Document* a_doc: m_documents)
          doc_hashes.push(a_doc->get_doc_hash());
        auto[root, verifies, version, levels, leaves] = CMerkle::generate(doc_hashes);
        Q_UNUSED(verifies);
        Q_UNUSED(version);
        Q_UNUSED(levels);
        Q_UNUSED(leaves);
        if (m_documents_root_hash != root)
        {
          CLog::log("Mismatch block DocRootHash for type(" + m_block_type + ") block(" + cutils::hash8c(m_block_hash) + ") creation date(" + m_block_creation_date + ")", "sec", "error");
          return {false, true};
        }

        // ext root hash control
        if (m_block_ext_root_hash != "")
        {
          auto[status, block_ext_root_hash] = calcBlockExtRootHash();
          if (!status || (block_ext_root_hash != m_block_ext_root_hash))
          {
            CLog::log("Mismatch block Ext DocRootHash for type(" + m_block_type + ") block(" + cutils::hash8c(m_block_hash) + ") creation date(" + m_block_creation_date + ")", "sec", "error");
            return {false, true};
          }
        }

        // re-calculate block hash
        String re_calc_block_hash = calcBlockHash();
        if (re_calc_block_hash != m_block_hash)
        {
          CLog::log("Mismatch block kHash. localy calculated(" + cutils::hash8c(re_calc_block_hash) +") remote(" + m_block_type + " / " + cutils::hash8c(m_block_hash) + ") " + safe_stringify_block(true), "sec", "error");
          return {false, true};
        }

    //    Block* tmp_block = new Block(JSonObject {
    //      {"bCDate", this.m_block_creation_date},
    //      {"bType", this.m_block_type},
    //      {"bHash", this.m_block_hash}});
        for (Document* a_doc: m_documents)
          if (!a_doc->fullValidate(this).status)
          {
    //        delete tmp_block;
            return {false, true};
          }

    //    delete tmp_block;
      }

      return {true, true};
    }

    String Block::getBlockHash() const
    {
      return m_block_hash;
    }

    */

    pub fn calc_block_hash(&self) -> CBlockHashT {
        if self.m_block_type == constants::block_types::GENESIS {
            return genesis_calc_block_hash(self);
        } else if self.m_block_type == constants::block_types::COINBASE {
            return self.m_if_coinbase_block.calc_block_hash(self);
        }

        panic!("Undefined block type: {}", self.m_block_type);
    }

    pub fn set_block_hash(&mut self, hash: &CBlockHashT)
    {
        self.m_block_hash = hash.parse().unwrap();
    }

    /*

    String Block::getBacker() const
    {
      return m_block_backer;
    }

    std::tuple<bool, String> Block::getBlockSignMsg() const
    {
      // by default blocks have no prt to signing
      return {false, ""};
    }

    String Block::dumpBlock() const
    {
      return "dumpBlock not implemented!";
    }
    */

    //old_name_was addBlockToDAG
    pub fn add_block_to_dag(&self, machine: &mut CMachine) -> (bool, String)
    {
        // duplicate check
        let (_status, records) = q_select(
            C_BLOCKS,
            vec!["b_hash"],     // fields
            vec![
                simple_eq_clause("b_hash", &self.m_block_hash),
            ],
            vec![
                &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
                &OrderModifier { m_field: "b_id", m_order: "ASC" },
            ],
            1,   // limit
            false,
        );
        if records.len() > 0
        { return (true, "Block already existed in DAG".to_string()); }

        // save hard copy of blocks(timestamped by receive date) to have backup
        // in case of curruptions in DAG or bootstrp the DAG, machine doesn't need to download again entire DAG
        // you can simply copy files from ~/backup-dag to folder ~/temporary/inbox
        let dag_backup = application().dag_backup();
        let file_name = application().get_now_sss() + "_" + &*self.m_block_type.clone() + "_" + &*self.m_block_hash.clone() + ".txt";
        let clone_id = application().id();
        if constants::DO_HARDCOPY_DAG_BACKUP {
            file_write(
                dag_backup,
                file_name,
                &self.safe_stringify_block(false),
                clone_id);
        }

        //TODO: implementing atomicity(transactional) either in APP or DB

        // insert into DB
        let confidence_string = cutils::convert_float_to_string(self.m_block_confidence, constants::FLOAT_LENGTH);
        let confidence_float = confidence_string.parse::<f64>().unwrap();
        let signals = cutils::serialize_json(&self.m_block_signals);
        let (_status, _sf_version, body) = wrap_safe_content_for_db(&self.safe_stringify_block(false), constants::WRAP_SAFE_CONTENT_VERSION);
        let docs_count = self.m_block_documents.len() as i32;
        let ancestors = self.m_block_ancestors.join(",");
        let ancestors_count = self.m_block_ancestors.len() as i32;
        let descendants = self.m_block_descendants.join(",");
        let cycle = application().get_coinbase_cycle_stamp(&self.m_block_creation_date);
        let b_trxs_count = 0;
        let b_receive_date = application().get_now();
        let b_confirm_date = application().get_now();
        let b_utxo_imported = constants::NO.to_string();

        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("b_hash", &self.m_block_hash as &(dyn ToSql + Sync)),
            ("b_type", &self.m_block_type as &(dyn ToSql + Sync)),
            ("b_confidence", &confidence_float as &(dyn ToSql + Sync)),
            ("b_body", &body as &(dyn ToSql + Sync)),
            ("b_docs_root_hash", &self.m_block_documents_root_hash as &(dyn ToSql + Sync)),
            ("b_ext_root_hash", &self.m_block_ext_root_hash as &(dyn ToSql + Sync)),
            ("b_signals", &signals as &(dyn ToSql + Sync)),
            ("b_trxs_count", &b_trxs_count as &(dyn ToSql + Sync)),
            ("b_docs_count", &docs_count as &(dyn ToSql + Sync)),
            ("b_ancestors", &ancestors as &(dyn ToSql + Sync)),
            ("b_ancestors_count", &ancestors_count as &(dyn ToSql + Sync)),
            ("b_descendants", &descendants as &(dyn ToSql + Sync)),
            ("b_creation_date", &self.m_block_creation_date as &(dyn ToSql + Sync)),
            ("b_receive_date", &self.m_block_receive_date as &(dyn ToSql + Sync)),
            ("b_confirm_date", &self.m_block_confirm_date as &(dyn ToSql + Sync)),
            ("b_cycle", &cycle as &(dyn ToSql + Sync)),
            ("b_backer", &self.m_block_backer as &(dyn ToSql + Sync)),
            ("b_receive_date", &b_receive_date as &(dyn ToSql + Sync)),
            ("b_confirm_date", &b_confirm_date as &(dyn ToSql + Sync)),
            ("b_utxo_imported", &b_utxo_imported as &(dyn ToSql + Sync))]);

        dlog(
            &format!("--- recording block in DAG Block({})", cutils::hash8c(&self.m_block_hash)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        q_insert(
            C_BLOCKS,     // table
            &values, // values to insert
            true);

        // add newly recorded block to cache in order to reduce DB load. TODO: improve it
        update_cached_blocks(
            machine,
            &self.m_block_type,
            &self.m_block_hash,
            &self.m_block_creation_date,
            &constants::NO.to_string());

        // recording block ext Info (if exist)
        let block_ext_info: String = self.stringify_block_ext_info();
        if block_ext_info != "" {
            self.insert_block_ext_info_to_db(&block_ext_info, &self.m_block_hash, &self.m_block_creation_date);
        }

        // adjusting leave blocks
        remove_from_leave_blocks(&self.m_block_ancestors);
        add_to_leave_blocks(&self.m_block_hash, &self.m_block_creation_date, &self.m_block_type);

        // insert block signals
        log_signals(&self);

        if self.m_block_documents.len() > 0
        {
            for doc_inx in 0..self.m_block_documents.len()
            {
                //FIXME: implement suspicious docs filtering!

                let a_doc: &Document = &self.m_block_documents[doc_inx];
                a_doc.apply_doc_first_impact(self);

                // connect documents and blocks
                a_doc.map_doc_to_block(&self.m_block_hash, doc_inx as CDocIndexT);
            }
        }

        // update ancestor's descendent info
        append_descendants(&self.m_block_ancestors, &vec![self.m_block_hash.clone()]);

        // sceptical_dag_integrity_controls
        let (status, _msg) = controls_of_new_block_insertion(&self.m_block_hash);
        if !status
        {
            dlog(
                &format!("Error in sceptical Data Integrity Check: block({}) ", cutils::hash8c(&self.m_block_hash)),
                constants::Modules::App,
                constants::SecLevel::Info);

            return (false, "Error in sceptical Data Integrity Check".to_string());
        }

        #[allow(unused_doc_comments)]
        /**
        {
            // TODO: remove this block(variable/mechanism) after fixing sqlite database lock problem
            if (CMachine::get().m_recorded_blocks_in_db == 0)
            {
                QueryRes
                res = DbModel::customQuery(
                    "db_comen_blocks",
                    "SELECT COUNT(*) AS count_blocks FROM c_blocks",
                    { "count_blocks" },
                    0,
                    {},
                    false,
                    true);
                CMachine::get().m_recorded_blocks_in_db = res.records[0].value("count_blocks").toInt();
            } else {
                CMachine::get().m_recorded_blocks_in_db + +;
            }
        }
         */
        return (true, "block was added to DAG".to_string());
    }

    /*

    bool Block::postAddBlockToDAG() const
    {
      // remove perequisity, if any block in parsing Q was needed to this block

      ParsingQHandler::removePrerequisites(m_block_hash);

      /**
      * sometimes (e.g. repayback blocks which can be created by delay and causing to add block to missed blocks)
      * we need to doublecheck if the block still is in missed blocks list and remove it
      */
      MissedBlocksHandler::removeFromMissedBlocks(getBlockHash());

      /**
      * inherit UTXO visibilities of ancestors of newly DAG-added block
      * current block inherits the visibility of it's ancestors
      * possibly first level ancestors can be floating signatures(which haven't entry in table trx_utxos),
      * so add ancestors of ancestors too, in order to being sure we keep good and reliable history in utxos
      */
      if (!StringList {constants::BLOCK_TYPES::FSign, constants::BLOCK_TYPES::FVote}.contains(m_block_type))
      {
        StringList ancestors = m_ancestors;
        ancestors = cutils::arrayAdd(ancestors, DAG::getAncestors(ancestors));
        ancestors = cutils::arrayAdd(ancestors, DAG::getAncestors(ancestors));
        ancestors = cutils::arrayUnique(ancestors);
        UTXOHandler::inheritAncestorsVisbility(
          ancestors,
          m_block_creation_date,
          getBlockHash());
      }

      return true;
    }



    bool Block::createDocuments(const QJsonValue& documents)
    {
      JSonArray docs = documents.toArray();
      for(CDocIndexT doc_inx = 0; doc_inx < static_cast<CDocIndexT>(docs.len()); doc_inx++)
      {
        Document* d = DocumentFactory::create(docs[doc_inx].toObject(), this, doc_inx);
        m_documents.push(d);
      }
      return true;
    }

    /**
     * @brief Block::getBlockExtInfo
     * @param blockHash
     * @return <status, extInfoExist?, extinfoJsonObj>
     */
    std::tuple<bool, bool, JSonArray> Block::getBlockExtInfo(const String& block_hash)
    {
      JSonArray block_ext_info;
      QueryRes res = DbModel::select(
        stbl_block_extinfos,
        {"x_block_hash", "x_detail"},
        {{"x_block_hash", block_hash}});
      if (res.records.len() != 1)
      {
        CLog::log("get Block Ext Infos: the block(" + cutils::hash8c(block_hash) + ") has not ext Info", "app", "trace");
        return {true, false, {}};
      }

      String serialized_block = BlockUtils::unwrapSafeContentForDB(res.records[0].value("x_detail").to_string()).content;
      block_ext_info = cutils::parseToJsonArr(serialized_block);
      return {true, true, block_ext_info};
    }

    std::tuple<bool, bool, JSonArray> Block::getBlockExtInfo() const
    {
      if (m_block_ext_info.len() > 0)
        return {true, true, m_block_ext_info[0].toArray()};

      return Block::getBlockExtInfo(m_block_hash);
    }

    bool Block::appendToDocuments(Document* doc)
    {
      m_documents.push(doc);
      return true;
    }

    bool Block::appendToExtInfo(const JSonArray& an_ext_info)
    {
      m_block_ext_info.push(an_ext_info);
      return true;
    }

*/
    //old_name_was getBlockExtInfoByDocIndex
    pub fn get_block_ext_info_by_doc_index(&self, document_index: usize) -> &Vec<JSonObject>
    {
        return &self.m_block_ext_info[document_index];
    }

    //old_name_was searchInBlockExtInfo
    pub fn search_in_block_ext_info(
        clauses: ClausesT,
        fields: Vec<&str>,
        order: OrderT,
        limit: u32) -> QVDRecordsT
    {
        let (_status, records) = q_select(
            C_BLOCK_EXTINFOS,
            fields,
            clauses,
            order,
            limit,
            false);
        return records;
    }
    /*

        /**
         *
         * @param {*} args
         * do a groupping and some general validations on entire documents of a Block
         * TODO: maybe enhance it to use memory buffer
         */

        TransientBlockInfo Block::groupDocsOfBlock(const String& stage) const
        {
          TransientBlockInfo transient_block_info {false, this, stage};


          if (m_documents.len() == 0)
            return transient_block_info;

          String now_ = application().get_now();
          for (CDocIndexT doc_inx = 0; doc_inx < m_documents.len(); doc_inx++)
          {
            Document *a_doc = m_documents[doc_inx];
            transient_block_info.m_doc_by_hash[a_doc->get_doc_hash()] = a_doc;

            if ((a_doc.m_doc_creation_date > m_block_creation_date) || (a_doc.m_doc_creation_date > now_))
            {
              CLog::log("Block has document with creationdate after block-creation Date! stage(" + stage + "), block(" + cutils::hash8c(m_block_hash) + ") Doc(" + cutils::hash8c(a_doc->get_doc_hash()) + ")", "app", "error");
              return transient_block_info;
            }
            transient_block_info.m_doc_index_by_hash[a_doc->get_doc_hash()] = doc_inx;

            if (!cutils::isValidVersionNumber(a_doc.m_doc_version))
            {
              CLog::log("invalid dVer in group Docs Of Block stage(" + stage + ") for doc(" + cutils::hash8c(a_doc->get_doc_hash()) + ")", "sec", "error");
              return transient_block_info;
            }


            // document length control
            Document *tmp_doc = DocumentFactory::create(a_doc->export_doc_to_json), this, doc_inx);

            DocLenT recalculate_doc_length = static_cast<DocLenT>(tmp_doc->calc_doc_length());
            if ((tmp_doc.m_doc_type != constants::DOC_TYPES::DPCostPay) &&
                ((tmp_doc.m_doc_length != recalculate_doc_length) ||
                 (tmp_doc.m_doc_length != a_doc.m_doc_length))
                )
            {
              String msg = "The doc stated dLen is not same as real length. stage(" + stage + ") doc type(" + tmp_doc.m_doc_type + "/" + cutils::hash8c(tmp_doc->get_doc_hash()) + ") stated dLen(" + String::number(a_doc.m_doc_length) + "), ";
              msg += " real length(" + String::number(tmp_doc->calc_doc_length()) + ")";
              CLog::log(msg, "sec", "error");

              delete tmp_doc;

              return transient_block_info;
            }
            delete tmp_doc;


            if (!transient_block_info.m_groupped_documents.keys().contains(a_doc.m_doc_type))
              transient_block_info.m_groupped_documents[a_doc.m_doc_type] = {};

            transient_block_info.m_groupped_documents[a_doc.m_doc_type].push(a_doc);

            if (a_doc->get_ref() != "")
            {
              if (Document::canBeACostPayerDoc(a_doc.m_doc_type))
              {
                transient_block_info.m_transactions_dict[a_doc->get_doc_hash()] = a_doc;
                transient_block_info.m_map_trx_hash_to_trx_ref[a_doc->get_doc_hash()] = a_doc->get_ref();
                transient_block_info.m_map_trx_ref_to_trx_hash[a_doc->get_ref()] = a_doc->get_doc_hash();
              } else {
                transient_block_info.m_map_referencer_to_referenced[a_doc->get_doc_hash()] = a_doc->get_ref();
                transient_block_info.m_map_referenced_to_referencer[a_doc->get_ref()] = a_doc->get_doc_hash();
              }
            }
          }

          StringList payedRefs1 = transient_block_info.m_map_trx_ref_to_trx_hash.keys();
          StringList payedRefs2;
          for(String key: transient_block_info.m_map_trx_hash_to_trx_ref.keys())
            payedRefs2.push(transient_block_info.m_map_trx_hash_to_trx_ref[key]);

          for (Document* a_doc: m_documents)
          {
            if (!Document::isNoNeedCostPayerDoc(a_doc.m_doc_type))
            {
              // there must be a transaction to pay for this document
              if (!payedRefs1.contains(a_doc->get_doc_hash()) || !payedRefs2.contains(a_doc->get_doc_hash()))
              {
                if ((a_doc.m_doc_type == constants::DOC_TYPES::FPost) && (a_doc.m_doc_class == constants::FPOST_CLASSES::DMS_Post))
                {
                  if ((getNonce() == "") || (m_block_creation_date > "2021-01-01 00:00:00"))
                  {
                    CLog::log("The document DMS_Post has not Nonce & not payed by no transaction. stage(" + stage + ") document(" + cutils::hash8c(a_doc->get_doc_hash()) + ") ", "sec", "error");
                    return transient_block_info;
                  }
                } else {
                  CLog::log("The document is not payed by no transaction. stage(" + stage + ") document(" + cutils::hash8c(a_doc->get_doc_hash()) + ") ", "sec", "error");
                  return transient_block_info;
                }
              }
            }
          }

          if (transient_block_info.m_map_trx_ref_to_trx_hash.keys().len() != transient_block_info.m_map_trx_hash_to_trx_ref.keys().len())
          {
            CLog::log("transaction count and ref count are different! stage(" + stage + ") mapTrxRefToTrxHash: " + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash) + " mapTrxHashToTrxRef: " + cutils::dumpIt(transient_block_info.m_map_trx_hash_to_trx_ref) + " ", "sec", "error");
            return transient_block_info;
          }

          for (String a_ref: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
          {
            if (!transient_block_info.m_transactions_dict.keys().contains(transient_block_info.m_map_trx_ref_to_trx_hash[a_ref]))
            {
              CLog::log("missed some1 transaction to support referenced documents. stage(" + stage + ") trxDict: " + cutils::dumpIt(transient_block_info.m_transactions_dict) + " mapTrxRefToTrxHash: " + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash) + " ", "sec", "error");
              return transient_block_info;
            }
          }
          if (cutils::arrayDiff (transient_block_info.m_map_trx_hash_to_trx_ref.keys(), transient_block_info.m_transactions_dict.keys()).len() != 0)
          {
            CLog::log("missed some transaction, to support referenced documents. stage(" + stage + ") trxDict: " + cutils::dumpIt(transient_block_info.m_transactions_dict) + " mapTrxRefToTrxHash: " + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash) + " ", "sec", "error");
            return transient_block_info;
          }
          for (String a_ref: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
          {
            if (!transient_block_info.m_doc_by_hash.keys().contains(a_ref))
            {
              CLog::log("missed a referenced document. stage(" + stage + ") referenced doc(" + cutils::hash8c(a_ref) + "), referencer doc(" + cutils::hash8c(transient_block_info.m_map_trx_ref_to_trx_hash[a_ref]) + ") ", "sec", "error");
              return transient_block_info;
            }
          }

          if (static_cast<uint32_t>(transient_block_info.m_doc_index_by_hash.keys().len()) != static_cast<uint32_t>(m_documents.len()))
          {
            CLog::log("There is duplicated doc.hash in block. stage(" + stage + ") block(" + cutils::hash8c(m_block_hash) + ") ", "sec", "error");
            return transient_block_info;
          }

          StringList doc_types = transient_block_info.m_groupped_documents.keys();
          doc_types.sort();
          for(String a_type: doc_types)
            CLog::log("block(" + cutils::hash8c(m_block_hash) + ") has " + String::number(transient_block_info.m_groupped_documents[a_type].len()) + " Document(s) of type(" + a_type + ") ", "app", "trace");

          transient_block_info.m_status = true;
          return transient_block_info;

        //  {
        //    true,
        //    trxDict,
        //    docByHash,
        //    grpdDocuments,
        //    docIndexByHash,
        //    mapTrxHashToTrxRef,
        //    mapTrxRefToTrxHash,
        //    mapReferencedToReferencer,
        //    mapReferencerToReferenced
        //  };
        }


        String Block::getNonce() const
        {
          cutils::exiter("m_nonce isn't implement for Block Base class", 0);
          return "";
        }

        std::vector<Document *> Block::getDocuments() const
        {
          return m_documents;
        }

        //std::tuple<bool, JSonObject> Block::selectBExtInfosFromDB(const String& block_hash)
        //{
        //  QueryRes bExtInfo = DbModel::select(
        //    "db_comen_blocks",
        //    stbl_block_extinfos,
        //    StringList {"x_block_hash", "x_detail"},     // fields
        //    {ModelClause("x_block_hash", block_hash)},
        //    {},
        //    1   // limit
        //  );
        //  if (bExtInfo.records.len() == 0)
        //    return {false, JSonObject {}};

        //  QVariant x_detail = bExtInfo.records[0].value("x_detail");
        //  String serializedBextInfo = x_detail.to_string();
        //  auto[unwrap_status, unwrap_ver, unwrap_content] = BlockUtils::unwrapSafeContentForDB(serializedBextInfo);

        //  JSonObject JsonObj = cutils::parseToJsonObj(unwrap_content);

        //  return {true, JsonObj};

        //}

        //std::tuple<bool, JSonObject> Block::selectBExtInfosFromDB() const
        //{
        //  return selectBExtInfosFromDB(m_block_hash);
        //}

    */
    //old_name_was insertToDB
    //old_name_was insertBlockExtInfoToDB
    pub fn insert_block_ext_info_to_db(
        &self,
        serialized_block_ext_info: &String,
        block_hash: &CBlockHashT,
        creation_date: &CDateT) -> bool
    {
        let (_status, _sf_version, the_safe_content) = wrap_safe_content_for_db(
            serialized_block_ext_info,
            constants::WRAP_SAFE_CONTENT_VERSION);
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("x_block_hash", &block_hash as &(dyn ToSql + Sync)),
            ("x_detail", &the_safe_content as &(dyn ToSql + Sync)),
            ("x_creation_date", &creation_date as &(dyn ToSql + Sync))
        ]);
        dlog(
            &format!("--- recording bExtInfo in DAG Block({})", cutils::hash8c(block_hash)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);


        return q_insert(
            C_BLOCK_EXTINFOS,     // table
            &values, // values to insert
            true);
    }
    /*

        /**
         * picks a doc from a block which is complatly loaded in memory
         * @param {*} block
         * @param {*} doc_hash
         * return {doc_index, doc_pointer}
         *
         */
        std::tuple<int64_t, Document*> Block::getDocumentByHash(const CDocHashT& doc_hash) const
        {
          auto documents = getDocuments();
          for (CDocIndexT doc_inx = 0; doc_inx < documents.len(); doc_inx++)
          {
            if (documents[doc_inx].m_doc_hash == doc_hash)
              return {doc_inx, documents[doc_inx]};
          }
          return {-1, nullptr};
        }

        std::tuple<int64_t, JSonObject> Block::getDocumentJByHash(
          const JSonObject& block,
          const CDocHashT& doc_hash)
        {
          JSonArray documents = block.value("docs").toArray();
          for (CDocIndexT doc_inx = 0; doc_inx < documents.len(); doc_inx++)
          {
            if (documents[doc_inx].toObject().value("dHash").to_string() == doc_hash)
              return {doc_inx, documents[doc_inx].toObject()};
          }
          return {-1, {}};
        }

        /**
        *
        * @param {string} blockHash
        * static retrieves complete version of recorded block in 2 different tables i_blocks & i_block_segwits
        */
        std::tuple<bool, JSonObject> Block::regenerateBlock(const CBlockHashT& block_hash)
        {
          JSonObject Jblock {};
          //listener.doCallSync('SPSH_before_regenerate_block', { block_hash });
          QVDRecordsT block_records = DAG::searchInDAG({{"b_hash", block_hash}});
          if (block_records.len() == 0)
          {
            // TODO: the block is valid and does not exist in local. or
            // invalid block invoked, maybe some penal for sender!
            CLog::log("The requested block to regenrate doesn't exist in DAG! Block("+ cutils::hash8c(block_hash) + ") ", "app", "warning");
            return {false, Jblock};
          }

          try {
            QVDicT the_block = block_records[0];
            Jblock = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(the_block.value("b_body").to_string()).content);

            auto[status, extExist, block_ext_info] = getBlockExtInfo(block_hash);
            if (extExist)
              Jblock["bExtInfo"] = block_ext_info;

            return {true, Jblock};

          } catch (std::exception) {
            return {false, Jblock};
          }

        }

        */
}
