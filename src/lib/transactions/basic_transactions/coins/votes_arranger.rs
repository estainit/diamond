use std::collections::HashMap;
use crate::{application, constants, cutils, dlog};
use crate::lib::custom_types::{CAddressT, CCoinCodeT, CDateT, CDocHashT, TimeBySecT, VString};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{CoinImportDataContainer, CoinVoterDocInfo, CoinVoterInfo, SpenderBlockStatedCreationDate};

//old_name_was doGroupByCoinAndVoter
pub fn do_group_by_coin_and_voter(
    block_inspect_container: &mut CoinImportDataContainer,
    invoked_doc_hash: &CDocHashT,
    _do_db_log: bool)
{
    let mut coins_and_voters_dict: HashMap<CCoinCodeT, HashMap<CAddressT, CoinVoterInfo>> = HashMap::new();

    // preparing coinsAndVotersDict
    let mut coin_spender_docs: VString = vec![]; // will be used to recognize colned transactions(if exist)
    for a_raw_vote in &block_inspect_container.m_raw_votes
    {
        let a_coin = &a_raw_vote.m_voting_coin;

        // integrate coin spends
        coin_spender_docs.push(a_raw_vote.m_spender_doc.clone());

        // group by coins
        if !coins_and_voters_dict.contains_key(a_coin)
        {
            coins_and_voters_dict.insert(a_coin.clone(), HashMap::new());
        }

        // group by voters
        let voter: CAddressT = a_raw_vote.m_voter.clone();
        if !coins_and_voters_dict[a_coin].contains_key(&voter)
        {
            // will be used to recognize if the 2 usage of coins have less than 6 hours different or not
            let mut tmp1 = coins_and_voters_dict[a_coin].clone();
            tmp1.insert(voter.clone(), CoinVoterInfo::new());
            coins_and_voters_dict.insert(a_coin.clone(), tmp1);
        }
        let spend_key = format!("{},{}", a_raw_vote.m_spend_date, a_raw_vote.m_spender_doc);

        // Spender Block's Stated Creation Date Structure
        let mut tmp1 = coins_and_voters_dict[a_coin].clone();
        let mut tmp2 = tmp1[&voter].clone();
        tmp2.m_spend_orders_info_by_spender_block_stated_creation_date
            .insert(
                spend_key,
                SpenderBlockStatedCreationDate {
                    m_spend_date: a_raw_vote.m_spend_date.clone(),
                    m_spend_doc: a_raw_vote.m_spender_doc.clone(),
                });
        tmp1.insert(voter.clone(), tmp2);
        coins_and_voters_dict.insert(a_coin.clone(), tmp1);

        // * the time between voting time and now, more old more trusty
        // * older than 12 hours definitely approved vote
        let mut tmp1 = coins_and_voters_dict[a_coin].clone();
        let mut tmp2 = tmp1[&voter].clone();
        tmp2.m_docs_info.insert(a_raw_vote.m_spender_doc.clone(), CoinVoterDocInfo {
            m_coin_spend_date: a_raw_vote.m_spend_date.clone(),
            m_spend_receive_order: a_raw_vote.m_receive_order,// order of receiving spends for same coin
            m_voter_percentage: a_raw_vote.m_voter_percentage,
            m_vote_date: a_raw_vote.m_vote_date.clone(),
        });
        tmp1.insert(voter.clone(), tmp2);
        coins_and_voters_dict.insert(a_coin.clone(), tmp1);

        let gap_inx_seg1 = cutils::padding_left(&a_raw_vote.m_receive_order.to_string(), 5);

        // * it is totally possible when the node creates votes(because of spending same coin in different times),
        // * hasn't the total spends in history, so creates a partially ordered votes.
        // * and did not send the vote about early usages of coin and only votes for 2 recently usages.
        // * say one years a go the coin C was spent in Trx1, and 11 month ago cheater again tried to spend it in Trx2
        // * and now again want to spend in Trx3.
        // * since machine has not Trx1 in history will say Trx2 is valid while it is not.
        // * to cover this issue here we add a second index (vote date).
        // * since the vote table NEVER truncated, we have entire spends(if they are case of double-spend)
        // * and related double-spends attempts history
        let gap_inx_seg2: CDateT = a_raw_vote.m_vote_date.clone();
        let gap_inx = format!("{},{}", gap_inx_seg1, gap_inx_seg2);
        let mut tmp1 = coins_and_voters_dict[a_coin].clone();
        let mut tmp2 = tmp1[&voter].clone();
        tmp2.m_receive_orders.insert(gap_inx.clone(), a_raw_vote.m_spender_doc.clone());
        tmp1.insert(voter.clone(), tmp2);
        coins_and_voters_dict.insert(a_coin.clone(), tmp1);
    }
    // let coins = coinsAndVotersDict.keys().join("");
    dlog(
        &format!(
            "Coins And Voters Dict for doc({}): coinsAndVotersDict:{:?}",
            cutils::hash8c(invoked_doc_hash),
            coins_and_voters_dict),
        constants::Modules::Trx,
        constants::SecLevel::Info);

//  if do_db_log
//    this._super.logSus({
//        lkey: '2.coinsAndVotersDict',
//        blockHash: '-',
//        docHash: invokedDocHash,
//        coins,
//        logBody: utils.stringify(coinsAndVotersDict)
//    });

    coin_spender_docs = cutils::array_unique(&coin_spender_docs);
    if (coin_spender_docs.len() == 1) && (coin_spender_docs[0] == *invoked_doc_hash)
    {
        // all voters for all coins are agree, that all coins are used in same document with same hash
        // so it is a cloned transaction
        let mut tmp = block_inspect_container.m_transactions_validity_check[invoked_doc_hash].clone();
        tmp.m_coins_and_voters_dict = coins_and_voters_dict;
        tmp.m_cloned = "Cloned".to_string();
        tmp.m_valid = true;
        block_inspect_container.m_transactions_validity_check.insert(invoked_doc_hash.clone(), tmp);

//    if do_db_log
//        this._super.logSus({
//            lkey: '3.cloned response',
//            blockHash: '-',
//            docHash: invokedDocHash,
//            coins,
//            logBody: utils.stringify(response)
//        });

        return;
    }

    // control if voter belives the coins are spended in less than 6 hours
    let full_cycle: TimeBySecT = application().get_cycle_by_seconds();
    let half_cycle: TimeBySecT = full_cycle / 2;

    let coin_keys = coins_and_voters_dict.keys().cloned().collect::<VString>();
    for a_coin in coin_keys
    {
        let voter_keys = coins_and_voters_dict[&a_coin].keys().cloned().collect::<VString>();
        for a_voter in voter_keys
        {
            let mut spend_keys = coins_and_voters_dict[&a_coin][&a_voter]
                .m_spend_orders_info_by_spender_block_stated_creation_date
                .keys()
                .cloned()
                .collect::<VString>();
            spend_keys.sort();

            // control if spend-keys are for one unique doc or different docs
            let mut key_group: HashMap<CDocHashT, u64> = HashMap::new();
            for an_spend_key in &spend_keys
            {
                let spender_doc =
                    &coins_and_voters_dict[&a_coin][&a_voter]
                        .m_spend_orders_info_by_spender_block_stated_creation_date[an_spend_key]
                        .m_spend_doc;
                if !key_group.contains_key(spender_doc)
                {
                    key_group.insert(spender_doc.clone(), 0);
                }
                key_group.insert(spender_doc.clone(), key_group[spender_doc] + 1);
            }

//      coinsAndVotersDict[aCoin][aVoter].m_spendTimes = cutils::arrayUnique(coinsAndVotersDict[aCoin][aVoter].m_spendTimes);
//      coinsAndVotersDict[aCoin][aVoter].m_spendTimes.sort();

//      VString spendTimes = coinsAndVotersDict[aCoin][aVoter].m_spendOIBSBSCD.keys();
//      spendTimes = cutils::arrayUnique(spendTimes);
//      spendTimes.sort();

            let mut tmp1 = coins_and_voters_dict[&a_coin].clone();
            let mut tmp2 = tmp1[&a_voter].clone();
            tmp2.m_spends_less_than_6_hour_new = false;
            tmp2.m_spends_less_than_6_hours = false;
            tmp1.insert(a_voter.clone(), tmp2);
            coins_and_voters_dict.insert(a_coin.clone(), tmp1);

            if key_group.keys().len() == 1
            {
                // there is only on spender document, so it is not the case of double-spending
            } else if key_group.keys().len() > 1
            {
                let mut can_control_less_6_condition = true;
                for a_key in key_group.keys()
                {
                    if key_group[a_key] > 1
                    {
                        // * it means there is at least one group of docs(maybe cloned) in combine with the another group(s)
                        // * the conflicts can be simple double-spend doc or a clone of double-spend
                        can_control_less_6_condition = false;
                    }
                }
                // * it is possible existing 2 different groups of documents
                // * 1. a document group with majority of occurrence which are cloned docs
                // * 2. and another group of doc which are cheating double spend docs
                // * definitely both groups are issued by one entity who can sign the coin(s)
                // *
                // * cloned       cloned      double
                // * cloned       cloned      cloned
                // *    |         double      double
                // *    |            |           |
                // *    v            |           |
                // * double          v           v
                // * cloned       cloned      cloned
                // *
                let mut tmp1 = coins_and_voters_dict[&a_coin].clone();
                let mut tmp2 = tmp1[&a_voter].clone();
                tmp2.m_can_control_less_6_condition = can_control_less_6_condition;
                tmp2.m_spends_less_than_6_hour_new = false;
                tmp1.insert(a_voter.clone(), tmp2);
                coins_and_voters_dict.insert(a_coin.clone(), tmp1);

                if can_control_less_6_condition
                {
                    let first_spend_by_signer_claim = &coins_and_voters_dict[&a_coin][&a_voter].m_spend_orders_info_by_spender_block_stated_creation_date[&spend_keys[0]];
                    let second_spend_by_signer_claim = &coins_and_voters_dict[&a_coin][&a_voter].m_spend_orders_info_by_spender_block_stated_creation_date[&spend_keys[1]];
                    let mut receive_order_by_machine_pov = coins_and_voters_dict[&a_coin][&a_voter].m_receive_orders.keys().cloned().collect::<VString>();
                    receive_order_by_machine_pov.sort();
                    // it is second doc by machine's POV
                    let second_spend_by_machine_pov = &coins_and_voters_dict[&a_coin][&a_voter].m_receive_orders[&receive_order_by_machine_pov[1]];
                    if application().time_diff(
                        first_spend_by_signer_claim.m_spend_date.clone(),
                        coins_and_voters_dict[&a_coin][&a_voter].m_docs_info[second_spend_by_machine_pov].m_vote_date.clone(),
                    ).as_seconds < full_cycle
                    {
                        // * if voter received a Trx before of 12 hours after stated spend time,
                        // * she can vote on it.
                        // * after 12 hours of spending a coin any claim is unacceptable

                        let mut tmp1 = coins_and_voters_dict[&a_coin].clone();
                        let mut tmp2 = tmp1[&a_voter].clone();
                        tmp2.m_spends_less_than_6_hours =
                            application().time_diff(
                                first_spend_by_signer_claim.m_spend_date.clone(),
                                second_spend_by_signer_claim.m_spend_date.clone()).as_seconds < half_cycle;
                        tmp1.insert(a_voter.clone(), tmp2);
                        coins_and_voters_dict.insert(a_coin.clone(), tmp1);
                    }


                    for inx in 1..spend_keys.len()
                    {
                        if coins_and_voters_dict[&a_coin][&a_voter]
                            .m_spend_orders_info_by_spender_block_stated_creation_date[&spend_keys[inx - 1]]
                            .m_spend_doc
                            !=
                            coins_and_voters_dict[&a_coin][&a_voter]
                                .m_spend_orders_info_by_spender_block_stated_creation_date[&spend_keys[inx]]
                                .m_spend_doc
                        {
                            // * they are not 2 cloned doc in sequence!
                            // * so must control time diff
                            if application().time_diff(
                                coins_and_voters_dict[&a_coin][&a_voter].m_spend_orders_info_by_spender_block_stated_creation_date[&spend_keys[inx - 1]].m_spend_date.clone(),
                                coins_and_voters_dict[&a_coin][&a_voter].m_spend_orders_info_by_spender_block_stated_creation_date[&spend_keys[inx]].m_spend_date.clone(),
                            ).as_seconds < half_cycle
                            {
                                // * two different docs are created in less than half a cylce time diff
                                // * so
                                let mut tmp1 = coins_and_voters_dict[&a_coin].clone();
                                let mut tmp2 = tmp1[&a_voter].clone();
                                tmp2.m_spends_less_than_6_hour_new = true;
                                tmp1.insert(a_voter.clone(), tmp2);
                                coins_and_voters_dict.insert(a_coin.clone(), tmp1);
                            }
                        }
                    }
                }
            } else {
                //
            }
        }
    }

    let mut tmp = block_inspect_container.m_transactions_validity_check[invoked_doc_hash].clone();
    tmp.m_coins_and_voters_dict = coins_and_voters_dict;
    block_inspect_container.m_transactions_validity_check.insert(invoked_doc_hash.clone(), tmp);

    return;
}


