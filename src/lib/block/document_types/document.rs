use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::coinbase_document::CoinbaseDocument;
use crate::lib::block::document_types::dna_proposal_document::DNAProposalDocument;
use crate::lib::block::document_types::polling_document::PollingDocument;
use crate::lib::custom_types::{CBlockHashT, CDocHashT, CDocIndexT, DocLenT, JSonObject};
use crate::lib::database::abs_psql::q_insert;
use crate::lib::database::tables::STBL_DOCS_BLOCKS_MAP;

#[derive(Clone, Serialize, Deserialize)]
pub struct Document
{
    pub m_doc_hash: String,
    pub m_doc_type: String,
    pub m_doc_class: String,
    pub m_doc_version: String,
    pub m_doc_title: String,
    pub m_doc_ref: String,
    pub m_doc_title_hash: String,
    pub m_doc_comment: String,
    pub m_doc_tags: String,
    pub m_doc_creation_date: String,
    pub m_block_creation_date: String,
    pub m_doc_ext_hash: String,
    pub m_doc_ext_info: Vec<JSonObject>,
    pub m_doc_length: DocLenT,

    pub m_if_proposal_doc: DNAProposalDocument,
    pub m_if_polling_doc: PollingDocument,
    pub m_if_basic_tx_doc: BasicTxDocument,
    pub m_if_coinbase: CoinbaseDocument,
}

impl Document
{
    pub fn new() -> Document {
        Document {
            m_doc_hash: "".to_string(),
            m_doc_type: "".to_string(),
            m_doc_class: "".to_string(),
            m_doc_version: constants::DEFAULT_DOCUMENT_VERSION.to_string(),
            m_doc_title: "".to_string(),
            m_doc_ref: "".to_string(),
            m_doc_title_hash: "".to_string(),
            m_doc_comment: "".to_string(),
            m_doc_tags: "".to_string(),
            m_doc_creation_date: "".to_string(),
            m_block_creation_date: "".to_string(),
            m_doc_ext_hash: "".to_string(),
            m_doc_ext_info: vec![],
            m_doc_length: 0,

            m_if_proposal_doc: DNAProposalDocument::new(),
            m_if_polling_doc: PollingDocument::new(),
            m_if_basic_tx_doc: BasicTxDocument::new(),
            m_if_coinbase: CoinbaseDocument::new(),
        }
    }

    pub fn getDocExtInfo(&self) -> &Vec<JSonObject>
    {
        return &self.m_doc_ext_info;
    }

    pub fn maybeAssignDocExtInfo(
        &mut self,
        block: &Block,
        doc_inx: CDocIndexT)
    {
        if self.m_doc_ext_info.len() == 0 {
            self.m_doc_ext_info = block.getBlockExtInfoByDocIndex(doc_inx as usize).clone();
        }
    }

