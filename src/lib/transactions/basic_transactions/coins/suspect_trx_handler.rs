use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::lib::custom_types::{CAddressT, CBlockHashT, CDocHashT, SharesPercentT, VString};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{CoinImportDataContainer, RawVote, TheVote, TheVotes};
use crate::lib::database::abs_psql::q_custom_query;
use crate::lib::database::tables::{C_TRX_SUSPECT_TRANSACTIONS, C_TRX_SUSPECT_TRANSACTIONS_FIELDS};
use crate::lib::services::dna::dna_handler::get_an_address_shares;
use crate::lib::transactions::basic_transactions::coins::votes_arranger::do_group_by_coin_and_voter;
use crate::lib::transactions::basic_transactions::coins::votes_arranger_coin_position::do_group_by_coin_and_position;
use crate::lib::transactions::basic_transactions::coins::votes_arranger_coin_spender::do_group_by_coin_and_spender;
use crate::lib::transactions::basic_transactions::coins::votes_arranger_final::do_sus_vote_res;

//old_name_was getSusInfoByBlockHash
pub fn get_sus_info_by_block_hash(block_hash: &CDocHashT) -> (
    bool /* has_sus_records */,
    HashMap<CBlockHashT, TheVotes> /* votes_dict */
)
{
    let mut votes_dict: HashMap<CBlockHashT, TheVotes> = HashMap::new();
    // retrieve votes about ALL votes about all coins which are used in given block and are spent in other blocks too
    let complete_query: String = format!(
        "SELECT st_voter, st_coin, st_spender_block, st_spender_doc, st_vote_date FROM {} \
        WHERE st_coin IN (SELECT st_coin FROM {} WHERE st_spender_block='{}') ORDER BY st_voter, st_coin",
        C_TRX_SUSPECT_TRANSACTIONS,
        C_TRX_SUSPECT_TRANSACTIONS,
        block_hash);
    let (_status, votes) = q_custom_query(
        &complete_query,
        &vec![],
        true);
    if votes.len() == 0
    {
        dlog(
            &format!(
                "Transaction Didn't recognized as an suspicious case. block({})", cutils::hash8c(&block_hash)),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        return (false, votes_dict);
    }
    dlog(
        &format!(
            "Found sus trxes({} records) for block({})",
            votes.len(),
            cutils::hash8c(block_hash)),
        constants::Modules::Trx,
        constants::SecLevel::Warning);


    // control if they are cloned trx?


    // calculate already votes

    for vote in votes
    {
        let spender_block = vote["st_spender_block"].to_string();
        if !votes_dict.contains_key(&spender_block)
        {
            votes_dict.insert(spender_block.clone(), TheVotes { m_votes: vec![], m_sum_percent: 0.0 });
        }

        let (_share_count, mut shares_percent) = get_an_address_shares(
            &vote["st_voter"].to_string(),
            &vote["st_vote_date"].to_string()); // dna handler to calculate
        if shares_percent < constants::MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER
        {
            shares_percent = cutils::i_floor_float(constants::MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER);
        }

        let mut tmp = votes_dict[&spender_block].clone();
        tmp.m_votes.push(
            TheVote
            {
                m_voter: vote["st_voter"].to_string(),
                m_shares_percent: shares_percent,
            });
        votes_dict.insert(spender_block, tmp);
    }

    let v_keys = votes_dict.keys().cloned().collect::<VString>();
    for blk_hash in v_keys
    {
        let mut unique_voters: HashMap<CAddressT, SharesPercentT> = HashMap::new();
        for a_vote in &votes_dict[&blk_hash].m_votes
        {
            unique_voters.insert(a_vote.m_voter.clone(), a_vote.m_shares_percent);
        };
        let mut tmp = votes_dict[&blk_hash].clone();
        tmp.m_sum_percent = 0.0;
        for voter in unique_voters.keys()
        {
            tmp.m_sum_percent += unique_voters[voter];
        };
        tmp.m_sum_percent = cutils::i_floor_float(tmp.m_sum_percent);
        votes_dict.insert(blk_hash.clone(), tmp);
    };
    return (true, votes_dict); //cloneStatus
}

//old_name_was getSusInfoByDocHash
pub fn get_sus_info_by_doc_hash(invoked_doc_hash: &CDocHashT) ->
(bool /* all_Coins_Are_Valid */, Vec<RawVote> /* raw_votes */)
{

    // retrieve all trx which using same inputs of given trx
    let complete_query = format!(
        "SELECT {} FROM {} WHERE st_coin IN (SELECT st_coin FROM {} WHERE st_spender_doc='{}') \
      ORDER BY st_voter, st_receive_order",
        C_TRX_SUSPECT_TRANSACTIONS_FIELDS.join(", "),
        C_TRX_SUSPECT_TRANSACTIONS,
        C_TRX_SUSPECT_TRANSACTIONS,
        invoked_doc_hash
    );
    let (_status, raw_votes) = q_custom_query(
        &complete_query,
        &vec![],
        false);

    if raw_votes.len() == 0
    {
        dlog(
            &format!(
                "InvokedDocHash({}) didn't recognized as an suspicious doc!",
                cutils::hash8c(invoked_doc_hash)),
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (
            true,
            vec![]);
        // rawVotes: [],
        //  coinsAndVotersDict: {},
        //  coinsAndOrderedSpendersDict: {},
        //  coinsAndPositionsDict: {},
        //  susVoteRes: {}
    }

    let mut new_row_votes: Vec<RawVote> = vec![];
    for a_row in raw_votes
    {
        new_row_votes.push(RawVote::load_from_record(&a_row));
    }

    return (
        false,
        new_row_votes);
//  ,
//    coinsAndVotersDict: {},
//    coinsAndOrderedSpendersDict: {},
//    coinsAndPositionsDict: {},
//    susVoteRes: {}
//  };
}

//old_name_was retrieveVoterPercentages
pub fn retrieve_voter_percentages(raw_votes: &mut Vec<RawVote>) ->
(bool /* status */, Vec<RawVote> /* raw_votes */)
{
    // retrieve voter shares
    let mut tmp_voter_date_shares_dict: HashMap<String, f64> = HashMap::new();
    for inx in 0..raw_votes.len()
    {
        let key = format!(
            "{}:{}",
            raw_votes[inx].m_voter,
            raw_votes[inx].m_vote_date);  // TODO: optimaized it to use start date f period instead of analog date in range, in  order to reduce map table
        if !tmp_voter_date_shares_dict.contains_key(&key)
        {
            let (_shares, mut percentage) =
                get_an_address_shares(&raw_votes[inx].m_voter, &raw_votes[inx].m_vote_date);
            if percentage < constants::MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER
            {
                dlog(
                    &format!(
                        "Shares == 0 for ({}): percentage({}) ",
                        cutils::short_bech16(&raw_votes[inx].m_voter),
                        percentage
                    ),
                    constants::Modules::Trx,
                    constants::SecLevel::Info);

                percentage = constants::MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER;

                dlog(
                    &format!(
                        "Shares == 0 for ({}): percentage({}) ",
                        cutils::short_bech16(&raw_votes[inx].m_voter),
                        percentage
                    ),
                    constants::Modules::Trx,
                    constants::SecLevel::Info);
            }
            tmp_voter_date_shares_dict.insert(key.clone(), percentage);
        }
        let mut tmp = raw_votes[inx].clone();
        tmp.m_voter_percentage = tmp_voter_date_shares_dict[&key].clone();
        raw_votes.insert(inx, tmp);
    }

    return (false, raw_votes.clone());
}

//old_name_was checkDocValidity
pub fn check_doc_validity(
    block_inspect_container: &mut CoinImportDataContainer,
    invoked_doc_hash: &CDocHashT,
    do_db_log: bool,
    do_console_log: bool)
{
    dlog(
        &format!(
            "Trx suspect transactions for doc({})", cutils::hash8c(invoked_doc_hash)),
        constants::Modules::Trx,
        constants::SecLevel::Info);

//  if (doDBLog)
//    this.logSus({
//      lkey: '1.suspect transactions',
//      blockHash: '-',
//      docHash: invokedDocHash,
//      refLocs: '-',
//      logBody: utils.stringify(rawVotes)});


    do_group_by_coin_and_voter(
        block_inspect_container,
        invoked_doc_hash,
        do_db_log);
    if block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_valid
    { return; }

    if do_console_log
    {
        dlog(
            &format!(
                "Coins-And-Voters-Dict: {:?}",
                block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_coins_and_voters_dict),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }


    // preparing coinsAndOrderedSpendersDict
    do_group_by_coin_and_spender(
        block_inspect_container,
        invoked_doc_hash,
        do_db_log);
    if do_console_log
    {
        dlog(
            &format!(
                "Coins-And-Ordered-Spenders-Dict: {:?}",
                block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_coins_and_ordered_spenders_dict),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }


    // preparing coinsAndPositionsDict
    do_group_by_coin_and_position(
        block_inspect_container,
        invoked_doc_hash,
        do_db_log);
    if do_console_log
    {
        dlog(
            &format!(
                "Coins And Positions Dict: {:?}",
                block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_coins_and_positions_dict,
            ),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }


    do_sus_vote_res(
        block_inspect_container,
        invoked_doc_hash,
        do_db_log);
    if do_console_log
    {
        dlog(
            &format!(
                "Sus Vote final Res: {:?}", block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_sus_vote_res),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
    }
    return;

//  return { , cvRes.m_coinsAndVotersDict, coinsAndPositionsDict, };
}


/*

QVDRecordsT SuspectTrxHandler::searchInSusTransactions(
  const ClausesT& clauses,
  const StringList& fields,
  const OrderT& order,
  const int limit)
{
  QueryRes posts = DbModel::select(
    stbl_trx_suspect_transactions,
    fields,
    clauses,
    order,
    limit);

  return posts.records;
}


*/