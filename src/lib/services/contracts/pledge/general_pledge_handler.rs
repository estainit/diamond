use std::collections::HashMap;
use crate::{application, constants};
use crate::lib::custom_types::{CDateT, GRecordsT};
use crate::lib::database::abs_psql::q_custom_query;
use crate::lib::database::tables::{C_PLEDGED_ACCOUNTS, C_PLEDGED_ACCOUNTS_FIELDS};


//old_name_was getPledgedAccounts
pub fn get_pledged_accounts(
    c_date: &CDateT,
    _only_actives: bool) -> GRecordsT
{
    // TODO: implement only_actives filter

    // retrieve activated before 2 last cycle
    let c_date = application().get_cb_coins_date_range(c_date).to;

    // *
    // *  effective date |<----- 12Hours ---->|  evaluating point  |
    // *                .|                    |  .                 |
    // *  pledge windows |                    |  .                 |
    // *                 |                    |                    |        Active Pledges
    // *  <+++++++++++++.|++                  |  .                 |        Open  (unknown end date)
    // *  <+++++++++++++.|++++++++++++++++++++|++.++++             |        Open (indicating end date)
    // *  <+++++++++++++.|++>                 |  .                 |        Closed (definit end date)
    // *                 |                    |                    |
    // *                 |                    |                    |        Inctive Pledges
    // *                .|<+++++++++++        |  .                 |        Open (unknown end date)
    // *                .| <++++++++++>       |  .                 |        Open (indicating end date)
    // *  <++++++++++++>.|                    |  .                 |        Closed (definit end date)
    // *  <+++++++++++> .|                    |  .                 |        Closed
    // *
    // * The real calculateDate is end of 2 cycle before the given cDate.
    // * so:
    // * query on cDate=2020-03-02 12:00:01 =>
    // *     SELECT * WHERE
    // *     (status='Open' AND activate_date<'2020-03-01 23:59:59') OR
    // *     (pgd_status='Close' AND close_date>'2020-03-01 23:59:59');
    // * which returns active pledges on given cDate, even Open or Close.
    // * active means the accounts which they must do repayback.
    // *
    // * while on same date to retrieve take-placed pledges (actives or inactives)
    // * query on 2020-03-02 12:00:01 => SELECT * WHERE
    // * (status='Open' AND activate_date<'2020-03-01 23:59:59') OR (pgd_status='Close' AND close_date>'2020-03-01 23:59:59');
    // *
    // * Note: the only indicator for pledge status is "pgd_status" and not start_date or close_date


    let mut complete_query: String = "".to_string();
    if constants::DATA_BASAE_AGENT == "psql"
    {
        complete_query = format!(
            "SELECT {} FROM {} WHERE (pgd_status='{}' AND pgd_activate_date<'{}') \
            OR (pgd_status='{}' AND pgd_close_date > '{}')",
            C_PLEDGED_ACCOUNTS_FIELDS.join(","),
            C_PLEDGED_ACCOUNTS,
            constants::OPEN,
            c_date,
            constants::CLOSE,
            c_date
        );
    }

    let (_status, records) = q_custom_query(
        &complete_query,
        &vec![],
        false);
    if records.len() == 0
    { return HashMap::new(); }


    let mut pledges_by_pledger: GRecordsT = HashMap::new();
    for a_contract in &records
    {
        let a_pledger: String = a_contract["pgd_pledger"].clone();
        if !pledges_by_pledger.contains_key(&a_pledger)
        {
            // it is possible an account pledged multi time because of different pledge contracts
            pledges_by_pledger.insert(a_pledger.clone(), vec![]);
        }
        let mut tmp = pledges_by_pledger[&a_pledger].clone();
        tmp.push(a_contract.clone());
        pledges_by_pledger.insert(a_pledger, tmp);
    }
    return pledges_by_pledger;
}