    pub fn setByJsonObj(
        &mut self,
        obj: &JSonObject,
        block: &Block,
        doc_index: CDocIndexT) -> bool
    {
        if obj["dHash"].to_string() != ""
        {
            self.m_doc_hash = obj["dHash"].to_string();
        }
        /*
                if obj["dType"].to_string() != "")
                m_doc_type = obj["dType"].to_string();

                if obj["dClass"].to_string() != "")
                m_doc_class = obj["dClass"].to_string();

                if obj["dRef"].to_string() != "")
                m_doc_ref = obj["dRef"].to_string();

                if obj["dCDate"].to_string() != "")
                m_doc_creation_date = obj["dCDate"].to_string();

                if obj["dVer"].to_string() != "")
                m_doc_version = obj["dVer"].to_string();

                if obj["dLen"].to_string() != "")
                m_doc_length = cutils::convertPaddedStringToInt(obj["dLen"].to_string());

                if obj["dComment"].to_string() != "")
                m_doc_comment = obj["dComment"].to_string();

                if obj["dTitle"].to_string() != "")
                m_doc_title = obj["dTitle"].to_string();

                if (obj.keys().contains("dExtInfo"))
                m_doc_ext_info = obj["dExtInfo"].toArray();

                if obj["dExtHash"].to_string() != "")
                m_doc_ext_hash = obj["dExtHash"].to_string();

                if obj["dTags"].to_string() != "")
                m_doc_tags = obj["dTags"].to_string();

        */

        let doc_type: String = obj["dType"].to_string();
        self.m_doc_type = doc_type.clone();

        if doc_type == constants::document_types::BasicTx
        {
            self.m_if_basic_tx_doc.setByJsonObj(obj);
        } else if doc_type == constants::document_types::DPCostPay
        {
            /*
            doc = new
            DPCostPayDocument(obj);
            */
        } else if doc_type == constants::document_types::Coinbase
        {
            /*
            doc = new
            CoinbaseDocument(obj);
            */
        } else if doc_type == constants::document_types::RpDoc
        {
            /*
            doc = new
            RepaymentDocument(obj);
            */
            // } else if doc_type == constants::document_types::FPost
            // {
            //     /*
            //     doc = new
            //     FreeDocument(obj);
            //     */
        } else if doc_type == constants::document_types::Ballot
        {
            /*
            doc = new
            BallotDocument(obj);
            */
        } else if doc_type == constants::document_types::Polling
        {
            /*
            doc = new
            PollingDocument(obj);
            */
        } else if doc_type == constants::document_types::AdmPolling
        {
            /*
            doc = new
            AdministrativePollingDocument(obj);
            */
        } else if doc_type == constants::document_types::DNAProposal
        {
            self.m_if_proposal_doc.setByJsonObj(obj);
        } else if doc_type == constants::document_types::Pledge
        {
            /*
            doc = new
            PledgeDocument(obj);
            */
        } else if doc_type == constants::document_types::ClosePledge
        {
            /*
            doc = new
            ClosePledgeDocument(obj);
            */
        } else if doc_type == constants::document_types::INameReg
        {
            /*
            doc = new
            INameRegDocument(obj);
            */
        } else if doc_type == constants::document_types::INameBind
        {
            /*
            doc = new
            INameBindDocument(obj);
            */
        }


        if (self.m_doc_ext_info.len() > 0) && (doc_index != -1) && (block.m_block_hash != "")
        {
            self.maybeAssignDocExtInfo(block, doc_index);
        }

        return true;
    }


    pub fn safeStringifyDoc(&self, ext_info_in_document: bool) -> String
    {
        let mut j_doc: JSonObject = self.exportDocToJson(ext_info_in_document);
        j_doc["dLen"] = constants::LEN_PROP_PLACEHOLDER.into();
        let serialized_j_doc: String = cutils::serialize_json(&j_doc);
        // recaluculate block final length
        j_doc["dLen"] = cutils::padding_length_value(
            serialized_j_doc.len().to_string(),
            constants::LEN_PROP_LENGTH)
            .into();

        dlog(
            &format!(
                "5 safe Sringify Doc({}):  {} / {} length:{} serialized document: {}",
                cutils::hash8c(&self.m_doc_hash),
                self.m_doc_type,
                self.m_doc_class,
                j_doc["dLen"],
                serialized_j_doc),
            constants::Modules::App,
            constants::SecLevel::Trace);

        return cutils::serialize_json(&j_doc);
    }

    pub fn getRef(&self) -> String
    {
        return self.m_doc_ref.clone();
    }
    /*

        String Document::getRefType() const
        {
        return "";
        }

        String Document::getProposalRef() const
        {
        return "";
        }

        String Document::getPayerTrxLinkBack() const
        {
        return "";
        }

        CMPAISValueT Document::getDocCosts() const
        {
        return -1;
        }

        bool Document::deleteInputs()
        {
        cutils::exiter("deleteInputs is n't implement for Document Base class document(" + m_doc_type + ")", 0);
        return false;
        }

        bool Document::deleteOutputs()
        {
        cutils::exiter("deleteOutputs is n't implement for Document Base class document(" + m_doc_type + ")", 0);
        return false;
        }

        bool Document::setDocumentInputs(const QJsonValue& obj)
        {
        cutils::exiter("set Document Inputs is n't implement for Document Base class document(" + m_doc_type + ")", 0);
        return false;
        }

        bool Document::setDocumentOutputs(const QJsonValue& obj)
        {
        cutils::exiter("set Document Outputs is n't implement for Document Base class document(" + m_doc_type + ")", 0);
        return false;
        }


        std::tuple<bool, JSonArray> Document::exportInputsToJson() const
        {
        return {false, JSonArray {}};
        }

        std::tuple<bool, JSonArray> Document::exportOutputsToJson() const
        {
        return {false, JSonArray {}};
        }
        */


