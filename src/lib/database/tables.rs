pub const C_KVALUE: &str = "c_kvalue";
pub const C_MACHINE_PROFILES: &str = "c_machine_profiles";
pub const C_LOGS_BROADCAST: &str = "c_logs_broadcast";

pub const C_MACHINE_NEIGHBORS: &str = "c_machine_neighbors";
pub const C_MACHINE_NEIGHBORS_FIELDS: [&str; 9] = ["n_id", "n_mp_code", "n_email", "n_pgp_public_key", "n_is_active", "n_connection_type", "n_info", "n_creation_date", "n_last_modified"];

pub const C_MACHINE_WALLET_ADDRESSES: &str = "c_machine_wallet_addresses";
pub const C_MACHINE_WALLET_ADDRESSES_FIELDS: [&str; 6] = ["wa_id", "wa_mp_code", "wa_address", "wa_title", "wa_detail", "wa_creation_date"];

pub const C_MACHINE_WALLET_FUNDS: &str = "c_machine_wallet_funds";
pub const C_MACHINE_WALLET_FUNDS_FIELDS: [&str; 11] = ["wf_id", "wf_mp_code", "wf_address", "wf_block_hash", "wf_trx_type", "wf_trx_hash", "wf_o_index", "wf_o_value", "wf_creation_date", "wf_mature_date", "wf_last_modified"];

pub const C_BLOCKS: &str = "c_blocks";
pub const C_BLOCKS_FIELDS: [&str; 19] = ["b_id", "b_hash", "b_type", "b_cycle", "b_confidence", "b_ext_root_hash", "b_docs_root_hash", "b_signals", "b_trxs_count", "b_docs_count", "b_ancestors_count", "b_ancestors", "b_descendants", "b_body", "b_creation_date", "b_receive_date", "b_confirm_date", "b_backer", "b_utxo_imported"];

pub const C_BLOCK_EXTINFOS: &str = "c_block_extinfos";
pub const C_BLOCK_EXTINFOS_FIELDS: [&str; 3] = ["x_block_hash", "x_detail", "x_creation_date"];

pub const C_DOCS_BLOCKS_MAP: &str = "c_docs_blocks_map";
pub const C_DOCS_BLOCKS_MAP_FIELDS: [&str; 4] = ["dbm_block_hash", "dbm_doc_index", "dbm_doc_hash", "dbm_last_control"];

pub const C_DNA_SHARES: &str = "c_dna_shares";
pub const C_DNA_SHARES_FIELDS: [&str; 14] = ["dn_id", "dn_doc_hash", "dn_shareholder", "dn_project_hash", "dn_help_hours", "dn_help_level", "dn_shares", "dn_title", "dn_descriptions", "dn_tags", "dn_votes_y", "dn_votes_n", "dn_votes_a", "dn_creation_date"];

pub const C_MACHINE_DRAFT_PROPOSALS: &str = "c_machine_draft_proposals";
pub const C_MACHINE_DRAFT_PROPOSALS_FIELDS: [&str; 4] = ["dbm_block_hash", "dbm_doc_index", "dbm_doc_hash", "dbm_last_control"];

pub const C_PROPOSALS: &str = "c_proposals";
pub const C_PROPOSALS_FIELDS: [&str; 4] = ["dbm_block_hash", "dbm_doc_index", "dbm_doc_hash", "dbm_last_control"];

pub const C_POLLINGS: &str = "c_pollings";
pub const C_POLLINGS_FIELDS: [&str; 27] = ["pll_id", "pll_hash", "pll_creator", "pll_type", "pll_class", "pll_ref", "pll_ref_type", "pll_ref_class", "pll_start_date", "pll_end_date", "pll_timeframe", "pll_version", "pll_comment", "pll_y_count", "pll_y_shares", "pll_y_gain", "pll_y_value", "pll_n_count", "pll_n_shares", "pll_n_gain", "pll_n_value", "pll_a_count", "pll_a_shares", "pll_a_gain", "pll_a_value", "pll_status", "pll_ct_done"];