/*
QVDRecordsT GeneralPledgeHandler::searchInPledgedAccounts(
  const ClausesT& clauses,
  const StringList& fields,
  const OrderT& order,
  const int& limit)
{

  QueryRes res = DbModel::select(
    stbl_pledged_accounts,
    fields,
    clauses,
    order,
    limit);
  return res.records;
}

QVDRecordsT GeneralPledgeHandler::searchInDraftPledges(
  const ClausesT& clauses,
  const StringList& fields,
  const OrderT& order,
  const int& limit)
{
  QueryRes res = DbModel::select(
    stbl_machine_draft_pledges,
    fields,
    clauses,
    order,
    limit);
  return res.records;
}

bool GeneralPledgeHandler::deleteDraftPledge(
  const ClausesT& clauses)
{
  QueryRes res = DbModel::dDelete(
    stbl_machine_draft_pledges,
    clauses);
  return true;
}

std::tuple<bool, uint32_t, String> GeneralPledgeHandler::validatePledgerSignedRequest(
  const DNAProposalDocument* proposal,
  const PledgeDocument* pledge,
  String stage,
  String cDate)
{
  String msg;

  QJsonObject dExtInfo = pledge->m_doc_ext_info[0].toObject();

  // does truly referenced the proposal
  if (proposal->getDocHash() != pledge->m_proposal_ref)
  {
    msg = "The proposal is not the Refered one in pledge req! proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref);
    CLog::log(msg, "sec", "error");
    return {false, 0, msg};
  }

  // pledger singature structure check
  QJsonObject pledger_unlock_set = dExtInfo["pledgerUSet"].toObject();
  bool is_valid_unlock = SignatureStructureHandler::validateSigStruct(
    pledger_unlock_set,
    pledge->m_pledger_address);
  if (!is_valid_unlock)
  {
    msg = "Invalid! given unlock structure for pledge req, proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref) + ") unlock:" + cutils::dumpIt(pledger_unlock_set);
    CLog::log(msg, "sec", "error");
    return {false, 0, msg};
  }

  // pledger signature & permission validate check
  String sign_message = pledge->getSignMsgAsPledger();
  QJsonArray pledger_signatures = dExtInfo["pledgerSignatures"].toArray();
  bool permited_to_pledge = false;
  for (CSigIndexT sign_inx = 0; sign_inx < pledger_signatures.len(); sign_inx++)
  {
    String a_signature = pledger_signatures[sign_inx].to_string();
    try {
      bool verify_sign_res = CCrypto::ECDSAVerifysignature(
        pledger_unlock_set["sSets"].toArray()[sign_inx].toObject()["sKey"].to_string(),
        sign_message,
        a_signature);
      if (!verify_sign_res)
      {
        msg = "The Pledge to proposal has invalid signature! proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref) + ") unlock:" + cutils::dumpIt(pledger_unlock_set);
        CLog::log(msg, "sec", "error");
        return {false, 0, msg};
      }
    } catch (std::exception) {
      msg = "The Pledge to proposal has invalid signature! proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref) + ") unlock:" + cutils::dumpIt(pledger_unlock_set);
      CLog::log(msg, "sec", "error");
      return {false, 0, msg};
    }
    if (pledger_unlock_set["sSets"].toArray()[sign_inx].toObject()["pPledge"].to_string() == constants::YES)
        permited_to_pledge = true;
  }
  if (!permited_to_pledge)
  {
    msg = "The pledger of Pledge to proposal has not permited to pledge! proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref) + ") unlock:" + cutils::dumpIt(pledger_unlock_set);
    CLog::log(msg, "sec", "error");
    return {false, 0, msg};
  }

  // control proposal cost
  auto[status, locally_recalculate_doc_dp_cost] = proposal->calcDocDataAndProcessCost(
    stage,
    pledge->m_doc_creation_date);
  if (!status)
    return {false, 0, "Failed in calculating DPCost"};

  auto[one_cycle_income, apply_cost] = ProposalHandler::calcProposalApplyCost(
    proposal->m_help_hours,
    proposal->m_help_level,
    cDate); // the creation date of the block in which contribute is recorded (start date)

  if (pledge->m_redeem_principal < locally_recalculate_doc_dp_cost + apply_cost)
  {
    msg = "Proposal costs is hiegher than requested loan! cost(" + cutils::microPAIToPAI6(apply_cost) + ") principal(" + cutils::microPAIToPAI6(pledge->m_redeem_principal) + ") proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref) + ") unlock:" + cutils::dumpIt(pledger_unlock_set);
    CLog::log(msg, "sec", "error");
    return {false, 0, msg};
  }
  if (pledge->m_redeem_repayment_amount > one_cycle_income)
  {
    msg = "Repayment amount is bigger than one cycle income! repay(" + cutils::microPAIToPAI6(pledge->m_redeem_repayment_amount) + ") one cycle(" + cutils::microPAIToPAI6(one_cycle_income) + ") proposal(" + cutils::hash8c(proposal->getDocHash()) + ") the pledge(" + cutils::hash8c(pledge->m_proposal_ref) + ") unlock:" + cutils::dumpIt(pledger_unlock_set);
    CLog::log(msg, "sec", "error");
    return {false, 0, msg};
  }

  // redeem terms control
  LoanDetails loan_details = LoanContractHandler::calcLoanRepayments(
    pledge->m_redeem_principal,
    pledge->m_redeem_annual_interest,
    pledge->m_redeem_repayment_amount,
    pledge->m_redeem_repayment_schedule);
  if (!loan_details.m_calculation_status)
  {
    msg = "Invalid repayment data! principal(" + String::number(pledge->m_redeem_principal) + ") " +
      " interest(" + String::number(pledge->m_redeem_annual_interest) + ")" +
      " amount(" + String::number(pledge->m_redeem_repayment_amount) + ")" +
      " schedule(" + String::number(pledge->m_redeem_repayment_schedule) + ")" +
      cutils::hash8c(pledge->getDocHash()) +")";
    CLog::log(msg, "sec", "error");
    return {false, 0, msg};
  }

  // so it seems a valid pledge request
  return {true, loan_details.m_repayments.len(), ""};

}

/**
 *
 * @param {*} pledge_hash
 * if pleger/plegee does not pay the costs by a valid transaction, so the pledge is not valid!
 * even proposal costs are payed properly?
 *
 */
