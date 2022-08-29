/*
#include <iostream>
#include <vector>
#include "init_sqlite.h"

std::vector<std::string> psql_init_query_delete = {
    "DROP TABLE IF EXISTS c_kvalue;",
    "DROP TABLE IF EXISTS c_machine_profiles;",
    "DROP TABLE IF EXISTS c_machine_neighbors;",
    "DROP TABLE IF EXISTS c_machine_wallet_addresses;",
    "DROP TABLE IF EXISTS c_machine_wallet_funds;",
    "DROP TABLE IF EXISTS c_machine_used_coins;",
    "DROP TABLE IF EXISTS c_block_extinfos;",
    "DROP TABLE IF EXISTS c_released_reserves;",
    "DROP TABLE IF EXISTS c_blocks;",
    "DROP TABLE IF EXISTS c_logs_block_import_report;",
    "DROP TABLE IF EXISTS c_cpacket_ticketing;",
    "DROP TABLE IF EXISTS c_machine_direct_messages;",
    "DROP TABLE IF EXISTS c_machine_draft_proposals;",
    "DROP TABLE IF EXISTS c_proposals;",
    "DROP TABLE IF EXISTS c_polling_profiles;",
    "DROP TABLE IF EXISTS c_pollings;",
    "DROP TABLE IF EXISTS c_ballots;",
    "DROP TABLE IF EXISTS c_machine_ballots;",
    "DROP TABLE IF EXISTS c_administrative_pollings;",
    "DROP TABLE IF EXISTS c_administrative_refines_history;",
    "DROP TABLE IF EXISTS c_machine_draft_pledges;",
    "DROP TABLE IF EXISTS c_pledged_accounts;",
    "DROP TABLE IF EXISTS c_machine_onchain_contracts;",
    "DROP TABLE IF EXISTS c_logs_broadcast;",
    "DROP TABLE IF EXISTS c_missed_blocks;",
    "DROP TABLE IF EXISTS c_sending_q;",
    "DROP TABLE IF EXISTS c_parsing_q;",
    "DROP TABLE IF EXISTS c_treasury;",
    "DROP TABLE IF EXISTS c_docs_blocks_map;",
    "DROP TABLE IF EXISTS c_trx_spend;",
    "DROP TABLE IF EXISTS c_trx_suspect_transactions;",
    "DROP TABLE IF EXISTS c_logs_suspect_transactions;",
    "DROP TABLE IF EXISTS c_trx_rejected_transactions;",
    "DROP TABLE IF EXISTS c_trx_output_time_locked;",
    "DROP TABLE IF EXISTS c_logs_time_locked;",
    "DROP TABLE IF EXISTS c_trx_coins;",
    "DROP TABLE IF EXISTS c_machine_block_buffer;",
    "DROP TABLE IF EXISTS c_machine_tmp_contents;",
    "DROP TABLE IF EXISTS c_shares;",
    "DROP TABLE IF EXISTS c_iname_records;",
    "DROP TABLE IF EXISTS c_machine_iname_records;",
    "DROP TABLE IF EXISTS c_iname_bindings;",
    "DROP TABLE IF EXISTS c_machine_iname_bindings;",
    "DROP TABLE IF EXISTS c_machine_iname_messages;",
    "DROP TABLE IF EXISTS c_collisions;",
    "DROP TABLE IF EXISTS c_signals;",
    "DROP TABLE IF EXISTS c_machine_posted_files;",
    "DROP TABLE IF EXISTS c_custom_posts;",
    "DROP TABLE IF EXISTS c_wiki_pages;",
    "DROP TABLE IF EXISTS c_wiki_contents;",
    "DROP TABLE IF EXISTS c_agoras;",
    "DROP TABLE IF EXISTS c_agoras_posts;",
    "DROP TABLE IF EXISTS c_nodes_snapshots;",

    // developers part
    "DROP TABLE IF EXISTS cdev_inbox_logs;",
    "DROP TABLE IF EXISTS cdev_parsing_q;",
    "DROP TABLE IF EXISTS cdev_sending_q;",
    "DROP TABLE IF EXISTS cdev_logs_trx_coins;"};

// *****************  Init query ************

*/

pub fn psql_tables_list<'l>() -> Vec<&'l str> {
    return vec![
        "c_administrative_pollings",
        "c_administrative_refines_history",
        "c_ballots",
        "c_blocks",
        "c_block_extinfos",
        "c_collisions",
        "c_agoras",
        "c_agoras_posts",
        "c_shares",
        "c_docs_blocks_map",
        "c_pledged_accounts",
        "c_iname_bindings",
        "c_iname_records",
        "c_kvalue",
        "c_logs_block_import_report",
        "c_logs_broadcast",
        "c_logs_suspect_transactions",
        "c_logs_time_locked",
        "c_machine_ballots",
        "c_machine_block_buffer",
        "c_machine_direct_messages",
        "c_machine_draft_pledges",
        "c_machine_draft_proposals",
        "c_machine_iname_messages",
        "c_machine_iname_bindings",
        "c_machine_iname_records",
        "c_machine_neighbors",
        "c_machine_onchain_contracts",
        "c_machine_posted_files",
        "c_machine_profiles",
        "c_machine_tmp_contents",
        "c_machine_used_coins",
        "c_machine_wallet_addresses",
        "c_machine_wallet_funds",
        "c_cpacket_ticketing",
        "c_missed_blocks",
        "c_nodes_snapshots",
        "c_parsing_q",
        "c_pollings",
        "c_polling_profiles",
        "c_proposals",
        "c_sending_q",
        "c_signals",
        "c_released_reserves",
//  "c_requests_for_release_reserved_coins",
        "c_treasury",
        "c_trx_rejected_transactions",
        "c_trx_spend",
        "c_trx_suspect_transactions",
        "c_trx_output_time_locked",
        "c_trx_coins",
        "c_wiki_contents",
        "c_wiki_pages",
        "cdev_inbox_logs",
        "cdev_logs_trx_coins",
        "cdev_parsing_q",
        "cdev_sending_q"
    ];
}

