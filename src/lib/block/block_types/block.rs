use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use crate::{application, ccrypto, constants, cutils, dlog};
use crate::cmerkle::{generate_m, MERKLE_VERSION};
use crate::cutils::{controlled_str_to_json, remove_quotes};
use crate::lib::block::block_types::block_coinbase::coinbase_block::CoinbaseBlock;
use crate::lib::block::block_types::block_genesis::genesis_block::b_genesis::{genesis_calc_block_hash};
use crate::lib::block::block_types::block_normal::normal_block::NormalBlock;
use crate::lib::block::block_types::block_repayback::repayback_block::RepaybackBlock;
use crate::lib::block::document_types::document::Document;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::block::document_types::floating_vote_document::FloatingVoteDocument;
use crate::lib::block_utils::{unwrap_safed_content_for_db, wrap_safe_content_for_db};
use crate::lib::custom_types::{BlockLenT, CBlockHashT, CDateT, CDocIndexT, ClausesT, JSonObject, OrderT, QVDRecordsT, QSDicT, CDocHashT, DocDicVecT, CMPAISValueT, VString, DocLenT};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::{q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_BLOCK_EXT_INFO};
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::services::society_rules::society_rules::get_max_block_size;
use crate::lib::utils::version_handler::is_valid_version_number;

#[derive(Debug)]
pub struct TransientBlockInfo
{
    pub m_status: bool,
    pub m_block: Block,
    pub m_stage: String,

    pub m_map_trx_hash_to_trx_ref: QSDicT,
    pub m_map_trx_ref_to_trx_hash: QSDicT,
    pub m_map_referencer_to_referenced: QSDicT,
    pub m_map_referenced_to_referencer: QSDicT,

    pub m_doc_by_hash: HashMap<String, Document>,
    pub m_transactions_dict: HashMap<String, Document>,
    pub m_grouped_documents: DocDicVecT,
    pub m_doc_index_by_hash: HashMap<CDocHashT, CDocIndexT>,

    pub m_block_total_output: CMPAISValueT,
    pub m_block_documents_hashes: VString,
    pub m_block_ext_info_hashes: VString,
    pub m_pre_requisites_ancestors: VString, // in case of creating a block which contains some ballots, the block explicitely includes the related polling blocks, in order to force and asure existance of polling recorded in DAG, befor applying the ballot(s)
}

impl TransientBlockInfo {
    pub fn new() -> Self {
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
            m_grouped_documents: HashMap::new(),
            m_doc_index_by_hash: HashMap::new(),
            m_block_total_output: 0,
            m_block_documents_hashes: vec![],
            m_block_ext_info_hashes: vec![],
            m_pre_requisites_ancestors: vec![],
        }
    }
}

#[allow(unused, dead_code)]
pub struct BlockApprovedDocument
{
    m_approved_doc: Document,
    m_approved_doc_hash: String,
    m_approved_doc_ext_info: Vec<DocExtInfo>,
    m_approved_doc_ext_hash: String,
}

