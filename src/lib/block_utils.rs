use serde_json::{json};
use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::custom_types::{JSonObject, VString};


//old_name_was ifAncestorsAreValid
pub fn if_ancestors_are_valid(ancestors: &VString) -> bool
{
    // TODO: since the address is in hex base, add hex char controll
    for an_ancestor in ancestors
    {
        if (an_ancestor == "")
            || (an_ancestor.len() != 64)
            || !cutils::has_only_hex_chars(an_ancestor)
        {
            return false;
        }
    }
    return true;
}


/*
/**
 * @brief BlockUtils::retrieveDPCostInfo
 * @param doc
 * @param backer
 * @return {treasury_incomes, backer_incomes}
 */
std::tuple<bool, uint64_t, uint64_t> BlockUtils::retrieveDPCostInfo(
  const Document* doc,
  const String& backer)
{
  std::vector<TOutput*> outputs = doc->get_outputs();
  /**
  * the block cost payment transaction is a document that always has to has no input and 2 outputs.
  * 0. TP_DP   Treasury Payment Data&  Process Cost
  * 1. backer fee
  */
  if (
    (outputs.len() != 2) ||
    (outputs[0].m_address != "TP_DP") ||
    (outputs[1].m_address != backer))
  {
    CLog::log("Invalid treasury payment because of receiver Doc(" + cutils::hash8c(doc->get_doc_hash()) + ") account(" + outputs[0].m_address + ") or Backer address(" + outputs[1].m_address + ")! ", "trx", "error");
    return {false, 0, 0};
  }

  String ddd = doc->safe_stringify_doc();
  DocLenT len = static_cast<DocLenT>(ddd.len());
  if (len > constants::MAX_DPCostPay_DOC_SIZE)
  {
    CLog::log("Invalid treasury payment doc length in Doc(" + cutils::hash8c(doc->get_doc_hash()) + ")! ", "trx", "error");
    return {false, 0, 0};
  }

  return { true, outputs[0].m_amount, outputs[1].m_amount };
}


*/

//old_name_was wrapSafeContentForDB
pub fn wrap_safe_content_for_db(content: &String, safe_ver: &str)
                                -> (bool, String, String)
{
    // to make a safe string to insert in db, jus convert it to base64
    if safe_ver == "0.0.0"
    {
        let b64 = ccrypto::b64_encode(content);
        let json_obj: JSonObject = json!({
          "sfVer": safe_ver,
          "content": b64});
        //    CLog::log("Safe Wrapped Content: " + cutils::serializeJson(jsonObj), "app");
        return (
            true,
            safe_ver.to_string(),
            cutils::controlled_json_stringify(&json_obj),
        );
    } else {
        let msg = format!("unknown sfVer version: {}", safe_ver);
        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Error);

        return (false,
                safe_ver.to_string(),
                msg,
        );
    }
}

//old_name_was unwrapSafeContentForDB
pub fn unwrap_safed_content_for_db(wrapped: &String) -> (bool, String, String)
{
    let _deserialization_status = false;
    let (status, json_object) = cutils::controlled_str_to_json(wrapped);
    if !status
    {
        let err_msg = "Invalid wrapped content! ".to_string();
        dlog(
            &err_msg,
            constants::Modules::App,
            constants::SecLevel::Error);
        dlog(
            &wrapped,
            constants::Modules::App,
            constants::SecLevel::Error);

        return (
            false,
            constants::DEFAULT_SAFE_VERSION.to_string(),
            err_msg);
    }

    let safe_version = remove_quotes(&json_object["sfVer"]);
    let content = remove_quotes(&json_object["content"]);
    let (status, content) = ccrypto::b64_decode(&content);
    if !status
    {
        dlog(
            &"Failed in b64 dec a safe content ".to_string(),
            constants::Modules::App,
            constants::SecLevel::Error);

        return (
            false,
            safe_version,
            "Invalid b64 wrapped content! ".to_string());
    }

    return (true, safe_version, content);
}

