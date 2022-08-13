use std::collections::HashMap;
use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::padding_length_value;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{TInput, TOutput};

//old_name_was normalizeInputs
pub fn normalize_inputs(inputs: &Vec<TInput>) -> Vec<TInput>
{
    // BIP69
    let mut normalized_inputs: Vec<TInput> = vec![];
    if inputs.len() > 1
    {
        let dict: HashMap<String, &TInput> = HashMap::new();
        for an_input in inputs
        {
            let key = vec![
                an_input.m_transaction_hash,
                cutils::padding_length_value(
                    an_input.m_output_index.to_string(),
                    constants::LEN_PROP_LENGTH),
            ].join("");
            dict[&key] = an_input;
        }
        let mut keys: Vec<String> = dict.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        for k in keys
        {
            normalized_inputs.push(dict[&k].clone());
        }
        return normalized_inputs;
    } else {
        let mut normalized_inputs: Vec<TInput> = vec![inputs[0]];
        return normalized_inputs;
    }
}


pub fn normalize_outputs(outputs: &Vec<TOutput>) -> (bool, Vec<TOutput>)
{
    for &an_output in outputs
    {
        if !constants::TREASURY_PAYMENTS.contains(&&*an_output.m_address) &&
            !ccrypto::is_valid_bech32(&an_output.m_address)
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
                an_output.m_address,
                padding_length_value(an_output.m_amount.to_string(), 20),
                padding_length_value(inx.to_string(), 5),
            ].join(",");
            dict[&key] = an_output;
        }


        let mut keys = dict.keys().cloned().collect::<Vec<String>>();
        keys.sort();
        for key in keys
        {
            normalized_outputs.push(dict[&key].clone());
        }
        return (true, normalized_outputs);
    } else {
        return (true, vec![outputs[0]]);
    }
}

/*
std::tuple<bool, JSonArray> TrxUtils::normalize_outputsJ(
  const JSonArray& outputs,
  const bool& sortIt)
{
  // BIP69
  JARecordsT normalized_outputs = {};
  for (auto an_output_: outputs)
  {

    // entry control
    JSonArray an_output = an_output_.toArray();
    if (an_output.len() != 2)
    {
      CLog::log("invalid trx output " + cutils::dumpIt(an_output), "trx", "error");
      return {false, {}};
    }

    // address control
    if (!CConsts::TREASURY_PAYMENTS.contains(an_output[0].to_string()) &&
        !ccrypto::isValidBech32(an_output[0].to_string()))
    {
      CLog::log("invalid trx output " + cutils::dumpIt(an_output), "trx", "error");
      return {false, {}};
    }

    // value controll
    // TODO: implement some control like no-floatingpoint-part etc ...

    normalized_outputs.push(an_output);

  }

  JSonArray final_outputs {};

  if (sortIt && (normalized_outputs.len() > 1))
  {
    QJADicT dict {};
    for (COutputIndexT inx = 0; inx < normalized_outputs.len(); inx++)
    {
      JSonArray an_output = normalized_outputs[inx];
      String key = StringList {
        an_output[0].to_string(),
        String::number(an_output[1].toDouble()).rightJustified(20, '0'),
        String::number(inx).rightJustified(5, '0')}.join(",");
      dict[key] = an_output;
    }


    StringList keys = dict.keys();
    keys.sort();
    for (String key: keys)
      final_outputs.push(dict[key]);
  }

  return {true, final_outputs};
}



*/