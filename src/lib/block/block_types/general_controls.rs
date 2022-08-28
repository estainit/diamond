use crate::{constants, cutils, dlog};
use crate::cmerkle::{generate_m, MERKLE_VERSION};
use crate::lib::block::block_types::block::Block;
use crate::lib::custom_types::VString;
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;
use crate::lib::services::society_rules::society_rules::get_max_block_size;
use crate::lib::utils::version_handler::is_valid_version_number;

impl Block {
    // old name was blockGeneralControls
    pub fn block_general_controls(&self) -> EntryParsingResult
    {
        let error_message: String;
        let block_identifier = self.get_block_identifier();

        if self.m_block_net != constants::SOCIETY_NAME
        {
            error_message = format!(
                "Invalid society communication! {} Society({})",
                block_identifier,
                self.m_block_net);
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

        // block create date control
        if cutils::is_greater_than_now(&self.m_block_creation_date)
        {
            error_message = format!(
                "Invalid block creation date! {} creation date({})",
                block_identifier,
                self.m_block_creation_date);
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

        if self.m_block_length > get_max_block_size(&self.m_block_type.to_string())
        {
            error_message = format!(
                "Invalid block length {} length({}) > MAX: {})",
                block_identifier,
                self.m_block_length,
                get_max_block_size(&self.m_block_type.to_string()));
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

        // Block length control
        if !self.control_block_length()
        {
            error_message = format!(
                "second Invalid block length {} type({})",
                block_identifier,
                self.m_block_type);
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


        if (self.m_block_version == "") || !is_valid_version_number(&self.m_block_version)
        {
            error_message = format!(
                "Invalid block Version {} {}",
                block_identifier, self.m_block_version);
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

        if !cutils::is_a_valid_date_format(&self.m_block_creation_date)
        {
            error_message = format!(
                "Invalid creation date {} creation date({})",
                block_identifier, self.m_block_creation_date);
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

        if cutils::is_greater_than_now(&self.m_block_creation_date)
        {
            error_message = format!(
                "Block whith future creation date is not acceptable({}) not {}!",
                self.m_block_creation_date, block_identifier);
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

        // ancestors control
        if self.m_block_ancestors.len() < 1
        {
            error_message = format!(
                "Invalid ancestors for {}", block_identifier);
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

        if !cutils::is_valid_hash(&self.m_block_hash)
        {
            error_message = format!(
                "Invalid block hash for {} {}",
                block_identifier, self.m_block_hash);
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

        // docRootHash control
        if self.m_block_documents.len() > 0
        {
            let mut doc_hashes: VString = vec![];
            for a_doc in &self.m_block_documents
            {
                doc_hashes.push(a_doc.get_doc_hash());
            }

            let (
                root,
                _verifies,
                _version,
                _levels,
                _leaves) = generate_m(
                doc_hashes,
                &"hashed".to_string(),
                &"keccak256".to_string(),
                &MERKLE_VERSION.to_string());

            if self.m_block_documents_root_hash != root
            {
                error_message = format!(
                    "Mismatch block DocRootHash for {}  hash: {} creation date({})",
                    block_identifier, self.m_block_hash, self.m_block_creation_date);
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

            // ext root hash control
            if self.m_block_ext_root_hash != ""
            {
                let (status, block_ext_root_hash) = self.calc_block_ext_root_hash();
                if !status || (block_ext_root_hash != self.m_block_ext_root_hash)
                {
                    error_message = format!(
                        "Mismatch block Doc ext RootHash for {}  hash: {} creation date({})",
                        block_identifier, self.m_block_hash, self.m_block_creation_date);
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
            }

            // re-calculate block hash
            let re_calc_block_hash: String = self.calc_block_hash();
            if re_calc_block_hash != self.m_block_hash
            {
                error_message = format!(
                    "Mismatch block bHash. {} localy calculated({}) remote({}):  \n remote body: {}",
                    block_identifier,
                    re_calc_block_hash,
                    self.m_block_hash,
                    self.safe_stringify_block(true));
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

            for a_doc in &self.m_block_documents
            {
                let (status, msg) = a_doc.full_validate(self);
                if !status
                {
                    let doc_identifier = format!(" document({}/#{})", a_doc.m_doc_type, cutils::hash6c(&a_doc.m_doc_hash));
                    error_message = format!(
                        "{}, contains invalid document, {} {}",
                        block_identifier, doc_identifier, msg);
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
            }
        }
        return EntryParsingResult {
            m_status: true,
            m_should_purge_record: true,
            m_message: "Fully validated block.".to_string(),
        };
    }
}