/*

const StringList to_string_fields = {
"b_hash", "b_type", "b_cycle", "b_ext_root_hash", "b_docs_root_hash", "b_creation_date",
"b_receive_date", "b_confirm_date", "b_backer", "b_coins_imported",

"dbm_block_hash", "dbm_doc_hash", "dbm_last_control",

"pr_hash", "pr_type", "pr_class", "pr_version", "pr_title", "pr_descriptions",
"pr_tags", "pr_project_id", "pr_polling_profile", "pr_contributor_account",
"pr_start_voting_date", "pr_conclude_date", "pr_approved",

"ppr_name", "ppr_activated", "ppr_perform_type", "ppr_amendment_allowed",
"ppr_votes_counting_method", "ppr_version",

"pll_start_date", "pll_hash", "pll_creator", "pll_type", "pll_class", "pll_ref",
"pll_ref_type", "pll_ref_class", "pll_version", "pll_comment",
"pll_status", "pll_ct_done",

"ba_hash", "ba_pll_hash", "ba_voter", "ba_creation_date",
"ba_receive_date",

"dn_creation_date", "dn_project_hash", "dn_shareholder", "dn_doc_hash", "dn_title",
"dn_descriptions", "dn_tags",

"tr_coin", "tr_block_hash", "tr_creation_date", "tr_cat", "tr_title",

"sp_coin", "sp_spend_loc", "sp_spend_date",

"ut_coin", "ut_ref_creation_date", "ut_o_address",

"pgd_hash", "pgd_type", "pgd_class", "pgd_version",
"pgd_pledger_sign_date", "pgd_pledgee_sign_date", "pgd_arbiter_sign_date",
"pgd_activate_date", "pgd_close_date",
"pgd_pledger", "pgd_pledgee", "pgd_arbiter", "pgd_status",

"sig_creation_date", "sig_block_hash", "sig_signaler", "sig_key", "sig_value",

"ag_parent", "ag_iname", "ag_hash", "ag_language", "ag_title",
"ag_description", "ag_tags", "ag_content_format_version", "ag_creator",
"ag_creation_date", "ag_last_modified",  "ag_doc_hash",

"ap_creation_date", "ap_ag_hash", "ap_doc_hash", "ap_creator", "ap_reply",
"ap_format_version", "ap_opinion",

"wkp_creation_date", "wkp_hash", "wkp_iname", "wkp_doc_hash",
"wkp_language", "wkp_format_version", "wkp_creator", "wkp_title",

"wkc_wkp_hash", "wkc_content",

"in_register_date", "in_hash", "in_name", "in_owner", "in_doc_hash", "in_is_settled",

"nb_creation_date", "nb_in_hash", "nb_doc_hash", "nb_bind_type", "nb_title",
"nb_comment", "nb_status",

"apr_creation_date", "apr_conclude_date", "apr_approved", "apr_hash", "apr_creator",
"apr_subject", "apr_values", "apr_comment", "apr_conclude_info",

"arh_apply_date", "arh_hash", "arh_subject",

"st_voter", "st_vote_date", "st_coin", "st_logger_block", "st_spender_block",
"st_spender_doc", "st_receive_order", "st_spend_date"

"rt_block_hash", "rt_doc_hash", "rt_coin",

"cycleStartDate"


};

const StringList to_int_fields = {
"b_trxs_count", "b_docs_count", "b_ancestors_count",

"pr_help_hours", "pr_help_level", "pr_voting_timeframe",

"pll_timeframe",
"pll_y_count", "pll_y_shares", "pll_y_gain", "pll_y_value",
"pll_n_count", "pll_n_shares", "pll_n_gain", "pll_n_value",
"pll_a_count", "pll_a_shares", "pll_a_gain", "pll_a_value",

"ba_voter_shares", "ba_vote", "ba_vote_c_diff", "ba_vote_r_diff",

"dn_help_hours", "dn_help_level", "dn_votes_y", "dn_votes_a", "dn_votes_n",

"pgd_repayment_offset", "pgd_repayment_amount", "pgd_repayment_schedule",

"ap_reply_point",

"transactionMinimumFee", "docExpenseDict", "basePricePerChar", "blockFixCost"

};

const StringList to_double_fields = {
"b_confidence",

"pgd_principal", "pgd_annual_interest",

"minShareToAllowedIssueFVote", "minShareToAllowedVoting", "minShareToAllowedSignCoinbase",

"ut_o_value",

"tr_value",

"getMinShareToAllowedIssueFVote", "getMinShareToAllowedVoting", "getMinShareToAllowedSignCoinbase",
"getTransactionMinimumFee", "getBasePricePerChar", "getBlockFixCost"


};

const StringList to_comma_splited_string_fields = {
"b_ancestors", "b_descendants",
};

const StringList to_wrap_unwrap_fields = {
"b_body", "b_signals",

"ap_attrs",

"nb_conf_info",

};

const StringList to_serialize_unserilize_fields = {
"b_body", "b_signals",

"ap_attrs",

"nb_conf_info",

};

const StringList to_string_to_double_fields = {
"arh_value"
};


QVDicT BlockUtils::convertToQVDicT(const JSonObject& record)
{
QVDicT out {};
StringList keys = record.keys();

for (String a_key: cutils::arrayIntersection(keys, to_string_fields))
out[a_key] = record[a_key).t]_string();

for (String a_key: cutils::arrayIntersection(keys, to_double_fields))
out[a_key] = record[a_key).t]Double();

for (String a_key: cutils::arrayIntersection(keys, to_int_fields))
out[a_key] = record[a_key).t]Int();

for (String a_key: cutils::arrayIntersection(keys, to_wrap_unwrap_fields))
{
String content = "";
try {
  auto wrapped = BlockUtils::wrapSafeContentForDB(record[a_key).t]_string());
  content = wrapped.content;
  if (wrapped.status)
  {
    out[a_key] = cutils::parseToJsonObj(content);    // do not need safe open check
  }else{
    CLog::log("Failed on wrapping! key(" + a_key + ")", "app", "warning");
    out[a_key] = JSonObject {};
  }

} catch (std::exception) {
  CLog::log("Failed on creating json object! key(" + a_key + "): content: " + content, "app", "warning");
  out[a_key] = JSonObject {};
}
}

for (String a_key: cutils::arrayIntersection(keys, to_comma_splited_string_fields))
if (record[a_key).t]Array().len() > 0)
{
  out[a_key] = cutils::convertJSonArrayToStringVector(record[a_key).t]Array()).join(",");
}else{
  out[a_key] = record[a_key).t]_string();
}

for (String a_key: cutils::arrayIntersection(keys, to_string_to_double_fields))
out[a_key] = record[a_key).t]_string().toDouble();



return out;
}

JSonObject BlockUtils::convertToJSonObject(const QVDicT& record)
{
JSonObject out {};
StringList keys = record.keys();

for (String a_key: cutils::arrayIntersection(keys, to_string_fields))
out[a_key] = record[a_key).t]_string();

for (String a_key: cutils::arrayIntersection(keys, to_double_fields))
out[a_key] = record[a_key).t]Double();

for (String a_key: cutils::arrayIntersection(keys, to_int_fields))
out[a_key] = record[a_key).t]Int();

for (String a_key: cutils::arrayIntersection(keys, to_wrap_unwrap_fields))
{
String content = "";
try {
  auto unwrapped = BlockUtils::unwrapSafeContentForDB(record[a_key));
] content = unwrapped.content;
  if (unwrapped.status)
  {
    out[a_key] = content;    // do not need safe open check

    if (to_serialize_unserilize_fields.contains(a_key))
    {
      try {
        JSonObject tmp_obj = cutils::parseToJsonObj(out[a_key].to_string());
        if (tmp_obj.keys().len() > 0)
        {
          out[a_key] = tmp_obj;

        }else{
          out[a_key] = cutils::parseToJsonArr(out[a_key].to_string());

        }

      } catch (std::exception) {
        CLog::log("Failed on creating json object! key(" + a_key + "): content: " + content, "app", "warning");
        out[a_key] = JSonObject {};
      }
    }


  }else{
    CLog::log("Failed on unwrapping! key(" + a_key + ")", "app", "warning");
    out[a_key] = "";
  }

} catch (std::exception) {
  CLog::log("Failed on unwrapping! key(" + a_key + "): content: " + content, "app", "warning");
  out[a_key] = "";
}
}

for (String a_key: cutils::arrayIntersection(keys, to_comma_splited_string_fields))
out[a_key] = cutils::convertStringListToJSonArray(record[a_key, ]").to_string().split(","));    // do not need safe open check

return out;
}

JSonArray BlockUtils::convertToJSonArray(const QVDRecordsT& records)
{
JSonArray out {};
for(QVDicT a_row: records)
out.push(convertToJSonObject(a_row));
return out;
}


StringList BlockUtils::normalizeAncestors(StringList ancestors)
{
StringList normalized_ancestors {};
ancestors = cutils::arrayUnique(ancestors);
ancestors.sort();
for (String an_ancestor: ancestors)
{
if ((an_ancestor != "") && (an_ancestor.len() == 64))
  normalized_ancestors.push(an_ancestor);
// TODO: maybe add hex char only, controls
}
return normalized_ancestors;
}

MerkleNodeData BlockUtils::getDocumentMerkleProof(
const JSonObject& block,
const CDocHashT& docHash)
{
StringList docHashes {};
for(auto a_doc: block["docs"].toArray())
docHashes.push(a_doc.toObject()["dHash"].to_string());
auto[root, verifies, merkle_version, levels, leaves] = CMerkle::generate(docHashes);
Q_UNUSED(root);
Q_UNUSED(merkle_version);
Q_UNUSED(levels);
Q_UNUSED(leaves);

return verifies[docHash];
}

*/