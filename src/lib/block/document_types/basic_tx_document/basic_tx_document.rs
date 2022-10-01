use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use substring::Substring;
use crate::{ccrypto, constants, cutils, dlog, machine};
use crate::cmerkle::{generate_m, MERKLE_VERSION};
use crate::constants::MAX_COINS_AMOUNT;
use crate::cutils::{remove_quotes};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::{Document, set_document_outputs};
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CDateT, CDocHashT, CMPAISValueT, CMPAIValueT, COutputIndexT, DocLenT, JSonArray, JSonObject, QV2DicT, VString, VVString};
use crate::lib::services::society_rules::society_rules::{get_base_price_per_char, get_doc_expense};
use crate::lib::transactions::basic_transactions::basic_transaction_template::{generate_bip69_input_tuples, generate_bip69_output_tuples};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{safe_stringify_unlock_set, stringify_inputs, stringify_outputs, TInput, TOutput};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;
use crate::lib::transactions::trx_utils::{normalize_inputs, normalize_outputs};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BasicTxDocument
{
    pub m_inputs: Vec<TInput>,
    pub m_outputs: Vec<TOutput>,
    pub m_data_and_process_payment_indexes: Vec<COutputIndexT>,  // dPIs
}

impl BasicTxDocument {
    pub fn new() -> Self {
        BasicTxDocument {
            m_inputs: vec![],
            m_outputs: vec![],
            m_data_and_process_payment_indexes: vec![],
        }
    }

    //old_name_was setByJsonObj
    pub fn set_doc_by_json_obj(&mut self, json_obj: &JSonObject) -> bool {
        self.m_data_and_process_payment_indexes = vec![];
        let mut inx: usize = 0;
        while !json_obj["dPIs"][inx].is_null() {
            self.m_data_and_process_payment_indexes
                .push(json_obj["dPIs"][inx]
                    .to_string()
                    .parse::<COutputIndexT>()
                    .unwrap());
            inx += 1;
        }

        if json_obj["inputs"].is_array() {
            self.set_document_inputs(&json_obj["inputs"]);
        }

        if json_obj["outputs"].is_array() {
            self.set_document_outputs(json_obj["outputs"].as_array().unwrap());
        }

        return true;
    }

    // //old_name_was safeStringifyDoc
    // pub fn safe_stringify_doc(&self, doc: &Document, ext_info_in_document: bool) -> String
    // {
    //     let document: JSonObject = self.export_doc_to_json(doc, ext_info_in_document);
    //     let res: String = cutils::controlled_json_stringify(&document);
    //     dlog(
    //         &format!("3 safe Stringify Doc({}): {}/{} length: {} the serialized document: {}",
    //                  cutils::hash8c(&doc.m_doc_hash),
    //                  doc.m_doc_type,
    //                  doc.m_doc_class,
    //                  cutils::sep_num_3(res.len() as i64),
    //                  res),
    //         constants::Modules::App,
    //         constants::SecLevel::TmpDebug);
    //
    //     return res;
    // }