    pub fn exportDocToJson_inner(&self, ext_info_in_document: bool) -> JSonObject
    {
        let mut document: JSonObject = json!({
            "dHash":self.m_doc_hash,
            "dType": self.m_doc_type,
            "dVer": self.m_doc_version,
            "dCDate": self.m_doc_creation_date,
            "dLen": constants::LEN_PROP_PLACEHOLDER, //cutils::padding_length_value(self.m_doc_length.to_string(), constants::LEN_PROP_LENGTH)
        });

        if self.m_doc_class != ""
        {
            document["dClass"] = self.m_doc_class.clone().into();
        }

        /*
        // maybe add inputs
        auto [has_input, Jinputs] = exportInputsToJson();
        if (has_input)
        document["inputs"] = Jinputs;

        // maybe add outputs
        auto [has_output, Joutputs] = exportOutputsToJson();
        if (has_output)
        document["outputs"] = Joutputs;
        */

        if self.m_doc_ref != "" {
            document["dRef"] = self.m_doc_ref.clone().into();
        }
        if self.m_doc_title != "" {
            document["dTitle"] = self.m_doc_title.clone().into();
        }
        if self.m_doc_comment != "" {
            document["dComment"] = self.m_doc_comment.clone().into();
        }

        if self.m_doc_tags != "" {
            document["dTags"] = self.m_doc_tags.clone().into();
        }

        if self.m_doc_ext_hash != "" {
            document["dExtHash"] = self.m_doc_ext_hash.clone().into();
        }

        if ext_info_in_document {
            document["dExtInfo"] = self.m_doc_ext_info.clone().into();
            // document["dExtInfo"] = serde_json::to_string(&self.m_doc_ext_info).unwrap().into();
        }

        return document;
    }

    pub fn exportDocToJson(&self, ext_info_in_document: bool) -> JSonObject
    {
        if self.m_doc_type == constants::document_types::BasicTx {
            return self.m_if_basic_tx_doc.exportDocToJson(self, ext_info_in_document);
        } else if self.m_doc_type == constants::document_types::DNAProposal {
            return self.m_if_proposal_doc.exportDocToJson(self, ext_info_in_document);
        }

        panic!("should not reach here");
        //return json!({});
    }

    /*
    QVector<COutputIndexT> Document::getDPIs() const
    {
    cutils::exiter("attribute data_and_process_payment_indexes is n't implement for Base class document(" + self.m_doc_type + ")", 0);
    return {};
    }

*/
    //old_name_was getDocHash
    pub fn get_doc_hash(&self) -> String
    {
        return self.m_doc_hash.clone();
    }
    /*


    std::vector<TInput*> Document::getInputs() const
    {
    return {};
    }

    std::vector<TOutput*> Document::getOutputs() const
    {
    return {};
    }

    String Document::getDocHashableString() const
    {
    return "";
    }

    CDocHashT Document::getDocRef() const
    {
    return m_doc_ref;
    }

*/
    //old_name_was calcDocHash
    pub fn calc_doc_hash(&self) -> CDocHashT {
        if self.m_doc_type == constants::document_types::BasicTx
        {
            return self.m_if_basic_tx_doc.calc_doc_hash(self);
        } else if self.m_doc_type == constants::document_types::DPCostPay
        {} else if self.m_doc_type == constants::document_types::Coinbase
        {} else if self.m_doc_type == constants::document_types::RpDoc
        {

            // } else if doc_type == constants::document_types::FPost
            // {
            //     /*
            //     doc = new
            //     FreeDocument(obj);
            //     */
        } else if self.m_doc_type == constants::document_types::Ballot
        {} else if self.m_doc_type == constants::document_types::Polling
        {} else if self.m_doc_type == constants::document_types::AdmPolling
        {} else if self.m_doc_type == constants::document_types::DNAProposal
        {
            return self.m_if_proposal_doc.calc_doc_hash(&self);
        } else if self.m_doc_type == constants::document_types::Pledge
        {} else if self.m_doc_type == constants::document_types::ClosePledge
        {} else if self.m_doc_type == constants::document_types::INameReg
        {} else if self.m_doc_type == constants::document_types::INameBind
        {}


        return "".to_string();
    }

    pub fn calcDocLength(&self) -> DocLenT {
        let doc_length: DocLenT = self.safeStringifyDoc(true).len();
        return doc_length;
    }

