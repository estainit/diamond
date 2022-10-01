use crate::{application, constants, cutils, dlog};
use crate::lib::block::block_types::block::{Block, TransientBlockInfo};
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{BlockLenT, VString};
use crate::lib::database::abs_psql::OrderModifier;
use crate::lib::database::tables::C_MACHINE_BLOCK_BUFFER_FIELDS;
use crate::lib::machine::machine_buffer::block_buffer::search_buffered_docs;
use crate::lib::utils::version_handler::is_valid_version_number;

// old name was retrieveAndGroupBufferedDocuments
pub fn retrieve_and_group_buffered_documents(
    block: &Block,
    transient_block_info: &mut TransientBlockInfo)
    -> (bool/* status */, bool/* should clear buffer */, String/* err_msg */)

{
    let mut msg: String;

    let buffered_docs = search_buffered_docs(
        vec![],
        Vec::from(C_MACHINE_BLOCK_BUFFER_FIELDS),
        vec![
            &OrderModifier { m_field: "bd_dp_cost", m_order: "DESC" },
            &OrderModifier { m_field: "bd_doc_class", m_order: "ASC" },
            &OrderModifier { m_field: "bd_insert_date", m_order: "ASC" },
        ],
        0);
    if buffered_docs.len() == 0
    { return (true, true, "There is no doc to append!".to_string()); }

    for serialized_doc in buffered_docs
    {
        let roughly_block_size: BlockLenT = block.safe_stringify_block(true).len();
        // size control TODO: needs a little tuning
        if roughly_block_size > ((constants::MAX_BLOCK_LENGTH_BY_CHAR * 80) / 100)
        { continue; }

        // TODO: it is too important in a unique block exit both trx and it's referred Doc(if is referred to a doc),
        // in some case there are even4 document which must exist together in same block
        // add some control to be sure about it.
        // now it is not the case until reaching buffer  total size bigger than a single block(almost 10 Mega Byte)

        let (status, a_js_doc) = cutils::controlled_str_to_json(&serialized_doc["bd_payload"]);
        if !status
        {
            msg = format!("Failed in de-ser bd-payload! {}", cutils::hash16c(&serialized_doc["bd_doc_hash"]));
            dlog(
                &msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);

            return (false, false, msg);
        }
        let (status, a_document) = Document::load_document(&a_js_doc, block, -1);
        if !status
        {
            msg = format!("Failed in load de-sered doc bd-payload! {}", cutils::hash16c(&serialized_doc["bd_doc_hash"]));
            dlog(
                &format!("{} {}", msg, a_js_doc),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        if !is_valid_version_number(&a_document.m_doc_version)
        {
            msg = format!(
                "invalid dVer for in retrieve And Group Buffered Documents {}",
                a_document.get_doc_identifier()
            );
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        if !cutils::is_a_valid_date_format(&a_document.m_doc_creation_date)
        {
            msg = format!(
                "Invalid date format block-creationDate({}) {}!",
                block.m_block_creation_date, a_document.get_doc_identifier()
            );
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        if a_document.m_doc_creation_date > block.m_block_creation_date
        {
            msg = format!(
                "Creating new block, document creation-date({}) is after block-creation-date({})! {}",
                a_document.m_doc_creation_date,
                block.m_block_creation_date,
                a_document.get_doc_identifier()
            );
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        if a_document.m_doc_creation_date > application().now()
        {
            msg = format!(
                "Creating new block, documents is created in future({})!, {}",
                a_document.m_doc_creation_date,
                a_document.get_doc_identifier()
            );
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        //  // length control
        //  if (parseInt(a_document.dLen) != utils.stringify(a_document).length) {
        //     msg = `create: The doc(${a_document.dType} / ${utils.hash6c(a_document.m_doc_hash)}), stated dLen(${a_document.dLen}), III. is not same as real length(${utils.stringify(a_document).length})!`
        //     clog.sec.error(msg);
        //     return { err: true, msg }
        //  }

        if !transient_block_info.m_grouped_documents.contains_key(&a_document.m_doc_type)
        {
            transient_block_info.m_grouped_documents.insert(a_document.m_doc_type.clone(), vec![]);
        }
        let mut tmp: Vec<Document> = transient_block_info.m_grouped_documents[&a_document.m_doc_type].clone();
        tmp.push(a_document.clone());
        transient_block_info.m_grouped_documents.insert(a_document.m_doc_type.clone(), tmp);

        if a_document.m_doc_ref != "".to_string()
        {
            if Document::can_be_a_cost_payer_doc(&a_document.m_doc_type)
            {
                transient_block_info.m_transactions_dict.insert(a_document.m_doc_hash.clone(), a_document.clone());
                transient_block_info.m_map_trx_hash_to_trx_ref.insert(a_document.get_doc_hash(), a_document.get_doc_ref());
                transient_block_info.m_map_trx_ref_to_trx_hash.insert(a_document.get_doc_ref(), a_document.get_doc_hash());
            } else {
                transient_block_info.m_map_referencer_to_referenced.insert(a_document.get_doc_hash(), a_document.get_doc_ref());
                transient_block_info.m_map_referenced_to_referencer.insert(a_document.get_doc_ref(), a_document.get_doc_hash());
            }
        }
        transient_block_info.m_doc_by_hash.insert(a_document.get_doc_hash(), a_document.clone());
    }

    if transient_block_info.m_map_trx_ref_to_trx_hash.keys().len()
        != transient_block_info.m_map_trx_hash_to_trx_ref.keys().len()
    {
        msg = format!(
            "Creating new block, create: transaction count and ref count are different! map trx ref to map trx hash to trx ref: {:#?} {:#?}",
            transient_block_info.m_map_trx_ref_to_trx_hash,
            transient_block_info.m_map_trx_hash_to_trx_ref
        );
        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, false, msg);
    }

    for a_reference in transient_block_info.m_map_trx_ref_to_trx_hash.keys()
    {
        if !transient_block_info.m_transactions_dict.contains_key(&transient_block_info.m_map_trx_ref_to_trx_hash[a_reference])
        {
            msg = format!(
                "Creating new block, missed some3 transaction to support referenced documents. \
                transactions dict: {:#?} map trx ref to trx hash: {:#?}",
                transient_block_info.m_transactions_dict,
                transient_block_info.m_map_trx_ref_to_trx_hash
            );
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }
    }

    if cutils::array_diff(
        &transient_block_info.m_map_trx_hash_to_trx_ref.keys().cloned().collect::<VString>(),
        &transient_block_info.m_transactions_dict.keys().cloned().collect::<VString>()).len() != 0
    {
        msg = format!(
            "Creating new block, missed some2 transaction to support referenced documents. transactions dict: {:?} \
              map trx ref to trx hash: {:?} ",
            transient_block_info.m_transactions_dict,
            transient_block_info.m_map_trx_ref_to_trx_hash
        );
        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, false, msg);
    }

    for group_code in transient_block_info.m_grouped_documents.keys().cloned().collect::<VString>()
    {
        msg = format!(
            "Creating new block, extracted {} documents of type({})",
            transient_block_info.m_grouped_documents[&group_code].len(),
            group_code
        );
        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Info);
    }

    for a_reference in &transient_block_info.m_map_trx_ref_to_trx_hash.keys().cloned().collect::<VString>()
    {
        if !transient_block_info.m_doc_by_hash.contains_key(a_reference)
        {
            msg = format!(
                "Creating new block, missed referenced document, which is supported by trx.ref({})",
                cutils::hash8c(a_reference)
            );
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }
    }

    dlog(
        &format!("Transient block info: {:#?}", transient_block_info),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    return (true, true, "Successfully grouped".to_string());
}
