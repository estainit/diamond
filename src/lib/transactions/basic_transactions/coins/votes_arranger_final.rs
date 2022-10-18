use std::collections::HashMap;
use crate::cutils;
use crate::lib::custom_types::{CCoinCodeT, VString};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{CoinAndPosition, CoinImportDataContainer, InOutside6hElm, SusVote};

//old_name_was doSusVoteRes
pub fn do_sus_vote_res(
    block_inspect_container: &mut CoinImportDataContainer,
    invoked_doc_hash: &String,
    _do_db_log: bool)
{
    let coins_and_positions_dict: HashMap<CCoinCodeT, HashMap<u32, CoinAndPosition>> =
        block_inspect_container.m_transactions_validity_check[invoked_doc_hash].m_coins_and_positions_dict.clone();

    let mut sus_vote_res: HashMap<CCoinCodeT, SusVote> = HashMap::new();

// preparing final response for invoked document, based on position 0 (the fist spender)
// responses for susvote
    let zero_u32: u32 = 0;
    let zero_usize: usize = 0;
    for a_coin in coins_and_positions_dict.keys()
    {
        if coins_and_positions_dict[a_coin][&zero_u32].m_outside_total <= coins_and_positions_dict[a_coin][&zero_u32].m_inside_total
        {
            sus_vote_res.insert(
                a_coin.clone(),
                SusVote {
                    m_valid: false,
                    m_action: "Donate".to_string(),
                    m_voters: coins_and_positions_dict[a_coin][&zero_u32].m_inside_6_hours[zero_usize].m_voters,
                    m_votes: coins_and_positions_dict[a_coin][&zero_u32].m_inside_6_hours[zero_usize].m_votes,
                });
        }


        if coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours.len() == 1
        {
            if coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours[zero_usize].m_doc_hash == *invoked_doc_hash
            {
                sus_vote_res.insert(
                    a_coin.clone(),
                    SusVote {
                        m_valid: true,
                        m_action: "-".to_string(),
                        m_voters: coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours[zero_usize].m_voters,
                        m_votes: coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours[zero_usize].m_votes,
                    });
            } else {
                sus_vote_res.insert(
                    a_coin.clone(),
                    SusVote {
                        m_valid: false,
                        m_action: "Reject".to_string(),
                        m_voters: coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours[zero_usize].m_voters,
                        m_votes: coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours[zero_usize].m_votes,
                    });
            }
        }

        if coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours.len() > 1
        {
            // in firt position, there are more than one docs with more than 6h gap time
            let mut vote_and_hash_dict: HashMap<String, InOutside6hElm> = HashMap::new();
            for an_outside in &coins_and_positions_dict[a_coin][&zero_u32].m_outside_6_hours
            {
                let the_key = format!(
                    "{}:{}:{}",
                    cutils::padding_left(&an_outside.m_votes.to_string(), 10),
                    cutils::padding_left(&an_outside.m_voters.to_string(), 10),
                    an_outside.m_doc_hash);
                vote_and_hash_dict.insert(the_key, an_outside.clone());
            }
            let mut votes = vote_and_hash_dict.keys().cloned().collect::<VString>();
            if votes.len() > 0
            {
                votes.sort();
                votes.reverse();
            }
            let the_max_vote = votes[zero_usize].clone();
            let max_vote_info = &vote_and_hash_dict[&the_max_vote];

            if max_vote_info.m_doc_hash == *invoked_doc_hash
            {
                sus_vote_res.insert(
                    a_coin.clone(),
                    SusVote {
                        m_valid: true,
                        m_action: "".to_string(),
                        m_voters: max_vote_info.m_voters,
                        m_votes: max_vote_info.m_votes,
                    });
            } else {
                sus_vote_res.insert(
                    a_coin.clone(),
                    SusVote {
                        m_valid: false,
                        m_action: "Reject".to_string(),
                        m_voters: max_vote_info.m_voters,
                        m_votes: max_vote_info.m_votes,
                    });
            }
        }
    }

//  if (do_db_log)
//    this._super.logSus({
//        lkey: '5.susVoteRes',
//        blockHash: '-',
//        docHash: invokedDocHash,
//        coins,
//        logBody: utils.stringify({ susVoteRes })
//    });

    let mut tmp = block_inspect_container.m_transactions_validity_check[invoked_doc_hash].clone();
    tmp.m_sus_vote_res = sus_vote_res;
    block_inspect_container.m_transactions_validity_check.insert(invoked_doc_hash.clone(), tmp);

    return;
}
