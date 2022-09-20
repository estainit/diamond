use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::{constants, cutils, dlog};
use crate::lib::custom_types::{CAddressT, CDocHashT, CMPAIValueT, COutputIndexT, GRecordsT, VString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{TInput, TOutput};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;

/*

#include "stable.h"

#include "lib/transactions/trx_utils.h"
#include "lib/ccrypto.h"


#include "rp_docdocument.h"



RepaymentDocument::RepaymentDocument(const QJsonObject& obj)
{
  setByJsonObj(obj);
}

RepaymentDocument::~RepaymentDocument()
{
  deleteInputs();
  deleteOutputs();
}

bool RepaymentDocument::deleteInputs()
{
  for (TInput* an_input: m_inputs)
    delete an_input;
  return true;
}

bool RepaymentDocument::deleteOutputs()
{
  for (TOutput* an_output: m_outputs)
    delete an_output;
  return true;
}

bool RepaymentDocument::setByJsonObj(const QJsonObject& obj)
{
  Document::setByJsonObj(obj);

  // maybe some drived class assigning

  if (obj["inputs"].toArray().size() > 0)
    setDocumentInputs(obj["inputs"]);

  if (obj["outputs"].toArray().size() > 0)
    setDocumentOutputs(obj["outputs"]);

  return true;
}

String RepaymentDocument::getRef() const
{
  return "";
}

std::tuple<bool, QJsonArray> RepaymentDocument::exportInputsToJson() const
{
  QJsonArray inputs {};
  for (TInput* an_input: m_inputs)
    inputs.push(QJsonArray{
      an_input->m_transaction_hash,
      an_input->m_output_index});
  return {true, inputs};
}

std::tuple<bool, QJsonArray> RepaymentDocument::exportOutputsToJson() const
{
  QJsonArray outputs {};
  for (TOutput* an_output: m_outputs)
    outputs.push(QJsonArray{
      an_output->m_address,
      QVariant::fromValue(an_output->m_amount).toDouble()});
  return {true, outputs};
}

QJsonObject RepaymentDocument::getRepayDocTpl()
{
  return QJsonObject {
    {"dHash", "0000000000000000000000000000000000000000000000000000000000000000"},
    {"dType", constants::DOC_TYPES::RpDoc},
    {"dClass", constants::DOC_TYPES::RpDoc},
    {"dVer", "0.0.0"},
    {"cycle", ""}, // 'yyyy-mm-dd am' / 'yyyy-mm-dd pm'
    {"inputs", QJsonArray{}},
    {"outputs", QJsonArray{}}};
}

*/

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RepaymentDocument
{
    pub m_doc_cycle: String,
    pub m_input: TInput,
    pub m_outputs: Vec<TOutput>,
}

impl RepaymentDocument
{
    pub fn new() -> Self
    {
        Self {
            m_doc_cycle: "".to_string(),
            m_input: TInput::new(),
            m_outputs: vec![],
        }
    }
}

//old_name_was calcRepaymentDetails
pub fn calc_repayment_details(
    coinbase_trx_hash: &CDocHashT,
    output_index: COutputIndexT,
    coinbased_output_value: CMPAIValueT,
    pledged_accounts_info: &GRecordsT,
    the_pledged_account: &CAddressT) -> RepaymentDocument
{
    dlog(
        &format!(
            "calc Repayment Details, coinbase-trx-hash: {}, output-index:{}, coinbased-output-value:{}, the-pledged-account:{}, pledged-accounts-info: {:#?}",
            coinbase_trx_hash,
            output_index,
            coinbased_output_value,
            the_pledged_account,
            pledged_accounts_info
        ),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let mut a_repayback_doc: RepaymentDocument = RepaymentDocument {
        m_doc_cycle: "".to_string(),
        m_input: TInput {
            m_transaction_hash: coinbase_trx_hash.to_string(),
            m_output_index: output_index,
            m_owner: "".to_string(),
            m_amount: 0,
            m_private_keys: vec![],
            m_unlock_set: UnlockSet::new(),
        },
        m_outputs: vec![],
    };

    let mut real_income: CMPAIValueT = coinbased_output_value;
    // create a repayment block cut repayment parts from income
    // repaymentInputs.push(refLoc);

    // order pledges by register date and cut by order
    let mut total_to_be_cut: CMPAIValueT = 0;
    let mut pledged_accounts_by_time: HashMap<String, &HashMap<String, String>> = HashMap::new();
    for a_pledged in &pledged_accounts_info[the_pledged_account]
    {
        let key: String = format!("{}_{}", a_pledged["pgd_activate_date"], a_pledged["pgd_hash"]); // make sure for all machine we have a certain order even for same date pledge activated contract
        pledged_accounts_by_time.insert(key, a_pledged);
        total_to_be_cut += a_pledged["pgd_repayment_amount"].parse::<CMPAIValueT>().unwrap();
    }
    dlog(
        &format!(
            "Pledged account income per cycle: Account({}) Income({}) total To Be Cut({})",
            cutils::short_bech16(the_pledged_account),
            real_income,
            total_to_be_cut),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    // cutting
    let mut keys: VString = pledged_accounts_by_time
        .keys()
        .cloned()
        .collect::<VString>();
    keys.sort();
    for a_repay in &keys
    {
        // TODO implement an efficient way to check if repayments already done? in this case close contract
        let to_cut: CMPAIValueT = pledged_accounts_by_time[a_repay]["pgd_repayment_amount"].parse::<CMPAIValueT>().unwrap();
        let repayment_account: CAddressT = pledged_accounts_by_time[a_repay]["pgd_pledgee"].clone();
        if real_income >= to_cut
        {
            dlog(
                &format!(
                    "Pledged Account cuttings. Account({} cutting {} PAIs repayments to pay({})",
                    cutils::short_bech16(the_pledged_account),
                    to_cut,
                    cutils::short_bech16(&repayment_account)),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);

            a_repayback_doc.m_outputs.push(
                TOutput {
                    m_address: repayment_account.clone(),
                    m_amount: to_cut,
                    m_output_character: "".to_string(),
                    m_output_index: 0,
                });
            real_income = real_income - to_cut;
        } else {
            dlog(
                &format!(
                    "Pledged Account({}) cutting {} mcPAIs (remained PAIs) which not covers completely repayments to ({})",
                    cutils::short_bech16(the_pledged_account),
                    real_income,
                    cutils::short_bech16(&repayment_account)),
                constants::Modules::App,
                constants::SecLevel::Warning);

            // repaymentOutputs.push([repaymentAccount, real_income]);
            a_repayback_doc.m_outputs.push(
                TOutput {
                    m_address: repayment_account.clone(),
                    m_amount: real_income,
                    m_output_character: "".to_string(),
                    m_output_index: 0,
                });
            real_income = 0;
        }
    }

    // pay what remains, to account itself
    if real_income > 0
    {
        dlog(
            &format!(
                "Pledged Account({}) get paying {} mc PAIs to pledger after cut all repayments",
                cutils::short_bech16(the_pledged_account),
                real_income),
            constants::Modules::App,
            constants::SecLevel::Info);
        a_repayback_doc.m_outputs.push(
            TOutput {
                m_address: the_pledged_account.clone(),
                m_amount: real_income,
                m_output_character: "".to_string(),
                m_output_index: 0,
            });
    }

    return a_repayback_doc;
}

/*
QJsonObject RepaymentDocument::exportDocToJson(const bool ext_info_in_document) const
{
  QJsonObject document = Document::exportDocToJson(ext_info_in_document);

  document.remove("dCDate");
  document.remove("dLen");

  // recaluculate doc final length
  document["cycle"] = m_doc_cycle;

  return document;
}


String RepaymentDocument::safeStringifyDoc(const bool ext_info_in_document) const
{
  QJsonObject Jdoc = exportDocToJson(ext_info_in_document);


  CLog::log("12 safe Sringify Doc(" + cutils::hash8c(m_doc_hash) + "): " + m_doc_type + " / " + m_doc_class + " length:" + String::number(cutils::serializeJson(Jdoc).len()) + " serialized document: " + cutils::serializeJson(Jdoc), "app", "trace");

  return cutils::serializeJson(Jdoc);
}

CDocHashT RepaymentDocument::calcDocHash() const
{
  String hashables = getDocHashableString();
  String hash = CCrypto::keccak256Dbl(hashables); // NOTE: absolutely using double hash for more security
  CLog::log("Hashable string for repayback block, doc hash(" + hash + ") hashables:" + hashables, "app", "trace");
  return hash;
}


String RepaymentDocument::getDocHashableString() const
{
  // in order to have almost same hash! we sort the attribiutes alphabeticaly
  String hashable_doc = "{";
  hashable_doc += "\"cycle\":\"" + m_doc_cycle + "\",";
  hashable_doc += "\"dClass\":\"" + m_doc_class + "\",";
  hashable_doc += "\"dType\":\"" + m_doc_type + "\",";

  hashable_doc += "\"inputs\":" + SignatureStructureHandler::stringifyInputs(m_inputs) + ",";
  hashable_doc += "\"outputs\":" + SignatureStructureHandler::stringifyOutputs(m_outputs) + "}";

  return hashable_doc;
}


String RepaymentDocument::getDocHashableString2(RepaymentDocument* a_doc)
{
  // in order to have almost same hash! we sort the attribiutes alphabeticaly
  String hashable_doc = "{";
  hashable_doc += "\"hash\":\"" + a_doc->m_doc_hash + "\",";
  hashable_doc += "\"dType\":\"" + a_doc->m_doc_type + "\",";
  hashable_doc += "\"dClass\":\"" + a_doc->m_doc_class + "\",";
  hashable_doc += "\"dVer\":\"" + a_doc->m_doc_version + "\",";
  hashable_doc += "\"cycle\":\"" + a_doc->m_doc_cycle + "\",";

  hashable_doc += "\"inputs\":" + SignatureStructureHandler::stringifyInputs(a_doc->m_inputs) + ",";
  hashable_doc += "\"outputs\":" + SignatureStructureHandler::stringifyOutputs(a_doc->m_outputs) + "}";

  return hashable_doc;
}


bool RepaymentDocument::setDocumentInputs(const QJsonValue& obj)
{
  QJsonArray inputs = obj.toArray();
  for(QJsonValueRef an_input: inputs)
  {
    QJsonArray io = an_input.toArray();
    TInput* i  = new TInput({io[0].to_string(), static_cast<COutputIndexT>(io[1].toVariant().toDouble())});
    m_inputs.push_back(i);
  }
  return true;
}

bool RepaymentDocument::setDocumentOutputs(const QJsonValue& obj)
{
  QJsonArray outputs = obj.toArray();
  for(QJsonValueRef an_output: outputs)
  {
    QJsonArray oo = an_output.toArray();
    TOutput *o  = new TOutput({oo[0].to_string(), static_cast<CMPAIValueT>(oo[1].toDouble())});
    m_outputs.push_back(o);
  }
  return true;
}


*/