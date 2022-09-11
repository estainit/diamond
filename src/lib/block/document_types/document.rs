use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use crate::{application, constants, cutils, dlog};
use crate::cutils::{remove_quotes};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::basic_tx_document::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::coinbase_document::CoinbaseDocument;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::block::document_types::null_document::NullDocument;
use crate::lib::block::document_types::proposal_document::ProposalDocument;
use crate::lib::block::document_types::polling_document::PollingDocument;
use crate::lib::custom_types::{CBlockHashT, CDateT, CDocHashT, CDocIndexT, CMPAIValueT, COutputIndexT, DocLenT, JSonObject, QV2DicT, VString, VVString};
use crate::lib::database::abs_psql::q_insert;
use crate::lib::database::tables::C_DOCS_BLOCKS_MAP;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{make_inputs_tuples, make_outputs_tuples, stringify_outputs, TInput, TOutput};

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub m_doc_ext_info: Vec<DocExtInfo>,
    pub m_doc_length: DocLenT,

    pub m_if_proposal_doc: ProposalDocument,
    pub m_if_polling_doc: PollingDocument,
    pub m_if_basic_tx_doc: BasicTxDocument,
    pub m_if_coinbase_doc: CoinbaseDocument,
    pub m_if_null_doc: NullDocument,

    pub m_empty_vec: Vec<String>,

}

