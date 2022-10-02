use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::{Document, trx_has_input, trx_has_not_input};
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CDocHashT, CDocIndexT, CInputIndexT, COutputIndexT, JSonObject, QVDicT, QVDRecordsT, VString};
use crate::lib::dag::dag::{get_block_hashes_by_doc_hashes, search_in_dag};
use crate::lib::database::abs_psql::{ModelClause};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::validate_sig_struct;


/*

//let resObj1 = {
//    calledAncestors: [],
//    ref: null,
//    err: false,
//    msg: ''
//}

 */

#[allow(unused, dead_code)]
pub fn setter1(res_obj: &mut QVDicT, values: &QVDicT)
{
    for key in values.keys()
    {
        res_obj.insert(key.clone(), values[key].clone());
    }

    // if (_.has(args, 'ref'))
    //     resObj1.ref = args.ref;

    // if (_.has(args, 'err'))
    //     resObj1.err = args.err;

    // if (_.has(args, 'msg'))
    //     resObj1.msg = args.msg;
}

#[allow(unused, dead_code)]
pub fn getter1(res_obj: &QVDicT, k: &String) -> String
{
    if k == ""
    {
        return "".to_string();
    }

// clog.trx.info(`SuperValidate, getter1111 called for key: ${k}`);
    if res_obj.contains_key(k)
    {
        return res_obj[k].clone();
    }

    return "".to_string();
}


