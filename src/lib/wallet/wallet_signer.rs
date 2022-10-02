use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog, machine};
use crate::cutils::unpack_coin_code;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CAddressT, CCoinCodeT, CInputIndexT, ClausesT, CMPAIValueT, LimitT, OrderT, QV2DicT, QVDRecordsT, VString};
use crate::lib::database::abs_psql::{ModelClause, q_insert, q_select};
use crate::lib::database::tables::C_MACHINE_USED_COINS;
use crate::lib::machine::machine_buffer::block_buffer::push_to_block_buffer;
use crate::lib::transactions::basic_transactions::basic_transaction_template::BasicTransactionTemplate;
use crate::lib::transactions::basic_transactions::coins::coins_handler::search_in_spendable_coins;
use crate::lib::transactions::basic_transactions::make_a_transaction::make_a_transaction_document;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{TInput, TOutput};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;
use crate::lib::wallet::wallet_address_handler::{create_and_insert_new_address_in_wallet, get_addresses_info};
use crate::lib::wallet::wallet_coins::{delete_from_funds, delete_from_funds_by_doc};


//old_name_was walletSigner
pub fn wallet_signer(
    candidate_coins: &VString,
    sending_amount: CMPAIValueT,
    fee_calc_method: &String,
    desired_trx_fee: CMPAIValueT,
    recipient: &CAddressT,
    change_back_mod: &String,
    change_back_address: &CAddressT,
    output_bill_size: CMPAIValueT,  // 0 means one output for entire sending coins
    comment: String) -> (bool, String)
{
    let message: String;

    let un_locker_index = 0; // if client didn't mention, chose the first unlock struct FIXME =0

    if candidate_coins.len() == 0
    {
        message = "No coin selected to spend!".to_string();
        dlog(
            &message,
            constants::Modules::Trx,
            constants::SecLevel::Warning);
        return (false, message);
    }

    if recipient == ""
    {
        message = "The recipient was missed!".to_string();
        dlog(
            &message,
            constants::Modules::Trx,
            constants::SecLevel::Warning);
        return (false, message);
    }

    if sending_amount == 0
    {
        message = "Missed sending amount!".to_string();
        dlog(
            &message,
            constants::Modules::Trx,
            constants::SecLevel::Warning);
        return (false, message);
    }

    // if desired_trx_fee == 0
    // {
    //   message = "Missed transaction fee!";
    //   CLog::log(message, "app", "warning");
    //   return {false, message};
    // }

    //TODO: double-control if mentioned coins are spendable and avoid double spending

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "ut_coin",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_coin in candidate_coins {
        c1.m_field_multi_values.push(a_coin as &(dyn ToSql + Sync));
    }
    let spendable_coins = search_in_spendable_coins(
        &vec![c1],
        &vec![],
        0);
    dlog(
        &format!("spendable coins: {:?}", spendable_coins),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    let mut envolved_spending_addresses: VString = vec![];
    let mut spendable_amount: CMPAIValueT = 0;
    let mut inputs: HashMap<CCoinCodeT, TInput> = HashMap::new();
    for a_coin in &spendable_coins
    {
        let coin_owner: CAddressT = a_coin["ut_o_address"].to_string();
        let coin_value: CMPAIValueT = a_coin["ut_o_value"].parse::<CMPAIValueT>().unwrap();

        envolved_spending_addresses.push(coin_owner.clone());
        spendable_amount += coin_value;

        let (tx_hash, output_inx) = unpack_coin_code(&a_coin["ut_coin"]);
        inputs.insert(
            a_coin["ut_coin"].to_string(),
            TInput {
                m_transaction_hash: tx_hash,
                m_output_index: output_inx,
                m_owner: coin_owner,
                m_amount: coin_value,
                m_private_keys: vec![],
                m_unlock_set: UnlockSet::new(),
            });
    }

    if spendable_amount < (sending_amount + desired_trx_fee)
    {
        message = format!(
            "Spending more than input fund! spendable_amount:{}, sending_amount: {} + desired_trx_fee:{}",
            spendable_amount, sending_amount, desired_trx_fee);
        dlog(
            &message,
            constants::Modules::Trx,
            constants::SecLevel::Warning);
        return (false, message);
    }
    let change_back_amount = spendable_amount - sending_amount - desired_trx_fee;

    let mut outputs: Vec<TOutput> = vec![];
    let mut change_back_address = change_back_address.clone();
    if change_back_mod == constants::change_back_mods::EXACT_ADDRESS
    {
        if change_back_address == ""
        {
            change_back_address = machine().get_backer_address();
        }
    } else if change_back_mod == constants::change_back_mods::BACKER_ADDRESS
    {
        change_back_address = machine().get_backer_address();
    } else if change_back_mod == constants::change_back_mods::A_NEW_ADDRESS
    {
        let (status, msg) = create_and_insert_new_address_in_wallet(
            constants::signature_types::BASIC,
            "1/1",
            constants::CURRENT_SIGNATURE_VERSION);
        if !status
        {
            return (false, msg);
        }
        change_back_address = msg;
    }

    outputs.push(TOutput {
        m_address: change_back_address,
        m_amount: change_back_amount,
        m_output_character: constants::OUTPUT_CHANGE_BACK.to_string(),
        m_output_index: 0,
    });

    if output_bill_size == 0
    {
        outputs.push(TOutput {
            m_address: recipient.to_string(),
            m_amount: sending_amount,
            m_output_character: constants::OUTPUT_NORMAL.to_string(),
            m_output_index: 0,
        });
    } else {
        // while (sending_amount >= output_bill_size)
        // {
        //     outputs.push(TOutput {
        //         recipient,
        //         output_bill_size,
        //         constants::OUTPUT_NORMAL,
        //     });
        //     sending_amount -= output_bill_size;
        // }
        // if (sending_amount > 0)
        // outputs.push(TOutput {
        //     recipient,
        //     sending_amount,
        //     constants::OUTPUT_NORMAL,
        // });
    }

    let wallet_controlled_addresses = get_addresses_info(
        envolved_spending_addresses,
        vec!["wa_address", "wa_detail"]);
    dlog(
        &format!("wallet controlled addresses records: {:#?}", wallet_controlled_addresses),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    let mut addresses_dict: QV2DicT = HashMap::new();
    for an_address in wallet_controlled_addresses
    {
        addresses_dict.insert(an_address["wa_address"].to_string(), an_address);
    }

    // find the unlockers
    for a_coin in &inputs.keys().cloned().collect::<VString>()
    {
        let mut an_input: TInput = inputs[a_coin].clone();

        if !addresses_dict.contains_key(&an_input.m_owner)
        {
            message = format!(
                "The input coin is not controlled by your wallet! coin: {:#?}",
                an_input);
            dlog(
                &message,
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, message);
        }

        let address_details: UnlockDocument = serde_json::from_str(
            &addresses_dict[&an_input.m_owner]["wa_detail"]).unwrap();
        an_input.m_unlock_set = address_details.m_unlock_sets[un_locker_index].clone();
        let salt = &an_input.m_unlock_set.m_salt;
        // let private_keys: VString = address_details.m_private_keys[salt];
        an_input.m_private_keys = address_details.m_private_keys[salt].clone();
        dlog(
            &format!("Coin to be spend: {:#?}", an_input.clone()),
            constants::Modules::Trx,
            constants::SecLevel::Info);
        inputs.insert(a_coin.clone(), an_input.clone());
    }

    let trx_template = BasicTransactionTemplate::new(
        inputs,
        outputs,
        fee_calc_method,
        desired_trx_fee,
        comment,
    );

    let (
        res_status,
        res_message,
        trx_doc,
        dp_cost) = make_a_transaction_document(trx_template);
    if !res_status
    { return (false, res_message); }

    dlog(
        &format!(
            "In wallet signer the signed basic transaction: {}",
            trx_doc.safe_stringify_doc(true)),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    // push transaction to Block buffer
    let mp_code = machine().get_selected_m_profile();
    let (buffer_push_res, buffer_push_message) = push_to_block_buffer(
        &trx_doc,
        dp_cost,
        &mp_code,
    );
    if !buffer_push_res
    {
        return (false, buffer_push_message);
    }

    // remove from wallet funds
    delete_from_funds_by_doc(&trx_doc);

    locally_mark_coin_as_used(&trx_doc);

    message = format!(
        "The transaction {} have been created and pushed to block buffer",
        trx_doc.get_doc_identifier());
    dlog(
        &message,
        constants::Modules::Trx,
        constants::SecLevel::Info);

    drop(trx_doc);

    return (true, message);
}

//old_name_was locallyMarkUTXOAsUsed
pub fn locally_mark_coin_as_used(doc: &Document)
{
    let mp_code: String = machine().get_selected_m_profile();
    let now_ = application().now();
    let inputs = doc.get_inputs();
    let doc_hash = doc.get_doc_hash();
    for inx in 0..inputs.len() as CInputIndexT
    {
        let coin_code: CCoinCodeT = inputs[inx as usize].get_coin_code();
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("lu_mp_code", &mp_code as &(dyn ToSql + Sync)),
            ("lu_coin", &coin_code as &(dyn ToSql + Sync)),
            ("lu_spend_loc", &doc_hash as &(dyn ToSql + Sync)),
            ("lu_insert_date", &now_ as &(dyn ToSql + Sync)),
        ]);
        dlog(
            &format!("Mark the coin as used coin: {:?}", values),
            constants::Modules::Trx,
            constants::SecLevel::Info);
        q_insert(
            C_MACHINE_USED_COINS,
            &values,
            false);
    }
}

//old_name_was excludeLocallyUsedCoins
pub fn exclude_locally_used_coins() -> bool
{
    let (_status, locally_used_coins) = q_select(
        C_MACHINE_USED_COINS,
        vec!["lu_coin"],
        vec![],
        vec![],
        0,
        false);
    let mp_code = machine().get_selected_m_profile();
    for a_coin in locally_used_coins
    {
        let (doc_hash, output_index) =
            cutils::unpack_coin_code(&a_coin["lu_coin"]);
        delete_from_funds(
            &doc_hash,
            output_index,
            &mp_code,
        );
    }
    return true;
}

//old_name_was searchLocallyMarkedUTXOs
#[allow(unused, dead_code)]
pub fn search_in_locally_marked_coins(
    clauses: ClausesT,
    fields: Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (_status, records) = q_select(
        C_MACHINE_USED_COINS,
        fields,
        clauses,
        order,
        limit,
        false);

    return records;
}

/*
std::tuple<bool, String, VString, JSonObject> Wallet::signByAnAddress(
  const CAddressT& signer_address,
  const String& sign_message,
  CSigIndexT unlocker_index)
{
  String msg;

  QVDRecordsT addresses_details = Wallet::getAddressesInfo({signer_address});
  if (addresses_details.len() != 1)
  {
    msg = "The address " + cutils::short_bech16(signer_address) + " is not controlled by current wallet/profile!";
    CLog::log(msg, "app", "error");
    return {false, msg, {}, {}};
  }

  JSonObject addrDtl = cutils::parseToJsonObj(addresses_details[0]["wa_detail"].to_string());
  VString signatures {};
  JSonObject dExtInfo {};
  JSonObject unlock_set = addrDtl["uSets"].toArray()[unlocker_index].toObject();
  JSonArray sSets = unlock_set["sSets"].toArray();

  for (CSigIndexT inx = 0; inx < sSets.len(); inx++)
  {
    auto[signing_res, signature_hex, signature] = CCrypto::ECDSAsignMessage(
      addrDtl["the_private_keys"].toObject()[unlock_set["salt"].to_string()].toArray()[inx].to_string(),
      sign_message);
    if (!signing_res)
    {
      msg = "Failed in sign of Salt(" + unlock_set["salt"].to_string() + ")";
      CLog::log(msg, "app", "error");
      return {false, msg, {}, {}};
    }
    signatures.push(String::fromStdString(signature_hex));
  }
  if (signatures.len() == 0)
  {
    msg = "The message couldn't be signed";
    CLog::log(msg, "app", "error");
    return {false, msg, {}, {}};
  }

  return {true, "", signatures, unlock_set};
}

*/