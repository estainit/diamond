use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CDocHashT, JSonObject, QVDicT, VString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::simple_eq_clause;

//old_name_was lookForDocByAnc
#[allow(dead_code, unused)]
pub fn look_for_doc_by_ancestor(
    block: &JSonObject,
    doc_hash: &CDocHashT,
    v_setter: &dyn Fn(&QVDicT, &QVDicT),
    v_getter: &dyn Fn(&QVDicT, &String) -> String,
    res_obj: &mut QVDicT)
{
    let tmp: QVDicT = HashMap::from([
        ("ref".to_string(), "".to_string()),
        ("calledAncestors".to_string(), "".to_string()),
        ("err".to_string(), "".to_string()),
        ("msg".to_string(), "".to_string())
    ]);
    v_setter(res_obj, &tmp);

    look_for_doc_by_anc_recursive(
        block,
        doc_hash,
        v_setter,
        v_getter,
        res_obj,
        0);
    return;
}

//old_name_was lookForDocByAncRecursive
#[allow(dead_code, unused)]
pub fn look_for_doc_by_anc_recursive(
    block: &JSonObject,
    doc_hash: &CDocHashT,
    v_setter: &dyn Fn(&QVDicT, &QVDicT),
    v_getter: &dyn Fn(&QVDicT, &String) -> String,
    res_obj: &QVDicT,
    mut level_anc: u16)
{
    let tmp_v = v_getter(res_obj, &"ref".to_string()).to_string();
    if tmp_v != "".to_string()
    {
        dlog(
            &format!(
                "{}.{} SuperValidate, founded ref={}",
                level_anc,
                level_anc,
                tmp_v),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }

    level_anc += 1;
    let msg: String = "".to_string();
    if constants::LOG_SUPER_VALIDATE_DEBUG
    {
        dlog(
            &format!(
                "{}.{} SuperValidate, look For Doc By Anc Recursive: looking for doc_hash({}) in block({})",
                level_anc,
                level_anc,
                cutils::hash8c(doc_hash),
                cutils::hash8c(&remove_quotes(&block["bHash"]))
            ),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }

    // check if invoked doc exist in this block
    if ![constants::block_types::FLOATING_SIGNATURE.to_string(), constants::block_types::FLOATING_VOTE.to_string()]
        .contains(&remove_quotes(&block["bType"]))
    {
        if Block::js_get_documents_hashes(&block).contains(&doc_hash)
        {
            dlog(
                &format!("{}.{} SuperValidate, OK. Found ref for doc_hash({}) in block {}",
                         level_anc,
                         level_anc,
                         cutils::hash8c(&doc_hash),
                         Block::get_js_block_identifier(block)),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);
            let tmp_v: QVDicT = HashMap::from([
                ("ref".to_string(), remove_quotes(&block["bHash"])),
                ("err".to_string(), "".to_string())]);
            v_setter(res_obj, &tmp_v);
            return;
        }
    }

    let ancestors: VString =
        cutils::convert_comma_separated_string_to_string_vector(&remove_quotes(&block["bAncestors"]));
    if constants::LOG_SUPER_VALIDATE_DEBUG
    {
        dlog(
            &format!("{}.{} SuperValidate, looping on ancestors({:?})",
                     level_anc,
                     level_anc,
                     ancestors
            ),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }
    if ancestors.len() == 0
    {
        dlog(
            &format!("{}.{} Super-Validate, .SCUUCM, block {} has no ancestors and still can not find the doc({})",
                     level_anc,
                     level_anc,
                     Block::get_js_block_identifier(block),
                     doc_hash
            ),
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return;
    }

    for ancestor_hash in ancestors
    {
        let ancestor_w_blocks = search_in_dag(
            vec![simple_eq_clause("b_hash", &ancestor_hash)],
            vec!["b_body"],
            vec![],
            0,
            false,
        );
        if ancestor_w_blocks.len() != 1
        {
            dlog(
                &format!("{}.{} SuperValidate, .SCUUCM, block {} has an ancestor({}) which doesn't exist in DAG ({})",
                         level_anc,
                         level_anc,
                         Block::get_js_block_identifier(block),
                         cutils::hash8c(&ancestor_hash),
                         ancestor_w_blocks.len()
                ),
                constants::Modules::Sec,
                constants::SecLevel::Error);

            let tmp_v: QVDicT = HashMap::from([
                ("err".to_string(), constants::YES.to_string()),
                ("msg".to_string(), msg)]);
            v_setter(res_obj, &tmp_v);
            return;
        }

        let ancestor_w_block: &QVDicT = &ancestor_w_blocks[0];
        let (_status, _sf_ver, content) = unwrap_safed_content_for_db(&ancestor_w_block["b_body"]);
        let (_status, j_block) = cutils::controlled_str_to_json(&content);
        // let ancBlock = blockUtils.openDBSafeObject(ancWBlock["b_body"].to_string()).content;
        if constants::LOG_SUPER_VALIDATE_DEBUG
        {
            dlog(
                &format!("{}.{} SuperValidate, refBlockHashFoundByAncestors ============({})",
                         level_anc,
                         level_anc,
                         v_getter(res_obj, &"ref".to_string()).to_string()
                ),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);
        }
        if (v_getter(res_obj, &"ref".to_string()).to_string() == "")
            && !cutils::convert_comma_separated_string_to_string_vector(&v_getter(res_obj, &"calledAncestors".to_string())).contains(&remove_quotes(&j_block["bHash"]))
        {
            let mut tmp: VString = cutils::convert_comma_separated_string_to_string_vector(&v_getter(res_obj, &"calledAncestors".to_string()));
            tmp.push(remove_quotes(&j_block["bHash"]));
            let tmp_v: QVDicT = HashMap::from([("calledAncestors".to_string(), tmp.join(","))]);
            v_setter(res_obj, &tmp_v);
            look_for_doc_by_anc_recursive(
                &j_block,
                doc_hash,
                v_setter,
                v_getter,
                res_obj,
                level_anc);
        }
    }
}