impl Document
{
    pub fn new() -> Self {
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

            m_if_proposal_doc: ProposalDocument::new(),
            m_if_polling_doc: PollingDocument::new(),
            m_if_basic_tx_doc: BasicTxDocument::new(),
            m_if_coinbase_doc: CoinbaseDocument::new(),
            m_if_null_doc: NullDocument::new(),
            m_empty_vec: vec![],
        }
    }

    //old_name_was getDocExtInfo
    #[allow(unused, dead_code)]
    pub fn get_doc_ext_info(&self) -> &Vec<DocExtInfo>
    {
        return &self.m_doc_ext_info;
    }

    //old_name_was maybeAssignDocExtInfo
    pub fn maybe_assign_doc_ext_info(
        &mut self,
        block: &Block,
        doc_inx: CDocIndexT)
    {
        if self.m_doc_ext_info.len() == 0 {
            self.m_doc_ext_info = block.get_block_ext_info_by_doc_index(doc_inx as usize).clone();
        }
    }

    pub fn set_doc_by_json_obj(
        &mut self,
        obj: &JSonObject,
        block: &Block,
        doc_index: CDocIndexT) -> bool
    {
        if !obj["dHash"].is_null()
        {
            self.m_doc_hash = remove_quotes(&obj["dHash"]);
        }

        if !obj["dType"].is_null()
        {
            self.m_doc_type = remove_quotes(&obj["dType"]);
        }

        if !obj["dClass"].is_null()
        {
            self.m_doc_class = remove_quotes(&obj["dClass"]);
        }

        if !obj["dRef"].is_null()
        {
            self.m_doc_ref = remove_quotes(&obj["dRef"]);
        }

        if !obj["dCDate"].is_null()
        {
            self.m_doc_creation_date = remove_quotes(&obj["dCDate"]);
        }

        if !obj["dVer"].is_null()
        {
            self.m_doc_version = remove_quotes(&obj["dVer"]);
        }

        if !obj["dLen"].is_null()
        {
            self.m_doc_length = remove_quotes(&obj["dLen"]).parse::<DocLenT>().unwrap();
        }

        if !obj["dComment"].is_null()
        {
            self.m_doc_comment = remove_quotes(&obj["dComment"]);
        }

        if !obj["dTitle"].is_null()
        {
            self.m_doc_title = remove_quotes(&obj["dTitle"]);
        }

        if !obj["dExtInfo"].is_null()
        {
            println!("iiii i i i i i iiiii: {}", obj["dExtInfo"]);
            //self.m_doc_ext_info = remove_quotes(&obj[");
        }

        if (self.m_doc_ext_info.len() > 0) && (doc_index != -1) && (block.m_block_hash != "")
        {
            self.maybe_assign_doc_ext_info(block, doc_index);
        }

        if !obj["dExtHash"].is_null()
        {
            self.m_doc_ext_hash = remove_quotes(&obj["dExtHash"]);
        }

        if !obj["dTags"].is_null()
        {
            self.m_doc_tags = remove_quotes(&obj["dTags"]);
        }


        let doc_type: String = remove_quotes(&obj["dType"]);
        self.m_doc_type = doc_type.clone();

        if doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.set_doc_by_json_obj(obj);
        } else if doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
        {
            /*
            doc = new
            DPCostPayDocument(obj);
            */
        } else if doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.set_doc_by_json_obj(obj);
        } else if doc_type == constants::document_types::REPAYMENT_DOCUMENT
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
        } else if doc_type == constants::document_types::BALLOT
        {
            /*
            doc = new
            BallotDocument(obj);
            */
        } else if doc_type == constants::document_types::POLLING
        {
            return self.m_if_polling_doc.set_doc_by_json_obj(obj);
        } else if doc_type == constants::document_types::ADMINISTRATIVE_POLLING
        {
            /*
            doc = new
            AdministrativePollingDocument(obj);
            */
        } else if doc_type == constants::document_types::PROPOSAL
        {
            return self.m_if_proposal_doc.set_doc_by_json_doc(obj);
        } else if doc_type == constants::document_types::PLEDGE
        {
            /*
            doc = new
            PledgeDocument(obj);
            */
        } else if doc_type == constants::document_types::CLOSE_PLEDGE
        {
            /*
            doc = new
            ClosePledgeDocument(obj);
            */
        } else if doc_type == constants::document_types::I_NAME_REGISTER
        {
            /*
            doc = new
            INameRegDocument(obj);
            */
        } else if doc_type == constants::document_types::I_NAME_BIND
        {
            /*
            doc = new
            INameBindDocument(obj);
            */
        }

        panic!("Invalid document type! {}", doc_type);
    }

    //old_name_was calcDocLength
    pub fn calc_doc_length(&self) -> DocLenT {
        let doc_length: DocLenT = self.safe_stringify_doc(true).len();
        return doc_length;
    }

    pub fn safe_stringify_doc(&self, ext_info_in_document: bool) -> String
    {
        let mut j_doc: JSonObject = self.export_doc_to_json(ext_info_in_document);
        j_doc["dLen"] = constants::LEN_PROP_PLACEHOLDER.into();
        let serialized_j_doc: String = cutils::controlled_json_stringify(&j_doc);
        // recaluculate block final length
        j_doc["dLen"] = cutils::padding_length_value(
            serialized_j_doc.len().to_string(),
            constants::LEN_PROP_LENGTH)
            .into();

        dlog(
            &format!(
                "5 safe Sringify A Doc({}):  {} / {} length:{} serialized document: {}",
                cutils::hash8c(&self.m_doc_hash),
                self.m_doc_type,
                self.m_doc_class,
                j_doc["dLen"],
                serialized_j_doc),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return cutils::controlled_json_stringify(&j_doc);
    }

    //old_name_was getRef
    pub fn get_ref(&self) -> String
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

        bool Document::set_document_inputs(const QJsonValue& obj)
        {
        cutils::exiter("set Document Inputs is n't implement for Document Base class document(" + m_doc_type + ")", 0);
        return false;
        }

        bool Document::set_document_outputs(const QJsonValue& obj)
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


    pub fn export_doc_to_json_super(&self, ext_info_in_document: bool) -> JSonObject
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
            document["dExtInfo"] = json!({
            "dExtInfo": self.m_doc_ext_info
        });
            // document["dExtInfo"] = serde_json::to_string(&self.m_doc_ext_info).unwrap().into();
        }

        return document;
    }

    pub fn export_doc_to_json(&self, ext_info_in_document: bool) -> JSonObject
    {
        if self.m_doc_type == constants::document_types::BASIC_TX {
            return self.m_if_basic_tx_doc.export_doc_to_json(self, ext_info_in_document);
        } else if self.m_doc_type == constants::document_types::COINBASE {
            return self.m_if_coinbase_doc.export_doc_to_json(self, ext_info_in_document);
        } else if self.m_doc_type == constants::document_types::PROPOSAL {
            return self.m_if_proposal_doc.export_doc_to_json(self, ext_info_in_document);
        }

        panic!("should not reach here");
        //return json!({});
    }


    #[allow(unused, dead_code)]
    pub fn get_dpis_super(&self) -> Vec<COutputIndexT>
    {
        panic!("attribute data_and_process_payment_indexes is n't implement for Base class document({})", self.m_doc_type);
        return vec![];
    }

    pub fn get_dpis(&self) -> &Vec<COutputIndexT>
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.get_dpis();
        } else if self.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
        {} else if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_null_doc.get_dpis();
        } else if self.m_doc_type == constants::document_types::REPAYMENT_DOCUMENT
        {} else if self.m_doc_type == constants::document_types::BALLOT
        {} else if self.m_doc_type == constants::document_types::POLLING
        {} else if self.m_doc_type == constants::document_types::ADMINISTRATIVE_POLLING
        {} else if self.m_doc_type == constants::document_types::PROPOSAL
        {} else if self.m_doc_type == constants::document_types::PLEDGE
        {} else if self.m_doc_type == constants::document_types::CLOSE_PLEDGE
        {} else if self.m_doc_type == constants::document_types::I_NAME_REGISTER
        {} else if self.m_doc_type == constants::document_types::I_NAME_BIND
        {}

        panic!("Invalid document type to get dpis! {}", self.m_doc_hash);
    }

    //old_name_was getDocHash
    pub fn get_doc_hash(&self) -> String
    {
        return self.m_doc_hash.clone();
    }

    pub fn get_doc_type(&self) -> String
    {
        return self.m_doc_type.clone();
    }

    pub fn get_doc_class(&self) -> String
    {
        return self.m_doc_class.clone();
    }

    pub fn get_doc_ref(&self) -> String
    {
        return self.m_doc_ref.clone();
    }

    /*


    Vec<TInput*> Document::get_inputs() const
    {
    return {};
    }

    Vec<TOutput*> Document::get_outputs() const
    {
    return {};
    }

    String Document::get_doc_hashable_string() const
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
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.calc_doc_hash(self);
        } else if self.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
        {} else if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.calc_doc_hash(&self);
        } else if self.m_doc_type == constants::document_types::REPAYMENT_DOCUMENT
        {

            // } else if doc_type == constants::document_types::FPost
            // {
            //     /*
            //     doc = new
            //     FreeDocument(obj);
            //     */
        } else if self.m_doc_type == constants::document_types::BALLOT
        {} else if self.m_doc_type == constants::document_types::POLLING
        {} else if self.m_doc_type == constants::document_types::ADMINISTRATIVE_POLLING
        {} else if self.m_doc_type == constants::document_types::PROPOSAL
        {
            return self.m_if_proposal_doc.calc_doc_hash(&self);
        } else if self.m_doc_type == constants::document_types::PLEDGE
        {} else if self.m_doc_type == constants::document_types::CLOSE_PLEDGE
        {} else if self.m_doc_type == constants::document_types::I_NAME_REGISTER
        {} else if self.m_doc_type == constants::document_types::I_NAME_BIND
        {}


        panic!("Invalid document type to calculate its hash! {}", self.get_doc_identifier());
    }

    // old name was stringifyOutputs
    pub fn stringify_outputs(&self) -> String
    {
        return stringify_outputs(&self.get_outputs());
    }

    pub fn make_outputs_tuples(&self) -> VVString
    {
        return make_outputs_tuples(&self.get_outputs());
    }

    pub fn make_inputs_tuples(&self) -> VVString
    {
        return make_inputs_tuples(&self.get_inputs());
    }

    // old name was getOutputs
    pub fn get_outputs(&self) -> &Vec<TOutput>
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.get_outputs();
        } else if self.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
        {} else if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.get_outputs();
        } else if self.m_doc_type == constants::document_types::REPAYMENT_DOCUMENT
        {} else if self.m_doc_type == constants::document_types::BALLOT
        {} else if self.m_doc_type == constants::document_types::POLLING
        {
            return self.m_if_null_doc.get_outputs();
        } else if self.m_doc_type == constants::document_types::ADMINISTRATIVE_POLLING
        {} else if self.m_doc_type == constants::document_types::PROPOSAL
        {} else if self.m_doc_type == constants::document_types::PLEDGE
        {} else if self.m_doc_type == constants::document_types::CLOSE_PLEDGE
        {} else if self.m_doc_type == constants::document_types::I_NAME_REGISTER
        {} else if self.m_doc_type == constants::document_types::I_NAME_BIND
        {}

        panic!("Invalid document type to outputs! {}", self.get_doc_identifier());
    }

    // old name was getInputs
    pub fn get_inputs(&self) -> &Vec<TInput>
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.get_inputs();
        } else if self.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
        {} else if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_null_doc.get_inputs();
        } else if self.m_doc_type == constants::document_types::REPAYMENT_DOCUMENT
        {} else if self.m_doc_type == constants::document_types::BALLOT
        {} else if self.m_doc_type == constants::document_types::POLLING
        {} else if self.m_doc_type == constants::document_types::ADMINISTRATIVE_POLLING
        {} else if self.m_doc_type == constants::document_types::PROPOSAL
        {} else if self.m_doc_type == constants::document_types::PLEDGE
        {} else if self.m_doc_type == constants::document_types::CLOSE_PLEDGE
        {} else if self.m_doc_type == constants::document_types::I_NAME_REGISTER
        {} else if self.m_doc_type == constants::document_types::I_NAME_BIND
        {}

        panic!("Invalid document type to get inputs! {}", self.get_doc_identifier());
    }

    pub fn validate_signatures(
        &self,
        used_coins_dict: &QV2DicT,
        exclude_coins: &VString,
        block_hash: &CBlockHashT) -> bool
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.validate_signatures(
                self,
                used_coins_dict,
                exclude_coins,
                block_hash);
        } else if self.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
        {} else if self.m_doc_type == constants::document_types::COINBASE
        {} else if self.m_doc_type == constants::document_types::REPAYMENT_DOCUMENT
        {} else if self.m_doc_type == constants::document_types::BALLOT
        {} else if self.m_doc_type == constants::document_types::POLLING
        {} else if self.m_doc_type == constants::document_types::ADMINISTRATIVE_POLLING
        {} else if self.m_doc_type == constants::document_types::PROPOSAL
        {} else if self.m_doc_type == constants::document_types::PLEDGE
        {} else if self.m_doc_type == constants::document_types::CLOSE_PLEDGE
        {} else if self.m_doc_type == constants::document_types::I_NAME_REGISTER
        {} else if self.m_doc_type == constants::document_types::I_NAME_BIND
        {}

        panic!("Invalid document type to validate_ signatures! {}", self.get_doc_identifier());
    }


    /*

    void Document::setDocHash()
    {
    m_doc_hash = calcDocHash();
    }

    */

    // old name was setDExtHash
    pub fn set_doc_ext_hash(&mut self)
    {
        self.m_doc_ext_hash = self.calc_doc_ext_info_hash();
    }

    // old name was setDocLength
    pub fn set_doc_length(&mut self)
    {
        self.m_doc_length = self.calc_doc_length();
    }

    // old name was calcDocExtInfoHash
    pub fn calc_doc_ext_info_hash(&self) -> String
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.calc_doc_ext_info_hash(self);
        } else if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.calc_doc_ext_info_hash(self);
        }
        panic!("Invalid document for calc hash {}", self.get_doc_identifier());
    }

    // old name was hasSignable
    pub fn has_signable(&self) -> bool
    {
        if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.has_signable(self);
        }

        // by default documents have no part to signing
        panic!("Invalid document for has Signable {}", self.get_doc_identifier());
    }

    // old name was veridfyDocSignature
    pub fn veridfy_doc_signature(&self) -> bool
    {
        if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.verify_doc_signature(self);
        }

        panic!("Invalid document for verify doc sig {}", self.get_doc_identifier());
    }

    // customValidateDoc
    pub fn custom_validate_doc(&self, block: &Block) -> (bool, String)
    {
        if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.custom_validate_doc(self, block);
        }

        panic!("Invalid document for 'custom Validate Doc' {}", self.get_doc_identifier());
    }


    pub fn get_doc_identifier(&self) -> String
    {
        let doc_identifier = format!(
            " doc({}/{}) ",
            self.m_doc_type,
            cutils::hash8c(&self.m_doc_hash));
        return doc_identifier;
    }

    // old name was fullValidate
    pub fn full_validate(&self, block: &Block) -> (bool, String)
    {
        let err_msg: String;

        // doc length control
        let doc_lenght: DocLenT = self.safe_stringify_doc(true).len();
        if doc_lenght < 1
        {
            err_msg = format!(
                "Doc length can not be zero or negative! {} {} doc class({})",
                block.get_block_identifier(), self.get_doc_identifier(), self.m_doc_class);
            dlog(
                &err_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, err_msg);
        }

        if doc_lenght > constants::MAX_DOC_LENGTH_BY_CHAR
        {
            err_msg = format!(
                "Doc length excced! {} {} doc class({})",
                block.get_block_identifier(), self.get_doc_identifier(), self.m_doc_class);
            dlog(
                &err_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, err_msg);
        }

        if (self.m_doc_length != 0) && (doc_lenght > self.m_doc_length)
        {
            err_msg = format!(
                "Doc real length is longer than stated length! {} {} + doc class({})",
                block.get_block_identifier(), self.get_doc_identifier(), self.m_doc_class);
            dlog(
                &err_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, err_msg);
        }

        // recalculate documents hash
        if self.get_doc_hash() != self.calc_doc_hash()
        {
            err_msg = format!(
                "Mismatch document Hash! {} {} locally calculated({})",
                block.get_block_identifier(), self.get_doc_identifier(), cutils::hash8c(&self.calc_doc_hash()));
            dlog(
                &err_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, err_msg);
        }

        // controll doc ext hash
        if self.m_doc_ext_hash != self.calc_doc_ext_info_hash()
        {
            err_msg = format!(
                "Mismatch doc ext Hash. {} {}. remote doc ext hash ({}) localy calculated({})  safe: {}",
                block.get_block_identifier(), self.get_doc_identifier(), self.m_doc_ext_hash, self.calc_doc_ext_info_hash(), self.safe_stringify_doc(true));
            dlog(
                &err_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, err_msg);
        }

        // controll document signatures (if exist)
        if self.has_signable()
        {
            if !self.veridfy_doc_signature()
            {
                err_msg = format!(
                    "Failed in verify doc signature {} {} ",
                    block.get_block_identifier(), self.get_doc_identifier());
                dlog(
                    &err_msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return (false, err_msg);
            }
        }

        // custom doc validation
        let (status, msg_) = self.custom_validate_doc(block);
        if !status
        {
            err_msg = format!(
                "Failed validate Doc {} {} {} ",
                block.get_block_identifier(), self.get_doc_identifier(), msg_);
            dlog(
                &err_msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, err_msg);
        }

        return (true, "is Valid doc".to_string());
    }

    /**
     * @brief Document::applyDocFirstImpact
     * depends on the document type, the node does some impacts on database
     * e.g. records a FleNS in DB
     */
    pub fn apply_doc_first_impact(&self, block: &Block) -> bool
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {} else if self.m_doc_type == constants::document_types::COINBASE
        {
            return self.m_if_coinbase_doc.apply_doc_first_impact(self, block);
        } else if self.m_doc_type == constants::document_types::PROPOSAL
        {
            return self.m_if_proposal_doc.apply_doc_first_impact(self, block);
        } else {}

        panic!("Invalid doc type in 'apply doc first impact' {}", self.m_doc_type);
    }

    //old_name_was calcDocDataAndProcessCost
    pub fn calc_doc_data_and_process_cost_supper(
        _stage: &String,
        _c_date: &CDateT,
        _extra_length: DocLenT) -> (bool, CMPAIValueT)
    {
        panic!("Base document has no method to calc dp cost!");
    }

    pub fn calc_doc_data_and_process_cost(
        &self,
        stage: &str,
        c_date: &CDateT,
        extra_length: DocLenT) -> (bool, CMPAIValueT)
    {
        if self.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.m_if_basic_tx_doc.calc_doc_data_and_process_cost(
                self,
                stage,
                c_date,
                extra_length);
        } else if self.m_doc_type == constants::document_types::COINBASE
        {} else if self.m_doc_type == constants::document_types::PROPOSAL
        {} else {}

        panic!("Invalid doc type in 'calc dp cost doc' {}", self.m_doc_type);
    }


    /*
        //JSonObject Document::exportJson() const
        //{
        //  JSonObject r;
        //  return r;
        //}


    String Document::getDocToBeSignedHash() const
    {
    // by default documents have no part to signing
    return "";
}

String Document::getDocSignMsg() const
{
return "";
}


GenRes Document::applyDocImpact2()
{
return
{
false
};
}

*/

    //old_name_was mapDocToBlock
    pub fn map_doc_to_block(&self,
                            block_hash: &CBlockHashT,
                            doc_index: CDocIndexT)
    {
        return map_doc_to_block(&self.m_doc_hash, block_hash, doc_index);
    }
    /*

    bool Document::trxHasInput(const String& document_type)
    {
    return StringList {
    constants::document_types::BASIC_TX,
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
    constants::document_types::COINBASE,
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
    return StringList{constants::document_types::BASIC_TX}.contains(dType);
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
    return StringList{constants::document_types::BASIC_TX}.contains(dType);
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
    constants::document_types::BASIC_TX,
    constants::DOC_TYPES::DPCostPay,
    constants::DOC_TYPES::RpDoc,
    constants::DOC_TYPES::RlDoc
    }.contains(dType));
    }

    String Document::stringify_inputs() const
    {
    return SignatureStructureHandler::stringify_inputs(get_inputs());
    }

    String Document::stringify_outputs() const
    {
    return SignatureStructureHandler::stringify_outputs(get_outputs());
    }

    // old name was trxHasOutput
    bool Document::docHasOutput(const String& document_type)
    {
    return StringList {
    constants::document_types::COINBASE,
    constants::document_types::BASIC_TX,
    constants::DOC_TYPES::RpDoc,
    constants::DOC_TYPES::RlDoc,}.contains(document_type);
    }

    bool Document::docHasOutput()
    {
    return docHasOutput(m_doc_type);
    }

    void Document::importCostsToTreasury(
    const Block* block,
    CoinImportDataContainer* block_inspect_container)
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

//old_name_was mapDocToBlock
pub fn map_doc_to_block(
    doc_hash: &CDocHashT,
    block_hash: &CBlockHashT,
    doc_index: CDocIndexT)
{
    let dbm_doc_index = doc_index;
    let dbm_last_control = application().now();
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
        C_DOCS_BLOCKS_MAP,     // table
        &values, // values to insert
        true);
}


//old_name_was setDocumentOutputs
pub fn set_document_outputs(obj: &Vec<JSonObject>) -> Vec<TOutput>
{
    let mut outputs: Vec<TOutput> = vec![];
    // JSonArray outputs = obj.toArray();
    for an_output in obj
    {
        // JSonArray oo = an_output.toArray();
        let o: TOutput = TOutput {
            m_address: remove_quotes(&an_output[0]),
            m_amount: remove_quotes(&an_output[1]).parse::<CMPAIValueT>().unwrap(),
            m_output_character: "".to_string(),
            m_output_index: 0,
        };
        // new TOutput({oo[0].to_string(), static_cast<CMPAIValueT>(oo[1].toDouble())});
        outputs.push(o);
    }
    return outputs;
}