bool GeneralPledgeHandler::removePledgeBecauseOfPaymentsFail(const String& pledge_hash)
{
  //sceptical test
  QueryRes exist = DbModel::select(
    stbl_pledged_accounts,
    {"pgd_hash"},
    {{"pgd_hash", pledge_hash}});
  if (exist.records.len() != 1)
  {
    CLog::log("Try to delete pledge strange result! " + cutils::dumpIt(exist.records), "sec", "error");
    return false;
  }

  DbModel::dDelete(
    stbl_pledged_accounts,
    {{"pgd_hash", pledge_hash}});

  return true;
}

bool GeneralPledgeHandler::reOpenPledgeBecauseOfPaymentsFail(const CDocHashT& pledge_hash)
{
  CLog::log("Do Apply reopen PledgeClosing pledge(" + cutils::hash8c(pledge_hash) + ") because of failed close payment!", "app", "warning");
  // change pledge status in table i_pledged_accounts
  DbModel::update(
    stbl_pledged_accounts,
    {{ "pgd_status", constants::OPEN},
    {"pgd_close_date", ""}},
    {{"pgd_hash", pledge_hash}});

  return true;
}

/**
 * @brief recognizeSignerTypeInfo
 * @param pledge
 * @param signer_address
 * return {status, signer_type, by_type}
 */
std::tuple<bool, String, String> GeneralPledgeHandler::recognizeSignerTypeInfo(
  const PledgeDocument* pledge,
  const CAddressT& signer_address)
{
  String by_type = "";
  String signer_type = "";

  HashMap<CAddressT, bool> addDict = {};
  if (signer_address == "")
  {
    auto[wstat, addInfo] = Wallet::searchWalletAdress({pledge->m_pledger_address, pledge->m_pledgee_address, pledge->m_arbiter_address});
    Q_UNUSED(wstat);
    if (addInfo.len()== 0)
    {
      CLog::log("Non of these 3 address not controlled by machine wallet Pledger(" + cutils::short_bech16(pledge->m_pledger_address) + ") Pledgee(" + cutils::short_bech16(pledge->m_pledgee_address) + ") Arbiter(" + cutils::short_bech16(pledge->m_arbiter_address) + ") Pledge(" + cutils::hash8c(pledge->getDocHash()) + ") ", "sec", "warning");    // non of these 3 address not controlled by machine wallet
      return {false, "", ""};
    }

    for (QVDicT anAdd: addInfo)
      addDict[anAdd["wa_address"].to_string()] = true;

  } else {
      addDict[signer_address] = true;
  }

  if (addDict.keys().contains(pledge->m_pledgee_address))
  {
    by_type = constants::PLEDGE_CONCLUDER_TYPES::ByPledgee;
    signer_type = "pledgee";

  } else if (addDict.keys().contains(pledge->m_arbiter_address))
  {
    by_type = constants::PLEDGE_CONCLUDER_TYPES::ByArbiter;
    signer_type = "arbiter";

  } else if (addDict.keys().contains(pledge->m_pledger_address))
  {
    by_type = constants::PLEDGE_CONCLUDER_TYPES::ByPledger;
    signer_type = "pledger";
  }

  if ((signer_type == "") || (by_type == ""))
  {
    CLog::log("Either signer_type or by_type is invalid signer_type(" + signer_type + ") or by_type(" + by_type + ") Pledge(" + cutils::hash8c(pledge->getDocHash()) + ") ", "sec", "warning");
    return {false, "", ""};
  }

  return {true, signer_type, by_type};
}

std::tuple<bool, String, String> GeneralPledgeHandler::recognizeSignerTypeInfo(
  Document* pledge,
  const CAddressT& signer_address)
{
  return recognizeSignerTypeInfo(dynamic_cast<PledgeDocument*>(pledge), signer_address);
}

bool GeneralPledgeHandler::insertAPledge(
  const Block& block,
  const PledgeDocument* pledge)
{

  CLog::log("Create a new pledge for pledger(" + cutils::short_bech16(pledge->m_pledger_address) + ") pledge(" + cutils::hash8c(pledge->getDocHash()) + ") ", "app", "trace");

  CDateT real_activate_date = cutils::getACycleRange(block.m_block_creation_date
    // forwardByCycle: iConsts.PLDEGE_ACTIVATE_OR_DEACTIVATE_MATURATION_CYCLE_COUNT
  ).from;

  QVDicT values {
    {"pgd_hash", pledge->m_doc_hash},
    {"pgd_type", pledge->m_doc_type},
    {"pgd_class", pledge->m_doc_class},
    {"pgd_version", pledge->m_doc_version},
    {"pgd_pledger_sign_date", pledge->m_pledger_sign_date},
    {"pgd_pledgee_sign_date", pledge->m_pledgee_sign_date},
    {"pgd_arbiter_sign_date", pledge->m_arbiter_sign_date},
    {"pgd_activate_date", real_activate_date},
    {"pgd_pledger", pledge->m_pledger_address},
    {"pgd_pledgee", pledge->m_pledgee_address},
    {"pgd_arbiter", pledge->m_arbiter_address},
    {"pgd_principal", QVariant::fromValue(pledge->m_redeem_principal)},
    {"pgd_annual_interest", pledge->m_redeem_annual_interest},
    {"pgd_repayment_offset", QVariant::fromValue(pledge->m_redeem_repayment_offset)},
    {"pgd_repayment_amount", QVariant::fromValue(pledge->m_redeem_repayment_amount)},
    {"pgd_repayment_schedule", QVariant::fromValue(pledge->m_redeem_repayment_schedule)},
    {"pgd_status", constants::OPEN} // by default by inserting a pledge is initialy open and active
  };
  CLog::log("Inserting new onchain pledge(" + cutils::hash8c(pledge->getDocHash()) + ") values:" + cutils::dumpIt(values), "app", "trace");
  DbModel::insert(
    stbl_pledged_accounts,
    values);

  return true;
}


