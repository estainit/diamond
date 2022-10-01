use serde_json::json;
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::{Block, TransientBlockInfo};
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CMPAISValueT, CMPAIValueT, VString, VVString};
use crate::lib::database::abs_psql::{OrderModifier, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_BLOCK_BUFFER_FIELDS};
use crate::lib::machine::machine_buffer::block_buffer::search_buffered_docs;
use crate::lib::services::society_rules::society_rules::get_block_fix_cost;

// old name was fetchBufferedTransactions
pub fn fetch_buffered_transactions(
    block: &mut Block,
    transient_block_info: &mut TransientBlockInfo)
    -> (bool /* creating block status */, bool /* should empty buffer */, String /* msg */)
{
    let mut msg: String;

    let buffered_txs = search_buffered_docs(
        vec![
            simple_eq_clause("bd_doc_type", &constants::document_types::BASIC_TX.to_string()),
        ],
        Vec::from(C_MACHINE_BLOCK_BUFFER_FIELDS),
        vec![
            &OrderModifier { m_field: "bd_dp_cost", m_order: "DESC" },
            &OrderModifier { m_field: "bd_doc_class", m_order: "ASC" },
            &OrderModifier { m_field: "bd_insert_date", m_order: "ASC" },
        ],
        0);

    // TODO: currently naively the query select most payer transaction first.
    // the algorithm must be enhanced. specially to deal with block size, and being sure
    // if prerequisites doc(e.g payer transaction and referenced document & sometimes documents) are all placed in same block!

    dlog(
        &format!("The NORMAL block will contain {} transactions", buffered_txs.len()),
        constants::Modules::App,
        constants::SecLevel::Info);

    if buffered_txs.len() == 0
    {
        return (
            false,
            false,
            "There is no transaction to append to block!".to_string());
    }

    let mut supported_p4p: VString = vec![];// extracting P4P (if exist)
    for serialized_trx in buffered_txs
    {
        let (_status, js_doc) = cutils::controlled_str_to_json(&serialized_trx["bd_payload"].to_string());
        let (_status, doc) = Document::load_document(&js_doc, &Block::new(), -1);
        let now_ = application().now();
        let (status, tmp_block) = Block::load_block(&json!({
              "bCDate": now_,
              "bType": constants::block_types::NORMAL,
              "bHash": constants::HASH_ZEROS_PLACEHOLDER}));
        if !status
        {
            msg = format!("Error in loading tmp block!");
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        let (status, msg_) = doc.custom_validate_doc(&tmp_block);
        if !status
        {
            msg = format!(
                "Error in validate Doc. {} block({})!, {}",
                doc.get_doc_identifier(), cutils::hash8c(&block.get_block_hash()), msg_);
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, false, msg);
        }

        if doc.m_doc_class == constants::trx_classes::P4P
        {
            supported_p4p.push(doc.m_doc_ref.clone());
        }

        for an_output in doc.get_outputs()
        {
            transient_block_info.m_block_total_output += an_output.m_amount as CMPAISValueT;
        }

        block.m_block_documents.push(doc);
    }

    let mut block_total_dp_cost: CMPAIValueT = 0;
    for doc in &block.m_block_documents
    {
        // collect backer fees
        let mut dp_cost: CMPAISValueT = 0;
        if supported_p4p.contains(&doc.get_doc_hash())
        {
            msg = format!(
                "Block {} trx {} is supported by p4p trx, so this trx must not pay trx-fee",
                block.get_block_identifier(),
                doc.get_doc_identifier());
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Info);
        } else {
            // find the backer output
            // check if trx is clone, in which client pays for more than one backer in a transaction
            // in order to ensure more backers put trx in DAG
            for &a_dp_index in doc.get_dpis()
            {
                if doc.get_outputs()[a_dp_index as usize].m_address == block.m_block_backer
                {
                    dp_cost = doc.get_outputs()[a_dp_index as usize].m_amount as CMPAISValueT;
                }
            }

            if dp_cost == 0
            {
                msg = format!(
                    "Can not create block, because at least one trx hasn't backer fee! transaction {} in Block {}",
                    doc.get_doc_identifier(),
                    block.get_block_identifier());
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, false, msg);
            }
        }

        block_total_dp_cost += dp_cost as CMPAIValueT;
        transient_block_info.m_block_documents_hashes.push(doc.get_doc_hash());
        transient_block_info.m_block_ext_info_hashes.push(doc.get_doc_ext_hash());
        //block.m_block_ext_info.push(trx["dExtInfo"]);
    }

    // create treasury payment
    let block_fix_cost: CMPAIValueT = get_block_fix_cost(&block.m_block_creation_date);
    let backer_fee: CMPAIValueT =
        cutils::c_floor((block_total_dp_cost as f64 * constants::BACKER_PERCENT_OF_BLOCK_FEE) / 100.0) as CMPAIValueT;

    if backer_fee < block_fix_cost
    {
        msg = format!(
            "The block can not cover broadcasting costs! \nblock Total DPCost({}) \n backer Net Fee({})",
            cutils::nano_pai_to_pai(block_total_dp_cost as CMPAISValueT),
            cutils::nano_pai_to_pai((backer_fee - block_fix_cost) as CMPAISValueT));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, false, msg);
    }
    let backer_net_fee: CMPAIValueT = backer_fee - block_fix_cost;

    let treasury: CMPAISValueT = (block_total_dp_cost - backer_net_fee) as CMPAISValueT;
    let output_tuples: VVString = vec![
        vec!["TP_DP".to_string(), treasury.to_string()],
        vec![machine().get_backer_address(), backer_net_fee.to_string()],
    ];

    let tmp_js = json!({
                "dType": constants::document_types::DATA_AND_PROCESS_COST_PAYMENT,
                "dCDate": block.m_block_creation_date,
                "outputs": output_tuples});
    let (status, mut dp_cost_trx) = Document::load_document(
        &tmp_js,
        block,
        -1);
    if !status
    {
        msg = format!("Failed in load-doc of DPCost {}", tmp_js);
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, false, msg);
    }

    dp_cost_trx.set_doc_hash();

    transient_block_info.m_block_documents_hashes.push(dp_cost_trx.get_doc_hash());

    let mut tmp_documents: Vec<Document> = vec![];
    tmp_documents.push(dp_cost_trx);
    for a_doc in &block.m_block_documents
    {
        tmp_documents.push(a_doc.clone());
    }
    block.m_block_documents = tmp_documents;

    // block.m_block_ext_info.push(QJsonArray{});   // althougt it is empty but must be exits, in order to having right index in block ext Infos array
    // transient_block_info.m_block_ext_info_hashes.push("-");   // althougt it is empty but must be exits, in order to having right index in block ext Infos array

    return (
        true,
        true,
        "Successfully appended transactions to block".to_string()
    );
}

