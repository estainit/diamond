use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::json;
use crate::{application, ccrypto, CMachine, constants, cutils, dlog, get_value, machine};
use crate::lib::block::block_types::block::{Block, TransientBlockInfo};
use crate::lib::block::block_types::block_normal::normal_block_handler::create_a_normal_block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{BlockLenT, CDateT, ClausesT, CMPAISValueT, CMPAIValueT, LimitT, OrderT, QVDRecordsT, VString, VVString};
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_delete, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_MACHINE_BLOCK_BUFFER, C_MACHINE_BLOCK_BUFFER_FIELDS};
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;
use crate::lib::services::society_rules::society_rules::get_block_fix_cost;
use crate::lib::utils::version_handler::is_valid_version_number;

impl CMachine {
    // js name was pushToBlockBuffer
    pub fn push_to_block_buffer(
        &self,
        doc: &Document,
        dp_cost: CMPAIValueT,
        mp_code: &String) -> (bool, String)
    {
        let msg: String;
        //listener.doCallAsync('APSH_before_push_doc_to_buffer_async', args);

        let dbl_chk = self.search_buffered_docs(
            vec![
                simple_eq_clause("bd_mp_code", mp_code),
                simple_eq_clause("bd_doc_hash", &doc.get_doc_hash()),
            ],
            vec!["bd_doc_hash"],
            vec![],
            0,
        );
        if dbl_chk.len() > 0
        {
            msg = format!("Tried to insert in buffer duplicated {}", doc.get_doc_identifier());
            dlog(
                &msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, msg);
        }

        let payload: String = doc.safe_stringify_doc(true);

        let doc_hash = doc.get_doc_hash();
        let doc_type = doc.get_doc_type();
        let doc_class = doc.get_doc_class();
        let dp_cost_i64 = dp_cost as i64;
        let payload_len_i32 = payload.len() as i32;
        let now_ = application().now();
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("bd_mp_code", &mp_code as &(dyn ToSql + Sync)),
            ("bd_insert_date", &now_ as &(dyn ToSql + Sync)),
            ("bd_doc_hash", &doc_hash as &(dyn ToSql + Sync)),
            ("bd_doc_type", &doc_type as &(dyn ToSql + Sync)),
            ("bd_doc_class", &doc_class as &(dyn ToSql + Sync)),
            ("bd_payload", &payload as &(dyn ToSql + Sync)),
            ("bd_dp_cost", &dp_cost_i64 as &(dyn ToSql + Sync)),
            ("bd_doc_len", &payload_len_i32 as &(dyn ToSql + Sync)),
        ]);
        let status: bool = q_insert(
            C_MACHINE_BLOCK_BUFFER,
            &values,
            true);
        dlog(
            &format!("Insert a document in block buffer, values: {:#?}", values),
            constants::Modules::App,
            constants::SecLevel::Info);
        if status
        {
            return (
                true,
                format!("The document have been pushed into buffer. {}", doc.get_doc_identifier())
            );
        }

        dlog(
            &format!("Failed in push doc to block buffer, values: {:#?}", values),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "Failed in push doc to block buffer".to_string());
    }

    //old_name_was searchBufferedDocs
    pub fn search_buffered_docs(
        &self,
        clauses: ClausesT,
        fields: Vec<&str>,
        order: OrderT,
        limit: LimitT) -> QVDRecordsT
    {
        let (_status, records) = q_select(
            C_MACHINE_BLOCK_BUFFER,
            fields,
            clauses,
            order,
            limit,
            false);
        return records;
    }

    // old name was broadcastBlock
    pub fn broadcast_block(
        &self,
        cost_pay_mode: &String,
        create_date_type: &String) -> (bool, String)
    {
        let msg: String = "".to_string();
        let mut block: Block = Block::new();
        let mut should_reset_block_buffer: bool = false;
        let mut cheating_creation_date: CDateT = "".to_string();
        let mut cheating_ancestors: VString = vec![];
        if create_date_type == "cheat"
        {
            cheating_creation_date = get_value("cheating_creation_date");
            // let tt = get_value("cheating_ancestors").split(",")
            cheating_ancestors = get_value("cheating_ancestors")
                .split(",")
                .collect::<Vec<&str>>()
                .iter()
                .map(|&x| x.to_string())
                .collect::<VString>()
        }


        if cost_pay_mode == "byPoW"
        {
            // TODO: implement it (if we really need POW payment block types)
            //res = await POWblockHandler.createAPOWBlock({
            //  creationDate: cheating_creation_date,
            //  ancestors: (!utils._nilEmptyFalse(cheating_ancestors)) ? utils.parse(cheating_ancestors) : null
            //});
        } else {
            let (status, block_, should_reset_block_buffer_, msg) = create_a_normal_block(
                &cheating_ancestors,
                &cheating_creation_date,
                cheating_ancestors.len() > 0);
            should_reset_block_buffer = should_reset_block_buffer_;
            if !status
            {
                dlog(
                    &format!("Failed in generating normal block! {}", msg),
                    constants::Modules::App,
                    constants::SecLevel::Error);
                return (false, format!("Failed in generating normal block! {}", msg));
            }
            block = block_;
        }


        // write file on hard output/send email
        let mut block_body = block.safe_stringify_block(true);
        dlog(
            &format!("About to sending a normal block to network block: {} {}", block.get_block_identifier(), block_body),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        // let push_res = push_into_sending_q(
        //   ,
        //   block.getBlockHash(),
        //   block_body,
        //   "Broadcasting the created normal block(" + cutils::hash8c(block.getBlockHash()) + ") " + cutils::getNow());
        block_body = ccrypto::b64_encode(&block_body);
        let (_code, body) = make_a_packet(
            vec![
                json!({
                "cdType": block.get_block_type(),
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "bHash": block.m_block_hash.clone(),
                // "ancestors": ancestors,
                "block": block_body,
            }),
            ],
            constants::DEFAULT_PACKET_TYPE,
            constants::DEFAULT_PACKET_VERSION,
            application().now(),
        );
        dlog(
            &format!(
                "prepared Normal block packet, before insert into DB {}: {}",
                block.get_block_identifier(),
                body),
            constants::Modules::App,
            constants::SecLevel::Info);

        let status = push_into_sending_q(
            &block.get_block_type(),
            &block.get_block_hash(),
            &body,
            &format!(
                "Broadcasting the created normal block {} {}",
                block.get_block_identifier(), application().now()
            ),
            &vec![],
            &vec![],
            false,
        );

        dlog(
            &format!(
                "Normal block generated & pushed to sending Q. push res({}) block {} {}",
                status,
                block.get_block_identifier(),
                application().now()
            ),
            constants::Modules::App,
            constants::SecLevel::Info);

        // remove from buffer
        if should_reset_block_buffer
        {
            let empty_string = "".to_string();
            let mut c1 = ModelClause {
                m_field_name: "bd_doc_hash",
                m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
                m_clause_operand: "IN",
                m_field_multi_values: vec![],
            };
            let hashes= block.get_documents_hashes();
            for a_hash in &hashes
            {
                c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
            }
            machine().remove_from_buffer(
                vec![
                    simple_eq_clause("bd_mp_code", &machine().get_selected_m_profile()),
                    c1,
                ]);
        }
        drop(&block);

        return (true, msg);
    }


    // old name was fetchBufferedTransactions
    pub fn fetch_buffered_transactions(
        &self,
        block: &mut Block,
        transient_block_info: &mut TransientBlockInfo) -> (bool /* creating block status */, bool /* should empty buffer */, String /* msg */)
    {
        let mut msg: String;

        let buffered_txs = self.search_buffered_docs(
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
      "bType": "futureBlockTrx",
      "bHash": "futureHashTrx"}));
            if !status
            {
                msg = format!("error in loading tmp block!");
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
            "Sucessfully appended transactions to block".to_string()
        );
    }

    // old name was retrieveAndGroupBufferedDocuments
    pub fn retrieve_and_group_buffered_documents(
        &self,
        block: &Block,
        transient_block_info: &mut TransientBlockInfo)
        -> (bool/* status */, bool/* should clear buffer */, String/* err_msg */)

    {
        let mut msg: String;

        let buffered_docs = self.search_buffered_docs(
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

    // old name was removeFromBuffer
    pub fn remove_from_buffer(&self, clauses: ClausesT) -> bool
    {
        q_delete(
            C_MACHINE_BLOCK_BUFFER,
            clauses,
            false);

        return true;
    }
}