bool GeneralPledgeHandler::addContractToDb(
  const String& live_contract_type,
  const String& live_contract_class,
  const String& live_contract_ref_hash,
  const String& live_contract_descriptions,
  const String& live_contract_body)
{
  QVDicT values {
    {"lc_type", live_contract_type},
    {"lc_class", live_contract_class},
    {"lc_ref_hash", live_contract_ref_hash},  // pledge document hash
    {"lc_descriptions", live_contract_descriptions},
    {"lc_body", live_contract_body}
  };
  DbModel::insert(
    stbl_machine_onchain_contracts,
    values);

  return true;
}


bool GeneralPledgeHandler::activatePledge(
  const Block& block,
  const PledgeDocument* pledge)
{
  insertAPledge(block, pledge);

  auto[status, signer_type, by_type] = recognizeSignerTypeInfo(pledge);
  Q_UNUSED(by_type);
  if(!status)
    return false;

  String desc = "The contract(" + cutils::hash16c(pledge->getDocHash()) + ") ";
  if (signer_type == "pledgee")
  {
    desc += "pledger(" + cutils::short_bech16(pledge->m_pledger_address) + ") pledgee(You: " + cutils::short_bech16(pledge->m_pledgee_address) + ") arbiter(" + cutils::short_bech16(pledge->m_arbiter_address) + ") ";
  } else if (signer_type == "arbiter")
  {
    desc += "pledger(" + cutils::short_bech16(pledge->m_pledger_address) + ") pledgee(" + cutils::short_bech16(pledge->m_pledgee_address) + ") arbiter(You: " + cutils::short_bech16(pledge->m_arbiter_address) + ") ";
  } else if (signer_type == "pledger")
  {
    desc += "pledger(You: " + cutils::short_bech16(pledge->m_pledger_address) + ") pledgee(" + cutils::short_bech16(pledge->m_pledgee_address) + ") arbiter(" + cutils::short_bech16(pledge->m_arbiter_address) + ") ";
  }

  // insert pledge contract in machine local db (if controlled by machine)
  auto wrap_res = BlockUtils::wrapSafeContentForDB(pledge->safeStringifyDoc());
  if (!wrap_res.status)
    return false;

  addContractToDb(
    pledge->m_doc_type,
    pledge->m_doc_class,
    pledge->m_doc_hash,
    desc,
    wrap_res.content);

  return true;
}

bool GeneralPledgeHandler::doApplyClosingPledge(
  const Block& block,
  const ClosePledgeDocument* pledge)
{
  /**
  * NOTE: if pgd_status=Open then pgd_close_date is a indicating date
  *       if pgd_status=Close then pgd_close_date is a definitive close date
  */

  CLog::log("Do Apply Pledge Closing for pledge(" + cutils::hash8c(pledge->getDocHash()) + ")", "app", "info");

  // change pledge status in table _pledged_accounts
  CDateT realCloseDate = cutils::getACycleRange(block.m_block_creation_date).from;

  DbModel::update(
    stbl_pledged_accounts,
    {{"pgd_status", constants::CLOSE},
    {"pgd_close_date", realCloseDate}},
    {{"pgd_hash", pledge->getDocHash()}});

  return true;
}

String GeneralPledgeHandler::renderPledgeDocumentToHTML(const QJsonObject& Jpledge)
{
  PledgeDocument pledge = PledgeDocument(Jpledge);
  String out = "";
  out += "Pledge Class: " + pledge.m_doc_class;
  out += "Pledge Comment: " + pledge.m_doc_comment;
  out += "Pledger: " + pledge.m_pledger_address;
  out += "Pledgee: " + pledge.m_pledgee_address;
  out += "Arbiter: " + pledge.m_arbiter_address;
  out += "Redeem Terms: " + pledge.m_doc_class;
  out += "\tPrincipal: " + cutils::microPAIToPAI6(pledge.m_redeem_principal);
  out += "\tAnnual Interest: " + String::number(pledge.m_redeem_annual_interest);
  out += "\tRepayment Offset: " + String::number(pledge.m_redeem_repayment_offset);
  out += "\tRepayment Amount: " + cutils::microPAIToPAI6(pledge.m_redeem_repayment_amount);
  out += "\tRepayment Schedule: " + String::number(pledge.m_redeem_repayment_schedule) + " times (2 time in day)";
  out += "Creation Date: " + pledge.m_doc_creation_date;
  out += "Pledger Sign Date: " + pledge.m_pledger_sign_date;
  return out;
}


