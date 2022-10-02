use std::collections::HashMap;
use serde_json::json;
use crate::lib::custom_types::{CCoinCodeT, CMPAISValueT, CMPAIValueT, DocLenT, JSonObject};
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::transactions::basic_transactions::basic_transaction_template::{BasicTransactionTemplate, export_doc_ext_to_json, generate_bip69_input_tuples, generate_bip69_output_tuples};
use crate::lib::transactions::basic_transactions::coins::coins_handler::Coin;
use crate::lib::transactions::basic_transactions::pre_validate_transaction_params::pre_validate_transaction_params;

// * @return {status, err_msg, transaction, transaction_dp_cost}
//old_name_was makeATransaction
pub fn make_a_transaction_document(mut tpl: BasicTransactionTemplate)
                                   -> (bool, String, Document, CMPAIValueT)
{
    let mut msg: String;
    let doc: Document = Document::new();

    // supportedP4PTrxLength = _.has(args, 'supportedP4PTrxLength') ? (args.supportedP4PTrxLength) : 0;

    if tpl.m_tpl_backers_addresses.len() == 0
    {
        let backer_address = machine().get_backer_address();
        tpl.m_tpl_backers_addresses.push(backer_address);  // in case of clone, a transaction can have more than one backer
    }

    dlog(
        &format!("Make a transaction, input params: {:#?}", &tpl),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    // adding DP Costs payment outputs
    let mut tmp_outputs = tpl.m_tpl_outputs.clone();
    for a_backer in &tpl.m_tpl_backers_addresses
    {
        tpl.append_tmp_dp_cost_output(&mut tmp_outputs, a_backer);
    }
    tpl.m_tpl_outputs = tmp_outputs;
    dlog(
        &format!("Make a transaction, outputs: {:#?}", &tpl.m_tpl_outputs),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let (
        pre_validate_status,
        pre_validate_res_msg,
        total_inputs_amount,
        total_outputs_amount_except_dp_costs) = pre_validate_transaction_params(&tpl);
    if !pre_validate_status
    {
        return (false, pre_validate_res_msg, doc, 0);
    }

    // pre-signing in order to estimate actual outcome transaction length
    tpl.sign_and_create_doc_ext_info();

    let ext_info = export_doc_ext_to_json(&tpl.m_tpl_doc_ext_info);
    let input_tuples = generate_bip69_input_tuples(&tpl.m_tpl_inputs);
    let (output_tuples, ordered_tpl_outputs) = generate_bip69_output_tuples(&tpl.m_tpl_outputs);
    tpl.m_tpl_outputs = ordered_tpl_outputs;
    let trx_json: JSonObject = json!({
        "dHash": constants::HASH_ZEROS_PLACEHOLDER,
        "dLen": constants::LEN_PROP_PLACEHOLDER,
        "dType": tpl.m_tpl_doc_type,
        "dClass": tpl.m_tpl_doc_class,
        "dVer": tpl.m_tpl_doc_ver,
        "dRef": tpl.m_tpl_doc_ref,
        "dPIs": vec![0],  // data & process cost, which are payed by one output(or more output for cloned transactions)
        "dComment": tpl.m_tpl_comment,
        "dCDate": tpl.m_tpl_creation_date,
        "inputs": input_tuples,
        "outputs": output_tuples,
        "dExtInfo": ext_info,
        "dExtHash": constants::HASH_ZEROS_PLACEHOLDER});
    dlog(
        &format!("trx_json in pre-transaction: {:#}", &trx_json),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);
    let (status, mut document) = Document::load_document(&trx_json, &Block::new(), -1);
    if !status
    {
        dlog(
            &format!("Failed in load doc from trx_json: {:#?}", &trx_json),
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, pre_validate_res_msg, doc, 0);
    }

    let offset_extra_chars: DocLenT = 0; // just an offset length to cover potentially extra chars of precise DPCost
    dlog(
        &format!("offset-extra-chars is: {:#?}",
                 cutils::sep_num_3(offset_extra_chars as i64)),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    document.set_doc_length();

    dlog(
        &format!(
            "pre calculation transaction 1: {}",
            document.safe_stringify_doc(true)),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let now_ = application().now();
    let (
        status,
        locally_recalculate_trx_dp_cost) =
        document.calc_doc_data_and_process_cost(
            constants::stages::CREATING,
            &now_,
            offset_extra_chars);

    dlog(
        &format!(
            "The transaction cost is ({})",
            cutils::nano_pai_to_pai(locally_recalculate_trx_dp_cost as CMPAISValueT)),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    if !status
    {
        msg = "Fail in calculate transaction cost!".to_string();
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);

        return (false, msg, doc, 0);
    }
    dlog(
        &format!(
            "pre calculation transaction 2: {}",
            document.safe_stringify_doc(true)),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let (afford_status, afford_msg, the_dp_cost) =
        tpl.can_afford_costs(locally_recalculate_trx_dp_cost);
    dlog(
        &format!(
            "can-afford-costs: {}, msg: {}, cost: {}",
            afford_status, afford_msg, the_dp_cost),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    if !afford_status
    {
        return (false, afford_msg, doc, 0);
    }
    tpl.set_dp_cost_amount(the_dp_cost);

    dlog(
        &format!(
            "sum total_inputs_amount: {} PAI",
            cutils::nano_pai_to_pai(total_inputs_amount as CMPAISValueT)),
        constants::Modules::Trx,
        constants::SecLevel::Info);
    dlog(
        &format!(
            "sum outputs Value(except dp costs): {} PAI",
            cutils::nano_pai_to_pai(total_outputs_amount_except_dp_costs as CMPAISValueT)),
        constants::Modules::Trx,
        constants::SecLevel::Info);
    dlog(
        &format!(
            "sum DPCost({} Backer): {} nano PAI",
            tpl.m_tpl_backers_addresses.len(),
            the_dp_cost * tpl.m_tpl_backers_addresses.len() as CMPAIValueT),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    // TODO: add possibility for user to pay different DPCost to different backers(in case of cloning trx)
    let change_back_amount: CMPAISValueT = (total_inputs_amount
        - total_outputs_amount_except_dp_costs
        - (the_dp_cost * tpl.m_tpl_backers_addresses.len() as CMPAIValueT)) as CMPAISValueT;
    dlog(
        &format!(
            "change back amount ({} PAI) because of DPCosts",
            cutils::nano_pai_to_pai(change_back_amount)),
        constants::Modules::Trx,
        constants::SecLevel::Info);


//  CMPAISValueT finalChange = transaction->m_outputs[change_back_output_index]->m_amount + change_back_amount;

    dlog(
        &format!(
            "Dump tpl befor set_ change_ back_ output_ amount {:#?}", tpl),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    if change_back_amount > 0
    {
        tpl.set_change_back_output_amount(change_back_amount as CMPAIValueT);
    } else {
        // there is no change back, so remove the change back output
        tpl.remove_change_back_output();
    }

    // test for un-equality, by adding 1 unit to one out put and making a dummy unequality
    //if (tpl.m_tpl_do_unequal)
    //  transaction->m_outputs[tpl.m_tpl_change_back_output_index]->m_amount++;

    dlog(
        &format!(
            "Dump tpl after set_ change_ back_ output_ amount and befor final assignment to document outputs {:#?} \n",
            tpl),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    document.m_if_basic_tx_doc.m_outputs = tpl.generate_doc_output_tuples();
    document.m_if_basic_tx_doc.m_data_and_process_payment_indexes = tpl.get_dp_cost_indexes();
    if document.m_if_basic_tx_doc.m_data_and_process_payment_indexes.len() == 0
    {
        msg = "Failed in find output by (address_plus_value)".to_string();
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);

        return (false, msg, doc, 0);
    }
    document.m_if_basic_tx_doc.m_data_and_process_payment_indexes.sort();
    document.m_if_basic_tx_doc.m_data_and_process_payment_indexes.dedup();


    // re-signing, because of output re-ordering
    tpl.sign_and_create_doc_ext_info();
    document.m_if_basic_tx_doc.m_outputs = tpl.m_tpl_outputs.clone();
    document.m_doc_ext_info = tpl.m_tpl_doc_ext_info.clone();
    document.set_doc_ext_hash();
    document.set_doc_length();
    document.set_doc_ext_hash();
    document.set_doc_hash();

    let now_ = application().now();
    let mut tmp_block: Block = Block::new();
    tmp_block.m_block_creation_date = now_;
    tmp_block.m_block_type = constants::block_types::NORMAL.to_string();
    tmp_block.m_block_hash = constants::HASH_ZEROS_PLACEHOLDER.to_string();
    tmp_block.m_block_documents = vec![document.clone()];

    msg = format!(
        "Block to full Validate: {:#?}",
        tmp_block);
    dlog(
        &msg,
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let (final_validate, msg_) = document.full_validate(&tmp_block);
    if !final_validate
    {
        msg = format!(
            "Failed in transaction full Validate, {} {}",
            msg_,
            tmp_block.safe_stringify_block(true));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, doc, 0);
    }
    drop(tmp_block);

    // double-check outputs
    for an_out in document.get_outputs()
    {
        if an_out.m_amount <= 0
        {
            msg = "at least one negative/zero output exist".to_string();
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, msg, doc, 0);
        }
    }

    // signatures control
    let mut used_coins_dict: HashMap<CCoinCodeT, Coin> = HashMap::new();
    for (coin_code, an_inp) in &tpl.m_tpl_inputs
    {
        // let vv: QVDicT = HashMap::from([
        //     ("coin_owner".to_string(), an_inp.m_owner.clone()),
        //     ("coin_value".to_string(), an_inp.m_amount.to_string())]);
        let a_coin = Coin {
            m_coin_owner: an_inp.m_owner.clone(),
            m_coin_value: an_inp.m_amount,
            m_coin_code: coin_code.clone(),
            m_creation_date: "".to_string(),
            m_ref_creation_date: "".to_string(),
            ut_visible_by: "".to_string(),
        };
        used_coins_dict.insert(coin_code.clone(), a_coin);
    }

    let signature_validate_res = document.m_if_basic_tx_doc.validate_tx_signatures(
        &document,
        &used_coins_dict,
        &vec![],
        &constants::HASH_ZEROS_PLACEHOLDER.to_string());
    if !signature_validate_res
    {
        msg = format!(
            "Failed in transaction validate signature. {}",
            document.safe_stringify_doc(true));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, doc, 0);
    }

    // control transaction hash
    let general_transaction_validate_res: bool = document.m_if_basic_tx_doc.validate_general_rules_for_transaction(&document);
    if !general_transaction_validate_res
    {
        msg = format!("Failed in transaction general validate rules {}", document.safe_stringify_doc(true));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, doc, 0);
    }

    // equation control
    let (
        equation_check_res,
        equation_msg,
        total_inputs_amounts,
        total_outputs_amounts) = document.m_if_basic_tx_doc.equation_check(
        &document,
        &used_coins_dict,
        &HashMap::new(),
        &constants::HASH_ZEROS_PLACEHOLDER.to_string());
    if !equation_check_res
    {
        msg = format!(
            "Failed in transaction equation control, {}, {}",
            equation_msg,
            document.safe_stringify_doc(true));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, doc, 0);
    }
    if total_inputs_amounts != total_outputs_amounts
    {
        msg = format!(
            "Failed input/output amount equation control, input: {} != output: {}, {}",
            total_inputs_amounts,
            total_outputs_amounts,
            document.safe_stringify_doc(true));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, doc, 0);
    }
    dlog(
        &format!(
            "Pre-Successfully created transaction: {}",
            document.safe_stringify_doc(true)),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    let now_ = application().now();
    let (_calc_status2, trx_dp_cost2) = document.m_if_basic_tx_doc.calc_doc_data_and_process_cost(
        &document,
        constants::stages::CREATING,
        &now_,
        0);

    if the_dp_cost < trx_dp_cost2
    {
        if document.m_if_basic_tx_doc.get_dpis().len() > 1
        {
            panic!("Multiple DPCost payment(cloned trx) is not supported yet {}", 101);
        }

        dlog(
            &format!("Failed in DPCost calc: trx_dp_cost2({}) {}",
                     cutils::nano_pai_to_pai(trx_dp_cost2 as CMPAISValueT),
                     document.safe_stringify_doc(true)),
            constants::Modules::Trx,
            constants::SecLevel::Error);

        panic!("Failed in transaction cost calculation!!!!!");
        //transaction->m_outputs[transaction->getDPIs()[0]]->m_amount = trx_dp_cost2;
    }

    return (true, "".to_string(), document, the_dp_cost);
}