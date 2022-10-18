use std::collections::HashMap;
use crate::{ccrypto, constants, cutils, dlog};
use crate::lib::custom_types::{CCoinCodeT, CDocHashT, VString};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{CoinAndPosition, CoinImportDataContainer, CoinOrderedSpender, InOutside6hElm, VoteData};

//old_name_was doGroupByCoinAndPosition
pub fn do_group_by_coin_and_position(
    block_inspect_container: &mut CoinImportDataContainer,
    invoked_doc_hash: &CDocHashT,
    _do_db_log: bool)
{
    let coins_and_ordered_spenders_dict: HashMap<CCoinCodeT, HashMap<u32, HashMap<CDocHashT, CoinOrderedSpender>>> =
        block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_coins_and_ordered_spenders_dict.clone();
    let mut coins_and_positions_dict: HashMap<CCoinCodeT, HashMap<u32, CoinAndPosition>> = HashMap::new();
    let coins = coins_and_ordered_spenders_dict.keys().cloned().collect::<VString>();
    for a_coin in coins
    {
        if !coins_and_positions_dict.contains_key(&a_coin)
        {
            coins_and_positions_dict.insert(a_coin.clone(), HashMap::new());// HashMap < u32, CoinAndPosition > {};
        }
        for position in 0..coins_and_ordered_spenders_dict[&a_coin].len()
        {
            let pos_u32 = position as u32;
            if !coins_and_positions_dict[&a_coin].contains_key(&pos_u32)
            {
                let mut tmp = coins_and_positions_dict[&a_coin].clone();
                tmp.insert(position as u32, CoinAndPosition::new());
                coins_and_positions_dict.insert(a_coin.clone(), tmp);
            }
            // integrating votes of all documents in this position
            let documents_in_this_position: HashMap<CDocHashT, CoinOrderedSpender> =
                coins_and_ordered_spenders_dict[&a_coin][&pos_u32].clone();
            for doc_hash in documents_in_this_position.keys()
            {
                let vote_data: &VoteData = &coins_and_ordered_spenders_dict[&a_coin][&pos_u32][doc_hash].m_vote_data;
                if vote_data.m_outside_6_votes_gain > vote_data.m_inside_6_votes_gain
                {
                    let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                    let mut tmp2 = tmp1[&pos_u32].clone();
                    tmp2.m_outside_total += vote_data.m_outside_6_votes_gain;
                    tmp1.insert(pos_u32, tmp2);
                    coins_and_positions_dict.insert(a_coin.clone(), tmp1);

                    let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                    let mut tmp2 = tmp1[&pos_u32].clone();
                    tmp2.m_outside_6_hours.push(InOutside6hElm {
                        m_doc_hash: doc_hash.to_string(),
                        m_votes: vote_data.m_outside_6_votes_gain,
                        m_voters: vote_data.m_outside_6_voters_count,
                    });
                    tmp1.insert(pos_u32, tmp2);
                    coins_and_positions_dict.insert(a_coin.clone(), tmp1);
                } else if vote_data.m_outside_6_votes_gain < vote_data.m_inside_6_votes_gain
                {
                    let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                    let mut tmp2 = tmp1[&pos_u32].clone();
                    tmp2.m_inside_total += vote_data.m_inside_6_votes_gain;
                    tmp1.insert(pos_u32, tmp2);
                    coins_and_positions_dict.insert(a_coin.clone(), tmp1);

                    let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                    let mut tmp2 = tmp1[&pos_u32].clone();
                    tmp2.m_inside_6_hours.push(InOutside6hElm {
                        m_doc_hash: doc_hash.to_string(),
                        m_votes: vote_data.m_inside_6_votes_gain,
                        m_voters: vote_data.m_inside_6_voters_count,
                    });
                    tmp1.insert(pos_u32, tmp2);
                    coins_and_positions_dict.insert(a_coin.clone(), tmp1);
                } else if vote_data.m_outside_6_votes_gain == vote_data.m_inside_6_votes_gain
                {
                    if vote_data.m_outside_6_voters_count > vote_data.m_inside_6_voters_count
                    {
                        let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                        let mut tmp2 = tmp1[&pos_u32].clone();
                        tmp2.m_outside_total += vote_data.m_outside_6_votes_gain;
                        tmp1.insert(pos_u32, tmp2);
                        coins_and_positions_dict.insert(a_coin.clone(), tmp1);

                        let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                        let mut tmp2 = tmp1[&pos_u32].clone();
                        tmp2.m_outside_6_hours.push(InOutside6hElm {
                            m_doc_hash: doc_hash.to_string(),
                            m_votes: vote_data.m_outside_6_votes_gain,
                            m_voters: vote_data.m_outside_6_voters_count,
                        });
                        tmp1.insert(pos_u32, tmp2);
                        coins_and_positions_dict.insert(a_coin.clone(), tmp1);
                    } else if vote_data.m_outside_6_voters_count < vote_data.m_inside_6_voters_count
                    {
                        let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                        let mut tmp2 = tmp1[&pos_u32].clone();
                        tmp2.m_inside_total += vote_data.m_inside_6_votes_gain;
                        tmp1.insert(pos_u32, tmp2);
                        coins_and_positions_dict.insert(a_coin.clone(), tmp1);

                        let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                        let mut tmp2 = tmp1[&pos_u32].clone();
                        tmp2.m_inside_6_hours.push(InOutside6hElm {
                            m_doc_hash: doc_hash.to_string(),
                            m_votes: vote_data.m_inside_6_votes_gain,
                            m_voters: vote_data.m_inside_6_voters_count,
                        });
                        tmp1.insert(pos_u32, tmp2);
                        coins_and_positions_dict.insert(a_coin.clone(), tmp1);
                    } else if vote_data.m_outside_6_voters_count == vote_data.m_inside_6_voters_count
                    {
                        let out_hash = ccrypto::keccak256(
                            &format!("{}:{}", vote_data.m_outside_6_voters_count, vote_data.m_outside_6_votes_gain)
                        );

                        let in_hash = ccrypto::keccak256(
                            &format!("{}:{}", vote_data.m_inside_6_voters_count, vote_data.m_inside_6_votes_gain)
                        );

                        if out_hash > in_hash
                        {
                            let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                            let mut tmp2 = tmp1[&pos_u32].clone();
                            tmp2.m_outside_total += vote_data.m_outside_6_votes_gain;
                            tmp1.insert(pos_u32, tmp2);
                            coins_and_positions_dict.insert(a_coin.clone(), tmp1);

                            let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                            let mut tmp2 = tmp1[&pos_u32].clone();
                            tmp2.m_outside_6_hours.push(InOutside6hElm {
                                m_doc_hash: doc_hash.to_string(),
                                m_votes: vote_data.m_outside_6_votes_gain,
                                m_voters: vote_data.m_outside_6_voters_count,
                            });
                            tmp1.insert(pos_u32, tmp2);
                            coins_and_positions_dict.insert(a_coin.clone(), tmp1);
                        } else {
                            let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                            let mut tmp2 = tmp1[&pos_u32].clone();
                            tmp2.m_inside_total += vote_data.m_inside_6_votes_gain;
                            tmp1.insert(pos_u32, tmp2);
                            coins_and_positions_dict.insert(a_coin.clone(), tmp1);

                            let mut tmp1 = coins_and_positions_dict[&a_coin].clone();
                            let mut tmp2 = tmp1[&pos_u32].clone();
                            tmp2.m_inside_6_hours.push(InOutside6hElm {
                                m_doc_hash: doc_hash.to_string(),
                                m_votes: vote_data.m_inside_6_votes_gain,
                                m_voters: vote_data.m_inside_6_voters_count,
                            });
                            tmp1.insert(pos_u32, tmp2);
                            coins_and_positions_dict.insert(a_coin.clone(), tmp1);
                        }
                    }
                }
            }
        }
    }

    dlog(
        &format!(
            "Coins-And-Positions-Dict for doc({}): {:?}",
            cutils::hash8c(invoked_doc_hash),
            coins_and_positions_dict
        ),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);


//  if (do_db_log)
//    this._super.logSus({
//        lkey: '4.coinsAndPositionsDict',
//        blockHash: '-',
//        docHash: invokedDocHash,
//        coins,
//        logBody: utils.stringify(coinsAndPositionsDict)
//    });

    let mut tmp = block_inspect_container.m_transactions_validity_check[invoked_doc_hash].clone();
    tmp.m_coins_and_positions_dict = coins_and_positions_dict;
    block_inspect_container.m_transactions_validity_check.insert(invoked_doc_hash.clone(), tmp);

    return;
}


// coinsAndPositionsDict sample:
// {
//     "e45630b91c0df1323e8a861464e9fdb8deb705c523cb977bca09d58c5409ef5c:0": {
//         "0": {
//             "inside6h": [],
//             "outside6h": [
//                 {
//                     "docHash": "ba35317697b2836c890faf5dc8b2046d46b9d637ebcd3c37144babb501b117dd",
//                     "votes": 99.99999999999,
//                     "voters": 2
//                 }
//             ],
//             "outsideTotal": 99.99999999999,
//             "insideTotal": 0
//         },
//         "1": {
//             "inside6h": [],
//             "outside6h": [
//                 {
//                     "docHash": "8830e21bfa191b477847587adcc2d4ba1ed1c9339205b6afe0b693c38f611acb",
//                     "votes": 99.99999999999,
//                     "voters": 2
//                 }
//             ],
//             "outsideTotal": 99.99999999999,
//             "insideTotal": 0
//         }
//     }
// }
//
// it means for the coin e45630:0 in position zero there is only one spender which is out of 6 hours
// and this spender is valid, while there is another spender in position 2 and difinetaly must be rejected
