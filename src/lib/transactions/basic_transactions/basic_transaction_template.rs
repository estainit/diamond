use std::collections::HashMap;
use serde_json::{json, Value};
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::document_types::basic_tx_document::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::custom_types::{CAddressT, CCoinCodeT, CDateT, CDocHashT, CMPAISValueT, CMPAIValueT, COutputIndexT, JSonArray, VString, VVString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{make_inputs_tuples, make_outputs_tuples, TInput, TOutput};

#[derive(Debug)]
pub struct BasicTransactionTemplate
{
    pub m_tpl_inputs: HashMap<CCoinCodeT, TInput>,
    pub m_tpl_outputs: Vec<TOutput>,

    pub m_tpl_fee_calc_method: String,
    pub m_tpl_pre_calculated_dp_cost: CMPAIValueT,

    pub m_tpl_comment: String,
    pub m_tpl_doc_ref: CDocHashT,

    pub m_tpl_change_back_output_index: i64,
    pub m_tpl_change_back_address: CAddressT,

    pub m_tpl_backers_addresses: VString,

    pub m_tpl_sig_hash: String,

    pub m_tpl_creation_date: CDateT,
    pub m_tpl_doc_type: String,
    pub m_tpl_doc_class: String,
    pub m_tpl_doc_ver: String,
    pub m_tpl_doc_ext_info: Vec<DocExtInfo>,
    pub m_tpl_doc_ext_hash: CDocHashT,

    // for test purpose
    pub m_tpl_do_unequal: bool,
}

impl BasicTransactionTemplate {
    pub fn new(
        inputs: HashMap<CCoinCodeT, TInput>,
        outputs: Vec<TOutput>,
        fee_calc_method: &String,
        desired_trx_fee: CMPAIValueT,
        d_comment: String,
    ) -> Self {
        let creation_date = application().now();
        Self {
            m_tpl_inputs: inputs,
            m_tpl_outputs: outputs,
            m_tpl_fee_calc_method: fee_calc_method.clone(),
            m_tpl_pre_calculated_dp_cost: desired_trx_fee,
            m_tpl_comment: d_comment,
            m_tpl_doc_ref: "".to_string(),
            m_tpl_change_back_output_index: 0,
            m_tpl_change_back_address: "".to_string(),
            m_tpl_backers_addresses: vec![],
            m_tpl_sig_hash: constants::sig_hashes::ALL.to_string(),
            m_tpl_creation_date: creation_date,
            m_tpl_doc_type: constants::document_types::BASIC_TX.to_string(),
            m_tpl_doc_class: constants::trx_classes::SIMPLE_TX.to_string(),
            m_tpl_doc_ver: "0.0.2".to_string(),
            m_tpl_doc_ext_info: vec![],
            m_tpl_doc_ext_hash: "".to_string(),
            m_tpl_do_unequal: false,
        }
    }


    /*

      static const CMPAIValueT TEMP_DPCOST_AMOUNT;


    class BasicTransactionHandler
    {
    public:
      BasicTransactionHandler();



    bool BasicTransactionTemplate::setChangabackOutputIndex()
    {
      for (auto an_out: m_tpl_outputs)
        if (an_out.m_output_character == constants::OUTPUT_CHANGE_BACK)
        {
          m_tpl_change_back_output_index = an_out.m_output_index;
          return true;
        }

      return false;
    }
*/

    //old_name_was generateDocOutputTuples
    pub fn generate_doc_output_tuples(&mut self) -> Vec<TOutput>
    {
        let mut final_out: Vec<TOutput> = vec![];
        let (outputs, ordered_tpl_outputs) = generate_bip69_output_tuples(&self.m_tpl_outputs);
        self.m_tpl_outputs = ordered_tpl_outputs;
        for an_output in outputs
        {
            let output: TOutput = TOutput {
                m_address: an_output[0].clone(),
                m_amount: an_output[1].parse::<CMPAIValueT>().unwrap(),
                m_output_character: "".to_string(),
                m_output_index: 0,
            };
            final_out.push(output);
        }
        return final_out;
    }

    //old_name_was getInputCoinsCodes
    pub fn get_input_coins_codes(&self) -> VString
    {
        let mut the_coins_codes = self.m_tpl_inputs.keys().cloned().collect::<VString>();
        the_coins_codes.sort(); //BIP69  inputs
        return the_coins_codes;
    }

    //old_name_was canAffordCosts
    pub fn can_afford_costs(
        &self,
        calculate_dp_cost: CMPAIValueT) -> (bool, String, CMPAIValueT)
    {
        let msg: String;
        if self.m_tpl_pre_calculated_dp_cost > 0
        {
            if calculate_dp_cost > self.m_tpl_pre_calculated_dp_cost
            {
                msg = format!(
                    "Transaction calculated cost({} PAIs) is higher than your desire({} PAIs)",
                    cutils::nano_pai_to_pai(calculate_dp_cost as CMPAISValueT),
                    cutils::nano_pai_to_pai(self.m_tpl_pre_calculated_dp_cost as CMPAISValueT));
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, msg, 0);
            } else {
                return (true, "Done".to_string(), self.m_tpl_pre_calculated_dp_cost);
            }
        }

        if (self.m_tpl_fee_calc_method == constants::transaction_fee_calculate_methods::EXACT_FEE)
            && (calculate_dp_cost > self.m_tpl_pre_calculated_dp_cost)
        {
            msg = format!(
                "Transaction locally recalculate trx dp cost({} PAIs) is higher than your max({} PAIs)",
                cutils::nano_pai_to_pai(calculate_dp_cost as CMPAISValueT),
                cutils::nano_pai_to_pai(self.m_tpl_pre_calculated_dp_cost as CMPAISValueT));
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, msg, 0);
        }

        return (true, "Done".to_string(), calculate_dp_cost);
    }

    //old_name_was appendTmpDPCostOutPut
    pub fn append_tmp_dp_cost_output(
        &self,
        current_outputs: &mut Vec<TOutput>,
        address: &CAddressT)
    {
        current_outputs.push(
            TOutput {
                m_address: address.clone(),
                m_amount: constants::TEMP_DP_COST_AMOUNT,
                m_output_character: constants::OUTPUT_DP_COST.to_string(),
                m_output_index: 0,
            }
        );
    }

    //old_name_was getDPCostIndexes
    pub fn get_dp_cost_indexes(&self) -> Vec<COutputIndexT>
    {
        let mut indexes: Vec<COutputIndexT> = vec![];
        for an_out in &self.m_tpl_outputs
        {
            if an_out.m_output_character == constants::OUTPUT_DP_COST.to_string()
            {
                indexes.push(an_out.m_output_index);
            }
        }
        return indexes;
    }


    //old_name_was signAndCreateDExtInfo
    pub fn sign_and_create_doc_ext_info(&mut self) -> bool
    {
        self.m_tpl_doc_ext_info = vec![];
        let input_tuples = generate_bip69_input_tuples(&self.m_tpl_inputs);
        let (output_tuples, ordered_tpl_outputs) = generate_bip69_output_tuples(&self.m_tpl_outputs);
        println!("llllll output_tuples: {:?}", output_tuples);
        self.m_tpl_outputs = ordered_tpl_outputs;
        for a_coin in &self.get_input_coins_codes()
        {
            let mut signatures: VVString = vec![];
            for a_private_key in &self.m_tpl_inputs[a_coin].m_private_keys
            {
                let (status, signature) = BasicTxDocument::signing_inputs_and_outputs(
                    a_private_key,
                    &input_tuples,
                    &output_tuples,
                    &self.m_tpl_sig_hash,
                    &self.m_tpl_doc_ref,
                    &self.m_tpl_creation_date);
                if !status
                {
                    return false;
                }
                signatures.push(vec![signature.clone(), self.m_tpl_sig_hash.clone()]);
                println!("sssssssss asv2 {}", signature);
                panic!("5555555");
            }
            let d_ext = DocExtInfo {
                m_unlock_set: self.m_tpl_inputs[a_coin].m_unlock_set.clone(),
                m_signatures: signatures,
            };
            self.m_tpl_doc_ext_info.push(d_ext);
        }

        return true;
    }

    // old name was setDPcostAmount
    pub fn set_dp_cost_amount(&mut self, amount: CMPAIValueT) -> bool
    {
        let mut inx: COutputIndexT = 0;
        while inx < self.m_tpl_outputs.len() as COutputIndexT
        {
            if self.m_tpl_outputs[inx as usize].m_output_character == constants::OUTPUT_DP_COST.to_string()
            {
                self.m_tpl_outputs[inx as usize].m_amount = amount;
            }
            inx += 1;
        }
        return true;
    }

    //old_name_was removeChangebackOutput
    pub fn remove_change_back_output(&mut self) -> bool
    {
        let mut new_outputs: Vec<TOutput> = vec![];
        for an_out in &self.m_tpl_outputs
        {
            if an_out.m_output_character != constants::OUTPUT_CHANGE_BACK
            {
                new_outputs.push(an_out.clone());
            }
        }
        self.m_tpl_outputs = new_outputs;
        return true;
    }

    //old_name_was setChangebackOutputAmount
    pub fn set_change_back_output_amount(&mut self, amount: CMPAIValueT) -> bool
    {
        let mut change_back_existed: bool = false;
        let mut inx: COutputIndexT = 0;
        while inx < self.m_tpl_outputs.len() as COutputIndexT
        {
            if self.m_tpl_outputs[inx as usize].m_output_character == constants::OUTPUT_CHANGE_BACK
            {
                self.m_tpl_outputs[inx as usize].m_amount = amount;
                change_back_existed = true;
            }
            inx += 1;
        }
        if change_back_existed
        {
            return true;
        }

        // create a new output
        if self.m_tpl_change_back_address == ""
        {
            self.m_tpl_change_back_address = machine().get_backer_address();
        }

        self.m_tpl_outputs.push(
            TOutput {
                m_address: self.m_tpl_change_back_address.clone(),
                m_amount: amount,
                m_output_character: constants::OUTPUT_CHANGE_BACK.to_string(),
                m_output_index: 0,
            });

        return true;
    }
}