// sample coinsAndVotersDict:
//
// {
//     "ac36d47b7244bd6a6928686ad37b87c24d2f10329ea7eb430615cafc23976a55:0": {
//         "im1xpjkywf48yckgepcvdnrgdrx8qurgdeevf3kyenyv9snvve5v5ung9axujl": {
//             "spendOIBSBSCD": {
//                 "2019-04-06 18:36:33:305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61": {
//                     "spdDate": "2019-04-06 18:36:33",
//                     "spdDoc": "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61"
//                 },
//                 "2019-04-06 19:16:58:0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1": {
//                     "spdDate": "2019-04-06 19:16:58",
//                     "spdDoc": "0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1"
//                 }
//             },
//             "docsInfo": {
//                 "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61": {
//                     "coinSpendDate": "2019-04-06 18:36:33",
//                     "rOrder": 0,
//                     "voterPercentage": 99.9700089973
//                 },
//                 "0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1": {
//                     "coinSpendDate": "2019-04-06 19:16:58",
//                     "rOrder": 1,
//                     "voterPercentage": 99.9700089973
//                 }
//             },
//             "rOrders": {
//                 "0000:2019-04-06 19:17:15": "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61",
//                 "0001:2019-04-06 19:17:15": "0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1"
//             },
//             "spendTimes": [
//                 "2019-04-06 18:36:33",
//                 "2019-04-06 19:16:58"
//             ],
//             "firstSpenderInfo": {
//                 "spDoc": "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61",
//                 "spTime": "2019-04-06 18:36:33"
//             },
//             "spendsLessThan6HNew": false,
//             "spendsLessThan6H": false
//         },
//
//         "im1xqukxefe8p3xxwpk89jx2wfjvdjkycfhvscrsde4xymnxvrrxy6xxjljrvp": {
//             "spendOIBSBSCD": {
//                 "2019-04-06 18:36:33:305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61": {
//                     "spdDate": "2019-04-06 18:36:33",
//                     "spdDoc": "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61"
//                 },
//                 "2019-04-06 19:16:58:0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1": {
//                     "spdDate": "2019-04-06 19:16:58",
//                     "spdDoc": "0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1"
//                 }
//             },
//             "docsInfo": {
//                 "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61": {
//                     "coinSpendDate": "2019-04-06 18:36:33",
//                     "rOrder": 0,
//                     "voterPercentage": 0.02999100269
//                 },
//                 "0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1": {
//                     "coinSpendDate": "2019-04-06 19:16:58",
//                     "rOrder": 1,
//                     "voterPercentage": 0.02999100269
//                 }
//             },
//             "rOrders": {
//                 "0000:2019-04-06 19:17:09": "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61",
//                 "0001:2019-04-06 19:17:09": "0c8a0f651a80a5e06d44ff5e2c7a71fe4990997e4d2fb23bce68af94446a8fd1"
//             },
//             "spendTimes": [
//                 "2019-04-06 18:36:33",
//                 "2019-04-06 19:16:58"
//             ],
//             "firstSpenderInfo": {
//                 "spDoc": "305e32fda515fa1e49f06fb00b1df26703e621c18e8fa60e71e65258486bca61",
//                 "spTime": "2019-04-06 18:36:33"
//             },
//             "spendsLessThan6HNew": false,
//             "spendsLessThan6H": false
//         }
//     }
// }