//old_name_was trackingBackTheCoins
pub fn tracking_back_the_coins(
    block: &Block,
    interested_docs: &VString,
    interested_coins: &VString) -> (bool /* status */, String /* msg */, QVDRecordsT /* coin_track */)
{
    let level: u16 = 0;
    let mut coin_track: QVDRecordsT = vec![];
    if constants::LOG_SUPER_VALIDATE_DEBUG
    {
        dlog(
            &format!("{}.SCUUCM, Block {}", level, block.get_block_identifier()),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        dlog(
            &format!("{}.SCUUCM, Block: {:?}", level, block),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }

    let mut interested_docs = interested_docs.clone();
    let mut interested_coins = interested_coins.clone();
    let (validate_status, validate_msg) = tracking_back_the_coins_recursive(
        &mut coin_track,
        &block.export_block_to_json(constants::SUPER_CONTROL_SHOULD_CONTROL_SIGNATURES_AS_WELL),
        &mut interested_docs,
        &mut interested_coins,
        "Generated".to_string(),
        level);

    if constants::LOG_SUPER_VALIDATE_DEBUG
    {
        dlog(
            &format!("{}.SCUUCM, coin Track: {:?}", level, coin_track),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }
    return (validate_status, validate_msg, coin_track);
}


//old_name_was trackingBackTheCoinsRecursive
pub fn tracking_back_the_coins_recursive(
    coin_track: &mut QVDRecordsT,
    block: &JSonObject,
    interested_docs: &mut VString,
    interested_coins: &mut VString,
    descendant: String,
    mut level: u16) -> (bool, String)
{
    let mut msg: String = "".to_string();
    level += 1;

    if (level == 1) && (interested_docs.len() == 0)
    {
        // log all docs as interested
        for a_doc in block["bDocs"].as_array().unwrap()
        {
            interested_docs.push(remove_quotes(&a_doc["dHash"]));
        }
        dlog(
            &format!(
                "Start to trace back the coins are spent in these docs {:?}",
                interested_docs),
            constants::Modules::Trx,
            constants::SecLevel::Info);
    }

    if (level == 1) && (interested_coins.len() == 0)
    {
        // log all coins as interested
        for a_doc_hash in interested_docs.clone()
        {
            let (_inx, the_doc) = Block::get_json_document_by_hash(block, &a_doc_hash);
            if !the_doc["inputs"].is_null()
            {
                for an_inp in the_doc["inputs"].as_array().unwrap()
                {
                    interested_coins.push(cutils::pack_coin_code(
                        &remove_quotes(&an_inp[0]),
                        remove_quotes(&an_inp[1]).parse::<COutputIndexT>().unwrap()));
                }
            }
        }
    }

    if level == 1
    {
        dlog(
            &format!(
                "Start to trace back the coins {:?} from {}",
                interested_coins,
                Block::get_js_block_identifier(block)),
            constants::Modules::Trx,
            constants::SecLevel::Info);
    }

    let trace_back: QVDicT = HashMap::from([
        ("level".to_string(), level.to_string()),
        ("bType".to_string(), remove_quotes(&block["bType"])),
        ("creationDate".to_string(), remove_quotes(&block["bCDate"])),
        ("bHash".to_string(), remove_quotes(&block["bHash"])),
        ("descendant".to_string(), descendant),
        ("interested_docs".to_string(), interested_docs.join(",")),
        ("interested_coins".to_string(), interested_coins.join(","))]);

    coin_track.push(trace_back);
    let documents = block["bDocs"].as_array().unwrap();
    let mut doc_inx: CDocIndexT = 0;
    while doc_inx < documents.len() as CDocIndexT
    {
        let js_doc = &documents[doc_inx as usize];
        let doc_hash = remove_quotes(&js_doc["dHash"]);
        let doc_type = remove_quotes(&js_doc["dType"]);

        if constants::LOG_SUPER_VALIDATE_DEBUG
        {
            dlog(
                &format!(
                    "{}.SCUUCM, A {}{} doc_inx: {}",
                    level,
                    Block::get_js_block_identifier(block),
                    Document::get_js_doc_identifier(js_doc),
                    doc_inx),
                constants::Modules::Trx,
                constants::SecLevel::Info);
        }
        // if it is not first level, so it MUST have certain interested doc_hash to trace back
        if (level != 1) && !interested_docs.contains(&doc_hash)
        { continue; }

        // * in first level we can ask for control of all/some docs in block
        // * if is insisted to control certain docs and current doc is not one of mentioned docs,
        // * so continue the loop
        if (level == 1) &&
            (interested_docs.len() > 0) &&
            (!interested_docs.contains(&doc_hash))
        { continue; }


        if constants::LOG_SUPER_VALIDATE_DEBUG
        {
            dlog(
                &format!(
                    "{}.SCUUCM, B control {}{}",
                    level,
                    Block::get_js_block_identifier(block),
                    Document::get_js_doc_identifier(js_doc)),
                constants::Modules::App,
                constants::SecLevel::TmpDebug);
        }

        if !trx_has_input(&doc_type)
        {
            // * transaction has not input, so it must be the very transaction in which the fresh output was created.
            // * either because of being Coinbase Trx or being the DPCostPay Trx
            if trx_has_not_input(&doc_type)
            {
                if interested_docs.contains(&doc_hash)
                    || ((level == 1) && (interested_docs.len() == 0))
                {
                    // we found one interested fresh coin creating location
                    let the_dict: QVDicT = HashMap::from([
                        ("level".to_string(), level.to_string()),
                        ("bType".to_string(), remove_quotes(&block["bType"])),
                        ("creationDate".to_string(), remove_quotes(&block["bCDate"])),
                        ("bHash".to_string(), remove_quotes(&block["bHash"])),
                        ("dType".to_string(), doc_type.clone()),
                        ("descendant".to_string(), doc_hash.clone()),
                        ("interested_docs".to_string(), vec![doc_hash.clone()].join(",")),
                        ("interested_coins".to_string(), vec!["fresh coin"].join(","))]);
                    coin_track.push(the_dict);
                }
                continue;
            } else {
                msg = format!(
                    "{}.SCUUCM, must be Coinbase or DPCostPay and it is {}{}",
                    level,
                    Block::get_js_block_identifier(block),
                    Document::get_js_doc_identifier(js_doc));
                dlog(
                    &msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);

                msg = format!(
                    "{}.SCUUCM, The trx must be Coinbase or DPCostPay and it is {}{}",
                    level,
                    Block::get_js_block_identifier(block),
                    Document::get_js_doc_identifier(js_doc));
                dlog(
                    &msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return (false, msg);
            }
        }

        let mut doc_ext_infos: Vec<DocExtInfo> = vec![];
        if constants::SUPER_CONTROL_SHOULD_CONTROL_SIGNATURES_AS_WELL
        {
            if !block["bExtInfo"].is_null()
            {
                let b_ext_info = block["bExtInfo"].as_array().unwrap();
                for a_doc_ext in b_ext_info[doc_inx as usize].as_array().unwrap()
                {
                    let (_status, tmp) = DocExtInfo::load_from_json(a_doc_ext);
                    doc_ext_infos.push(tmp);
                }
            }
        }

        let inputs = js_doc["inputs"].as_array().unwrap();
        if constants::LOG_SUPER_VALIDATE_DEBUG
        {
            dlog(
                &format!(
                    "{}.SCUUCM, 1, the {}{} has some {} inputs to trace back. inputs: {:?}",
                    level,
                    Block::get_js_block_identifier(block),
                    Document::get_js_doc_identifier(js_doc),
                    inputs.len(),
                    inputs),
                constants::Modules::Trx,
                constants::SecLevel::Info);
        }

        let mut input_index: CInputIndexT = 0;
        while input_index < inputs.len() as CInputIndexT
        {
            let input = &inputs[input_index as usize];

            let coin_creator_doc_hash: CDocHashT = remove_quotes(&input[0]);
            let coin_creator_doc_output_index: COutputIndexT = remove_quotes(&input[1]).parse::<COutputIndexT>().unwrap();

            // mapping input ref to proper container block
            let (ref_block_hashes, _map_doc_to_block) = get_block_hashes_by_doc_hashes(
                &vec![coin_creator_doc_hash.clone()],
                &constants::COMPLETE.to_string());
            if ref_block_hashes.len() == 0
            {
                msg = format!(
                    "{}.SCUUCM, the coin-code: {} does not mapped to it's container-block!",
                    level,
                    cutils::pack_coin_code(&coin_creator_doc_hash, coin_creator_doc_output_index));
                dlog(
                    &msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);
                return (false, msg);
            }

            if constants::LOG_SUPER_VALIDATE_DEBUG
            {
                dlog(
                    &format!(
                        "{}.SCUUCM, coin-creator-doc-hash: {}, ref-block-hashes: {:?} {}{} ",
                        level,
                        cutils::hash8c(&coin_creator_doc_hash),
                        cutils::short_strings_list(&ref_block_hashes, 8),
                        Block::get_js_block_identifier(block),
                        Document::get_js_doc_identifier(js_doc)),
                    constants::Modules::App,
                    constants::SecLevel::Info);
            }

            // another paranoic test
            let control_coin_by_ancestors_links: bool = false;
            if control_coin_by_ancestors_links
            {
                // go back to history by ancestors link to find reference location(which block creates this ref as output)
                // for coin_creator_doc_hash
                let res_obj: QVDicT = HashMap::new();
                // look_for_doc_by_ancestor(
                //     block,
                //     &coin_creator_doc_hash, //doc_hash
                //     &setter1,
                //     &getter1,
                //     &mut res_obj);

                dlog(
                    &format!(
                        "{}.SCUUCM, getter1 final res ssssssssssssssss JSON.stringify(getter1()) ",
                        level),
                    constants::Modules::App,
                    constants::SecLevel::Info);

                if getter1(&res_obj, &"err".to_string()) == constants::YES.to_string()
                {
                    msg = getter1(&res_obj, &"msg".to_string());
                    dlog(
                        &msg,
                        constants::Modules::Trx,
                        constants::SecLevel::Error);
                    return (false, msg);
                }

                if !ref_block_hashes.contains(&getter1(&res_obj, &"ref".to_string()))
                {
                    msg = format!(
                        "{}.SCUUCM, the doc_hash({}) which is used in doc {} in block({}) is referring to 2 diff blocks by Ancestors({}) & i_docs_blocks_map({:?})",
                        level,
                        cutils::hash8c(&coin_creator_doc_hash),
                        Document::get_js_doc_identifier(&js_doc),
                        cutils::hash8c(&remove_quotes(&block["bHash"])),
                        getter1(&res_obj, &"ref".to_string()),
                        cutils::short_strings_list(&ref_block_hashes, 8)
                    );
                    dlog(
                        &msg,
                        constants::Modules::Sec,
                        constants::SecLevel::Error);
                    return (false, msg);
                }
            }

            // retrieve container block info
            let empty_string = "".to_string();
            let mut c1 = ModelClause {
                m_field_name: "b_hash",
                m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
                m_clause_operand: "IN",
                m_field_multi_values: vec![],
            };
            for a_hash in &ref_block_hashes {
                c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
            }
            let block_records = search_in_dag(
                vec![c1],
                vec!["b_body"],
                vec![],
                0,
                false);
            if block_records.len() != cutils::array_unique(&ref_block_hashes).len()
            {
                msg = format!("{}.SCUUCM, some of the blocks({:?}) do not exist in DAG ({})",
                              level,
                              cutils::short_strings_list(&ref_block_hashes, 8),
                              block_records.len()
                );
                dlog(
                    &msg,
                    constants::Modules::Sec,
                    constants::SecLevel::Error);

                return (false, msg);
            }
            for a_block_record in block_records
            {
                let (_status, _sf_ver, content) = unwrap_safed_content_for_db(&a_block_record["b_body"]);
                let (_status, ref_block) = cutils::controlled_str_to_json(&content);
                if
                (block["bType"] != constants::block_types::REPAYMENT_BLOCK)
                    && (application().time_diff(
                    remove_quotes(&ref_block["bCDate"]),
                    remove_quotes(&block["bCDate"])).as_minutes < application().get_cycle_by_minutes())
                {
                    msg = format!(
                        "{}.SCUUCM, block({} {}) uses an output of block({}) {}) before being maturated",
                        level,
                        cutils::hash8c(&remove_quotes(&block["bHash"])),
                        block["bCDate"].to_string(),
                        cutils::hash8c(&remove_quotes(&ref_block["bCDate"])),
                        ref_block["bCDate"].to_string()
                    );
                    dlog(
                        &msg,
                        constants::Modules::Sec,
                        constants::SecLevel::Error);
                    return (false, msg);
                }

                // looking for refBlock.output = block.input
                #[allow(unused_assignments)]
                let mut input_exist_in_referred_block = false;
                for a_ref_doc in ref_block["bDocs"].as_array().unwrap()
                {
                    if a_ref_doc["dHash"].to_string() == coin_creator_doc_hash
                    {
                        input_exist_in_referred_block |= true;

                        if constants::SUPER_CONTROL_SHOULD_CONTROL_SIGNATURES_AS_WELL
                        {
                            if !trx_has_not_input(&remove_quotes(&a_ref_doc["dType"]))
                            {
                                // also output address of ref controls!
                                let is_valid_unlock = validate_sig_struct(
                                    &doc_ext_infos[input_index as usize].m_unlock_set,
                                    &a_ref_doc["outputs"][coin_creator_doc_output_index as usize][0].to_string(),
                                    &HashMap::new());
                                if is_valid_unlock != true
                                {
                                    msg = format!(
                                        "{}.SCUUCM, Invalid given unlock structure for trx: {} input: {}",
                                        level,
                                        coin_creator_doc_hash,
                                        input_index);
                                    dlog(
                                        &msg,
                                        constants::Modules::Sec,
                                        constants::SecLevel::Error);
                                    return (false, msg);
                                }
                            }
                        }
                        let mature_cycles_count = application().get_mature_cycles_count(&remove_quotes(&a_ref_doc["dType"]));
                        if mature_cycles_count > 1
                        {
                            // control maturity
                            let cycle_by_min = application().get_cycle_by_minutes();
                            if ![constants::block_types::REPAYMENT_BLOCK.to_string()].contains(&remove_quotes(&block["bType"]))
                                &&
                                (application().time_diff(
                                    remove_quotes(&ref_block["bCDate"]),
                                    remove_quotes(&block["bCDate"])).as_minutes
                                    < mature_cycles_count as u64 * cycle_by_min)
                            {
                                msg = format!(
                                    "{}.SCUUCM, block({} {}) uses an output({} {}) of block({} {} \
                                    before being maturated by {} cycles ",
                                    level,
                                    cutils::hash8c(&remove_quotes(&block["bHash"])),
                                    block["bCDate"].to_string(),
                                    a_ref_doc["dType"].to_string(),
                                    cutils::hash8c(&remove_quotes(&a_ref_doc["dHash"])),
                                    Block::get_js_block_identifier(&ref_block),
                                    ref_block["bCDate"].to_string(),
                                    mature_cycles_count
                                );
                                dlog(
                                    &msg,
                                    constants::Modules::Sec,
                                    constants::SecLevel::Error);
                                return (false, msg);
                            }
                        }

                        // controlling the ref of ref
                        return tracking_back_the_coins_recursive(
                            coin_track,
                            &ref_block,
                            &mut vec![a_ref_doc["dHash"].to_string()],
                            &mut vec![],
                            js_doc["dHash"].to_string(),
                            level);
                    }
                }

                if !input_exist_in_referred_block
                {
                    msg = format!(
                        "{}.SCUUCM, the input({}) which is used in block({}) do not exist in block({})",
                        level,
                        cutils::hash8c(&coin_creator_doc_hash),
                        cutils::hash8c(&remove_quotes(&block["bHash"])),
                        cutils::hash8c(&remove_quotes(&ref_block["bHash"])));
                    dlog(
                        &msg,
                        constants::Modules::Sec,
                        constants::SecLevel::Error);
                    return (false, msg);
                }
            }
            input_index += 1;
        }
        doc_inx += 1;
    }
    return (true, msg);
}

/*

//String doSerial_blocks(const QVDicT& item)
//{
// String out =
//   item["b_hash"].to_string() +
//   item["b_creation_date"].to_string() +
//   item["b_ancestors"].to_string() +
//   item["b_type"].to_string() +
//   item["b_utxo_imported"].to_string();
// return out;
//}


*/
