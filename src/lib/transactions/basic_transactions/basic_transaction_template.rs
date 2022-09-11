use std::collections::HashMap;
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::document_types::basic_tx_document::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::custom_types::{CAddressT, CCoinCodeT, CDateT, CDocHashT, CMPAISValueT, CMPAIValueT, COutputIndexT, VString, VVString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{make_inputs_tuples, make_outputs_tuples, TInput, TOutput};

#[derive(Debug)]
pub struct BasicTransactionTemplate
{
    pub m_tpl_inputs: HashMap<CCoinCodeT, TInput>,
    pub m_tpl_outputs: Vec<TOutput>,

    pub m_tpl_max_dp_cost: CMPAIValueT,
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
        max_dp_cost: CMPAIValueT,
        desired_trx_fee: CMPAIValueT,
        d_comment: String,
    ) -> Self {
        let creation_date = application().now();
        BasicTransactionTemplate {
            m_tpl_inputs: inputs,
            m_tpl_outputs: outputs,
            m_tpl_max_dp_cost: max_dp_cost,
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
                    cutils::micro_pai_to_pai_6(&(calculate_dp_cost as CMPAISValueT)),
                    cutils::micro_pai_to_pai_6(&(self.m_tpl_pre_calculated_dp_cost as CMPAISValueT)));
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, msg, 0);
            } else {
                return (true, "Done".to_string(), self.m_tpl_pre_calculated_dp_cost);
            }
        }

        if (self.m_tpl_max_dp_cost > 0) && (calculate_dp_cost > self.m_tpl_max_dp_cost)
        {
            msg = format!(
                "Transaction locally recalculate trx dp cost({} PAIs) is higher than your max({} PAIs)",
                cutils::micro_pai_to_pai_6(&(calculate_dp_cost as CMPAISValueT)),
                cutils::micro_pai_to_pai_6(&(self.m_tpl_max_dp_cost as CMPAISValueT)));
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
        // let mut dExtInfo: Vec<DocExtInfo> = vec![];
        let input_tuples = generate_bip69_input_tuples(&self.m_tpl_inputs);
        let (output_tuples, ordered_tpl_outputs) = generate_bip69_output_tuples(&self.m_tpl_outputs);
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
                signatures.push(vec![signature, self.m_tpl_sig_hash.clone()]);
            }
            let d_ext = DocExtInfo {
                m_unlock_set: self.m_tpl_inputs[a_coin].m_unlock_set.clone(),
                m_signatures: signatures,
            };
            self.m_tpl_doc_ext_info.push(d_ext);
            //         QJsonObject {
            //         {
            //             "uSet",
            //             m_tpl_inputs[a_coin].m_unlock_set,
            //         },
            //         { "signatures", signatures }
            // });
        }
        // self.m_tpl_doc_ext_info = dExtInfo;

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

        // creat a new output
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

    /*
        String BasicTransactionTemplate::dumpMe()
        {
          String out = "\n";
          for (auto an_input: m_tpl_inputs)
            out += "\n" + an_input.dumpMe();

          out += "\nOutputs: ";
          for (auto an_output: m_tpl_outputs)
            out += "\n" + an_output.m_address + "(" + an_output.m_output_character + " / " + String::number(an_output.m_output_index) + ") \t " + cutils::microPAIToPAI6(an_output.m_amount);

          out += "\nmax DP cost: " + cutils::microPAIToPAI6(m_tpl_max_DP_cost);
          out += "\nPre calculated cost: " + cutils::microPAIToPAI6(m_tpl_pre_calculated_dp_cost);

          out += "\nComment: " + m_tpl_comment;
          out += "\ndoc ref: " + m_tpl_doc_ref;

          out += "\nchange back outputindex: " + String::number(m_tpl_change_back_output_index);
          out += "\nChange back address: " + m_tpl_change_back_address;

          out += "\nBackers addresses: " + m_tpl_backers_addresses.join(", ");
          out += "\nSig hash: " + m_tpl_sig_hash;
          out += "\nCreation date: " + m_tpl_creation_date;
          out += "\nDoc type: " + m_tpl_doc_type;
          out += "\nDoc class: " + m_tpl_doc_class;
          out += "\nDoc Version: " + m_tpl_doc_ver;
          out += "\nDo Unequal: " + cutils::dumpIt(m_tpl_do_unequal);
          out += "\n";

          return out;
        }





        //  -  -  -  Basic transaction handler part
        BasicTransactionHandler::BasicTransactionHandler()
        {

        }
        */
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

    // let mut indexes: Vec<COutputIndexT> = vec![];
    // for an_out in &tpl_outputs
    // {
    //     indexes.push(an_out.m_output_index);
    // }
    //
    // indexes.sort();
    //
    // let mut ordered_outputs: Vec<TOutput> = vec![];
    // for an_index in indexes
    // {
    //     for an_out in &tpl_outputs
    //     {
    //         if an_out.m_output_index == an_index
    //         {
    //             ordered_outputs.push(an_out.clone());
    //         }
    //     }
    // }
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
        dict.insert(a_key.clone(), tmp_out);
        normalized_outputs.push(dict[a_key].clone());
        real_output_inx += 1;
    }

    return (true, normalized_outputs);
}