bool GeneralPledgeHandler::pledgerSignsPledge(PledgeDocument* pledge_document)
{
  String msg;
  if (pledge_document->m_pledger_address == "")
  {
    msg = "The pledger ${pledger} is null";
    CLog::log(msg, "app", "error");
    return false;
  }

  if (!CCrypto::isValidBech32(pledge_document->m_pledger_address))
  {
    msg = "The pledger " + pledge_document->m_pledger_address + " is not a valid Bech32 address!";
    CLog::log(msg, "app", "error");
    return false;
  }

  QVDRecordsT pledgerAddInfo = Wallet::getAddressesInfo({pledge_document->m_pledger_address});
  if (pledgerAddInfo.len() != 1)
  {
    msg = "The Invoice Address which is going to be pledged, is not controlled by your wallet! address(" + pledge_document->m_pledger_address + ")";
    CLog::log(msg, "app", "error");
    return false;
  }

  QJsonObject addrDtl = cutils::parseToJsonObj(pledgerAddInfo[0]["wa_detail"].to_string());
  StringList signatures {};
  QJsonObject dExtInfo {};
  for (auto an_unlock_set_: addrDtl["uSets"].toArray())
  {
    // if already signed exit
    if (signatures.len() > 0)
      continue;
    String pPledge = constants::NO;
    QJsonObject an_unlock_set = an_unlock_set_.toObject();
    QJsonArray sSets = an_unlock_set["sSets"].toArray();
    for (auto aSign: sSets)
      if (aSign.toObject()["pPledge"].to_string() == constants::YES)
        pPledge = constants::YES;

    if (pPledge == constants::YES)
    {
      pledge_document->m_doc_ext_info = QJsonArray{QJsonObject {{"pledgerUSet", an_unlock_set}}};
      String sign_message = pledge_document->getSignMsgAsPledger(); //{ pledge: pledge_document, dExtInfo: pledge_document.dExtInfo });
      for (CSigIndexT inx = 0; inx < sSets.len(); inx++)
      {
        auto[signing_res, signature_hex, signature] = CCrypto::ECDSAsignMessage(
          addrDtl["the_private_keys"].toObject()[an_unlock_set["salt"].to_string()].toArray()[inx].to_string(),
          sign_message);
        if (!signing_res)
        {
          msg = "Failed in sign pledge, Salt(" + an_unlock_set["salt"].to_string() + ")";
          CLog::log(msg, "app", "error");
          return false;
        }
        signatures.push(String::fromStdString(signature_hex));
      }
    }
  }
  if (signatures.len() == 0)
  {
    msg = "The pledger Signs Pledge couldn't sign it";
    CLog::log(msg, "app", "error");
    return false;
  }

  pledge_document->m_doc_ext_info = QJsonArray{QJsonObject {
    {"pledgerUSet", pledge_document->m_doc_ext_info[0].toObject()["pledgerUSet"].toObject()},
    {"pledgerSignatures", cutils::convertStringListToJSonArray(signatures)}}};

  CLog::log("pledge document: " + pledge_document->safeStringifyDoc(), "app", "info");

  return true;
}