pub fn export_doc_ext_to_json(ext_info: &Vec<DocExtInfo>) -> JSonArray
{
    let mut out_js_arr: JSonArray = json!([]);
    for an_ext in ext_info
    {
        out_js_arr.as_array_mut().unwrap().push(an_ext.export_to_json());
    }
    out_js_arr
}

pub fn convert_json_to_doc_ext(ext_info: &Vec<Value>) -> Vec<DocExtInfo>
{
    let mut out: Vec<DocExtInfo> = vec![];
    for an_ext in ext_info
    {
        let (status, d_ext) = DocExtInfo::load_from_json(&an_ext);
        if !status
        {
            dlog(
                &format!("Failed in DocExtInfo::load-from-json: {}", an_ext),
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return vec![];
        }
        out.push(d_ext);
    }
    out
}

//old_name_was generateInputTuples
pub fn generate_bip69_input_tuples(tpl_inputs: &HashMap<CCoinCodeT, TInput>) -> VVString
{
    let mut the_coins_codes: VString = tpl_inputs
        .keys()
        .cloned()
        .collect::<VString>();

    the_coins_codes.sort(); //BIP69  inputs

    let mut sorted_inputs: Vec<TInput> = vec![];
    for a_coin in &the_coins_codes
    {
        sorted_inputs.push(tpl_inputs[a_coin].clone());
    }
    return make_inputs_tuples(&sorted_inputs);
}


//old_name_was generateOutputTuples
pub fn generate_bip69_output_tuples(tpl_outputs: &Vec<TOutput>) -> (VVString, Vec<TOutput>)
{
    let (status, tpl_outputs) = set_output_indexes(tpl_outputs);
    if !status
    {
        return (vec![], vec![]);
    }
    return (make_outputs_tuples(&tpl_outputs), tpl_outputs);
}

//old_name_was setOutputIndexes
pub fn set_output_indexes(tpl_outputs: &Vec<TOutput>) -> (bool, Vec<TOutput>)
{
    // BIP 69 for outputs
    let mut normalized_outputs: Vec<TOutput> = vec![];

    if tpl_outputs.len() < 1
    { return (false, normalized_outputs); }

    let mut dict: HashMap<String, TOutput> = HashMap::new();
    let mut dummy_inx: u64 = 0;
    let mut keys: VString = vec![];
    for an_output in tpl_outputs
    {
        let key: String = vec![
            cutils::padding_left(&an_output.m_amount.to_string(), 24),
            an_output.m_address.clone(),
            cutils::padding_left(&dummy_inx.to_string(), 5)]
            .join(",");

        dict.insert(key.clone(), an_output.clone());
        keys.push(key);
        dummy_inx += 1;
    }

    keys.sort();
    keys.reverse();
    let mut real_output_inx: COutputIndexT = 0;
    for a_key in &keys
    {
        let mut tmp_out = dict[a_key].clone();
        tmp_out.m_output_index = real_output_inx;
        normalized_outputs.push(tmp_out);
        real_output_inx += 1;
    }

    return (true, normalized_outputs);
}