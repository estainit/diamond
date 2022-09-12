use std::collections::HashMap;
use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::padding_length_value;
use crate::lib::custom_types::{COutputIndexT, VString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{TInput, TOutput};

//old_name_was normalizeInputs
pub fn normalize_inputs(inputs: &Vec<TInput>) -> Vec<TInput>
{
    // BIP69
    let mut normalized_inputs: Vec<TInput> = vec![];
    if inputs.len() > 1
    {
        let mut dict: HashMap<String, &TInput> = HashMap::new();
        for an_input in inputs
        {
            let key = vec![
                an_input.m_transaction_hash.clone(),
                cutils::padding_length_value(
                    an_input.m_output_index.to_string(),
                    constants::LEN_PROP_LENGTH),
            ].join("");
            dict.insert(key, an_input);
        }
        let mut keys: Vec<String> = dict.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        for k in keys
        {
            normalized_inputs.push(dict[&k].clone());
        }
        return normalized_inputs;
    } else {
        let normalized_inputs: Vec<TInput> = vec![inputs[0].clone()];
        return normalized_inputs;
    }
}


pub fn normalize_outputs(outputs: &Vec<TOutput>) -> (bool, Vec<TOutput>)
{
    for an_output in outputs
    {
        if !ccrypto::is_valid_bech32(&an_output.m_address) &&
            !constants::TREASURY_PAYMENTS.contains(&an_output.m_address.as_str())

        {
            dlog(
                &format!("invalid trx output: {}", an_output.dump()),
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, vec![]);
        }

        // TODO: implement some control like no-floatingpoint-part etc ...
    }

    let mut normalized_outputs: Vec<TOutput> = vec![];

    if outputs.len() > 1
    {
        let mut dict: HashMap<String, TOutput> = HashMap::new();
        for inx in 0..outputs.len()
        {
            let an_output = outputs[inx].clone();
            let key = vec![
                an_output.m_address.clone(),
                padding_length_value(an_output.m_amount.to_string(), 20),
                padding_length_value(inx.to_string(), 5),
            ].join(",");
            dict.insert(key, an_output);
        }


        let mut keys = dict.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        for key in keys
        {
            normalized_outputs.push(dict[&key].clone());
        }
        return (true, normalized_outputs);
    } else {
        return (true, vec![outputs[0].clone()]);
    }
}

//old_name_was normalizeOutputsJ
pub fn normalize_rp_outputs(
    outputs: &Vec<TOutput>,
    should_sort: bool) -> (bool, Vec<TOutput>)
{
    // BIP69
    let mut normalized_outputs: Vec<TOutput> = vec![];
    for an_output in outputs
    {
        // address control
        if !constants::TREASURY_PAYMENTS.contains(&an_output.m_address.as_str()) &&
            !ccrypto::is_valid_bech32(&an_output.m_address)
        {
            dlog(
                &format!("invalid trx output {:#?}", an_output),
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, vec![]);
        }

        // value controll
        // TODO: implement some control like no-floatingpoint-part etc ...

        normalized_outputs.push(an_output.clone());
    }

    let mut final_outputs: Vec<TOutput> = vec![];

    if should_sort && (normalized_outputs.len() > 1)
    {
        let mut dict: HashMap<String, TOutput> = HashMap::new();
        let mut inx: COutputIndexT = 0;
        while inx < normalized_outputs.len() as COutputIndexT
        {
            let an_output: &TOutput = &normalized_outputs[inx as usize];
            let key: String = vec![
                an_output.m_address.clone(),
                cutils::padding_left(&an_output.m_amount.to_string(), 20),
                cutils::padding_left(&inx.to_string(), 5)].join(",");
            dict.insert(key, an_output.clone());
            inx += 1;
        }


        let mut keys: VString = dict.keys().cloned().collect::<VString>();
        keys.sort();
        for key in keys
        {
            final_outputs.push(dict[&key].clone());
        }
    }

    return (true, final_outputs);
}
