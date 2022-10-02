use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CCoinCodeT, CDocHashT, CDocIndexT, CMPAISValueT, CMPAIValueT, QV2DicT};
use crate::lib::transactions::basic_transactions::coins::coins_handler::Coin;

//old_name_was validateEquation
pub fn validate_equation(
    block: &Block,
    used_coins_dict: &HashMap<CCoinCodeT, Coin>,
    invalid_coins_dict: &QV2DicT) -> bool
{
    let mut validate_res: bool;

    // transaction details check
    let mut inputs_amounts_dict: HashMap<CDocHashT, CMPAIValueT> = HashMap::new();
    let mut outputs_amounts_dict: HashMap<CDocHashT, CMPAIValueT> = HashMap::new();
    for doc_inx in 0..block.get_documents().len() as CDocIndexT
    {
        if block.get_documents()[doc_inx as usize].is_basic_transaction()
        // if Document::is_basic_transaction(&block.get_documents()[doc_inx as usize].m_doc_type)
        {
            let mut doc: Document = block.get_documents()[doc_inx as usize].clone();
            if doc.m_doc_ext_info.len() == 0
            {
                doc.maybe_assign_doc_ext_info(block, doc_inx);
            }

            // signatures control
            // validate_tx_signatures
            validate_res = doc.m_if_basic_tx_doc.validate_tx_signatures(
                &doc,
                &used_coins_dict,
                &vec![],
                &block.get_block_hash());
            if !validate_res
            { return false; }

            // control transaction hash
            validate_res = doc.m_if_basic_tx_doc.validate_general_rules_for_transaction(&doc);
            if !validate_res
            { return false; }

            // equation control
            let (
                equation_check_res,
                equation_msg,
                total_inputs_amounts,
                total_outputs_amounts) =
                doc.m_if_basic_tx_doc.equation_check(
                    &doc,
                    used_coins_dict,
                    invalid_coins_dict,
                    &block.get_block_hash());
            if !equation_check_res
            {
                dlog(
                    &format!("{}", equation_msg),
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return false;
            }

            inputs_amounts_dict.insert(doc.get_doc_hash(), total_inputs_amounts);
            outputs_amounts_dict.insert(doc.get_doc_hash(), total_outputs_amounts);

            if doc.get_inputs().len() > 0
            {
                for input in doc.get_inputs()
                {
                    let a_coin_code: CCoinCodeT = input.get_coin_code();
                    if used_coins_dict.contains_key(&a_coin_code)
                    {
                        if used_coins_dict[&a_coin_code.clone()].m_coin_value >= constants::MAX_COINS_AMOUNT
                        {
                            dlog(
                                &format!("The transaction has input bigger than MAX_SAFE_INTEGER! trx {} Block {} value: {}",
                                         doc.get_doc_identifier(),
                                         block.get_block_identifier(),
                                         cutils::nano_pai_to_pai(used_coins_dict[&a_coin_code].m_coin_value as CMPAISValueT)
                                ),
                                constants::Modules::Sec,
                                constants::SecLevel::Error);
                            return false;
                        }

                        let mut tmp_amount = inputs_amounts_dict[&doc.get_doc_hash()];
                        tmp_amount += used_coins_dict[&a_coin_code].m_coin_value;
                        inputs_amounts_dict.insert(doc.get_doc_hash(), tmp_amount);
                    } else {
                        // * trx uses already spent outputs! so try invalid_coins_dict
                        // * probably it is a double-spend case, which will be decided after 12 hours, in importing step
                        // * BTW ALL trx must have balanced equation (even duoble-spendeds)
                        if invalid_coins_dict.contains_key(&a_coin_code)
                        {
                            if invalid_coins_dict[&a_coin_code]["coinGenOutputValue"].parse::<CMPAIValueT>().unwrap() >= constants::MAX_COINS_AMOUNT
                            {
                                dlog(
                                    &format!("The transaction has inv-input bigger than MAX_SAFE_INTEGER! trx {} Block {}  value: {}",
                                             doc.get_doc_identifier(),
                                             block.get_block_identifier(),
                                             cutils::nano_pai_to_pai(invalid_coins_dict[&a_coin_code]["coinGenOutputValue"].parse::<CMPAISValueT>().unwrap())
                                    ),
                                    constants::Modules::Sec,
                                    constants::SecLevel::Error);
                                return false;
                            }

                            let mut tmp_amount = inputs_amounts_dict[&doc.get_doc_hash()];
                            tmp_amount += invalid_coins_dict[&a_coin_code]["coinGenOutputValue"].parse::<CMPAIValueT>().unwrap();
                            inputs_amounts_dict.insert(doc.get_doc_hash(), tmp_amount);
                        } else {
                            dlog(
                                &format!(
                                    "The input absolutely missed! not in tables neither in DAG! coin({})  trx {} Block {}",
                                    a_coin_code,
                                    doc.get_doc_identifier(),
                                    block.get_block_identifier()
                                ),
                                constants::Modules::Sec,
                                constants::SecLevel::Error);
                            return false;
                        }
                    }
                }
            }

            if doc.get_outputs().len() > 0
            {
                for output in doc.get_outputs()
                {
                    if output.m_address != cutils::strip_output_address(&output.m_address)
                    {
                        dlog(
                            &format!(
                                "The transaction has not digit characters! trx {} Block {}",
                                doc.get_doc_identifier(),
                                block.get_block_identifier()
                            ),
                            constants::Modules::Sec,
                            constants::SecLevel::Error);
                        return false;
                    }

                    if output.m_amount == 0
                    {
                        dlog(
                            &format!(
                                "The transaction has zero output! trx {} Block {}",
                                doc.get_doc_identifier(),
                                block.get_block_identifier()
                            ),
                            constants::Modules::Sec,
                            constants::SecLevel::Error);
                        return false;
                    }

                    // if output.m_amount < 0
                    // {
                    //   msg = "The transaction has negative output! trx(" + doc.m_doc_type + " / " + cutils::hash8c(doc.m_doc_hash) + ") Block(" + cutils::hash8c(block.getBlockHash()) + ")";
                    //   CLog::log(msg, "sec", "error");
                    //   return false;
                    // }

                    if output.m_amount >= constants::MAX_COINS_AMOUNT
                    {
                        dlog(
                            &format!(
                                "The transaction has output bigger than MAX_SAFE_INTEGER! trx {} Block {}",
                                doc.get_doc_identifier(),
                                block.get_block_identifier()
                            ),
                            constants::Modules::Sec,
                            constants::SecLevel::Error);
                        return false;
                    }

                    let mut tmp_amount = outputs_amounts_dict[&doc.get_doc_hash()];
                    tmp_amount += output.m_amount;
                    outputs_amounts_dict.insert(doc.get_doc_hash(), tmp_amount);
                }
            }
            dlog(
                &format!(
                    "The inputs_amounts_dict must be equal outputs. input({}) == ({} output) {} {} ",
                    cutils::nano_pai_to_pai(inputs_amounts_dict[&doc.get_doc_hash()] as CMPAISValueT),
                    cutils::nano_pai_to_pai(outputs_amounts_dict[&doc.get_doc_hash()] as CMPAISValueT),
                    doc.get_doc_identifier(),
                    block.get_block_identifier(),
                ),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);

            if inputs_amounts_dict[&doc.get_doc_hash()] != outputs_amounts_dict[&doc.get_doc_hash()]
            {
                dlog(
                    &format!("The transaction is not balanced! {} {} input({}) != ( output {})",
                             doc.get_doc_identifier(),
                             block.get_block_identifier(),
                             cutils::nano_pai_to_pai(inputs_amounts_dict[&doc.get_doc_hash()] as CMPAISValueT),
                             cutils::nano_pai_to_pai(outputs_amounts_dict[&doc.get_doc_hash()] as CMPAISValueT)
                    ),
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return false;
            }
        }
    }

    // calculate block total inputs & outputs
    let mut total_amount_block_inputs: CMPAIValueT = 0;
    let mut total_amount_block_outputs: CMPAIValueT = 0;
    for a_doc_hash in inputs_amounts_dict.keys()
    {
        total_amount_block_inputs += inputs_amounts_dict[a_doc_hash];
        total_amount_block_outputs += outputs_amounts_dict[a_doc_hash];
    }
    if total_amount_block_inputs != total_amount_block_outputs
    {
        dlog(
            &format!("Unbalanced total in/out of Block {} ",
                     block.get_block_identifier(),
            ),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return false;
    }

    dlog(
        &format!(
            "Valid in/out equation Block {} value({})",
            block.get_block_identifier(),
            cutils::nano_pai_to_pai(total_amount_block_inputs as CMPAISValueT)
        ),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    return true;
}
