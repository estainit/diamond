use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::block::node_signals_handler::log_signals;
use crate::lib::block_utils::wrap_safe_content_for_db;
use crate::lib::custom_types::CDocIndexT;
use crate::lib::dag::dag::append_descendants;
use crate::lib::dag::leaves_handler::{add_to_leave_blocks, remove_from_leave_blocks};
use crate::lib::dag::sceptical_dag_integrity_control::controls_of_new_block_insertion;
use crate::lib::database::abs_psql::{OrderModifier, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::C_BLOCKS;
use crate::lib::file_handler::file_handler::file_write;

impl Block {
    //old_name_was addBlockToDAG
    pub fn add_block_to_dag(&self) -> (bool, String)
    {
        // duplicate check
        let (_status, records) = q_select(
            C_BLOCKS,
            vec!["b_hash"],     // fields
            vec![
                simple_eq_clause("b_hash", &self.m_block_hash),
            ],
            vec![
                &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
                &OrderModifier { m_field: "b_id", m_order: "ASC" },
            ],
            1,   // limit
            false,
        );
        if records.len() > 0
        { return (true, "Block already existed in DAG".to_string()); }

        // save hard copy of blocks(timestamped by receive date) to have backup
        // in case of curruptions in DAG or bootstrp the DAG, machine doesn't need to download again entire DAG
        // you can simply copy files from ~/backup-dag to folder ~/temporary/inbox
        let dag_backup = application().dag_backup();
        let file_name = application().get_now_sss() + "_" + &*self.m_block_type.clone() + "_" + &*self.m_block_hash.clone() + ".txt";
        let clone_id = application().id();
        if constants::DO_HARD_COPY_DAG_BACKUP {
            file_write(
                dag_backup,
                file_name,
                &self.safe_stringify_block(false),
                clone_id);
        }

        //TODO: implementing atomicity(transactional) either in APP or DB

        // insert into DB
        let confidence_string = cutils::convert_float_to_string(self.m_block_confidence, constants::FLOAT_LENGTH);
        let confidence_float = confidence_string.parse::<f64>().unwrap();
        let signals = serde_json::to_string(&self.m_block_signals).unwrap();
        let (_status, _sf_version, body) = wrap_safe_content_for_db(&self.safe_stringify_block(false), constants::WRAP_SAFE_CONTENT_VERSION);
        let docs_count = self.m_block_documents.len() as i32;
        let ancestors = self.m_block_ancestors.join(",");
        let ancestors_count = self.m_block_ancestors.len() as i32;
        let descendants = self.m_block_descendants.join(",");
        let cycle = application().get_coinbase_cycle_stamp(&self.m_block_creation_date);
        let b_trxs_count = 0;
        let b_receive_date = application().now();
        let b_confirm_date = application().now();
        let b_coins_imported = constants::NO.to_string();

        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("b_hash", &self.m_block_hash as &(dyn ToSql + Sync)),
            ("b_type", &self.m_block_type as &(dyn ToSql + Sync)),
            ("b_confidence", &confidence_float as &(dyn ToSql + Sync)),
            ("b_body", &body as &(dyn ToSql + Sync)),
            ("b_docs_root_hash", &self.m_block_documents_root_hash as &(dyn ToSql + Sync)),
            ("b_ext_root_hash", &self.m_block_ext_root_hash as &(dyn ToSql + Sync)),
            ("b_signals", &signals as &(dyn ToSql + Sync)),
            ("b_trxs_count", &b_trxs_count as &(dyn ToSql + Sync)),
            ("b_docs_count", &docs_count as &(dyn ToSql + Sync)),
            ("b_ancestors", &ancestors as &(dyn ToSql + Sync)),
            ("b_ancestors_count", &ancestors_count as &(dyn ToSql + Sync)),
            ("b_descendants", &descendants as &(dyn ToSql + Sync)),
            ("b_creation_date", &self.m_block_creation_date as &(dyn ToSql + Sync)),
            ("b_receive_date", &self.m_block_receive_date as &(dyn ToSql + Sync)),
            ("b_confirm_date", &self.m_block_confirm_date as &(dyn ToSql + Sync)),
            ("b_cycle", &cycle as &(dyn ToSql + Sync)),
            ("b_backer", &self.m_block_backer as &(dyn ToSql + Sync)),
            ("b_receive_date", &b_receive_date as &(dyn ToSql + Sync)),
            ("b_confirm_date", &b_confirm_date as &(dyn ToSql + Sync)),
            ("b_coins_imported", &b_coins_imported as &(dyn ToSql + Sync))]);

        dlog(
            &format!("--- recording block in DAG Block({})", cutils::hash8c(&self.m_block_hash)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        q_insert(
            C_BLOCKS,     // table
            &values, // values to insert
            true);

        // // add newly recorded block to cache in order to reduce DB load. TODO: improve it
        // update_cached_blocks(
        //     machine,
        //     &self.m_block_type,
        //     &self.m_block_hash,
        //     &self.m_block_creation_date,
        //     &constants::NO.to_string());

        // recording block ext Info (if exist)
        let block_ext_info: String = self.stringify_block_ext_info();
        if block_ext_info != "" {
            self.insert_block_ext_info_to_db(&block_ext_info, &self.m_block_hash, &self.m_block_creation_date);
        }

        // adjusting leave blocks
        remove_from_leave_blocks(&self.m_block_ancestors);
        add_to_leave_blocks(&self.m_block_hash, &self.m_block_creation_date, &self.m_block_type);

        // insert block signals
        log_signals(&self);

        if self.m_block_documents.len() > 0
        {
            for doc_inx in 0..self.m_block_documents.len()
            {
                //FIXME: implement suspicious docs filtering!

                let a_doc: &Document = &self.m_block_documents[doc_inx];
                a_doc.apply_doc_first_impact(self);

                // connect documents and blocks
                a_doc.map_doc_to_block(&self.m_block_hash, doc_inx as CDocIndexT);
            }
        }

        // update ancestor's descendent info
        append_descendants(&self.m_block_ancestors, &vec![self.m_block_hash.clone()]);

        // sceptical_dag_integrity_controls
        let (status, _msg) = controls_of_new_block_insertion(&self.m_block_hash);
        if !status
        {
            dlog(
                &format!("Error in sceptical Data Integrity Check: {}",
                         self.get_block_identifier()),
                constants::Modules::App,
                constants::SecLevel::Info);

            return (false, "Error in sceptical Data Integrity Check".to_string());
        }

        #[allow(unused_doc_comments)]
        /**
        {
            // TODO: remove this block(variable/mechanism) after fixing sqlite database lock problem
            if (CMachine::get().m_recorded_blocks_in_db == 0)
            {
                QueryRes
                res = DbModel::customQuery(
                    "db_comen_blocks",
                    "SELECT COUNT(*) AS count_blocks FROM c_blocks",
                    { "count_blocks" },
                    0,
                    {},
                    false,
                    true);
                CMachine::get().m_recorded_blocks_in_db = res.records[0].value("count_blocks").toInt();
            } else {
                CMachine::get().m_recorded_blocks_in_db + +;
            }
        }
         */
        return (true, "block was added to DAG".to_string());
    }
}