pub const C_BALLOTS: &str = "c_ballots";
pub const C_BALLOTS_FIELDS: [&str; 10] = ["ba_hash", "ba_pll_hash", "ba_creation_date", "ba_receive_date", "ba_voter", "ba_voter_shares", "ba_vote", "ba_comment", "ba_vote_c_diff", "ba_vote_r_diff"];

pub const C_POLLING_PROFILES: &str = "c_polling_profiles";
pub const C_POLLING_PROFILES_FIELDS: [&str; 6] = ["ppr_name", "ppr_activated", "ppr_perform_type", "ppr_amendment_allowed", "ppr_votes_counting_method", "ppr_version"];

pub const C_ADMINISTRATIVE_REFINES_HISTORY: &str = "c_administrative_refines_history";
pub const C_ADMINISTRATIVE_REFINES_HISTORY_FIELDS: [&str; 5] = ["arh_id", "arh_hash", "arh_subject", "arh_value", "arh_apply_date"];

pub const C_ADMINISTRATIVE_POLLINGS: &str = "c_administrative_pollings";
pub const C_ADMINISTRATIVE_POLLINGS_FIELDS: [&str; 10] = ["apr_id", "apr_hash", "apr_creator", "apr_subject", "apr_values", "apr_comment", "apr_creation_date", "apr_conclude_date", "apr_approved", "apr_conclude_info"];

pub const C_SIGNALS: &str = "c_signals";
pub const C_SIGNALS_FIELDS: [&str; 6] = ["sig_id", "sig_block_hash", "sig_signaler", "sig_key", "sig_value", "sig_creation_date"];

pub const C_TREASURY: &str = "c_treasury";
pub const C_TREASURY_FIELDS: [&str; 8] = ["tr_id", "tr_cat", "tr_title", "tr_descriptions", "tr_creation_date", "tr_block_hash", "tr_coin", "tr_value"];

pub const C_SENDING_Q: &str = "c_sending_q";
pub const C_SENDING_Q_FIELDS: [&str; 8] = ["sq_id", "sq_type", "sq_code", "sq_title", "sq_sender", "sq_receiver", "sq_connection_type", "sq_payload"];
pub const CDEV_SENDING_Q: &str = "cdev_sending_q";

pub const C_PARSING_Q: &str = "c_parsing_q";
pub const C_PARSING_Q_FIELDS: [&str; 13] = ["pq_id", "pq_type", "pq_code", "pq_sender", "pq_connection_type", "pq_receive_date", "pq_payload", "pq_prerequisites", "pq_parse_attempts", "pq_v_status", "pq_creation_date", "pq_insert_date", "pq_last_modified"];
pub const CDEV_PARSING_Q: &str = "cdev_parsing_q";

pub const C_MACHINE_BLOCK_BUFFER: &str = "c_machine_block_buffer";
pub const C_MACHINE_BLOCK_BUFFER_FIELDS: [&str; 9] = ["bd_id", "bd_mp_code", "bd_insert_date", " bd_doc_hash", "bd_doc_type", "bd_doc_class", "bd_payload", "bd_dp_cost", "bd_doc_len"];

pub const C_MACHINE_ONCHAIN_CONTRACTS: &str = "c_machine_onchain_contracts";
pub const C_MACHINE_ONCHAIN_CONTRACTS_FIELDS: [&str; 6] = ["lc_id", "lc_type", "lc_class", "lc_ref_hash", "lc_descriptions", "lc_body"];

pub const C_MISSED_BLOCKS: &str = "c_missed_blocks";
pub const C_MISSED_BLOCKS_FIELDS: [&str; 6] = ["mb_block_hash", "mb_insert_date", "mb_invoke_attempts", "mb_last_invoke_date", "mb_descendants_count", "mb_descendants"];

pub const C_CPACKET_TICKETING: &str = "c_cpacket_ticketing";
pub const C_CPACKET_TICKETING_FIELDS: [&str; 5] = ["msg_id", "msg_file_id", "msg_try_count", "msg_creation_date", "msg_last_modified"];










