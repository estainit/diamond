use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, dlog};
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CDocHashT, ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::{q_insert, q_select};
use crate::lib::database::tables::{C_TRX_REJECTED_TRANSACTIONS, C_TRX_REJECTED_TRANSACTIONS_FIELDS};

//old_name_was searchInRejectedTrx
#[allow(unused, dead_code)]
pub fn search_in_rejected_trx(
    clauses: ClausesT,
    mut fields: Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    if fields.len() == 0
    {
        fields = Vec::from(C_TRX_REJECTED_TRANSACTIONS_FIELDS)
    }

    let (_status, records) = q_select(
        C_TRX_REJECTED_TRANSACTIONS,
        fields,
        clauses,
        order,
        limit,
        false);

    return records;
}

//old_name_was addTransaction
pub fn add_to_rejected_transactions(
    block_hash: &CBlockHashT,
    doc_hash: &CDocHashT,
    coin: &CCoinCodeT) -> bool
{
    let now_ = application().now();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("rt_block_hash", &block_hash as &(dyn ToSql + Sync)),
        ("rt_doc_hash", &doc_hash as &(dyn ToSql + Sync)),
        ("rt_coin", &coin as &(dyn ToSql + Sync)),
        ("rt_insert_date", &now_ as &(dyn ToSql + Sync)),
    ]);
    dlog(
        &format!("Add a new rejected coin: {:?}", values),
        constants::Modules::Trx,
        constants::SecLevel::Warning);

    q_insert(
        C_TRX_REJECTED_TRANSACTIONS,
        &values,
        true);

    return true;
}