impl BlockApprovedDocument {
    #[allow(unused, dead_code)]
    pub fn new() -> Self {
        BlockApprovedDocument {
            m_approved_doc: Document::new(),
            m_approved_doc_hash: "".to_string(),
            m_approved_doc_ext_info: vec![],
            m_approved_doc_ext_hash: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block
{
    pub m_block_net: String,
    pub m_block_length: BlockLenT,
    pub m_block_hash: String,
    pub m_block_type: String,
    pub m_block_version: String,
    pub m_block_ancestors: VString,
    pub m_block_descendants: VString,
    pub m_block_signals: Vec<JSonObject>,
    pub m_block_backer: String,
    pub m_block_confidence: f64,
    pub m_block_creation_date: String,
    pub m_block_receive_date: String,
    pub m_block_confirm_date: String,
    pub m_block_descriptions: String,
    pub m_block_documents_root_hash: String,
    pub m_block_ext_root_hash: String,
    pub m_block_documents: Vec<Document>,
    pub m_block_ext_info: Vec<Vec<DocExtInfo>>,
    pub m_block_floating_votes: Vec<FloatingVoteDocument>, // TODO: to be implemented later

    pub m_if_coinbase_block: CoinbaseBlock,
    pub m_if_normal_block: NormalBlock,
    pub m_if_repayback_block: RepaybackBlock,
}

impl Block {
    pub fn new() -> Self {
        Block {
            m_block_net: constants::SOCIETY_NAME.to_string(),
            m_block_descriptions: "".to_string(),
            m_block_length: 0,
            m_block_hash: "".to_string(),
            m_block_type: "".to_string(),
            m_block_version: constants::DEFAULT_BLOCK_VERSION.to_string(),
            m_block_ancestors: vec![],
            m_block_descendants: vec![],
            m_block_signals: vec![],
            m_block_backer: "".to_string(),
            m_block_confidence: 0.0,
            m_block_creation_date: "".to_string(),
            m_block_receive_date: "".to_string(),
            m_block_confirm_date: "".to_string(),
            m_block_documents: vec![],
            m_block_documents_root_hash: "".to_string(),
            m_block_ext_info: vec![],
            m_block_ext_root_hash: "".to_string(),
            m_block_floating_votes: vec![],

            m_if_coinbase_block: CoinbaseBlock::new(),
            m_if_normal_block: NormalBlock::new(),
            m_if_repayback_block: RepaybackBlock::new(),
        }
    }

    // old name was getDocuments
    pub fn get_documents(&self) -> &Vec<Document>
    {
        return &self.m_block_documents;
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
      if (values["b_id"] "").to_string() != "")
        m_id = values["b_id"] "").toUInt();
      if (values["b_hash"] "").to_string() != "")
        m_hash = values["b_hash"] "").to_string();
      if (values["b_type"] "").to_string() != "")
        m_type = values["b_type"] "").to_string();
      if (values["b_cycle"] "").to_string() != "")
        m_cycle = values["b_cycle"] "").to_string();

      if (values["b_confidence"] "").to_string() != "")
        m_confidence = values["b_confidence"] "").toFloat();

      if (values["b_ext_root_hash"] "").to_string() != "")
        m_ext_root_hash = values["b_ext_root_hash"] "").to_string();
      if (values["b_docs_root_hash"] "").to_string() != "")
        m_documents_root_hash = values["b_docs_root_hash"] "").to_string();
      if (values["b_signals"] "").to_string() != "")
        m_signals = values["b_signals"] "").to_string();
      if (values["b_trxs_count"] "").to_string() != "")
        m_trxs_count = values["b_trxs_count"] "").toUInt();
      if (values["b_docs_count"] "").to_string() != "")
        m_docs_count = values["b_docs_count"] "").toUInt();

      if (values["b_ancestors_count"] "").to_string() != "")
        m_ancestors_count = values["b_ancestors_count"] "").toUInt();

      if (values["b_ancestors"] "").to_string() != "")
        m_ancestors = values["b_ancestors"] "").to_string().split(",");

      if (values["b_descendants"] "").to_string() != "")
        m_descendents = values["b_descendants"] "").to_string().split(",");

      if (values["b_body"] "").to_string() != "")
        m_body = values["b_body"] "").to_string();
      if (values["b_creation_date"] "").to_string() != "")
        m_creation_date = values["b_creation_date"] "").to_string();
      if (values["b_receive_date"] "").to_string() != "")
        m_receive_date = values["b_receive_date"] "").to_string();
      if (values["b_confirm_date"] "").to_string() != "")
        m_confirm_date = values["b_confirm_date"] "").to_string();
      if (values["b_backer"] "").to_string() != "")
        m_block_backer = values["b_backer"] "").to_string();
      if (values["b_coins_imported"] "").to_string() != "")
        m_coin_imported = values["b_coins_imported"] "").to_string();
      return true;
    }

    //  -  -  -  Block

    Block::~Block()
    {
      // delete documents
      for(Document* d: m_documents)
        delete d;
    }
     */


    // * the tests to avoid injection/maleformed data in received Jsons
    // * FIXME: add more tests
    // old name was objectAssignmentsControlls
    pub fn object_assignments_controls(&self) -> bool
    {
        if !cutils::is_valid_hash(&self.m_block_hash)
        {
            dlog(
                &format!("Invalid blockHash after js assignment: {}", self.m_block_hash),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }

        if (self.m_block_type != constants::block_types::COINBASE)
            && !ccrypto::is_valid_bech32(&self.m_block_backer)
        {
            dlog(
                &format!("Invalid block backer after js assignment: {}", self.m_block_backer),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }


        return true;
    }

    //old_name_was stringifyBExtInfo
    pub fn stringify_block_ext_info(&self) -> String
    {
        if !vec![constants::block_types::COINBASE,
                 constants::block_types::NORMAL,
                 constants::block_types::POW].contains(&self.m_block_type.as_str())
        {
            dlog(
                &format!("DUMMY BREAKPOINT LOG :)"),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }
        let mut block_ext_info: Vec<Vec<DocExtInfo>> = vec![];
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
            cutils::controlled_json_stringify(&j_block).len().to_string(),
            constants::LEN_PROP_LENGTH).into();

        let out = cutils::controlled_json_stringify(&j_block);
        dlog(
            &format!(
                "Safe stringified block(Base class) Block({}) length({}) the block: {}",
                cutils::hash8c(&self.m_block_hash), out.len(), out),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return out;
    }

    pub fn get_docs_count(&self) -> CDocIndexT
    {
        self.m_block_documents.len() as CDocIndexT
    }

    //old name was getDocumentsHashes
    pub fn get_documents_hashes(&self) -> VString
    {
        let mut hashes: VString = vec![];
        for a_doc in &self.m_block_documents
        {
            hashes.push(a_doc.get_doc_hash());
        }
        return hashes;
    }

    #[allow(unused, dead_code)]
    pub fn js_get_documents_hashes(block: &JSonObject) -> VString
    {
        let mut hashes: VString = vec![];
        for a_doc in block["bDocs"].as_array().unwrap()
        {
            hashes.push(remove_quotes(&a_doc["dHash"]));
        }
        return hashes;
    }

    // old name was calcDocumentsRootHash
    pub fn calc_documents_root_hash(&self) -> (bool, CDocHashT)
    {
        let (
            root,
            _verifies,
            _merkle_version,
            _levels,
            _leaves) =
            generate_m(
                self.get_documents_hashes(),
                &"hashed".to_string(),
                &"keccak256".to_string(),
                &MERKLE_VERSION.to_string());
        return (true, root);
    }

    //old_name_was fillInBlockExtInfo
    pub fn fill_in_block_ext_info(&mut self) -> bool
    {
        self.m_block_ext_info = vec![];
        for doc in &self.m_block_documents
        {
            self.m_block_ext_info.push(doc.get_doc_ext_info().clone());
            // self.m_block_ext_info.push(compactUnlockersArray(doc.get_doc_ext_info()));
        }
        return true;
    }

    //old_name_was exportDocumentsToJSon
    pub fn export_documents_to_json(&self, ext_info_in_document: bool) -> Vec<JSonObject>
    {
        let mut documents: Vec<JSonObject> = vec![];
        for a_doc in &self.m_block_documents {
            let mut j_doc = a_doc.export_doc_to_json(ext_info_in_document);
            j_doc["dLen"] = cutils::padding_length_value(
                a_doc.calc_doc_length().to_string(),
                constants::LEN_PROP_LENGTH)
                .into();
            documents.push(j_doc);
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

        if self.m_block_type == constants::block_types::COINBASE
        {
            out = self.m_if_coinbase_block.export_block_to_json(
                self,
                &mut out,
                ext_info_in_document);
        }
        return out;
    }

    // old name was calcBlockExtRootHash
    pub fn calc_block_ext_root_hash_super(&self) -> (bool, String)
    {
        // for POW blocks the block has only one document and the dExtHash of doc and bExtHash of block are equal
        let mut doc_ext_hashes: VString = vec![];
        for a_doc in &self.m_block_documents
        { doc_ext_hashes.push(a_doc.get_doc_ext_hash()); }
        let (
            documents_exts_root_hash,
            _final_verifies,
            _version,
            _levels,
            _leaves) = generate_m(
            doc_ext_hashes,
            &"hashed".to_string(),
            &"keccak256".to_string(),
            &MERKLE_VERSION.to_string());
        return (true, documents_exts_root_hash);
    }

    // old name was calcBlockExtRootHash
    pub fn calc_block_ext_root_hash(&self) -> (bool, CDocHashT)
    {
        if self.m_block_type == constants::block_types::NORMAL
        {
            return self.calc_block_ext_root_hash_super();
        } else if self.m_block_type == constants::block_types::COINBASE
        {
            return self.m_if_coinbase_block.calc_block_ext_root_hash(self);
        }

        panic!("Missed 'calc Block Ext Root Hash method in {}", self.get_block_identifier());
    }

    // old_name_was calcAndSetBlockLength
    pub fn calc_and_set_block_length(&mut self)
    {
        let stringyfied_block: String = self.safe_stringify_block(false);
        self.m_block_length = stringyfied_block.len() as BlockLenT;
    }

    // old_name_was calcBlockLength
    pub fn calc_block_length(&self) -> BlockLenT
    {
        let serialized_block = self.safe_stringify_block(true);
        let (_status, deser_block) = cutils::controlled_str_to_json(&serialized_block);
        let block_length = remove_quotes(&deser_block["bLen"]).to_string().parse::<BlockLenT>().unwrap();
        block_length
    }

    pub fn get_ancestors(&self) -> VString
    {
        self.m_block_ancestors.clone()
    }

    pub fn get_block_identifier(&self) -> String
    {
        let block_identifier = format!(
            " block({} {}) ",
            self.m_block_type,
            cutils::hash8c(&self.m_block_hash));
        return block_identifier;
    }

    pub fn get_js_block_identifier(j_block: &JSonObject) -> String
    {
        let block_identifier = format!(
            " block({} {}) ",
            remove_quotes(&j_block["bType"]),
            cutils::hash8c(&remove_quotes(&j_block["bHash"])));
        return block_identifier;
    }

    pub fn handle_received_block(&self) -> EntryParsingResult
    {
        if self.m_block_type == constants::block_types::COINBASE
        {
            return self.m_if_coinbase_block.handle_received_block(&self);
        }

        return self.handle_received_block_super();
    }

    pub fn handle_received_block_super(&self) -> EntryParsingResult
    {
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: false,
            m_message: format!("This method not implemented for Base Class {}", self.get_block_identifier()),
        };
    }

    #[allow(unused, dead_code)]
    pub fn get_max_block_size(&self) -> BlockLenT
    {
        return get_max_block_size(&self.m_block_type);
    }

    pub fn control_block_length(&self) -> bool
    {
        if self.m_block_type == constants::block_types::COINBASE
        {
            return self.m_if_coinbase_block.control_block_length(self);
        }

        return self.control_block_length_parent();
    }

    pub fn control_block_length_parent(&self) -> bool
    {
        let stringifyed_block: String = self.safe_stringify_block(true);
        if stringifyed_block.len() != self.m_block_length

        {
            dlog(
                &format!("Mismatch (Base class)block length Block({}) local length({}) remote length({}) stringyfied block: {}",
                         cutils::hash8c(&self.m_block_hash), stringifyed_block.len(), self.m_block_length, stringifyed_block),
                constants::Modules::Sec,
                constants::SecLevel::Error);

            return false;
        }
        return true;
    }


    pub fn get_creation_date(&self) -> String
    {
        return self.m_block_creation_date.clone();
    }

    pub fn get_block_hash(&self) -> String
    {
        return self.m_block_hash.clone();
    }

    pub fn get_block_type(&self) -> String
    {
        return self.m_block_type.clone();
    }

    //old_name_was calcBlockHash
    pub fn calc_block_hash(&self) -> CBlockHashT {
        if self.m_block_type == constants::block_types::NORMAL {
            return self.m_if_normal_block.calc_block_hash(self);
        } else if self.m_block_type == constants::block_types::COINBASE {
            return self.m_if_coinbase_block.calc_block_hash(self);
        } else if self.m_block_type == constants::block_types::GENESIS {
            return genesis_calc_block_hash(self);
        }

        panic!("Undefined block type in (calc block hash): {}", self.m_block_type);
    }

    //old name was setBlockHash
    pub fn set_block_hash(&mut self, hash: &CBlockHashT)
    {
        self.m_block_hash = hash.clone();
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

    // old name was createDocuments
    pub fn create_block_documents(&mut self, documents: &Vec<JSonObject>) -> bool
    {
        // JSonArray docs = documents.toArray();
        for doc_inx in 0..documents.len() as CDocIndexT
        {
            let (status, doc) = Document::load_document(
                &documents[doc_inx as usize],
                self,
                doc_inx);
            if !status
            {
                dlog(
                    &format!(
                        "Loading block documents failed {} {} doc body: {}",
                        self.get_block_identifier(),
                        doc.get_doc_identifier(),
                        cutils::controlled_json_stringify(&documents[doc_inx as usize])),
                    constants::Modules::App,
                    constants::SecLevel::TmpDebug);
                return false;
            }
            self.m_block_documents.push(doc);
        }
        return true;
    }

    #[allow(unused, dead_code)]
    pub fn get_block_ext_info(&self) ->
    (
        bool/* status */,
        bool/* block_has_ext_info */,
        Vec<DocExtInfo>/* block_ext_info */)
    {
        if self.m_block_ext_info.len() > 0
        {
            return (true, true, self.m_block_ext_info[0].clone());
        }

        return get_block_ext_info(&self.m_block_hash);
    }
    /*
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
    pub fn get_block_ext_info_by_doc_index(&self, document_index: usize) -> &Vec<DocExtInfo>
    {
        return &self.m_block_ext_info[document_index];
    }

    //old_name_was searchInBlockExtInfo
    #[allow(unused, dead_code)]
    pub fn search_in_block_ext_info(
        clauses: ClausesT,
        fields: Vec<&str>,
        order: OrderT,
        limit: u32) -> QVDRecordsT
    {
        let (_status, records) = q_select(
            C_BLOCK_EXT_INFO,
            fields,
            clauses,
            order,
            limit,
            false);
        return records;
    }


    // * do a groupping and some general validations on entire documents of a Block
    // old name was groupDocsOfBlock
    pub fn group_docs_of_block(&self, stage: &String) -> TransientBlockInfo
    {
        let mut transient_block_info = TransientBlockInfo::new();
        transient_block_info.m_status = false;
        transient_block_info.m_block = self.clone();
        transient_block_info.m_stage = stage.clone();


        if self.get_docs_count() == 0
        { return transient_block_info; }

        let now_ = application().now();
        for doc_inx in 0..self.m_block_documents.len() as CDocIndexT
        {
            let a_doc: &Document = &self.m_block_documents[doc_inx as usize];
            transient_block_info.m_doc_by_hash.insert(a_doc.get_doc_hash(), a_doc.clone());

            if (a_doc.m_doc_creation_date > self.m_block_creation_date)
                || (a_doc.m_doc_creation_date > now_)
            {
                dlog(
                    &format!(
                        "Block has document with creation-date after block-creation Date! stage({}), \
                        block({}) Doc({})",
                        stage,
                        cutils::hash8c(&self.m_block_hash),
                        cutils::hash8c(&a_doc.get_doc_hash())
                    ),
                    constants::Modules::App,
                    constants::SecLevel::Error);

                return transient_block_info;
            }
            transient_block_info.m_doc_index_by_hash.insert(a_doc.get_doc_hash(), doc_inx);

            if !is_valid_version_number(&a_doc.m_doc_version)
            {
                dlog(
                    &format!(
                        "Invalid dVer in group Docs Of Block stage({}) for doc({})",
                        stage,
                        cutils::hash8c(&a_doc.get_doc_hash())
                    ),
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return transient_block_info;
            }

            // document length control
            let (status, tmp_doc) = Document::load_document(
                &a_doc.export_doc_to_json(true), self, doc_inx);
            if !status
            {
                dlog(
                    &format!(
                        "Failed in load-doc: stage:{}, {}",
                        stage,
                        a_doc.export_doc_to_json(true)
                    ),
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
            }

            let recalculate_doc_length: DocLenT = tmp_doc.calc_doc_length();
            // println!("a_doc: {:#?}", a_doc);
            // println!("tmp_doc: {:#?}", tmp_doc);
            if (tmp_doc.get_doc_type() != constants::document_types::DATA_AND_PROCESS_COST_PAYMENT)
                &&
                (recalculate_doc_length != a_doc.get_doc_length())
            {
                let msg = format!(
                    "The doc stated dLen is not same as real length. stage({}) doc {} stated dLen({}), real length({})",
                    stage,
                    tmp_doc.get_doc_identifier(),
                    a_doc.calc_doc_length(),
                    tmp_doc.calc_doc_length());
                dlog(
                    &msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);

                drop(tmp_doc);

                return transient_block_info;
            }
            drop(tmp_doc);

            if !transient_block_info.m_grouped_documents.contains_key(&a_doc.m_doc_type)
            {
                transient_block_info.m_grouped_documents.insert(a_doc.m_doc_type.clone(), vec![]);
            }

            let mut tmp = transient_block_info.m_grouped_documents[&a_doc.m_doc_type].clone();
            tmp.push(a_doc.clone());
            transient_block_info.m_grouped_documents.insert(a_doc.m_doc_type.clone(), tmp);

            if a_doc.get_doc_ref() != "".to_string()
            {
                if Document::can_be_a_cost_payer_doc(&a_doc.get_doc_type())
                {
                    transient_block_info.m_transactions_dict.insert(a_doc.get_doc_hash(), a_doc.clone());
                    transient_block_info.m_map_trx_hash_to_trx_ref.insert(a_doc.get_doc_hash(), a_doc.get_doc_ref());
                    transient_block_info.m_map_trx_ref_to_trx_hash.insert(a_doc.get_doc_ref(), a_doc.get_doc_hash());
                } else {
                    transient_block_info.m_map_referencer_to_referenced.insert(a_doc.get_doc_hash(), a_doc.get_doc_ref());
                    transient_block_info.m_map_referenced_to_referencer.insert(a_doc.get_doc_ref(), a_doc.get_doc_hash());
                }
            }
        }

        let payed_refs_1: VString = transient_block_info.m_map_trx_ref_to_trx_hash.keys().cloned().collect::<VString>();
        let mut payed_refs_2: VString = vec![];
        for key in transient_block_info.m_map_trx_hash_to_trx_ref.keys().cloned().collect::<VString>()
        {
            payed_refs_2.push(transient_block_info.m_map_trx_hash_to_trx_ref[&key].clone());
        }

        for a_doc in &self.m_block_documents
        {
            if !Document::is_no_need_cost_payer_doc(&a_doc.get_doc_type())
            {
                // there must be a transaction to pay for this document
                if !payed_refs_1.contains(&a_doc.get_doc_hash()) || !payed_refs_2.contains(&a_doc.get_doc_hash())
                {
                    if (a_doc.get_doc_type() == constants::document_types::FREE_POST)
                        && (a_doc.get_doc_class() == constants::free_post_classes::DMS_POST)
                    {
                        // if ((getNonce() == "") || (m_block_creation_date > "2021-01-01 00:00:00"))
                        // {
                        //   CLog::log("The document DMS_Post has not Nonce & not payed by no transaction. stage(" + stage + ") document(" + cutils::hash8c(a_doc.get_doc_hash()) + ") ", "sec", "error");
                        //   return transient_block_info;
                        // }
                    } else {
                        dlog(
                            &format!(
                                "The document is not payed by no transaction. stage({}) document {}",
                                stage, a_doc.get_doc_identifier()),
                            constants::Modules::Sec,
                            constants::SecLevel::Error);
                        return transient_block_info;
                    }
                }
            }
        }

        if transient_block_info.m_map_trx_ref_to_trx_hash.keys().len()
            != transient_block_info.m_map_trx_hash_to_trx_ref.keys().len()
        {
            dlog(
                &format!(
                    "transaction count and ref count are different! stage({}) mapTrxRefToTrxHash: {:#?} mapTrxHashToTrxRef: {:#?}",
                    stage,
                    transient_block_info.m_map_trx_ref_to_trx_hash,
                    transient_block_info.m_map_trx_hash_to_trx_ref),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return transient_block_info;
        }

        for a_ref in transient_block_info.m_map_trx_ref_to_trx_hash.keys().cloned().collect::<VString>()
        {
            if !transient_block_info.m_transactions_dict.contains_key(
                &transient_block_info.m_map_trx_ref_to_trx_hash[&a_ref])
            {
                dlog(
                    &format!(
                        "Missed some1 transaction to support referenced documents. stage({}) trxDict: {:#?} \
                        mapTrxRefToTrxHash: {:#?}",
                        stage,
                        transient_block_info.m_transactions_dict,
                        transient_block_info.m_map_trx_ref_to_trx_hash),
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return transient_block_info;
            }
        }
        if cutils::array_diff(
            &transient_block_info.m_map_trx_hash_to_trx_ref.keys().cloned().collect::<VString>(),
            &transient_block_info.m_transactions_dict.keys().cloned().collect::<VString>()).len() != 0
        {
            dlog(
                &format!(
                    "Missed some transaction, to support referenced documents. stage({}) trxDict: {:#?} mapTrxRefToTrxHash: {:#?}",
                    stage,
                    transient_block_info.m_transactions_dict,
                    transient_block_info.m_map_trx_ref_to_trx_hash),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return transient_block_info;
        }

        for a_ref in &transient_block_info.m_map_trx_ref_to_trx_hash.keys().cloned().collect::<VString>()
        {
            if !transient_block_info.m_doc_by_hash.contains_key(a_ref)
            {
                dlog(
                    &format!(
                        "Missed a referenced document. stage({}) referenced-doc({}), referencer doc {}",
                        stage,
                        cutils::hash8c(a_ref),
                        cutils::hash8c(&transient_block_info.m_map_trx_ref_to_trx_hash[a_ref])),
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return transient_block_info;
            }
        }

        if transient_block_info.m_doc_index_by_hash.keys().len()
            != self.m_block_documents.len()
        {
            dlog(
                &format!(
                    "There is duplicated doc.hash in block. stage({}) block({}) ",
                    stage,
                    cutils::hash8c(&self.get_block_hash())),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return transient_block_info;
        }

        let mut doc_types: VString = transient_block_info.m_grouped_documents.keys().cloned().collect::<VString>();
        doc_types.sort();
        for a_type in doc_types
        {
            dlog(
                &format!(
                    "The block: {} has {} Document(s) of type({}) ",
                    self.get_block_identifier(),
                    transient_block_info.m_grouped_documents[&a_type].len(),
                    a_type),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }

        transient_block_info.m_status = true;
        return transient_block_info;
    }

    /*

         String Block::getNonce() const
         {
           cutils::exiter("m_nonce isn't implement for Block Base class", 0);
           return "";
         }

         Vec<Document *> Block::getDocuments() const
         {
           return m_documents;
         }

         //std::tuple<bool, JSonObject> Block::selectBExtInfosFromDB(const String& block_hash)
         //{
         //  QueryRes bExtInfo = DbModel::select(
         //    "db_comen_blocks",
         //    C_BLOCK_EXT_INFO,
         //    VString {"x_block_hash", "x_detail"},     // fields
         //    {ModelClause("x_block_hash", block_hash)},
         //    {},
         //    1   // limit
         //  );
         //  if (bExtInfo.records.len() == 0)
         //    return {false, JSonObject {}};

         //  QVariant x_detail = bExtInfo.records[0]["x_detail"];
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
            C_BLOCK_EXT_INFO,
            &values,
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
*/
    //old_name_was getDocumentJByHash
    pub fn get_json_document_by_hash(
        block: &JSonObject,
        doc_hash: &CDocHashT) -> (CDocIndexT, JSonObject)
    {
        let documents = block["bDocs"].as_array().unwrap();
        for doc_inx in 0..documents.len() as CDocIndexT
        {
            if remove_quotes(&documents[doc_inx as usize]["dHash"]) == doc_hash.to_string()
            { return (doc_inx, documents[doc_inx as usize].clone()); }
        }
        return (-1, json!({}));
    }
}

//old_name_was regenerateBlock
pub fn regenerate_block(block_hash: &CBlockHashT) -> (bool, JSonObject)
{
    //listener.doCallSync('SPSH_before_regenerate_block', { block_hash });
    let records = search_in_dag(
        vec![simple_eq_clause("b_hash", block_hash)],
        vec!["b_body"],
        vec![],
        0,
        false);
    if records.len() == 0
    {
        // TODO: the block is valid and does not exist in local. or
        // invalid block invoked, maybe some penal for sender!
        dlog(
            &format!(
                "The requested block to regenerate, which doesn't exist in DAG! {}",
                block_hash
            ),
            constants::Modules::App,
            constants::SecLevel::Warning);
        return (false, json!({}));
    }

    let (status, _sf_ver, serialized_block) =
        unwrap_safed_content_for_db(&records[0]["b_body"].to_string());
    if !status
    {
        dlog(
            &format!(
                "Failed in un-wrap safe record{},  {:?}",
                block_hash,
                &records
            ),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, json!({}));
    }
    let (status, mut j_block) = controlled_str_to_json(&serialized_block);
    if !status
    {
        dlog(
            &format!(
                "Failed in deser safe un-wrapped record{},  {:?}",
                block_hash,
                &serialized_block
            ),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, json!({}));
    }

    let (status, ext_info_exist, block_ext_info) = get_block_ext_info(block_hash);
    if !status
    {
        return (false, json!({}));
    }

    if ext_info_exist
    {
        j_block["bExtInfo"] = json!({
            "bExtInfo": block_ext_info
        });
    }

    return (true, j_block);
}

//old_name_was getBlockExtInfo
pub fn get_block_ext_info(block_hash: &String) ->
(
    bool/* status */,
    bool/* block_has_ext_info */,
    Vec<DocExtInfo>/* block_ext_info */)
{
    let (status, records) = q_select(
        C_BLOCK_EXT_INFO,
        vec!["x_block_hash", "x_detail"],
        vec![simple_eq_clause("x_block_hash", block_hash)],
        vec![],
        0, false);

    if !status
    {
        return (false, false, vec![]);
    }

    if records.len() != 1
    {
        dlog(
            &format!("get Block Ext Info: the block({}) has not ext Info", cutils::hash8c(block_hash)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return (true, false, vec![]);
    }

    let (status, _sf_ver, serialized_block) = unwrap_safed_content_for_db(
        &records[0]["x_detail"].to_string());
    if !status
    {
        dlog(
            &format!("Failed on un-wrapping safe wrapped doc ext info {}",
                     records[0]["x_detail"].to_string()),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);
        return (true, false, vec![]);
    }

    let mut is_valid: bool = true;
    let block_ext_info: Vec<DocExtInfo> = match serde_json::from_str(&serialized_block) {
        Ok(r) => r,
        Err(e) => {
            is_valid = false;
            dlog(
                &format!(
                    "Failed on json deser un-wrapped doc ext info {} {}",
                    e,
                    serialized_block),
                constants::Modules::App,
                constants::SecLevel::Error);
            vec![]
        }
    };
    if !is_valid
    {
        return (false, false, vec![]);
    }

    return (true, true, block_ext_info);
}