    /*

    void Document::setDocHash()
    {
    m_doc_hash = calcDocHash();
    }

    void Document::setDocLength()
    {
    m_doc_length = calcDocLength();
    }

    void Document::setDExtHash()
    {
    m_doc_ext_hash = calcDocExtInfoHash();
    }

    GenRes Document::customValidateDoc(const Block* block) const
    {
    Q_UNUSED(block);
    return {true, ""};
    }

    GenRes Document::fullValidate(const Block* block) const
    {
    String msg;
    // doc length controll
    DocLenT doc_lenght = safeStringifyDoc(true).len();
    if(doc_lenght < 1)
    {
    msg = "Doc length can not be zero or negative! block(" + block.m_block_type + " / " + cutils::hash8c(block.m_block_hash) + ") doc(" + m_doc_type + " / " + cutils::hash8c(get_doc_hash()) + ") doc class(" + m_doc_class + ")";
    CLog::log(msg, "sec", "error");
    return {false, msg};
    }
    if(doc_lenght > constants::MAX_DOC_LENGTH_BY_CHAR)
    {
    CLog::log("Doc length excced! block(" + block.m_block_type + " / " + cutils::hash8c(block.m_block_hash) + ") doc(" + m_doc_type + " / " + cutils::hash8c(get_doc_hash()) + ") doc class(" + m_doc_class + ")", "sec", "error");
    return {false, msg};
    }
    if ((m_doc_length != 0) && (doc_lenght > m_doc_length))
    {
    msg = "Doc real length is biger than stated length! block(" + block.m_block_type + " / " + cutils::hash8c(block.m_block_hash) + ") doc(" + m_doc_type + " / " + cutils::hash8c(get_doc_hash()) + ") doc class(" + m_doc_class + ")";
    CLog::log(msg, "sec", "error");
    return {false, msg};
    }

    // recalculate documents hash
    if(get_doc_hash() != calcDocHash())
    {
    msg = "Mismatch document Hash! doc(" + m_doc_type + " / " + cutils::hash8c(get_doc_hash()) + ") localy calculated(" + cutils::hash8c(calcDocHash()) +") block(" + block.m_block_type + " / " + cutils::hash8c(block.m_block_hash) + ") ";
    CLog::log(msg, "sec", "error");
    return {false, msg};
    }

    // controll doc ext hash
    if(m_doc_ext_hash != calcDocExtInfoHash())
    {
    msg = "Mismatch doc ext Hash. remote doc ext hash(" + m_doc_ext_hash + ") localy calculated(" + calcDocExtInfoHash() +") block(" + cutils::hash8c(block.m_block_hash) + ") " + safeStringifyDoc();
    CLog::log(msg, "sec", "error");
    return {false, msg};
    }

    // controll document signatures (if exist)
    if(hasSignable())
    if(!veridfyDocSignature())
      return {false, "Failed in vrify doc signature"};

    // general doc validation
    GenRes custom_validate = customValidateDoc(block);
    if(!custom_validate.status)
    {
    msg = "Failed validate Doc " +custom_validate.msg+ " hash(" + m_doc_ext_hash + ") block(" + cutils::hash8c(block.m_block_hash) + ")";
    CLog::log(msg, "sec", "error");
    return {false, msg};
    }

    return {true, "is Valid"};
    }

    */
    /**
     * @brief Document::applyDocFirstImpact
     * depends on the document type, the node does some impacts on database
     * e.g. records a FleNS in DB
     */
    pub fn apply_doc_first_impact(&self, block: &Block) -> bool
    {
        if self.m_doc_type == constants::document_types::DNAProposal
        {
            return self.m_if_proposal_doc.apply_doc_first_impact(self, block);
        } else {}
        return false;
    }