    pub fn calc_doc_ext_info_hash(&self, doc: &Document) -> String
    {
        let mut hashes: VString = vec![];
        for an_ext_info in &doc.m_doc_ext_info
        {
            let hash_ables: String = format!(
                "uSet:{},signatures:{}",
                safe_stringify_unlock_set(&an_ext_info.m_unlock_set),
                serde_json::to_string(&an_ext_info.m_signatures).unwrap()
            );

            let the_hash = ccrypto::keccak256(&hash_ables);
            dlog(
                &format!(
                    "Doc Ext Root Hash Hash-ables {} Regenerated Ext hash: {}\nhash-ables: {}",
                    doc.get_doc_identifier(), the_hash, hash_ables),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
            hashes.push(the_hash);
        }
        let (
            root,
            _final_verifies,
            _version,
            _levels,
            _leaves) = generate_m(
            hashes,
            &"hashed".to_string(),
            &"keccak256".to_string(),
            &MERKLE_VERSION.to_string());
        return root;
    }

    pub fn calc_doc_data_and_process_cost(
        &self,
        doc: &Document,
        stage: &str,
        c_date: &CDateT,
        extra_length: DocLenT) -> (bool, CMPAIValueT)
    {
        let mut doc_len: DocLenT = doc.m_doc_length;
        if stage == constants::stages::CREATING
        {
            dlog(
                &format!(
                    "calc-doc-data-and-process-cost, doc-len: {}, calc-doc-length: {}",
                    doc_len, doc.calc_doc_length()),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }
        // if stage == constants::stages::CREATING
        // {
        //     doc_len += extra_length + constants::TRANSACTION_PADDING_LENGTH;
        // }

        if stage == constants::stages::VALIDATING
        {
            if doc_len != doc.calc_doc_length()
            {
                dlog(
                    &format!(
                        "The trx len and local re-calc len are not same in validating! stage({}) remote Len({}) local Len({}) {}",
                        stage, doc_len, doc.calc_doc_length(), doc.get_doc_identifier()),
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, 0);
            }
        } else {
            if doc_len < doc.calc_doc_length()
            {
                dlog(
                    &format!(
                        "The trx len and local re-calc len are not same! stage({}) remote Len({}) local Len({}) {}",
                        stage, doc_len, doc.calc_doc_length(), doc.get_doc_identifier()),
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, 0);
            }
        }

        if doc.m_doc_class == constants::trx_classes::P4P
        {
            // the transaction which new transaction is going to pay for
            doc_len = doc_len * self.get_dpis().len();
        }

        let mut the_cost: CMPAIValueT =
            (doc_len as CMPAIValueT *
                get_base_price_per_char(c_date) *
                get_doc_expense(
                    &doc.m_doc_type.as_str(),
                    doc_len,
                    &doc.m_doc_class.as_str(),
                    c_date)) as CMPAIValueT;

        if stage == constants::stages::CREATING
        {
            dlog(
                &format!(
                    "get-base-price-per-char: {}, get-doc-expense: {}",
                    get_base_price_per_char(c_date),
                    get_doc_expense(
                        &doc.m_doc_type.as_str(),
                        doc_len,
                        &doc.m_doc_class.as_str(),
                        c_date)),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }

        if stage == constants::stages::CREATING
        {
            the_cost = the_cost * machine().get_machine_service_interests(
                &doc.m_doc_type,
                &doc.m_doc_class,
                doc_len,
                extra_length,
                doc.get_dpis().len() as u8) as CMPAIValueT;
        }
        return (true, the_cost as CMPAIValueT);
    }

    /*
        bool BasicTxDocument::deleteInputs()
        {
          for (TInput* an_input: m_inputs)
            delete an_input;
          return true;
        }

        bool BasicTxDocument::deleteOutputs()
        {
          for (TOutput* an_output: m_outputs)
            delete an_output;
          return true;
        }

        */
    //old_name_was exportDocToJson
    pub fn export_doc_to_json(&self, doc: &Document, ext_info_in_document: bool) -> JSonObject
    {
        let mut document: JSonObject = doc.export_doc_to_json_super(ext_info_in_document);

        // // impacting uSets
        // if ext_info_in_document
        // {
        //     document["dExtInfo"] = compact_unlockers_array(&document["dExtInfo"]);
        // }

        if self.m_data_and_process_payment_indexes.len() > 0
        {
            document["dPIs"] = self.m_data_and_process_payment_indexes.clone().into();
        }

        return document;
    }


    // pub fn exportInputsToJson(&self) ->(bool, Vec<JSonObject>)
    // {
    // let mut inputs:Vec<JSonObject>=vec![];
    // for an_input in  self.m_inputs
    // {
    //     inputs.push(JSonArray {
    //         an_input.m_transaction_hash,
    //         an_input.m_output_index,
    //     });
    //
    // }
    //     return {true, inputs};
    // }


    /*
        std::tuple<bool, JSonArray> BasicTxDocument::exportOutputsToJson() const
        {
        JSonArray outputs {};
        for (TOutput* an_output: m_outputs)
        outputs.push(JSonArray{
          an_output.m_address,
          QVariant::fromValue(an_output.m_amount).toDouble()});
        return {true, outputs};
        }
    */

    pub fn get_doc_hashable_string(&self, doc: &Document) -> String
    {
        let doc_hash_ables: String = format!(
            "dCDate:{},dClass:{},dComment:{},dExtHash:{},dLen:{},dPIs:{},dRef:{},dType:{},dVer:{},inputs:{},outputs:{}",
            doc.m_doc_creation_date,
            doc.m_doc_class,
            doc.m_doc_comment,
            doc.m_doc_ext_hash,
            cutils::padding_length_value(doc.m_doc_length.to_string(), constants::LEN_PROP_LENGTH),
            serde_json::to_string(&self.m_data_and_process_payment_indexes).unwrap(),
            doc.get_doc_ref(),
            doc.m_doc_type,
            doc.m_doc_version,
            stringify_inputs(self.get_inputs()),
            stringify_outputs(self.get_outputs())
        );
        return doc_hash_ables;
    }

    // TODO: some unit test for pure hashable
    //old_name_was extractHPureParts_simple
    pub fn extract_hash_pure_parts_simple(&self, _doc: &Document) -> String
    {
        // the hTrx MUST be constant and NEVER change the order of attribiutes (alphabetical)
        // in case of change the version MUST be changed and the code treats it in new manner
        let normalized_inputs: Vec<TInput> = normalize_inputs(&self.m_inputs);


        let (status, normalized_outputs) = normalize_outputs(&self.m_outputs);
        if !status {
            return "".to_string();
        }

        let mut doc_hahsables: String = "{".to_string();
        doc_hahsables += &*("\"inputs\":".to_owned() + &stringify_inputs(&normalized_inputs) + ",");
        doc_hahsables += &*("\"outputs\":".to_owned() + &stringify_outputs(&normalized_outputs) + "}");

        return doc_hahsables.to_string();
    }

    //old_name_was extractTransactionPureHashableParts
    pub fn extract_transaction_pure_hashable_parts(&self, doc: &Document) -> String
    {
        if doc.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.extract_hash_pure_parts_simple(doc);
        }
        return "".to_string();
    }

    //old_name_was getPureHash
    pub fn get_pure_hash(&self, doc: &Document) -> String
    {
        let hashable_string = self.extract_transaction_pure_hashable_parts(doc);
        let the_hash = ccrypto::keccak256_dbl(&hashable_string);    // NOTE: useing double hash for more security
        dlog(
            &format!("get Pure Hash res! hash({}) version({}) hashable string: ({}) trx({})",
                     cutils::hash8c(&the_hash), doc.m_doc_version, hashable_string, cutils::hash8c(&doc.get_doc_hash())),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        return the_hash;
    }

    pub fn calc_doc_hash(&self, doc: &Document) -> String
    {
        let hashables = self.get_doc_hashable_string(doc);
        let mut the_hash = ccrypto::keccak256_dbl(&hashables); // NOTE: absolutely using double hash for more security

        // generate deterministic part of trx hash
        let pure_hash = self.get_pure_hash(doc);
        the_hash = pure_hash.substring(32, 64).to_string() + &*the_hash.substring(32, 64).to_string();

        dlog(
            &format!("\nHashable string for Basic Trx doc({} / {}) hash({}) version({}) hashables: {}",
                     doc.m_doc_type, doc.m_doc_class, cutils::hash8c(&the_hash), doc.m_doc_version, hashables),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return the_hash;
    }

    /*
        bool BasicTxDocument::applyDocFirstImpact(const Block& block) const
        {
        return false;
        }

        String BasicTxDocument::get_ref() const
        {
        return m_doc_ref;
        }

    */
    pub fn get_dpis(&self) -> &Vec<COutputIndexT>
    {
        return &self.m_data_and_process_payment_indexes;
    }

    //old_name_was getInputs
    pub fn get_inputs(&self) -> &Vec<TInput>
    {
        return &self.m_inputs;
    }

    //old_name_was getOutputs
    pub fn get_outputs(&self) -> &Vec<TOutput>
    {
        return &self.m_outputs;
    }

    //old_name_was setDocumentInputs
    pub fn set_document_inputs(&mut self, obj: &JSonArray) -> bool
    {
        // JSonArray inputs = obj.toArray();
        for an_input in obj.as_array().unwrap() {
            // JSonArray io = an_input.toArray();
            println!("ffffff 55: {}", an_input);
            let input_dtl = an_input.as_array().unwrap();
            println!("ffffff 56: {:?}", input_dtl);
            println!("ffffff output_index 1: {:?}", &input_dtl[1]);
            let output_index = remove_quotes(&input_dtl[1]);
            println!("ffffff output_index 2: {}", output_index);
            let inp_entry: TInput = TInput {
                m_transaction_hash: remove_quotes(&input_dtl[0]),
                m_output_index: remove_quotes(&input_dtl[1]).parse::<COutputIndexT>().unwrap(),
                m_owner: "".to_string(),
                m_amount: 0,
                m_private_keys: vec![],
                m_unlock_set: UnlockSet::new(),
            };
            // ({io[0].to_string(), static_cast< COutputIndexT > (io[1].toVariant().toDouble())});
            self.m_inputs.push(inp_entry);
        }
        return true;
    }

    //old_name_was setDocumentOutputs
    pub fn set_document_outputs(&mut self, obj: &Vec<JSonObject>) -> bool
    {
        self.m_outputs = set_document_outputs(obj);
        // // JSonArray outputs = obj.toArray();
        // for an_output in obj.as_array().unwrap() {
        //     // JSonArray oo = an_output.toArray();
        //     let o: TOutput = TOutput {
        //         m_address: an_output[0].to_string(),
        //         m_amount: an_output[1].as_u64().unwrap(),
        //         m_output_character: "".to_string(),
        //         m_output_index: 0,
        //     };
        //     // new TOutput({oo[0].to_string(), static_cast<CMPAIValueT>(oo[1].toDouble())});
        //     self.m_outputs.push(o);
        // }
        return true;
    }

    /*

        bool BasicTxDocument::appendOutput(
        CAddressT address,
        CMPAIValueT value)
        {
        TOutput *o  = new TOutput({address, static_cast<CMPAIValueT>(value)});
        m_outputs.push(o);
        return true;
        }

        bool BasicTxDocument::removeOutputByIndex(COutputIndexT index)
        {
        delete m_outputs[index];
        m_outputs.erase(m_outputs.begin() + index, m_outputs.begin() + index + 1);
        return true;
        }


        int64_t BasicTxDocument::getOutputIndexByAddressValue( address_plus_value:&String)
        {
        for (COutputIndexT inx = 0; inx < m_outputs.len(); inx++)
        {
        if (m_outputs[inx].m_address + m_outputs[inx].m_amount == address_plus_value)
          return inx;
        }
        return -1;
        }

        JSonArray generate_bip69_input_tuples()
        {
        JSonArray input_tuples {};
        for(TInput* the_in: m_inputs)
        {
        JSonArray elm {the_in.m_transaction_hash, QVariant::fromValue(the_in.m_output_index).toDouble()};
        input_tuples.push(elm);
        }
        return input_tuples;
        }

        JSonArray BasicTxDocument::generate_bip69_output_tuples()
        {
        JSonArray output_tuples {};
        for(TOutput* out: m_outputs)
        {
        JSonArray elm {out.m_address, QVariant::fromValue(out.m_amount).toDouble()};
        output_tuples.push(elm);
        }
        return output_tuples;
        }

*/
    // old name was hasSignable
    pub fn has_sign_ables(&self, _doc: &Document) -> bool
    {
        return true;
    }

    //old_name_was getInputOutputSignables
    pub fn get_input_output_signables1(
        &self,
        doc: &Document,
        sig_hash: &String,
        c_date: &CDateT) -> String
    {
        let mut inp_dict: HashMap<CCoinCodeT, TInput> = HashMap::new();
        for an_input in &self.m_inputs
        {
            inp_dict.insert(an_input.get_coin_code(), an_input.clone());
        }
        let inputs: VVString = generate_bip69_input_tuples(&inp_dict);

        let (outputs, _ordered_tpl_outputs) = generate_bip69_output_tuples(&self.m_outputs);
        return Self::get_input_output_signables2(
            constants::document_types::BASIC_TX,
            &constants::DEFAULT_DOCUMENT_VERSION.to_string(),
            c_date,
            &inputs,
            &outputs,
            sig_hash,
            &doc.get_doc_ref());
    }

    //old_name_was getInputOutputSignables
    pub fn get_input_output_signables2(
        doc_type: &str,
        doc_version: &String,
        c_date: &CDateT,
        inputs: &VVString,
        outputs: &VVString,
        sig_hash: &String,
        document_ref: &CDocHashT) -> String
    {
        let (inputs, outputs) = Self::extract_input_outputs_based_on_sig_hash(
            &inputs,
            &outputs,
            sig_hash);

        let doc_sign_ables: String = format!(
            "dCDate:{},dRef:{},dType:{},dVer:{},inputs:{},outputs:{},sigHash:{}",
            c_date,
            document_ref,
            doc_type,
            doc_version,
            serde_json::to_string(&inputs).unwrap(),
            serde_json::to_string(&outputs).unwrap(),
            sig_hash);
        return doc_sign_ables;
    }

    //old_name_was signingInputOutputs
    pub fn signing_inputs_and_outputs(
        private_key: &String,
        inputs: &VVString,
        outputs: &VVString,
        sig_hash: &String,
        document_ref: &CDocHashT,
        c_date: &CDateT) -> (bool, String)
    {
        let sign_ables: String = BasicTxDocument::get_input_output_signables2(
            constants::document_types::BASIC_TX,
            &constants::DEFAULT_DOCUMENT_VERSION.to_string(),
            c_date,
            inputs,
            outputs,
            sig_hash,
            document_ref);
        let the_hash = ccrypto::keccak256_dbl(&sign_ables); // because of securiy, MUST use double hash
        dlog(
            &format!(
                "trx sign_ables: {} {}",
                sign_ables, the_hash),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        let (status, signature_hex, _signature) =
            ccrypto::ecdsa_sign_message(
                private_key,
                &the_hash
                    .substring(
                        0,
                        constants::SIGN_MSG_LENGTH as usize)
                    .to_string());
        if !status
        {
            dlog(
                &format!(
                    "Failed in sign trx sign_ables: {}",
                    sign_ables),
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, "Failed in sign trx sign_ables".to_string());
        }
        dlog(
            &format!(
                "YYY-The transaction has been signed. signature({}) sign-ables-hash({}) sign-ables-clear: {}",
                signature_hex, the_hash, sign_ables),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        return (true, signature_hex);
    }

    // old name was canHaveTimeLockedInput
    pub fn can_have_time_locked_inputs(&self) -> bool
    {
        return false;
        // // TODO: implement input time lock (both flexible and strict input timelock) ASAP
        // return (
        // (self.m_doc_type == constants::document_types::BASIC_TX) &&
        // VString{constants::TRX_CLASSES::SimpleTx, constants::TRX_CLASSES::P4P}.contains(m_doc_class)
        // );
    }

    /*
        std::tuple<Vec<TInput*>, Vec<TOutput*> > extract_input_outputs_based_on_sig_hash(String sig_hash)
        {
        if (sig_hash == constants::SIGHASH::ALL)
        return {m_inputs, m_outputs};

        if (sig_hash == constants::SIGHASH::NONE)
        return {m_inputs, {}};

        return {m_inputs, m_outputs};
        }

        std::tuple<Vec<TInput*>, Vec<TOutput*> >extract_input_outputs_based_on_sig_hash(
        Vec<TInput*> inputs,
        Vec<TOutput*> outputs,
        String sig_hash)
        {
        if (sig_hash == constants::SIGHASH::ALL) {
        // change nothing
        } else if (sig_hash == constants::SIGHASH::NONE) {
        outputs = {};
        }
        return { inputs, outputs };
        }
    */

    // old name was validateGeneralRulsForTransaction
    pub fn validate_general_rules_for_transaction(&self, doc: &Document) -> bool
    {
        //  let localHash = trxHashHandler.doHashTransaction(transaction);
        //  if (localHash != transaction.hash) {
        //    msg = `Mismatch trx hash locally recalculated:${localHash} received: ${transaction.hash} block(${utils.hash6c(blockHash)})`;
        //    return { err: true, msg: msg };
        //  }


        if cutils::is_greater_than_now(&doc.m_doc_creation_date)
        {
            dlog(
                &format!("Transaction with future creation date is not acceptable {}", doc.get_doc_identifier()),
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return false;
        }

        return true;
    }

    // old name was equationCheck
    pub fn equation_check(
        &self,
        doc: &Document,
        used_coins_dict: &QV2DicT,
        invalid_coins_dict: &QV2DicT,
        block_hash: &CBlockHashT) -> (bool, String, CMPAIValueT, CMPAIValueT)
    {
        let msg: String;
        let mut total_inputs_amounts: CMPAIValueT = 0;
        let mut total_outputs_amounts: CMPAIValueT = 0;

        if self.m_inputs.len() > 0
        {
            for an_input in &self.m_inputs
            {
                let a_coin_code: CCoinCodeT = an_input.get_coin_code();
                if used_coins_dict.contains_key(&a_coin_code)
                {
                    if used_coins_dict[&a_coin_code]["coin_value"].parse::<CMPAIValueT>().unwrap()
                        >= MAX_COINS_AMOUNT
                    {
                        msg = format!(
                            "The transaction has input bigger than MAX_SAFE_INTEGER! {} Block({})  value: {}",
                            doc.get_doc_identifier(), cutils::hash8c(block_hash),
                            cutils::nano_pai_to_pai(used_coins_dict[&a_coin_code]["coin_value"].parse::<CMPAISValueT>().unwrap()));
                        dlog(
                            &msg,
                            constants::Modules::Sec,
                            constants::SecLevel::Error);

                        return (false, msg, 0, 0);
                    }
                    total_inputs_amounts += used_coins_dict[&a_coin_code]["coin_value"]
                        .parse::<CMPAIValueT>()
                        .unwrap();
                } else {
                    // * trx uses already spent outputs! so try invalid_coins_dict
                    // * probably it is a double-spend case, which will be decided after 12 hours, in importing step
                    // * BTW ALL trx must have balanced equation (even double-spends)
                    if invalid_coins_dict.contains_key(&a_coin_code)
                    {
                        if invalid_coins_dict[&a_coin_code]["coinGenOutputValue"]
                            .parse::<CMPAIValueT>()
                            .unwrap() >= MAX_COINS_AMOUNT
                        {
                            msg = format!(
                                "The transaction has inv-input bigger than MAX_SAFE_INTEGER! {} Block({})  value: {}",
                                doc.get_doc_identifier(),
                                cutils::hash8c(block_hash),
                                cutils::nano_pai_to_pai(invalid_coins_dict[&a_coin_code]["coinGenOutputValue"].parse::<CMPAISValueT>().unwrap()));
                            dlog(
                                &msg,
                                constants::Modules::Sec,
                                constants::SecLevel::Error);
                            return (false, msg, 0, 0);
                        }
                        total_inputs_amounts += invalid_coins_dict[&a_coin_code]["coinGenOutputValue"].parse::<CMPAIValueT>().unwrap();
                    } else {
                        msg = format!(
                            "The input absolutely missed! not in tables neither in DAG! coin({}) {} Block({})",
                            a_coin_code,
                            doc.get_doc_identifier(),
                            cutils::hash8c(block_hash)
                        );
                        dlog(
                            &msg,
                            constants::Modules::Sec,
                            constants::SecLevel::Error);
                        return (false, msg, 0, 0);
                    }
                }
            }
        }

        if self.m_outputs.len() > 0
        {
            for output in &self.m_outputs
            {
                if output.m_address != cutils::strip_output_address(&output.m_address)
                {
                    msg = format!(
                        "The transaction has not digit characters in bech 32 address({})! {} Block({})",
                        output.m_address,
                        doc.get_doc_identifier(),
                        cutils::hash8c(block_hash)
                    );
                    dlog(
                        &msg,
                        constants::Modules::Sec,
                        constants::SecLevel::Error);
                    return (false, msg, 0, 0);
                }

                if output.m_amount == 0
                {
                    msg = format!(
                        "The transaction has zero output! {} Block({})",
                        doc.get_doc_identifier(),
                        cutils::hash8c(block_hash));
                    dlog(
                        &msg,
                        constants::Modules::Sec,
                        constants::SecLevel::Error);
                    return (false, msg, 0, 0);
                }

                // if output.m_amount < 0
                // {
                //     msg = format!(
                //         "The transaction has negative output! {} Block({})",
                //         doc.get_doc_identifier(),
                //         cutils::hash8c(block_hash));
                //     dlog(
                //         &msg,
                //         constants::Modules::Sec,
                //         constants::SecLevel::Error);
                //     return (false, msg, 0, 0);
                // }

                if output.m_amount >= MAX_COINS_AMOUNT
                {
                    msg = format!(
                        "The transaction has output bigger than MAX_SAFE_INTEGER! {} Block({})",
                        doc.get_doc_identifier(),
                        cutils::hash8c(block_hash));
                    dlog(
                        &msg,
                        constants::Modules::Sec,
                        constants::SecLevel::Error);
                    return (false, msg, 0, 0);
                }

                total_outputs_amounts += output.m_amount;
            }
        }

        return (true, "Done".to_string(), total_inputs_amounts, total_outputs_amounts);
    }


    /*
                CMPAISValueT BasicTxDocument::getDocCosts() const
                {
                CMPAISValueT costs = 0;
                for (auto a_cost_index: get_dpis())
                costs += m_outputs[a_cost_index].m_amount;
                return costs;
                }
        */

    //old_name_was extractInputOutputs
    pub fn extract_input_outputs_based_on_sig_hash<I, O>(
        inputs: &Vec<I>,
        outputs: &Vec<O>,
        sig_hash: &String) -> (Vec<I>, Vec<O>)
        where I: Clone, O: Clone
    {
        if sig_hash == constants::sig_hashes::ALL
        {
            // change nothing
            return (inputs.clone(), outputs.clone());
        } else if sig_hash == constants::sig_hashes::NONE
        {
            let out: Vec<O> = vec![];
            return (inputs.clone(), out);
        }
        return (inputs.clone(), outputs.clone());

        ////TODO: implement CUSTOM
        //case iConsts.SIGHASH.CUSTOM:
        //    let cust = {
        //        inputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4'],
        //        outputs: ['ALL']
        //    }
        //    cust = {
        //        inputs: 'ALL',
        //        outputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4']
        //    }
        //    cust = {
        //        inputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4'],
        //        outputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4']
        //    }
        //    inputs = args.inputs;
        //    outputs = args.outputs;
        //    break;
        // since they have conflict with BIP69 all are disabled
        // instead, use custom
        // case iConsts.SIGHASH['SINGLE']:
        //     inputs = args.inputs;
        //     outputs = [args.outputs[args.selectedIndex]];
        //     break;
        // case iConsts.SIGHASH['ALL|ANYONECANPAY']:
        //     inputs = [args.inputs[args.selectedIndex]];
        //     outputs = args.outputs;
        //     break;
        // case iConsts.SIGHASH['NONE|ANYONECANPAY']:
        //     inputs = [args.inputs[args.selectedIndex]];
        //     outputs = [];
        //     break;
        // case iConsts.SIGHASH['SINGLE|ANYONECANPAY']:
        //     inputs = [args.inputs[args.selectedIndex]];
        //     outputs = [args.outputs[args.selectedIndex]];
        //     break;
    }

    // old name was customValidateDoc
    pub fn custom_validate_doc(&self, doc: &Document, _block: &Block) -> (bool, String)
    {
        let msg: String;
        if doc.m_doc_class == constants::trx_classes::P4P
        {
            if !constants::SUPPORTS_P4P_TRANSACTION
            {
                msg = "Network! still doen't support P4P transactions!".to_string();
                dlog(
                    &msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Fatal);
                return (false, msg);
            }
        }

        for an_output in &self.m_outputs
        {
            if an_output.m_amount == 0
            {
                msg = format!("creating block, the transaction has zero output! {}", doc.get_doc_identifier());
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, msg);
            }
        }

        return (true, "custom validate trx done".to_string());
    }
}