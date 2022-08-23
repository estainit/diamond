
// *****************  Init Dev query ************
pub fn psql_init_query_dev<'l>() -> Vec<&'l str> {
    return vec![
        "
    CREATE TABLE IF NOT EXISTS cdev_inbox_logs (
    il_id bigserial primary key,
    il_title text NOT NULL,
    il_creation_date varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS cdev_parsing_q (
    pq_id bigserial primary key,
    pq_type varchar(32) NOT NULL,
    pq_code varchar(256) NOT NULL,
    pq_sender varchar(256) NOT NULL,   -- the sender's email
    pq_connection_type varchar(256) NOT NULL,   -- public or private
    pq_receive_date varchar(32) NOT NULL, -- receiving time in local node but utc-timezone
    pq_payload TEXT NOT NULL,   -- stringified body of block
    pq_prerequisites TEXT NULL,   -- stringified array of block hash which are needed to validate this block
    pq_parse_attempts INT NULL,  -- parse attempts, to avoid blocking on one block
    pq_v_status varchar(64) NULL,   -- the validation status of block
    pq_creation_date varchar(32) NOT NULL, -- the block creation date
    pq_insert_date varchar(32) NOT NULL,
    pq_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS cdev_sending_q (
    sq_id bigserial primary key,
    sq_type varchar(32) NOT NULL,
    sq_code varchar(256) NOT NULL,
    sq_title varchar(256) NOT NULL,
    sq_sender varchar(256) NOT NULL,   -- the sender's email
    sq_receiver varchar(256) NOT NULL,   -- the receiver's email
    sq_connection_type varchar(256) NOT NULL,   -- public or private
    sq_payload TEXT NOT NULL,   -- stringified body of block
    sq_send_attempts INT NULL,  -- send attempts, to avoid blocking on one block
    sq_creation_date varchar(32) NOT NULL,
    sq_last_modified varchar(32) NOT NULL
    );
",
        "
    CREATE TABLE IF NOT EXISTS cdev_logs_trx_utxos
    (
    ul_id bigserial primary key,
    ul_action varchar(16) NOT NULL,    -- added/deleted to/from table
    ul_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,    -- creation date of the block which mentioned in visible_by field
    ul_creation_date varchar(32) NOT NULL,    -- creation date of the block which mentioned in visible_by field
    ul_clone_code varchar(256) NOT NULL,    -- group code to avoid duplicate inputs
    ul_ref_loc varchar(512) NOT NULL,    -- ReferenceTransactionHash : ParseInt(OutputIndexNumber).toSting() // may be not usefull->: OutputAddress : ParseInt(OutputValue).toSting()
    ul_o_address varchar(256) NOT NULL,    -- owner's address
    ul_o_value BIGINT NOT NULL,    -- spendable value
    ul_visible_by varchar(256) NOT NULL,    -- the Block which has this transaction in it's history
    ul_ref_creation_date varchar(32) NOT NULL    -- the date the local machin insert it
    );
",
        "
    CREATE INDEX IF NOT EXISTS  index_logs_trx_utxos_ul_action ON cdev_logs_trx_utxos(ul_action);
",
        "
    CREATE INDEX IF NOT EXISTS  index_logs_trx_utxos_ul_creation_date ON cdev_logs_trx_utxos(ul_creation_date);
",
        "
    CREATE INDEX IF NOT EXISTS  index_logs_trx_utxos_ul_ref_loc ON cdev_logs_trx_utxos(ul_ref_loc);
",
        "
    CREATE INDEX IF NOT EXISTS  index_logs_trx_utxos_ul_visible_by ON cdev_logs_trx_utxos(ul_visible_by);
",
        "
    CREATE INDEX IF NOT EXISTS  index_logs_trx_utxos_ul_o_address ON cdev_logs_trx_utxos(ul_o_address);
"];
}

/*

// *****************  TO be deleted ************

-- alter table c_kvalue alter column kv_value type text;
--alter table cdev_logs_trx_utxos alter column ul_clone_code drop not null;
-- ALTER TABLE IF EXISTS c_machine_used_coins RENAME TO c_machine_used_coins;
-- ALTER TABLE c_machine_draft_proposals RENAME COLUMN pd_comments TO pd_comment;
-- ALTER TABLE c_iname_records ADD COLUMN in_is_settled varchar(1) DEFAULT 'N';
-- ALTER TABLE c_trx_suspect_transactions ADD COLUMN st_logger_doc bigserial primary key;
-- ALTER TABLE c_trx_suspect_transactions ADD COLUMN st_logger_doc varchar(256) NOT NULL;
-- ALTER TABLE c_trx_spend RENAME COLUMN sp_ref_loc to sp_coin;

// [ CASCADE | RESTRICT ];
DROP TRIGGER IF EXISTS trigger_name ON table_name;
DROP FUNCTION IF EXISTS name







CREATE OR REPLACE FUNCTION func_c_shares_calc_shares()
RETURNS TRIGGER AS $example_table$
BEGIN
UPDATE c_shares SET dn_shares = dn_help_hours * dn_help_level WHERE NEW.dn_id = dn_id;
RETURN NEW;
END;
$example_table$ LANGUAGE plpgsql;
CREATE TRIGGER trigger_c_shares_calc_shares AFTER INSERT ON c_shares FOR EACH ROW EXECUTE PROCEDURE func_c_shares_calc_shares();






-- trigger to add log
CREATE OR REPLACE FUNCTION added_utxo() RETURNS TRIGGER AS
$BODY$
BEGIN
INSERT INTO
cdev_logs_trx_utxos(ul_action, ul_creation_date, ul_ref_loc, ul_o_address, ul_o_value, ul_visible_by, ul_ref_creation_date)
VALUES('add', new.ut_creation_date, new.ut_coin, new.ut_o_address, new.ut_o_value, new.ut_visible_by, new.ut_ref_creation_date);
RETURN new;
END;
$BODY$
language plpgsql;
DROP TRIGGER IF EXISTS trigger_added_utxo ON c_trx_utxos;
CREATE TRIGGER trigger_added_utxo  AFTER INSERT ON c_trx_utxos FOR EACH ROW EXECUTE PROCEDURE added_utxo();
CREATE OR REPLACE FUNCTION deleted_utxo() RETURNS TRIGGER AS
$BODY$
BEGIN
INSERT INTO
cdev_logs_trx_utxos(ul_action, ul_creation_date, ul_ref_loc, ul_o_address, ul_o_value, ul_visible_by, ul_ref_creation_date)
VALUES('del', old.ut_creation_date, old.ut_coin, old.ut_o_address, old.ut_o_value, old.ut_visible_by, old.ut_ref_creation_date);
RETURN old;
END;
$BODY$
language plpgsql;
DROP TRIGGER IF EXISTS  trigger_deleted_utxo ON c_trx_utxos;
CREATE TRIGGER trigger_deleted_utxo BEFORE DELETE ON c_trx_utxos FOR EACH ROW EXECUTE PROCEDURE deleted_utxo();

-- insert into c_trx_utxos (ut_creation_date,ut_coin,ut_o_address,ut_o_value,ut_visible_by,ut_ref_creation_date)
-- values ('2019-01-14 09:45:34', 'refLoc', 'oAdd', '1212', 'utVis32', 'utRef');

INSERT INTO c_kvalue (kv_key, kv_value, kv_last_modified) VALUES ('k1', 'v1', 'm1');

)

)"};


*/