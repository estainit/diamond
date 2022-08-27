use crate::{application, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_coinbase::coinbase_block::CoinbaseBlock;
use crate::lib::block::block_types::block_coinbase::do_generate_coinbase_block::do_generate_coinbase_block;
use crate::lib::constants;
use crate::lib::dlog::dlog;
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::services::polling::polling_handler::do_conclude_treatment;

impl CoinbaseBlock {
    // old name was validateCoinbaseBlock
    pub fn validate_coinbase_block(&self, block_super_remote: &Block) -> EntryParsingResult
    {
        let block_identifier = block_super_remote.get_block_identifier();
        let error_message;
        let (
            _cycle_stamp,
            from_, to_,
            _from_hour, _to_hour) = application().get_coinbase_info(
            &"".to_string(), &self.m_cycle);

        dlog(
            &format!("\n{} Validate Coinbase {} cycle:({}) from:({}) to:({})",
                     block_identifier, block_super_remote.get_block_identifier(), self.m_cycle, from_, to_),
            constants::Modules::CB,
            constants::SecLevel::Info);

        // in case of syncing, we force machine to (maybe)conclude the open pollings
        // this code should be somewhere else, because conceptually it has nothing with coinbase flow!
        if machine().is_in_sync_process(false)
        {
            do_conclude_treatment(&block_super_remote.m_block_creation_date, false);
        }

        let (_status, mut local_regenerated_coinbase) = do_generate_coinbase_block(
            &self.m_cycle,
            constants::stages::REGENERATING,
            &block_super_remote.m_block_version);

        // re-write remote values on local values
        local_regenerated_coinbase.m_block_ancestors = block_super_remote.m_block_ancestors.clone();

        local_regenerated_coinbase.m_block_hash = constants::HASH_ZEROS_PLACEHOLDER.to_string();
        let block_length = local_regenerated_coinbase.calc_block_length();
        local_regenerated_coinbase.m_block_length = block_length;

        let local_regenerated_hash = local_regenerated_coinbase.calc_block_hash();
        local_regenerated_coinbase.set_block_hash(&local_regenerated_hash);

        if local_regenerated_coinbase.get_block_hash() != block_super_remote.m_block_hash
        {
            error_message = format!(
                "coinbase lock hash mismatch : remote{}, {} local({})",
                block_identifier,
                block_super_remote.m_block_hash,
                local_regenerated_coinbase.get_block_hash());
            dlog(
                &error_message,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: error_message,
            };
        }

        dlog(
            &format!("dummy dumping after calculating it's length(serialized) Remote: {}", serde_json::to_string(&block_super_remote).unwrap()),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        dlog(
            &format!("dummy dumping after calculating it's length(serialized) local_regenerated_coinbase: {}", serde_json::to_string(&local_regenerated_coinbase).unwrap()),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);

        //  CLog::log("dummy dumping local_regenerated_coinbase beofr calculating it's length(object): " + cutils::dumpIt(local_regenerated_coinbase) , "cb", "info");

        if local_regenerated_coinbase.m_block_documents_root_hash != block_super_remote.m_block_documents_root_hash
        {
            error_message = format!(
                "Discrepancy in bDocsRootHash locally created coinbase bDocsRootHash local({}) Remot({})",
                local_regenerated_coinbase.m_block_documents_root_hash,
                block_super_remote.m_block_documents_root_hash);
            dlog(
                &error_message,
                constants::Modules::CB,
                constants::SecLevel::Error);
            dlog(
                &format!("Remote block: {}", serde_json::to_string(&block_super_remote).unwrap()),
                constants::Modules::CB,
                constants::SecLevel::Error);
            dlog(
                &format!("Local regenerated block: {}", serde_json::to_string(&local_regenerated_coinbase).unwrap()),
                constants::Modules::CB,
                constants::SecLevel::Error);

            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: error_message,
            };
        }

        if local_regenerated_coinbase.m_block_hash != block_super_remote.m_block_hash
        {
            error_message = format!(
                "Discrepancy in block hash the local({}) the remot({})",
                local_regenerated_coinbase.m_block_hash,
                block_super_remote.m_block_hash);
            dlog(
                &error_message,
                constants::Modules::CB,
                constants::SecLevel::Error);
            dlog(
                &format!("Remote block: {}", serde_json::to_string(&block_super_remote).unwrap()),
                constants::Modules::CB,
                constants::SecLevel::Error);
            dlog(
                &format!("Local regenerated block: {}", serde_json::to_string(&local_regenerated_coinbase).unwrap()),
                constants::Modules::CB,
                constants::SecLevel::Error);

            return EntryParsingResult {
                m_status: false,
                m_should_purge_record: true,
                m_message: error_message,
            };
        }

        dlog(
            &format!("remoteConfidence({})", block_super_remote.m_block_confidence),
            constants::Modules::CB,
            constants::SecLevel::Info);

        error_message = format!("Valid Coinbase block has received. {}", block_super_remote.get_block_identifier());
        dlog(
            &error_message,
            constants::Modules::CB,
            constants::SecLevel::Info);
        return EntryParsingResult {
            m_status: true,
            m_should_purge_record: true,
            m_message: error_message,
        };
    }
}