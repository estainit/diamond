use std::collections::HashMap;
use crate::cmerkle::{generate_m, MERKLE_VERSION};
use crate::{application, cutils};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_coinbase::cb1_maybe_create_coinbase_block::{calc_definite_releasable_micro_pai_per_one_cycle_now_or_before, TmpHolder};
use crate::lib::block::document_types::document::Document;
use crate::lib::constants;
use crate::lib::custom_types::{CDateT, CMPAIValueT, TimeByMinutesT};
use crate::lib::dlog::dlog;
use crate::lib::services::treasury::treasury_handler::calc_treasury_incomes;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use crate::lib::utils::dumper::dump_vec_of_t_output;


//old_name_was createCBCore
pub fn generate_coinbase_core(
    cycle_: &str,
    mode: &str,
    version: &str) -> (bool, Block)
{
    dlog(
        &format!("create CBCore cycle({}) mode({})", cycle_, mode),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let c_date: CDateT;

    let mut cycle = cycle_.to_string();

    if cycle == ""
    {
        c_date = application().now();
        cycle = application().get_coinbase_cycle_stamp(&c_date);
    } else {
        if application().cycle_length() == 1
        {
            // normally the cycle time is 12 hours
            c_date = cycle.clone();
        } else {
            // here is for test net in which the cycle time is accelerated to have a faster block generator(even 2 minutes)
            let minutes = cycle.split(" ")
                .collect::<Vec<&str>>()[1]
                .to_string()
                .parse::<TimeByMinutesT>()
                .unwrap() * application().get_cycle_by_minutes();

            let minutes_ = application().convert_minutes_to_hhmm(minutes);
            c_date = cycle.split(" ").collect::<Vec<&str>>()[0].to_string().clone() + " " + &minutes_ + ":00";
        }
    }

    let block_creation_date: CDateT = application().get_coinbase_range_by_cycle_stamp(&cycle).from;
    let mut doc: Document = Document::new();
    doc.m_doc_type = constants::document_types::COINBASE.to_string();
    doc.m_doc_class = constants::DEFAULT.to_string();
    doc.m_doc_length = 0;
    doc.m_doc_hash = constants::HASH_ZEROS_PLACEHOLDER.to_string();
    doc.m_if_coinbase_doc.m_doc_cycle = cycle.clone();

    let (from_date, to_date, incomes) = calc_treasury_incomes(&c_date);
    dlog(
        &format!("The treasury incomes for coinbase c_date({}) treasury incomes({}) micro PAIs from Date({}) toDate({})",
                 c_date, cutils::sep_num_3(incomes as i64), from_date, to_date),
        constants::Modules::CB,
        constants::SecLevel::Info);

    doc.m_if_coinbase_doc.m_treasury_incomes = incomes;
    doc.m_if_coinbase_doc.m_treasury_from = from_date;
    doc.m_if_coinbase_doc.m_treasury_to = to_date;

    // create coinbase outputs
    let mut tmp_out_dict: HashMap<CMPAIValueT, HashMap<String, TmpHolder>> = HashMap::new();
    let mut holders: Vec<String> = vec![];
    let mut dividends: Vec<CMPAIValueT> = vec![];

    // minted: 2,251,799,813.685248
    // burned:
    // share1:    21,110,874.142979
    // share2:       985,017.380061
    // share3:       422,068.382528
    // share4:            38.230606
    let (
        one_cycle_issued,
        total_shares,
        share_amount_per_holder) =
        calc_definite_releasable_micro_pai_per_one_cycle_now_or_before(
            &block_creation_date);

    dlog(
        &format!("share amount per holder: {:?}", &share_amount_per_holder),
        constants::Modules::CB,
        constants::SecLevel::Info);

    doc.m_if_coinbase_doc.m_minted_coins = one_cycle_issued;

    let cycle_coins: CMPAIValueT = doc.m_if_coinbase_doc.m_treasury_incomes as CMPAIValueT + doc.m_if_coinbase_doc.m_minted_coins as CMPAIValueT;
    dlog(
        &format!("cycle sum minted coins+treasury({} bicro PAIs)",
                 cutils::sep_num_3(cycle_coins as i64)),
        constants::Modules::CB,
        constants::SecLevel::Info);

    for (a_holder, a_share) in share_amount_per_holder
    {
        let dividend: CMPAIValueT = ((a_share * cycle_coins as f64) / total_shares) as CMPAIValueT;
        // let dividend = utils.floor((utils.iFloorFloat(a_share / sumShares) * cycleCoins));

        holders.push(a_holder.clone());
        dividends.push(dividend);
        if !tmp_out_dict.contains_key(&dividend)
        {
            tmp_out_dict.insert(dividend, HashMap::new()); //HashMap < String, TmpHolder >=
        }
        let mut inner_hashmap = tmp_out_dict[&dividend].clone();
        inner_hashmap.insert(a_holder.clone(), TmpHolder { holder: a_holder, dividend });
        tmp_out_dict.insert(dividend, inner_hashmap);
    }
    // in order to have unique hash for coinbase block (even created by different backers) sort it by sahres desc, addresses asc
    dividends.sort();
    dividends.dedup();// make a unique vec
    // dividends.erase(last, dividends.end());
    dividends.reverse();

    holders.sort();
    holders.reverse();

    let mut outputs: Vec<TOutput> = vec![];
    for dividend in dividends
    {
        for a_holder in &holders
        {
            if tmp_out_dict.contains_key(&dividend)
            {
                let keys = tmp_out_dict[&dividend].keys().cloned().collect::<Vec<String>>();
                if keys.contains(&a_holder) {
                    let output_arr: TOutput = TOutput {
                        m_address: tmp_out_dict[&dividend][&a_holder.clone()].holder.clone(),
                        m_amount: tmp_out_dict[&dividend][&a_holder.clone()].dividend.clone(),
                        m_output_character: "".to_string(),
                        m_output_index: 0,
                    };
                    outputs.push(output_arr);
                }
            }
        }
    }
    doc.m_if_coinbase_doc.m_outputs = outputs;
    dlog(
        &format!("Coinbase outputs on Cycle({}): details: {}", cycle, dump_vec_of_t_output(&doc.m_if_coinbase_doc.m_outputs)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    // let doc: Document = load_document(&doc, &Block::new(), 0);

    doc.m_doc_length = doc.calc_doc_length();
    dlog(
        &format!(
            "5 safe Sringify B length:{} ",
            doc.m_doc_length),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    doc.m_doc_hash = doc.calc_doc_hash(); // trxHashHandler.doHashTransaction(trx)

    let mut block: Block = Block::new();
    block.m_block_net = constants::SOCIETY_NAME.to_string();
    block.m_block_type = constants::block_types::COINBASE.to_string();
    block.m_block_hash = constants::HASH_ZEROS_PLACEHOLDER.to_string();
    block.m_block_version = version.to_string();
    block.m_if_coinbase_block.m_cycle = cycle.clone();

    let (root, _verifies, _merkle_version, _levels, _leaves) =
        generate_m(
            vec![doc.m_doc_hash.clone()],
                   &"hashed".to_string(),
                   &"keccak256".to_string(),
                   &MERKLE_VERSION.to_string());
    block.m_block_documents = vec![doc];
    block.m_block_documents_root_hash = root;
    block.m_block_creation_date = block_creation_date;

    return (true, block);
}
