use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::q_select;
use crate::lib::database::tables::{C_TRX_REJECTED_TRANSACTIONS, C_TRX_REJECTED_TRANSACTIONS_FIELDS};

//old_name_was searchInRejectedTrx
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

/*

bool RejectedTransactionsHandler::addTransaction(
  const CBlockHashT& block_hash,
  const CDocHashT& doc_hash,
  const CCoinCodeT& coin)
{
  QVDicT values {
    {"rt_block_hash", block_hash},
    {"rt_doc_hash", doc_hash},
    {"rt_coin", coin},
    {"rt_insert_date", cutils::getNow()}};

  CLog::log("Add a new rejected coin: " + cutils::dumpIt(values), "trx", "warning");

  DbModel::insert(
    stbl_trx_rejected_transactions,
    values);

  return true;
}

*/