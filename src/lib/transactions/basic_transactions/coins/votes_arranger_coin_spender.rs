use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::lib::custom_types::{CAddressT, CCoinCodeT, CDocHashT, VString};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{CoinImportDataContainer, CoinOrderedSpender, CoinVoterInfo, VoteData};

// * here we re-orginizing votes based on transactions.
// * that is, making 2 dimensional array of coin and transaction-order and put in this member the transaction shole scores
//old_name_was doGroupByCoinAndSpender
pub fn do_group_by_coin_and_spender(
    block_inspect_container: &mut CoinImportDataContainer,
    invoked_doc_hash: &String,
    _do_db_log: bool)
{
    let coins_and_voters_dict: HashMap<CCoinCodeT, HashMap<CAddressT, CoinVoterInfo>> =
        block_inspect_container
            .m_transactions_validity_check[invoked_doc_hash]
            .m_coins_and_voters_dict
            .clone();

    let mut coins_and_ordered_spenders_dict: HashMap<CCoinCodeT, HashMap<u32, HashMap<CDocHashT, CoinOrderedSpender>>> = HashMap::new();
    // let mut coins = coins_and_voters_dict.keys().cloned().collect::<VString>();
    for a_coin in coins_and_voters_dict.keys()
    {
        if !coins_and_ordered_spenders_dict.contains_key(a_coin)
        {
            coins_and_ordered_spenders_dict.insert(a_coin.clone(), HashMap::new());// = HashMap < u32, HashMap < CDocHashT, CoinOrderedSpender > > {};
        }
        for voter in coins_and_voters_dict[a_coin].keys()
        {
            let a_coin_voter = &coins_and_voters_dict[a_coin][voter];

            let mut ordered_keys = a_coin_voter.m_receive_orders.keys().cloned().collect::<VString>();
            ordered_keys.sort();

            let mut considered_docs: VString = vec![];
            for ord_index in 0..ordered_keys.len()
            {
                let ord_index_u32 = ord_index as u32;
                let doc_hash = &a_coin_voter.m_receive_orders[&ordered_keys[ord_index]];

                if considered_docs.contains(doc_hash)
                { continue; }

                considered_docs.push(doc_hash.clone());

                let the_doc = &a_coin_voter.m_docs_info[doc_hash];

                if !coins_and_ordered_spenders_dict[a_coin].contains_key(&ord_index_u32)
                {
                    let mut tmp = coins_and_ordered_spenders_dict[a_coin].clone();
                    tmp.insert(ord_index_u32, HashMap::new());
                    coins_and_ordered_spenders_dict.insert(a_coin.clone(), tmp);
                }
                if !coins_and_ordered_spenders_dict[a_coin][&ord_index_u32].contains_key(doc_hash)
                {
                    let mut tmp1 = coins_and_ordered_spenders_dict[a_coin].clone();
                    let mut tmp2 = tmp1[&ord_index_u32].clone();
                    // coins_and_ordered_spenders_dict[a_coin][&ord_index_u32].insert(doc_hash.clone(), CoinOrderedSpender {
                    tmp2.insert(doc_hash.clone(), CoinOrderedSpender {
                        m_vote_data: VoteData {
                            m_doc_hash: doc_hash.clone(),
                            m_inside_6_voters_count: 0,
                            m_inside_6_votes_gain: 0.0,
                            m_outside_6_voters_count: 0,
                            m_outside_6_votes_gain: 0.0,
                            m_vote_gain: 0.0,
                            m_details: Default::default(),
                        },
                        m_docs: vec![],
                    });
                    tmp1.insert(ord_index_u32.clone(), tmp2);
                    coins_and_ordered_spenders_dict.insert(a_coin.clone(), tmp1);
                }
                // let votePower = utils.calcLog(theDoc.gapTime, (iConsts.getCycleBySeconds() * 2)).gain;
                // if (votePower == 0)
                //     votePower = utils.iFloorFloat(iConsts.MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER);
                let vote_power_f64 = 1.0;  // TODO improve votePower
                let vote_gain = cutils::i_floor_float(the_doc.m_voter_percentage * vote_power_f64);

                // coins_and_ordered_spenders_dict[a_coin][&ord_index_u32][doc_hash].m_vote_data.m_vote_gain += vote_gain;
                let mut tmp1 = coins_and_ordered_spenders_dict[a_coin].clone();
                let mut tmp2 = tmp1[&ord_index_u32].clone();
                let mut tmp3 = tmp2[doc_hash].clone();
                tmp3.m_vote_data.m_vote_gain += vote_gain;
                tmp2.insert(doc_hash.clone(), tmp3);
                tmp1.insert(ord_index_u32.clone(), tmp2);
                coins_and_ordered_spenders_dict.insert(a_coin.clone(), tmp1);

                if a_coin_voter.m_spends_less_than_6_hours
                { //spendsLessThan6HNew
                    // coins_and_ordered_spenders_dict[a_coin][&ord_index_u32][doc_hash].m_vote_data.m_inside_6_voters_count += 1;
                    // coins_and_ordered_spenders_dict[a_coin][&ord_index_u32][doc_hash].m_vote_data.m_inside_6_votes_gain += vote_gain;
                    let mut tmp1 = coins_and_ordered_spenders_dict[a_coin].clone();
                    let mut tmp2 = tmp1[&ord_index_u32].clone();
                    let mut tmp3 = tmp2[doc_hash].clone();
                    tmp3.m_vote_data.m_inside_6_voters_count += 1;
                    tmp3.m_vote_data.m_inside_6_votes_gain += vote_gain;
                    tmp2.insert(doc_hash.clone(), tmp3);
                    tmp1.insert(ord_index_u32.clone(), tmp2);
                    coins_and_ordered_spenders_dict.insert(a_coin.clone(), tmp1);
                } else {
                    let mut tmp1 = coins_and_ordered_spenders_dict[a_coin].clone();
                    let mut tmp2 = tmp1[&ord_index_u32].clone();
                    let mut tmp3 = tmp2[doc_hash].clone();
                    tmp3.m_vote_data.m_outside_6_voters_count += 1;
                    tmp3.m_vote_data.m_outside_6_votes_gain += vote_gain;
                    tmp2.insert(doc_hash.clone(), tmp3);
                    tmp1.insert(ord_index_u32.clone(), tmp2);
                    coins_and_ordered_spenders_dict.insert(a_coin.clone(), tmp1);
                }

                // coins_and_ordered_spenders_dict[a_coin][&ord_index_u32][doc_hash].m_vote_data.m_details[voter] = the_doc.clone();
                // coins_and_ordered_spenders_dict[a_coin][&ord_index_u32][doc_hash].m_docs.push(the_doc.clone());
                let mut tmp1 = coins_and_ordered_spenders_dict[a_coin].clone();
                let mut tmp2 = tmp1[&ord_index_u32].clone();
                let mut tmp3 = tmp2[doc_hash].clone();
                tmp3.m_docs.push(the_doc.clone());
                tmp3.m_vote_data.m_details.insert(voter.clone(), the_doc.clone());
                tmp2.insert(doc_hash.clone(), tmp3);
                tmp1.insert(ord_index_u32.clone(), tmp2);
                coins_and_ordered_spenders_dict.insert(a_coin.clone(), tmp1);
            }
        }
    }

    dlog(
        &format!(
            "Coins And Spenders Dict for doc({}) coins And Ordered Spenders Dict: {:?}",
            cutils::hash8c(invoked_doc_hash),
            coins_and_ordered_spenders_dict
        ),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

//  if (do_db_log)
//    this._super.logSus({
//        lkey: '3.coinsAndOrderedSpendersDict',
//        blockHash: '-',
//        docHash: invokedDocHash,
//        coins,
//        logBody: utils.stringify(coinsAndOrderedSpendersDict)
//    });

    let mut tmp = block_inspect_container.m_transactions_validity_check[invoked_doc_hash].clone();
    tmp.m_coins_and_ordered_spenders_dict = coins_and_ordered_spenders_dict;
    block_inspect_container.m_transactions_validity_check.insert(invoked_doc_hash.clone(), tmp);
    return;
}


// coinsAndOrderedSpendersDict sample: {
//     "e45630b91c0df1323e8a861464e9fdb8deb705c523cb977bca09d58c5409ef5c:0": [
//
//         first position
//         {
//             "ba35317697b2836c890faf5dc8b2046d46b9d637ebcd3c37144babb501b117dd": {
//                 "voteData": {
//                     "docHash": "ba35317697b2836c890faf5dc8b2046d46b9d637ebcd3c37144babb501b117dd",
//                     "inside6VotersCount": 0,
//                     "inside6VotesGain": 0,
//                     "outside6VotersCount": 2,
//                     "outside6VotesGain": 99.99999999999,
//                     "voteGain": 99.99999999999,
//                     "details": {
//                         "im1xpjkywf48yckgepcvdnrgdrx8qurgdeevf3kyenyv9snvve5v5ung9axujl": {
//                             "creationDate": "2019-04-01 22:22:36",
//                             "rOrder": 0,
//                             "shares": 99.98680174217,
//                             "gapTime": 1723
//                         },
//                         "im1xpsnyd3k8q6kvvrzxq6xyctyxp3xxc3cxdnryep5xg6rgvehvgcx2fu6vs2": {
//                             "creationDate": "2019-04-01 22:22:36",
//                             "rOrder": 0,
//                             "shares": 0.01319825782,
//                             "gapTime": 1730
//                         }
//                     }
//                 },
//                 "docs": [
//                     {
//                         "creationDate": "2019-04-01 22:22:36",
//                         "rOrder": 0,
//                         "shares": 99.98680174217,
//                         "gapTime": 1723
//                     },
//                     {
//                         "creationDate": "2019-04-01 22:22:36",
//                         "rOrder": 0,
//                         "shares": 0.01319825782,
//                         "gapTime": 1730
//                     }
//                 ]
//             }
//         },
//
//         second position
//         {
//             "8830e21bfa191b477847587adcc2d4ba1ed1c9339205b6afe0b693c38f611acb": {
//                 "voteData": {
//                     "docHash": "8830e21bfa191b477847587adcc2d4ba1ed1c9339205b6afe0b693c38f611acb",
//                     "inside6VotersCount": 0,
//                     "inside6VotesGain": 0,
//                     "outside6VotersCount": 2,
//                     "outside6VotesGain": 99.99999999999,
//                     "voteGain": 99.99999999999,
//                     "details": {
//                         "im1xpjkywf48yckgepcvdnrgdrx8qurgdeevf3kyenyv9snvve5v5ung9axujl": {
//                             "creationDate": "2019-04-01 22:50:58",
//                             "rOrder": 1,
//                             "shares": 99.98680174217,
//                             "gapTime": 21
//                         },
//                         "im1xpsnyd3k8q6kvvrzxq6xyctyxp3xxc3cxdnryep5xg6rgvehvgcx2fu6vs2": {
//                             "creationDate": "2019-04-01 22:50:58",
//                             "rOrder": 1,
//                             "shares": 0.01319825782,
//                             "gapTime": 28
//                         }
//                     }
//                 },
//                 "docs": [
//                     {
//                         "creationDate": "2019-04-01 22:50:58",
//                         "rOrder": 1,
//                         "shares": 99.98680174217,
//                         "gapTime": 21
//                     },
//                     {
//                         "creationDate": "2019-04-01 22:50:58",
//                         "rOrder": 1,
//                         "shares": 0.01319825782,
//                         "gapTime": 28
//                     }
//                 ]
//             }
//         }
//     ]
// }