std::tuple<bool, PledgeDocument*> GeneralPledgeHandler::doPledgeAddress(
  const CAddressT& pledger_address,
  const CAddressT& pledgee_address,
  const CDocHashT& proposal_ref,
  const CMPAIValueT principal,
  const double annual_interest,
  const CMPAIValueT repayment_amount,
  const uint64_t repayments_number,

  const uint64_t repayment_schedule,
  const uint64_t repayment_offset,
  const CAddressT& arbiter_address,
  const CAddressT& document_type,
  const CAddressT& document_class,
  const CDateT& pledger_sign_date,
  const CDateT& creation_date)
{
  String msg;

  if (pledger_address == "") {
    msg = "The pledger ${pledger} is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  if (pledgee_address == "") {
    msg = "The pledgee_address ${pledger} is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  if ((document_class == constants::PLEDGE_CLASSES::PledgeP) && (proposal_ref == ""))
  {
    msg = "The proposal_ref ${proposal_ref} is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  // the PAIs are loaned
  if (principal < 1)
  {
    msg = "The principal " + String::number(principal)+ " is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  if (annual_interest <= 0)
  {
    msg = "The annual_interest ${annual_interest} is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  // the amount is cutting from income and payed to Pledgee
  if (repayment_amount < 1)
  {
    msg = "The Repayment Amount " + String::number(repayment_amount) + " is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  // the amount is cutting from income and payed to Pledgee
  if (repayments_number < 1)
  {
    msg = "The Repayments number" + String::number(repayments_number) + " is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  // how many times do repayment in a year
  if (repayment_schedule < 1)
  {
    msg = "The Repayments schedule" + String::number(repayment_schedule) + " is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  // starting to pay the first repayment after n hours
  // how many minutes is offset. in other word, the first repayment can started after x minutes from approving pledge and giving loan
  // TODO: offset bigger than zero must be implemented
  if (repayment_offset > 0)
  {
    msg = "The repayment_offset " + String::number(repayment_offset) + " is invalid!";
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }


  // validating interest and repayments number and principal are in equation?
  LoanDetails loan_details = LoanContractHandler::calcLoanRepayments(
    principal,
    annual_interest,
    repayment_amount,
    repayment_schedule);

  if (!loan_details.m_calculation_status)
  {
    msg = "Falied on loan calculating details" + loan_details.dumpMe();
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }


  PledgeDocument* pledge_document = new PledgeDocument(QJsonObject {
    {"dType", document_type},
    {"dClass", document_class},
    {"dVer", "0.0.2"}});

  pledge_document->m_doc_creation_date = creation_date;
  pledge_document->m_pledger_sign_date = pledger_sign_date;
  pledge_document->m_pledger_address = pledger_address;
  pledge_document->m_pledgee_address = pledgee_address;
  pledge_document->m_arbiter_address = arbiter_address;
  pledge_document->m_proposal_ref = proposal_ref;
  pledge_document->m_redeem_principal = principal;
  pledge_document->m_redeem_annual_interest = annual_interest;
  pledge_document->m_redeem_repayment_offset = repayment_offset;
  pledge_document->m_redeem_repayment_amount = repayment_amount;
  pledge_document->m_redeem_repayment_schedule = repayment_schedule;
  pledge_document->m_redeem_repayments_number = loan_details.m_repayments.len();

  bool pledger_sign_res = pledgerSignsPledge(pledge_document);
  if (!pledger_sign_res) {
    msg = "Falied on pledger Signs Pledge" + pledge_document->safeStringifyDoc();
    CLog::log(msg, "app", "error");
    return {false, nullptr};
  }

  pledge_document->setDocLength();
  CLog::log("Signed pledge document: " + pledge_document->safeStringifyDoc(), "app", "info");

  return {true, pledge_document};
}

// js name was savepldgDraft
bool GeneralPledgeHandler::saveDraftPledgedProposal(
  PledgeDocument* pledge_document,
  const String& mp_code)
{
  QVDicT values {
    {"dpl_mp_code", mp_code},
    {"dpl_type", pledge_document->m_doc_type},
    {"dpl_class", pledge_document->m_doc_class},
    {"dpl_version", pledge_document->m_doc_version},
    {"dpl_comment", pledge_document->m_doc_comment},
    {"dpl_pledger", pledge_document->m_pledger_address},
    {"dpl_pledgee", pledge_document->m_pledgee_address},
    {"dpl_arbiter", pledge_document->m_arbiter_address},
    {"dpl_doc_ref", pledge_document->m_proposal_ref},
    {"dpl_body", BlockUtils::wrapSafeContentForDB(pledge_document->safeStringifyDoc()).content},
    {"dpl_req_date", pledge_document->m_pledger_sign_date}};

  return DbModel::insert(
    stbl_machine_draft_pledges,
    values);
}

bool GeneralPledgeHandler::createAndRecordPPTBundle(
  DNAProposalDocument* proposal,
  PledgeDocument* pledge,
  BasicTxDocument* proposalPayerTrx,
  BasicTxDocument* pledgeDocPayerTrx,
  const CDateT& creation_date)
{

  QJsonObject bundle {
    {"pledgeeSignedPledge", pledge->exportDocToJson()},
    {"pledgeDocPayerTrx", pledgeDocPayerTrx->exportDocToJson()},

    {"proposal", proposal->exportDocToJson()},
    {"proposalPayerTrx", proposalPayerTrx->exportDocToJson()}};

  String bundle_str = cutils::serializeJson(bundle);
  CDocHashT hash = CCrypto::keccak256(bundle_str);
  bundle["hash"] = hash;
  return TmpContentsHandler::insertTmpContent(
    constants::BundlePPT,
    "Basic",
    hash,
    BlockUtils::wrapSafeContentForDB(bundle_str).content);
}

/**
 * @brief GeneralPledgeHandler::handleReceivedProposalLoanRequest
 * @return {status, should remove record}
 */
std::tuple<bool, bool> GeneralPledgeHandler::handleReceivedProposalLoanRequest(
  const String& sender,
  const QJsonObject& payload,
  const String& connection_type,
  const CDateT& receive_date)
{
  CLog::log("payload in handle Received Proposal Loan Request: " + cutils::serializeJson(payload), "app", "info");

  String msg;
  QJsonObject Jproposal = payload["proposal"].toObject();
  QJsonObject Jpledge = payload["pledgerSignedPledge"].toObject();

  DNAProposalDocument* proposal = new DNAProposalDocument(Jproposal);
  PledgeDocument* pledge = new PledgeDocument(Jpledge);

  String cdVer = payload["cdVer"].to_string();
  if ((cdVer == "") || !cutils::isValidVersionNumber(cdVer))
  {
    msg = "missed cdVer gql in handle Received Proposal Loan Request";
    CLog::log(msg, "app", "error");
    return {false, true};
  }

  // do a bunch of control on pledgeRequest
  auto[status_pledger_sign, repayments_number, validate_res_msg] = validatePledgerSignedRequest(
    proposal,
    pledge,
    constants::STAGES::Validating,
    cutils::getNow());
  if (!status_pledger_sign)
  {
    msg = "Failed in validate Pledger Signed Request " + validate_res_msg;
    CLog::log(msg, "app", "error");
    delete proposal;
    delete pledge;
    return {false, true};
  }

  QJsonObject payload_to_record {
    {"proposal", proposal->exportDocToJson()},
    {"pledgerSignedPledge", pledge->exportDocToJson()},
  };
  String payload_to_record_str = BlockUtils::wrapSafeContentForDB(cutils::serializeJson(payload_to_record)).content;
  CDocHashT content_hash = CCrypto::keccak256(payload_to_record_str);
  delete proposal;
  delete pledge;

  bool res = TmpContentsHandler::insertTmpContent(
    constants::receivedPLR,
    constants::receivedPLR,
    content_hash,
    payload_to_record_str,
    constants::NEW,
    receive_date);

  return {res, true};
}


std::tuple<bool, String> GeneralPledgeHandler::pledgeeSignsPledge(PledgeDocument* pledge)
{
  String msg;
  if (pledge->m_pledgee_sign_date == "")
    pledge->m_pledgee_sign_date = cutils::getNow();

  if (pledge->m_pledgee_address == "")
  {
    msg = "The pledgee is missed";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  if (!CCrypto::isValidBech32(pledge->m_pledgee_address))
  {
    msg = "The pledgee(" + pledge->m_pledgee_address + " is not a valid Bech32 address!";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  QVDRecordsT pledgeeAddInfo = Wallet::getAddressesInfo({pledge->m_pledgee_address});
  if (pledgeeAddInfo.len() != 1)
  {
    msg = "The pledgee Address(" + cutils::short_bech16(pledge->m_pledgee_address) + ") is not controlled by your wallet!";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  QJsonObject addrDtl = cutils::parseToJsonObj(pledgeeAddInfo[0]["wa_detail"].to_string());
  StringList signatures {};
  QJsonObject dExtInfo = pledge->m_doc_ext_info[0].toObject();
  for (auto an_unlock_set_: addrDtl["uSets"].toArray())
  {
    // if already signed exit
    if (signatures.len() > 0)
      continue;

    QJsonObject an_unlock_set = an_unlock_set_.toObject();
    QJsonArray sSets = an_unlock_set["sSets"].toArray();

    dExtInfo["pledgeeUSet"] = an_unlock_set;
    String sign_message = pledge->getSignMsgAsPledgee();
    for (int64_t inx = 0; inx < sSets.len(); inx++)
    {
      auto[signing_res, signature_hex, signature] = CCrypto::ECDSAsignMessage(
        addrDtl["the_private_keys"].toObject()[an_unlock_set["salt"].to_string()].toArray()[inx].to_string(),
        sign_message);
      if (signing_res)
        signatures.push(String::fromStdString(signature_hex));
    }
  }
  if (signatures.len() == 0)
  {
    msg = "The pledgee Signs Pledge couldn't sign it";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  dExtInfo["pledgeeSignatures"] = cutils::convertStringListToJSonArray(signatures);
  pledge->m_doc_ext_info = QJsonArray {dExtInfo};
  CLog::log("The pledgee-signed pledge Contract: " + pledge->safeStringifyDoc(), "app", "info");

  return {true, "Pledgee signed contract"};
}

std::tuple<bool, String> GeneralPledgeHandler::pledgeeSignsProposalLoanRequestBundle(
  DNAProposalDocument* proposal,
  PledgeDocument* pledge)
{
  String msg;

  CLog::log("going to validate proposal: " + proposal->safeStringifyDoc());
  Block* tmp_block1 = new Block(QJsonObject {
    {"bCDate", cutils::getNow()},
    {"bType", "futureBlockproposal"},
    {"bHash", "futureHashproposal"}});
  GenRes full_validate = proposal->fullValidate(tmp_block1);
  delete tmp_block1;
  if (!full_validate.status)
    return {false, "Failed in proposal full Validate, " + full_validate.msg};


  // do a bunch of control on pledgeRequest
  auto[status_pledger_sign, repayments_number, validate_res_msg] = validatePledgerSignedRequest(
    proposal,
    pledge,
    constants::STAGES::Creating,
    cutils::getNow());
  if (!status_pledger_sign)
      return {false, "Invalid pledger signature! " + validate_res_msg};

  // (START) dummy pledge siging in order to calculate pledge length after signature
  pledge->m_transaction_ref = "0000000000000000000000000000000000000000000000000000000000000000";
  auto[sign_status, sign_msg] = pledgeeSignsPledge(pledge);
  if (!sign_status)
    return {sign_status, sign_msg};

  pledge->m_doc_ext_hash = "0000000000000000000000000000000000000000000000000000000000000000";
  pledge->setDocLength();

  auto[cost_status, pledge_dp_cost] = pledge->calcDocDataAndProcessCost(
    constants::STAGES::Creating,
    cutils::getNow());
  if (!cost_status)
  {
    msg = "Failed in plege/proposal cost calculation";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  // create a transaction for payment
  auto changeback_res = Wallet::getAnOutputAddress(
    true,
    constants::SIGNATURE_TYPES::Basic,
    "1/1");
  if (!changeback_res.status)
  {
    msg = "Failed in create changeback address for pledge!";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CAddressT change_back_address = changeback_res.msg;

  std::vector<TOutput> outputs1 {
    TOutput{change_back_address, 1, constants::OUTPUT_CHANGEBACK},
    TOutput{"TP_PROPOSAL", pledge->m_redeem_principal, constants::OUTPUT_TREASURY}};

  auto[coins_status1, coins_msg1, spendable_coins1, spendable_amount1] = Wallet::getSomeCoins(
    cutils::CFloor(pledge->m_redeem_principal * 1.3),  // an small portion bigger to support DPCosts
    constants::COIN_SELECTING_METHOD::PRECISE);
  for(auto a_coin: spendable_coins1)
    CLog::log("Spendables 1: " + a_coin.dumpMe(), "app", "info");

  if (!coins_status1)
    return {false, coins_msg1};

  if (spendable_coins1.keys().len() == 0)
  {
    msg = "Wallet couldn't find! proper UTXOs to spend 1";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto trx_template1 = BasicTransactionTemplate {
    spendable_coins1,
    outputs1,
    static_cast<CMPAIValueT>(pledge->m_redeem_principal * 0.7),  // max trx fee
    0,    // pre calculated dDPCost
    "Payed(by Pledgee) for applying proposal to Vote process",
    proposal->getDocHash()};
  auto[res_status1, res_msg1, proposal_payer_trx, dp_cost1] = BasicTransactionHandler::makeATransaction(trx_template1);
  if (!res_status1)
    return {false, res_msg1};

  CLog::log("Signed trx for proposal cost:" + proposal_payer_trx->safeStringifyDoc(true), "app", "info");
  pledge->m_transaction_ref = proposal_payer_trx->getDocHash();

  // re-sign pledge for final results
  auto[sign_status2, sign_msg2] = pledgeeSignsPledge(pledge);
  if (!sign_status2)
    return {sign_status2, sign_msg2};

  pledge->setDExtHash();

  if (pledge->m_doc_length != pledge->safeStringifyDoc().len())
  {
    msg = "Worng pldge contract length(${utils.stringify(pledge).length})calculation: ${utils.stringify(pledge)}";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  pledge->setDocHash(); // = this.calcHashDPledge(pledge);

  CLog::log("full Validate completed pledge contract: " + pledge->safeStringifyDoc(true), "app", "info");
  Block* tmp_block2 = new Block(QJsonObject {
    {"bCDate", cutils::getNow()},
    {"bType", "futureBlockpledge"},
    {"bHash", "futureHashpledge"}});
  GenRes final_validate_pledge = pledge->fullValidate(tmp_block2);
  delete tmp_block2;
  if (!final_validate_pledge.status)
    return {false, "Failed in pledge full Validate, " + final_validate_pledge.msg};



  // pay for pledge doc too
  std::vector<TOutput> outputs2 {
    TOutput{change_back_address, 1, constants::OUTPUT_CHANGEBACK},
    TOutput{"TP_PLEDGE", pledge_dp_cost, constants::OUTPUT_TREASURY}};

  auto[coins_status2, coins_msg2, spendable_coins2, spendable_amount2] = Wallet::getSomeCoins(
    cutils::CFloor(pledge_dp_cost * 2), // an small portion bigger to support DPCosts
    constants::COIN_SELECTING_METHOD::PRECISE,
    0,
    spendable_coins1.keys());  // avoid double spending inputs
  for(auto a_coin: spendable_coins2)
    CLog::log("Spendables 2: " + a_coin.dumpMe(), "app", "info");

  if (!coins_status2)
    return {coins_status2, coins_msg2};

  if (spendable_coins2.keys().len() == 0)
  {
    msg = "Wallet couldn't find! proper UTXOs to spend 2";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto trx_template2 = BasicTransactionTemplate {
    spendable_coins2,
    outputs2,
    static_cast<CMPAIValueT>(pledge_dp_cost * 2),  // max trx fee
    0,    // dDPCost
    "Payed(by Pledgee) for PledgeP document cost",
    pledge->getDocHash()};
  auto[res_status2, res_msg2, pledge_payer_trx, dp_cost2] = BasicTransactionHandler::makeATransaction(trx_template2);
  if (!res_status2)
    return {false, res_msg2};

  CLog::log("Signed trx for pledge cost:" + pledge_payer_trx->safeStringifyDoc(true), "app", "info");


  // mark UTXOs as used in local machine
  Wallet::locallyMarkUTXOAsUsed(proposal_payer_trx);
  Wallet::locallyMarkUTXOAsUsed(pledge_payer_trx);

  // push whole Proposal, Pledge & transactions all in a bundle in
  bool bundleProposalPledgeTrx = createAndRecordPPTBundle(
    proposal,
    pledge,
    proposal_payer_trx,
    pledge_payer_trx);

  return {bundleProposalPledgeTrx, "Done"};
}
*/