    /*
        //JSonObject Document::exportJson() const
        //{
        //  JSonObject r;
        //  return r;
        //}

        std::tuple<bool, CMPAIValueT> Document::calcDocDataAndProcessCost(
        const String& stage,
        String cDate,
        const uint32_t& extraLength) const
        {
        Q_UNUSED(stage);
        Q_UNUSED(cDate);
        Q_UNUSED(extraLength);
        return {false, 0};
        }

        bool Document::hasSignable() const
        {
        // by default documents have no part to signing
        return false;
        }

        String Document::getDocToBeSignedHash() const
        {
        // by default documents have no part to signing
        return "";
        }

        String Document::getDocSignMsg() const
        {
        return "";
        }

        bool Document::veridfyDocSignature() const
        {
        return true;
        }

        GenRes Document::applyDocImpact2()
        {
        return
        {
        false
        };
        }

        */
    pub fn mapDocToBlock(&self,
                         block_hash: &CBlockHashT,
                         doc_index: CDocIndexT)
    {
        return mapDocToBlock(&self.m_doc_hash, block_hash, doc_index);
    }
    /*

    bool Document::trxHasInput(const String& document_type)
    {
    return StringList {
    constants::DOC_TYPES::BasicTx,
    constants::DOC_TYPES::RpDoc
    }.contains(document_type);
    }

    bool Document::trxHasInput()
    {
    return Document::trxHasInput(m_doc_type);
    }

    bool Document::trxHasNotInput(const String& document_type)
    {
    return StringList {
    constants::DOC_TYPES::Coinbase,
    constants::DOC_TYPES::DPCostPay,
    constants::DOC_TYPES::RlDoc
    }.contains(document_type);
    }

    bool Document::trxHasNotInput()
    {
    return Document::trxHasNotInput(m_doc_type);
    }

    bool Document::isBasicTransaction(const String& dType)
    {
    // Note: the iConsts.DOC_TYPES.Coinbase  and iConsts.DOC_TYPES.DPCostPay altough are transactions, but have special tretmnent
    // Note: the iConsts.DOC_TYPES.RpDoc altough is a transaction, but since is created directly by node and based on validated coinbase info, so does not need validate
    return StringList{constants::DOC_TYPES::BasicTx}.contains(dType);
    }

    bool Document::isBasicTransaction()
    {
    return isBasicTransaction(m_doc_type);
    }

    bool Document::isDPCostPayment(const String& dType)
    {
    return StringList{constants::DOC_TYPES::DPCostPay}.contains(dType);
    }

    bool Document::isDPCostPayment()
    {
    return isDPCostPayment(m_doc_type);
    }

    bool Document::canBeACostPayerDoc(const String& dType)
    {
    return StringList{constants::DOC_TYPES::BasicTx}.contains(dType);
    }

    /**
    * The documents which do not need another doc to pay their cost.
    * instead they can pay the cost of another docs
    *
    * @param {} dType
    */
    bool Document::isNoNeedCostPayerDoc(const String& dType)
    {
    return (StringList {
    constants::DOC_TYPES::BasicTx,
    constants::DOC_TYPES::DPCostPay,
    constants::DOC_TYPES::RpDoc,
    constants::DOC_TYPES::RlDoc
    }.contains(dType));
    }

    String Document::stringify_inputs() const
    {
    return SignatureStructureHandler::stringify_inputs(getInputs());
    }

    String Document::stringify_outputs() const
    {
    return SignatureStructureHandler::stringify_outputs(getOutputs());
    }

    // old name was trxHasOutput
    bool Document::docHasOutput(const String& document_type)
    {
    return StringList {
    constants::DOC_TYPES::Coinbase,
    constants::DOC_TYPES::BasicTx,
    constants::DOC_TYPES::RpDoc,
    constants::DOC_TYPES::RlDoc,}.contains(document_type);
    }

    bool Document::docHasOutput()
    {
    return docHasOutput(m_doc_type);
    }

    void Document::importCostsToTreasury(
    const Block* block,
    UTXOImportDataContainer* block_inspect_container)
    {
    cutils::exiter("Import Costs To Treasury not implemented for base class block Type(" + block.m_block_type + ")", 97);
    }

    QVDRecordsT Document::searchInDocBlockMap(
    const ClausesT& clauses,
    const StringList fields,
    const OrderT order,
    const uint64_t limit)
    {
    QueryRes res = DbModel::select(
    stbl_docs_blocks_map,
    fields,
    clauses,
    order,
    limit);
    return res.records;
    }

    */
}

pub fn mapDocToBlock(
    doc_hash: &CDocHashT,
    block_hash: &CBlockHashT,
    doc_index: CDocIndexT)
{
    let dbm_doc_index = doc_index;
    let dbm_last_control = cutils::get_now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("dbm_block_hash", &block_hash as &(dyn ToSql + Sync)),
        ("dbm_doc_index", &dbm_doc_index as &(dyn ToSql + Sync)),
        ("dbm_doc_hash", &doc_hash as &(dyn ToSql + Sync)),
        ("dbm_last_control", &dbm_last_control as &(dyn ToSql + Sync))]);
    dlog(
        &format!("--- connecting Doc({}) to Block({})",
                 cutils::hash8c(doc_hash), cutils::hash8c(block_hash)),
        constants::Modules::App,
        constants::SecLevel::Info);
    q_insert(
        STBL_DOCS_BLOCKS_MAP,     // table
        &values, // values to insert
        true);
}