pub fn psql_init_query<'l>() -> Vec<&'l str> {
    return vec![
        "
    CREATE TABLE IF NOT EXISTS c_machine_profiles
    (
    mp_code varchar(32) UNIQUE NOT NULL,
    mp_name varchar(256) NOT NULL,
    mp_settings text NUll,
    mp_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_kvalue
    (
    kv_id bigserial primary key,
    kv_key varchar(256) NOT NULL,
    kv_value text NUll,
    kv_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE UNIQUE INDEX IF NOT EXISTS index_kvalue_kv_key ON c_kvalue(kv_key);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_neighbors
    (
    n_id bigserial primary key,
    n_mp_code varchar(32) NOT NULL,
    n_email varchar(256) NOT NULL,
    n_pgp_public_key text NULL,
    n_is_active varchar(1) DEFAULT 'Y',
    n_connection_type varchar(32) NOT NULL DEFAULT 'Public',
    n_info text NULL,
    n_creation_date varchar(32) NOT NULL,
    n_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_wallet_addresses
    (
    wa_id bigserial primary key,
    wa_mp_code varchar(32) NOT NULL,
    wa_address varchar(256) NOT NULL,    -- BECH32 format
    wa_title varchar(256) NULL,
    wa_detail text NOT NULL,    -- include proofs, pairkeys, merkle root....
    wa_creation_date varchar(32) NOT NULL
    );
",
        "
    CREATE INDEX IF NOT EXISTS index_wallet_addresses_wa_mp_code ON c_machine_wallet_addresses(wa_mp_code);
",
        "
    CREATE INDEX IF NOT EXISTS index_wallet_addresses_wa_address ON c_machine_wallet_addresses(wa_address);
",
        "
    ALTER TABLE c_machine_wallet_addresses ADD CONSTRAINT c_machine_wallet_addresses_ma UNIQUE (wa_mp_code, wa_address);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_wallet_funds
    (
    wf_id bigserial primary key,
    wf_mp_code varchar(32) NOT NULL,    -- machine profile id
    wf_address varchar(256) NOT NULL,    -- BECH32 format
    wf_block_hash varchar(256) NOT NULL,    -- reference transaction's block hash
    wf_trx_type varchar(32) NOT NULL,    -- reference transaction's type
    wf_trx_hash varchar(256) NOT NULL,    -- reference transaction's hash
    wf_o_index INT,    -- output index
    wf_o_value BIGINT,    -- output value
    wf_creation_date varchar(32) NOT NULL,
    wf_mature_date varchar(32) NOT NULL,
    wf_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_wallet_funds_wf_mp_code ON c_machine_wallet_funds(wf_mp_code);
",
        "
    CREATE INDEX IF NOT EXISTS  index_wallet_funds_wf_address ON c_machine_wallet_funds(wf_address);
",
        "
    ALTER TABLE c_machine_wallet_funds ADD CONSTRAINT c_machine_wallet_funds_thoi UNIQUE (wf_trx_hash, wf_o_index);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_used_coins
    (
    lu_mp_code varchar(32) NOT NULL,    -- machine profile id
    lu_coin varchar(512) NOT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    lu_spend_loc varchar(256) NOT NULL,    -- TrxHash
    lu_insert_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_block_extinfos
    (
    x_block_hash varchar(256) NOT NULL,    -- block's root hash
    x_detail text NULL,    -- stringify json extra-information details
    x_creation_date varchar(32) NOT NULL    -- this value comes from block creation date and will be used to (somethime)drop old extra-information
    );
",
        "
    CREATE UNIQUE INDEX IF NOT EXISTS index_block_extinfos_block_hash ON c_block_extinfos(x_block_hash);
",
        "
    CREATE TABLE IF NOT EXISTS c_released_reserves
    (
    rb_hash varchar(256) UNIQUE NOT NULL,    -- block root hash
    rb_eye_block varchar(256) NOT NULL,    -- reference block root hash
    rb_reserve_number varchar(1) NOT NULL,    -- coud be 1,2,3 or 4
    rb_release_date varchar(32) NOT NULL,
    rb_cycle varchar(32) NOT NULL
    );
",
        "
 ALTER TABLE c_released_reserves ADD CONSTRAINT c_released_reserves_er UNIQUE (rb_eye_block, rb_reserve_number);
",
        "
    CREATE TABLE IF NOT EXISTS c_blocks
    (
    b_id bigserial primary key,
    b_hash varchar(256) UNIQUE NOT NULL,    -- block root hash
    b_type varchar(32) NOT NULL,    -- block type (genesis/coinbase/normal)
    b_cycle varchar(32) NOT NULL,    -- the coin base cycle
    b_confidence DOUBLE PRECISION NULL,    -- if the block is coinbase it denots to percentage of share of signers
    b_ext_root_hash varchar(256) NULL,    -- it was ext_infos_root_hash segwits/zippedInfo... root hashes
    b_docs_root_hash varchar(256) NULL,    -- it was docs_root_hash documents root hash
    b_signals text NULL,    -- comma seperated signals
    b_trxs_count INT,    -- transaction counts
    b_docs_count INT,    -- documents counts
    b_ancestors_count INT NOT NULL,    -- ancestors counts
    b_ancestors text NOT NULL,    -- comma seperated block ancestors
    b_descendants text NULL,    -- comma seperated block descendents
    b_body text NOT NULL,    -- stringify json block full body
    b_creation_date varchar(32) NOT NULL,    -- the block creation date which stated by block creator
    b_receive_date varchar(32) NOT NULL,    -- the block receive date in local, only for statistics
    -- TODO: change code if this field exist, insert and if not ignored
    b_confirm_date varchar(32) NOT NULL,    -- the block confirmation date in local node
    b_backer varchar(128) NULL,    -- the BECH32 address of who got paid because of creating this block
    b_coins_imported varchar(1) NOT NULL DEFAULT 'N'    -- does the coins imported to c_trx_coins table?
    );
",
        "
    CREATE UNIQUE INDEX IF NOT EXISTS index_blocks_block_hash ON c_blocks(b_hash);
",
        "
    CREATE INDEX IF NOT EXISTS  index_blocks_creation_date ON c_blocks(b_creation_date);
",
        "
    CREATE INDEX IF NOT EXISTS  index_blocks_block_type ON c_blocks(b_type);
",
        "
    CREATE INDEX IF NOT EXISTS  index_blocks_backer ON c_blocks(b_backer);
",
        "
    CREATE INDEX IF NOT EXISTS  index_blocks_coins_imported ON c_blocks(b_coins_imported);
",
        "
    CREATE TABLE IF NOT EXISTS c_logs_block_import_report
    (
    li_id bigserial primary key,
    li_block_hash varchar(256) NOT NULL,
    li_title varchar(256) NOT NULL,
    li_report TEXT NULL,
    li_insert_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_cpacket_ticketing
    (
    msg_id bigserial primary key,
    msg_file_id varchar(256) NOT NULL,
    msg_try_count INT DEFAULT 0,
    msg_creation_date varchar(32) NOT NULL,
    msg_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_direct_messages
    (
    dm_id bigserial primary key,
    dm_mp_code varchar(32) NOT NULL,
    dm_sender TEXT NOT NULL,
    dm_receiver TEXT NOT NULL,
    dm_message TEXT NOT NULL,
    dm_creation_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_draft_proposals
    (
    pd_id bigserial primary key,
    pd_mp_code varchar(32) NOT NULL,
    pd_hash varchar(256) NOT NULL,
    pd_type varchar(32) NOT NULL,
    pd_class varchar(32) NOT NULL,
    pd_version varchar(8) NOT NULL,
    pd_title TEXT NOT NULL,
    pd_comment TEXT NULL,
    pd_tags TEXT NULL,
    pd_project_hash varchar(256) NOT NULL,
    pd_help_hours INT NOT NULL,
    pd_help_level INT NOT NULL,
    pd_voting_timeframe DOUBLE PRECISION NOT NULL,
    pd_polling_profile varchar(256) NOT NULL,
    pd_contributor_account varchar(128) NOT NULL,
    pd_body TEXT NULL,
    pd_creation_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_proposals
    (
    pr_id bigserial primary key,
    pr_hash varchar(256) NOT NULL UNIQUE,
    pr_type varchar(32) NOT NULL,
    pr_class varchar(32) NOT NULL,
    pr_version varchar(8) NOT NULL,
    pr_title TEXT NOT NULL,
    pr_descriptions TEXT NULL,
    pr_tags TEXT NULL,
    pr_project_id varchar(256) NOT NULL,
    pr_help_hours INT4 NOT NULL,
    pr_help_level INT4 NOT NULL,
    pr_voting_timeframe DOUBLE PRECISION NOT NULL,    --by hours
    pr_polling_profile varchar(32) NOT NULL,    -- vote counting method(Plurality, ...)
    pr_contributor_account varchar(128) NOT NULL,
    pr_start_voting_date varchar(32) NOT NULL,
    pr_conclude_date varchar(32) NULL,    -- the date in which the poroposal approved or rejected, date must be yyy-mm-dd 00:00:00 or yyy-mm-dd 12:00:00
    pr_approved varchar(1) NOT NULL DEFAULT 'N'
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_polling_profiles
    (
    ppr_name varchar(64) UNIQUE NOT NULL,    -- an asci-char name to refer a polling profile
    ppr_activated varchar(1) NOT NULL,
    ppr_perform_type varchar(32) NOT NULL,
    ppr_amendment_allowed varchar(1) NOT NULL,
    ppr_votes_counting_method varchar(32) NOT NULL,
    ppr_version varchar(8) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_pollings
    (
    pll_id bigserial primary key,
    pll_hash varchar(256) NOT NULL UNIQUE,
    pll_creator varchar(128) NOT NULL,    -- bech32 address of polling creator/owner, will be usefull in case of changing voting longivity
    pll_type varchar(32) NOT NULL,
    pll_class varchar(256) NOT NULL,    -- external key which refers to c_polling_profiles.(hash of polling profile) in polling documetn this fileds is equal to dClass which is by default iConsts.POLLING_PROFILE_CLASSES.Basic.ppName

    pll_ref varchar(256) NOT NULL,    -- hash of related doc/subject as an external-key
    pll_ref_type varchar(32) NOT NULL,    -- the type of doc which are voting for, iConsts.POLLING_REF_TYPE = [Proposal, ReqForRelRes] ...
    pll_ref_class varchar(32) NOT NULL,

    pll_start_date varchar(32) NOT NULL,
    pll_end_date varchar(32) NOT NULL,
    pll_timeframe DOUBLE PRECISION NOT NULL,    -- by hours and for Yes votes,
    pll_version varchar(8) NOT NULL,
    pll_comment TEXT NULL,

    pll_y_count BIGINT NOT NULL,    -- number of voters for Yes
    pll_y_shares BIGINT NOT NULL,    -- total shares of voterd for Yes
    pll_y_gain BIGINT NOT NULL,    -- gain of vote which multiple voter shares
    pll_y_value BIGINT NOT NULL,    -- final vote value for Yes

    pll_n_count BIGINT NOT NULL,    -- number of voters for No
    pll_n_shares BIGINT NOT NULL,    -- total shares of voterd for No
    pll_n_gain BIGINT NOT NULL,    -- gain of vote which multiple voter shares
    pll_n_value BIGINT NOT NULL,    -- final vote value for No

    pll_a_count BIGINT NOT NULL,    -- number of voters for Abstain
    pll_a_shares BIGINT NOT NULL,    -- total shares of voterd for Abstain
    pll_a_gain BIGINT NOT NULL,    -- gain of vote which multiple voter shares
    pll_a_value BIGINT NOT NULL,    -- final vote value for Abstain

    pll_status varchar(1) NOT NULL,    -- Open/Closed/Renewed
    pll_ct_done varchar(1) NOT NULL DEFAULT 'N'    -- does concluded or not
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_ballots
    (
    ba_hash varchar(256) NOT NULL UNIQUE,   -- the ballot hash
    ba_pll_hash varchar(256) NOT NULL,      -- the polling which are voting for
    ba_creation_date varchar(32) NOT NULL , -- block.creation date
    ba_receive_date varchar(32) NOT NULL ,  -- local receive date
    ba_voter varchar(128) NOT NULL,         -- bech32 address of voter
    ba_voter_shares BIGINT NOT NULL,        -- voter share on voting date
    ba_vote SMALLINT NOT NULL,              -- Yes/No/Abstain  Fuzzy logic implementation, accept from -100 to +100 and gains on Yes or No votes
    ba_comment TEXT NULL,                   -- voter commetns
    ba_vote_c_diff INT NOT NULL,            -- how many minutes passed from voting start time? it stated by voter(picking from container block, creation date)
    ba_vote_r_diff INT NOT NULL             -- how many minutes passed from voting start time? the date in which local machine received the vote. it could be diffrent in different machines
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_votes ON c_ballots(ba_pll_hash);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_ballots
    (
    lbt_id bigserial primary key,
    lbt_mp_code varchar(32) NOT NULL,
    lbt_hash varchar(256) NOT NULL,    -- the ballot hash
    lbt_pll_hash varchar(256) NOT NULL,    -- the polling which are voting for
    lbt_creation_date varchar(32) NOT NULL ,    -- block.creation date
    lbt_voter varchar(128) NOT NULL,    -- bech32 address of voter
    lbt_voter_shares BIGINT NOT NULL,    -- voter share on voting date
    lbt_voter_percent DOUBLE PRECISION NOT NULL,    -- voter percentage on voting date
    lbt_vote SMALLINT NOT NULL    -- Yes/No/Abstain  Fuzzy logic implementation, accept from 0 to 100 and gains on Yes or No votes
    );
",
        "
 ALTER TABLE c_machine_ballots ADD CONSTRAINT c_machine_ballots_polvot UNIQUE (lbt_pll_hash, lbt_voter);
",
        "
    CREATE TABLE IF NOT EXISTS c_administrative_pollings
    (
    apr_id bigserial primary key,
    apr_hash varchar(256) UNIQUE NOT NULL,    -- doc hash
    apr_creator varchar(128) NOT NULL,        -- bech32 address of polling request creator
    apr_subject varchar(256) NULL,
    apr_values TEXT NOT NULL,
    apr_comment TEXT NULL,
    apr_creation_date varchar(32) NOT NULL,   -- this is block.creation date which will convert to block creation cycle e.g. yyyy-mm-dd 00:00:00/12:00:00
    apr_conclude_date varchar(32) NULL,
    apr_approved varchar(1) NOT NULL DEFAULT 'N',
    apr_conclude_info TEXT NUll
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_administrative_refines_history
    (
    arh_id bigserial primary key,
    arh_hash varchar(256) UNIQUE NOT NULL,    -- admPolling doc hash
    arh_subject varchar(256) NULL,
    arh_value TEXT NOT NULL,                  -- depends on arh_subject, this field could be plain-text or serialized values
    arh_apply_date varchar(32) NOT NULL       -- if the date is >= than arh_apply_date, so the valid value is what recorded in this record
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_administrative_refine_history_arh_subject ON c_administrative_refines_history(arh_subject);
",
        "
    CREATE INDEX IF NOT EXISTS  index_administrative_refine_history_arh_apply_date ON c_administrative_refines_history(arh_apply_date);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_draft_pledges
    (
    dpl_id bigserial primary key,
    dpl_mp_code varchar(32) NOT NULL,
    dpl_type varchar(32) NOT NULL,
    dpl_class varchar(32) NOT NULL,
    dpl_version varchar(8) NOT NULL,
    dpl_comment TEXT NULL,
    dpl_pledger varchar(128) NOT NULL,    -- BECH32 format
    dpl_pledgee varchar(128) NOT NULL,    -- BECH32 format
    dpl_arbiter varchar(128) NULL,    -- BECH32 format
    dpl_doc_ref varchar(256) NOT NULL,
    dpl_body TEXT NULL,
    dpl_req_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_pledged_accounts
    (
    pgd_hash varchar(256) NOT NULL,
    pgd_type varchar(32) NOT NULL,    -- e.g. PledgeP
    pgd_class varchar(32) NOT NULL,
    pgd_version varchar(8) NOT NULL,
    pgd_pledger_sign_date varchar(32) NOT NULL,
    pgd_pledgee_sign_date varchar(32) NOT NULL,
    pgd_arbiter_sign_date varchar(32) NULL,
    pgd_activate_date varchar(32) NOT NULL,    -- the real date of activatine pledge, this date is the begining of 2 cycle later than the recording pledge in DAG
    pgd_close_date varchar(32) NULL,    -- the real date of closing pledge, this date is same as container-block.creation date
    pgd_pledger varchar(128) NOT NULL,    -- BECH32 format
    pgd_pledgee varchar(128) NOT NULL,    -- BECH32 format
    pgd_arbiter varchar(128) NULL,    -- BECH32 format
    pgd_principal BIGINT NOT NULL,
    pgd_annual_interest DOUBLE PRECISION NOT NULL,
    pgd_repayment_offset INT NOT NULL,    -- starting to pay the first repayment after n hours
    pgd_repayment_amount BIGINT NOT NULL,    -- the amount is cutting from income (generaly n each cycle) and payed to Pledgee
    pgd_repayment_schedule BIGINT NOT NULL,    -- the repayments count in a year(tipicaly 365 * 2 = repayment in every cycle)
    pgd_status varchar(1) NOT NULL    -- Open/Closed
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_onchain_contracts
    (
    lc_id bigserial primary key,
    lc_type varchar(32) NOT NULL,    -- e.g. PledgeP
    lc_class varchar(32) NOT NULL,
    lc_ref_hash varchar(256) NOT NULL,
    lc_descriptions TEXT NULL,    -- some descriptions
    lc_body TEXT NULL    -- some descriptions
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_logs_broadcast
    (
    lb_id bigserial primary key,
    lb_type varchar(32) NOT NULL,
    lb_code varchar(256) NOT NULL,
    lb_title varchar(256) NOT NULL,
    lb_sender varchar(256) NOT NULL,    -- the sender's email
    lb_receiver varchar(256) NOT NULL,    -- the receiver's email
    lb_connection_type varchar(256) NOT NULL,    -- public or private
    lb_send_date varchar(32) NOT NULL
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_log_broadcast_lb_type ON c_logs_broadcast(lb_type);
",
        "
    CREATE INDEX IF NOT EXISTS  index_log_broadcast_lb_code ON c_logs_broadcast(lb_code);
",
        "
    CREATE INDEX IF NOT EXISTS  index_log_broadcast_lb_receiver ON c_logs_broadcast(lb_receiver);
",
        "
    CREATE TABLE IF NOT EXISTS c_missed_blocks
    (
    mb_block_hash varchar(256) NOT NULL,
    mb_insert_date varchar(32) NOT NULL,    -- the datein which machine discovered she missed this block
    mb_invoke_attempts INT4 NULL,    -- invoke attempts, to avoid blocking on one block
    mb_last_invoke_date varchar(32) NOT NULL,
    mb_descendants_count INT4 NULL,    -- number of blocks whom need this block (potential descendents)
    mb_descendants text    -- potentially descendents hash
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_sending_q
    (
    sq_id bigserial primary key,
    sq_type varchar(32) NOT NULL,
    sq_code varchar(256) NOT NULL,
    sq_title varchar(256) NOT NULL,
    sq_sender varchar(256) NOT NULL,    -- the sender's email
    sq_receiver varchar(256) NOT NULL,    -- the receiver's email
    sq_connection_type varchar(256) NOT NULL,    -- public or private
    sq_payload TEXT NOT NULL,    -- stringified body of block
    sq_send_attempts INT NULL,    -- send attempts, to avoid blocking on one block
    sq_creation_date varchar(32) NOT NULL,
    sq_last_modified varchar(32) NOT NULL
    );
",
        "
 ALTER TABLE c_sending_q ADD CONSTRAINT c_sending_q_bsr UNIQUE (sq_type, sq_code, sq_sender, sq_receiver);
",
        "
    CREATE TABLE IF NOT EXISTS c_parsing_q
    (
    pq_id bigserial primary key,
    pq_type varchar(32) NOT NULL,
    pq_code varchar(256) NOT NULL,
    pq_sender varchar(256) NOT NULL,    -- the sender's email
    pq_connection_type varchar(256) NOT NULL,    -- public or private
    pq_receive_date varchar(32) NOT NULL,    -- receiving time in local node but utc-timezone
    pq_payload TEXT NOT NULL,    -- stringified body of block
    pq_prerequisites TEXT NULL,    -- stringified array of block hash which are needed to validate this block
    pq_parse_attempts INT4 NULL,    -- parse attempts, to avoid blocking on one block
    pq_v_status varchar(64) NULL,    -- the validation status of block
    pq_creation_date varchar(32) NOT NULL,    -- the block creation date
    pq_insert_date varchar(32) NOT NULL,
    pq_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_treasury
    (
    tr_id bigserial primary key,
    tr_cat varchar(32) NOT NULL,    -- e.g. TP_DP, TP_PROPOSAL, TP_PLEDGE...
    tr_title varchar(256) NOT NULL,    -- title
    tr_descriptions TEXT NULL,    -- some descriptions
    tr_creation_date varchar(32) NOT NULL,    -- creation date of the block in which payed to treasury, in case of donate creation date of double-spended refLoc
    tr_block_hash varchar(256) NOT NULL,    -- the block hash in which payed to treasury, in case of donate creation blockHash of double-spended refLoc
    tr_coin varchar(512) UNIQUE NOT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert, in case of donate ref-loc of double-spended refLoc
    tr_value BIGINT NOT NULL    -- payed pai to treasury
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_docs_blocks_map
    (
    dbm_block_hash varchar(256) NOT NULL,
    dbm_doc_index INT NOT NULL,
    dbm_doc_hash varchar(256) NOT NULL,
    dbm_last_control varchar(32) NOT NULL
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_docs_blocks_map_dbm_block_hash ON c_docs_blocks_map(dbm_block_hash);
",
        "
    CREATE INDEX IF NOT EXISTS  index_docs_blocks_map_dbm_doc_hash ON c_docs_blocks_map(dbm_doc_hash);
",
        "
    CREATE TABLE IF NOT EXISTS c_trx_spend
    (
    sp_coin varchar(512) NOT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    sp_spend_loc varchar(1024) NOT NULL,    -- [blockHash, trxHash].join(':');
    sp_spend_date varchar(32) NOT NULL    -- the date stated in block.creation date of spend trx container
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_trx_suspect_transactions
    (
    st_id bigserial primary key,
    st_voter varchar(128) NOT NULL,    -- the bech32 address of who created susVote block
    st_vote_date varchar(32) NOT NULL,    -- susVote block creation date
    st_coin varchar(512) NOT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    st_logger_block varchar(256) NOT NULL,
    st_spender_block varchar(256) NOT NULL,
    st_spender_doc varchar(256) NOT NULL,    -- for each invalid ref MUST insert a row (even in same block)
    st_receive_order INT DEFAULT 0,
    st_spend_date varchar(32) NOT NULL    -- suspect block's creation date
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_logs_suspect_transactions
    (
    ls_id bigserial primary key,
    ls_lkey varchar(32) NOT NULL,    -- log key
    ls_block_hash varchar(256) NOT NULL,
    ls_doc_hash varchar(256) NOT NULL,    -- for each invalid ref MUST insert a row (even in same block)
    ls_coins TEXT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    ls_log_body TEXT NULL,
    ls_creation_date varchar(32) NOT NULL    -- suspect block's creation date
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_trx_rejected_transactions
    (
    rt_block_hash varchar(256) NOT NULL,
    rt_doc_hash varchar(256) NOT NULL,    -- for each invalid ref MUST insert a row (even in same block)
    rt_coin varchar(512) NOT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    rt_insert_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_trx_output_time_locked
    (
    ot_block_hash varchar(256) NOT NULL,    -- block hash
    ot_doc_hash varchar(256) NOT NULL,    -- transaction hash
    ot_pure_hash varchar(256) NOT NULL,    -- transaction pure hash(only iputs & outputs)
    ot_ref_loc varchar(512) NOT NULL,
    ot_doc_body TEXT NULL,    -- stringified body of transaction
    ot_redeem_time varchar(32) NOT NULL,    -- ISO 8601 format
    ot_doc_max_redeem BIGINT NOT NULL,    -- max redeem of doc inputs (by minutes)
    ot_cycle varchar(32) NOT NULL,
    ot_ref_creation_date varchar(32) NOT NULL,
    ot_coin_imported varchar(1) NOT NULL DEFAULT 'N'    -- does coins imported to c_trx_coins table?
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_logs_time_locked
    (
    lt_lkey varchar(32) NOT NULL,    -- log key
    lt_block_hash varchar(256) NOT NULL,
    lt_doc_hashes TEXT NOT NULL,    -- for each invalid ref MUST insert a row (even in same block)
    lt_ref_locs TEXT NOT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    lt_log_body TEXT NULL,    -- TrxHash : ParseInt(OutputIndexNumber).toSting() : OutputAddress NOTE: the ref transaction MUST be created before 12 hours ago. so control it before insert
    lt_creation_date varchar(32) NOT NULL    -- suspect block's creation date
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_trx_coins
    (
    ut_id bigserial primary key,
    ut_creation_date varchar(32) NOT NULL,    -- creation date of the block which mentioned in visible_by field
    -- ut_clone_code varchar(256) NOT NULL,    -- group code to avoid duplicate inputs
    ut_coin varchar(512) NOT NULL,    -- ReferenceTransactionHash : ParseInt(OutputIndexNumber).toSting() // may be not usefull->: OutputAddress : ParseInt(OutputValue).toSting()
    ut_o_address varchar(256) NOT NULL,    -- owner's address
    ut_o_value BIGINT NOT NULL,    -- spendable value
    ut_visible_by varchar(256) NOT NULL,    -- the Block which has this transaction in it's history
    ut_ref_creation_date varchar(32) NOT NULL    -- the date the local machin insert it
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_trx_coins_ut_creation_date ON c_trx_coins(ut_creation_date);
",
        "
    CREATE INDEX IF NOT EXISTS  index_trx_coins_ut_ref_loc ON c_trx_coins(ut_coin);
",
        "
    CREATE INDEX IF NOT EXISTS  index_trx_coins_ut_visible_by ON c_trx_coins(ut_visible_by);
",
        "
    CREATE INDEX IF NOT EXISTS  index_trx_coins_ut_o_address ON c_trx_coins(ut_o_address);
",
        "
    ALTER TABLE c_trx_coins ADD CONSTRAINT c_trx_coins_refLocVis UNIQUE (ut_coin, ut_visible_by);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_block_buffer
    (
    bd_id bigserial primary key,
    bd_mp_code varchar(32) NOT NULL,
    bd_insert_date varchar(32) NOT NULL,
    bd_doc_hash varchar(256) NOT NULL UNIQUE,
    bd_doc_type varchar(32) NOT NULL,    -- basicTx, Proposal, ...
    bd_doc_class varchar(32) NOT NULL,    -- mOfn, p4p, ...
    bd_payload TEXT NOT NULL,    -- stringifyed body of a single transaction
    bd_dp_cost BIGINT NOT NULL,    -- the backer's fee
    bd_doc_len INT NOT NULL    -- transaction lengths by char
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_tmp_contents
    (
    tc_id bigserial primary key,
    tc_mp_code varchar(32) NOT NULL,
    tc_insert_date varchar(32) NOT NULL,
    tc_content_status varchar(32) NULL,
    tc_content_hash varchar(256) NOT NULL UNIQUE,
    tc_content_type varchar(32) NOT NULL,
    tc_content_class varchar(32) NOT NULL,
    tc_payload TEXT NOT NULL    -- stringifyed body of a single transaction
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_shares
    (
    dn_id bigserial primary key,
    dn_doc_hash varchar(256) NOT NULL UNIQUE,    -- document hash in blockchain
    dn_shareholder varchar(128) NOT NULL,    -- the bech32 address of share holder
    dn_project_hash varchar(256) NOT NULL,
    dn_help_hours INT4 NOT NULL,
    dn_help_level INT4 NOT NULL,    -- 1 to 7, the signer states the importance of the help
    dn_shares INT8 NOT NULL DEFAULT 0,
    dn_title TEXT NULL,
    dn_descriptions TEXT NULL,    -- some descriptions
    dn_tags TEXT NULL,    -- some useful tags fro statistic reason (e.g js, performance, UI, optimise, consultant, translate ...)
    dn_votes_y BIGINT DEFAULT 0,    -- sum of shares of who voted positively
    dn_votes_n BIGINT DEFAULT 0,    -- sum of shares of who voted negatively
    dn_votes_a BIGINT DEFAULT 0,    -- sum of shares of who voted negatively
    dn_creation_date varchar(32) NOT NULL    -- it is the time of approval proposal
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_shares_dn_shareholder ON c_shares(dn_shareholder);
",
        "
    CREATE INDEX IF NOT EXISTS  index_shares_dn_creation_date ON c_shares(dn_creation_date);
",
        "
    CREATE INDEX IF NOT EXISTS  index_shares_dn_project_hash ON c_shares(dn_project_hash);
",
        "
    CREATE TABLE IF NOT EXISTS c_iname_records
    (
    in_doc_hash varchar(256) NOT NULL,    -- the document which contains iName register request
    in_name varchar(256) NOT NULL UNIQUE,    -- currently it supports only asci chars. TODO: implement unicode name services
    in_hash varchar(256) NOT NULL UNIQUE,
    in_owner varchar(128) NOT NULL,    -- bech32 address of owner
    in_is_settled varchar(1) NOT NULL DEFAULT 'N',    -- if definately registered for this owner?
    in_register_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_iname_records
    (
    imn_id bigserial primary key,
    imn_mp_code varchar(32) NOT NULL,   -- machine profile
    imn_doc_hash varchar(256) NOT NULL,    -- the document which contains iName register request
    imn_name varchar(256) NOT NULL UNIQUE,    -- currently it supports only asci chars. TODO: implement unicode name services
    imn_hash varchar(256) NOT NULL UNIQUE,
    imn_owner varchar(128) NOT NULL,    -- bech32 address of owner
    imn_info TEXT NULL,    -- reserved for future usage
    imn_register_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_iname_bindings
    (
    nb_doc_hash varchar(256) NOT NULL,   -- docHash of binding doc
    nb_in_hash varchar(256) NOT NULL,   -- iNameHash of related iname record
    nb_bind_type varchar(32) NOT NULL,    -- type of binded info e.g. iPGP, BitcoinAddress, EthereumAddress
    nb_conf_info TEXT NOT NULL, -- an stringified string of settings
    nb_title varchar(512) NOT NULL,
    nb_comment TEXT NOT NULL,
    nb_status varchar(1) NOT NULL DEFAULT 'V',
    nb_creation_date varchar(32) NOT NULL
    );
",
        "
 ALTER TABLE c_iname_bindings ADD CONSTRAINT c_iname_bindings_nt UNIQUE (nb_in_hash, nb_title);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_iname_bindings
    (
    mcb_mp_code varchar(32) NOT NULL,   -- machine profile
    mcb_in_hash varchar(256) NOT NULL,   -- iNameHash of related iname record
    mcb_bind_type varchar(32) NOT NULL,    -- type of binded info e.g. iPGP, BitcoinAddress, EthereumAddress
    mcb_label varchar(512) NOT NULL ,
    mcb_conf_info TEXT NOT NULL, -- an stringified string of settings
    mcb_comment TEXT NOT NULL,
    mcb_status varchar(1) NOT NULL DEFAULT 'V',
    mcb_creation_date varchar(32) NOT NULL
    );
",
        "
 ALTER TABLE c_machine_iname_bindings ADD CONSTRAINT c_machine_iname_bindings_pitl UNIQUE (mcb_mp_code, mcb_in_hash, mcb_bind_type, mcb_label);
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_iname_messages
    (
    mim_id bigserial primary key,
    mim_mp_code varchar(32) NOT NULL,   -- machine profile
    mim_type varchar(32) NOT NULL,   -- message type e.g. Plain, Pledge, PublicKey...
    mim_direction varchar(2) NOT NULL,
    mim_doc_hash varchar(256) NOT NULL UNIQUE,   -- container document hash
    mim_sender_in_hash varchar(256) NOT NULL,   -- iNameHash of related iname record
    mim_sender_key_label varchar(512) NOT NULL,    -- type of binded info e.g. iPGP, BitcoinAddress, EthereumAddress
    mim_message TEXT NOT NULL, -- an stringified string of settings
    mim_receiver_in_hash varchar(256) NOT NULL,   -- iNameHash of related iname record
    mim_receiver_key_label varchar(512) NOT NULL,    -- type of binded info e.g. iPGP, BitcoinAddress, EthereumAddress
    mim_status varchar(2) NOT NULL DEFAULT 'UN', -- Unread, Read
    mim_receive_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_collisions
    (
    cl_id bigserial primary key,
    cl_voter varchar(128) NOT NULL,    -- the bech32 address of who created susVote block
    cl_collision_ref varchar(512) NOT NULL,    -- the reference of what has collision (e.g. for iName collisioning it is keccak(iName))
    cl_block_hash varchar(256) NOT NULL,
    cl_doc_hash varchar(256) NOT NULL,    -- the hash of document which is container of conflicted data(e.g. IName reg document) for each invalid ref MUST insert a row (even in same block)
    cl_creation_date varchar(32) NOT NULL,    -- collision document block's creation date
    cl_receive_date varchar(32) NOT NULL    -- local machine(cl_voter time) of discover this collision
    );
",
        "
  ALTER TABLE c_collisions ADD CONSTRAINT c_collisions_vcbd UNIQUE (cl_voter, cl_collision_ref, cl_block_hash, cl_doc_hash);
",
        "
    CREATE TABLE IF NOT EXISTS c_signals
    (
    sig_id bigserial primary key,
    sig_block_hash varchar(256) NOT NULL,
    sig_signaler varchar(128)  NULL,    -- the bech32 address of who created block (backer address)
    sig_key varchar(64) NOT NULL,
    sig_value TEXT NULL,
    sig_creation_date varchar(32) NOT NULL  -- the block creation date
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_machine_posted_files
    (
    mpf_id bigserial primary key,
    mpf_mp_code varchar(32) NOT NULL,   -- machine profile
    mpf_name varchar(300) NOT NULL UNIQUE,    -- fine name  256 char hash and the rest for extnsion
    mpf_doc_hash varchar(256) NOT NULL,
    mpf_mime varchar(128) NOT NULL,    -- content mime type
    mpf_creation_date varchar(32) NOT NULL,
    mpf_signer varchar(128) NOT NULL    -- BECH32 address of signer doc
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_custom_posts
    (
    cp_id bigserial primary key,
    cp_in_hash varchar(256) NOT NULL,    -- related domain hash
    cp_hash varchar(256) NOT NULL UNIQUE,    -- the hash of document which is container of this content
    cp_url TEXT NOT NULL UNIQUE,    -- page url e.g. imagine/home    TODO: having url-hash, it looks not useful recording pure url
    cp_url_hash varchar(256) NOT NULL UNIQUE,    -- page url hash e.g. keccak('imagine/home')   TODO: for the sacke of space! could be better to use shorten hash e.g. 32 char
    cp_mime varchar(128) NOT NULL,    -- content mime type
    cp_content TEXT NOT NULL,    -- static html content/ .png, .pdf, .wav, .mp4 ...
    cp_creation_date varchar(32) NOT NULL,
    cp_last_modified varchar(32) NOT NULL,
    cp_author varchar(128) NOT NULL    -- BECH32 address
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_wiki_pages
    (
    wkp_id bigserial primary key,
    wkp_iname varchar(256) NOT NULL,    -- the hash iName(if Agora belaongs to an iName)
    wkp_title varchar(256) NOT NULL,    -- wiki page's title
    wkp_doc_hash varchar(256) NOT NULL,    -- the hash of document in which the Wiki is registered. every update changes this hash
    wkp_hash varchar(256) NOT NULL UNIQUE,    -- the unique hash of combination of iName & titleHash
    wkp_language varchar(3) NOT NULL DEFAULT 'eng',
    wkp_format_version varchar(8) DEFAULT '0.0.0',
    wkp_creation_date varchar(32) NOT NULL,
    wkp_last_modified varchar(32) NOT NULL,
    wkp_creator varchar(128) NOT NULL    -- BECH32 address of creator or modifier
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_wiki_contents
    (
    wkc_wkp_hash varchar(256) NOT NULL UNIQUE,    -- the unique hash of combination of iName & titleHash
    wkc_content TEXT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_agoras
    (
    ag_id bigserial primary key,
    ag_doc_hash varchar(256) NOT NULL,    -- the hash of document in which the Agora is registered
    ag_title varchar(256) NOT NULL,    -- Agora's title
    ag_hash varchar(256) NOT NULL UNIQUE,    -- the unique hash of combination of parentHash/titleHash
    ag_iname varchar(256) NOT NULL,    -- the iName(if Agora belaongs to an iName)
    ag_full_category TEXT NOT NULL,
    ag_parent varchar(256) NULL,    -- the hash of parent category(if it is child)
    ag_language varchar(3) NOT NULL DEFAULT 'eng',
    ag_description TEXT NULL,
    ag_content_format_version varchar(8) NOT NULL DEFAULT '0.0.0',
    ag_tags TEXT NULL,
    ag_creation_date varchar(32) NOT NULL,
    ag_last_modified varchar(32) NOT NULL,
    ag_creator varchar(128) NOT NULL,    -- BECH32 address
    ag_controlled_by_machine varchar(1) NOT NULL DEFAULT 'N',
    ag_mp_code varchar(32) NOT NULL    -- BECH32 address
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_agoras_posts
    (
    ap_id bigserial primary key,
    ap_doc_hash varchar(256) NOT NULL UNIQUE,    -- postHash=docHash
    ap_ag_hash varchar(256) NOT NULL,    -- the hash of Owner Agora
    ap_opinion TEXT NULL,
    ap_attrs TEXT NULL,
    ap_format_version varchar(8) NOT NULL DEFAULT '0.0.0',
    ap_reply varchar(256) NULL,    -- if it is a reply, this is the doc_hash of the post that replyed to
    ap_reply_point SMALLINT NULL,    -- if it is a reply
    ap_creation_date varchar(32) NOT NULL,
    ap_creator varchar(128) NOT NULL    -- BECH32 address
    );
",
        "
    CREATE TABLE IF NOT EXISTS c_nodes_snapshots
    (
    nss_id bigserial primary key,
    nss_label varchar(256) NOT NULL UNIQUE,
    nss_content TEXT NULL,
    nss_creation_date varchar(32) NOT NULL
    );
"];
}
