use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::json;
use crate::cmerkle::generate_m;
use crate::{application, ccrypto, cutils, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_factory::load_block;
use crate::lib::block::block_types::block_floating_signature::floating_signature_block::aggrigate_floating_signatures;
use crate::lib::block::document_types::document::Document;
use crate::lib::constants;
use crate::lib::custom_types::{CDateT, CMPAIValueT, SharesCountT, QVDRecordsT, TimeByMinutesT, QSDicT, TimeBySecT, BlockLenT};
use crate::lib::dag::dag::{get_most_confidence_coinbase_block_from_dag, search_in_dag};
use crate::lib::dag::leaves_handler::{get_leave_blocks, has_fresh_leaves, LeaveBlock};
use crate::lib::dag::missed_blocks_handler::get_missed_blocks_to_invoke;
use crate::lib::database::abs_psql::{ModelClause, simple_eq_clause};
use crate::lib::dlog::dlog;
use crate::lib::machine::machine_neighbor::get_neighbors;
use crate::lib::messaging_protocol::dag_message_handler::set_maybe_ask_for_latest_blocks_flag;
use crate::lib::messaging_protocol::dispatcher::make_a_packet;
use crate::lib::sending_q_handler::sending_q_handler::push_into_sending_q;
use crate::lib::services::dna::dna_handler::{get_machine_shares, get_shares_info};
use crate::lib::services::treasury::treasury_handler::calc_treasury_incomes;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use crate::lib::utils::dumper::{dump_hashmap_of_qvd_records, dump_it, dump_vec_of_str, dump_vec_of_t_output};

#[derive(Clone)]
struct TmpHolder {
    holder: String,
    dividend: CMPAIValueT,
}

// pub fn get_coinbase_block_template() -> Block {
//     let mut block: Block = Block::new();
//     block.m_block_type = constants::block_types::COINBASE.to_string();
//
//     return coinbaseBlockVersion0;
// }

// pub fn get_coinbase_doc_template_object() -> JSonObject
// {
//     let j_doc: JSonObject = json!({
//         "dHash": "",
//         "dType": constants::document_types::COINBASE,
//         "dVer": "0.0.0",
//         "dCycle": "", // 'yyyy-mm-dd am' / 'yyyy-mm-dd pm'
//         "treasuryFrom": "", // incomes from date
//         "treasuryTo": "", // incomes to date
//         "treasuryIncomes": 0, // incomes value
//         "mintedCoins": 0,
//         "outputs": json!([])
//     });
//     return j_doc;
// }

/**
 *
 * @param {the time for which cycle is calculated} cycle
 *
 * coinbase core is only shares and dividends,
 * so MUST BE SAME IN EVERY NODES.
 * the inputs are newly minted coins and treasury incomes
 *
 */

//old_name_was createCBCore
pub fn create_coinbase_core(
    cycle_: &str,
    mode: &str,
    version: &str) -> (bool, Block)
{

    dlog(
        &format!("create CBCore cycle({cycle_}) mode({mode})"),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let c_date: CDateT;

    let mut cycle = cycle_.to_string();

    if cycle == ""
    {
        c_date = application().get_now();
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
    // doc.m_if_coinbase = get_coinbase_doc_template_object();

    doc.m_if_coinbase.m_doc_cycle = cycle.clone();

    let (from_date, to_date, incomes) = calc_treasury_incomes(&c_date);
    dlog(
        &format!("The treasury incomes for coinbase c_date({}) treasury incomes({}) micro PAIs from Date({}) toDate({})",
                 c_date, cutils::sep_num_3(incomes as i64), from_date, to_date),
        constants::Modules::CB,
        constants::SecLevel::Info);

    doc.m_if_coinbase.m_treasury_incomes = incomes;
    doc.m_if_coinbase.m_treasury_from = from_date;
    doc.m_if_coinbase.m_treasury_to = to_date;

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

    doc.m_if_coinbase.m_minted_coins = one_cycle_issued;

    let cycle_coins: CMPAIValueT = doc.m_if_coinbase.m_treasury_incomes as CMPAIValueT + doc.m_if_coinbase.m_minted_coins as CMPAIValueT;
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
                        m_output_charachter: "".to_string(),
                        m_output_index: 0,
                    };
                    outputs.push(output_arr);
                }
            }
        }
    }
    doc.m_if_coinbase.m_outputs = outputs;
    dlog(
        &format!("Coinbase recalculated outputs on Cycle({}): details: {}", cycle, dump_vec_of_t_output(&doc.m_if_coinbase.m_outputs)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    // let doc: Document = load_document(&doc, &Block::new(), 0);
    doc.m_doc_hash = doc.calc_doc_hash(); // trxHashHandler.doHashTransaction(trx)

    let (status, mut block) = load_block(&json!({
        "bNet": constants::SOCIETY_NAME,
        "bType": constants::block_types::COINBASE,
        "bLen": 12,
    }));
    if !status {
        dlog(
            &format!("Failed in load block from predefined JSON obj: {:?}", serde_json::to_string(&block).unwrap()),
            constants::Modules::CB,
            constants::SecLevel::Error);
        return (false, block);
    }

    block.m_block_version = version.to_string();

    block.m_if_coinbase_block.m_cycle = cycle.clone();

    let (root, _verifies, _merkle_version, _levels, _leaves) =
        generate_m(vec![doc.m_doc_hash.clone()], &"hashed".to_string(), &"".to_string(), &"".to_string());
    block.m_block_documents = vec![doc];
    block.m_block_documents_root_hash = root;
    block.m_block_creation_date = block_creation_date;

    return (true, block);
}

/*

/**
*
* @param {*} cycle
*
* although the coinbase core is only shares (which are equal in entire nodes)
* but the final coinbase block consists of also ancestors links (which are participating in block hash)
* so it could be possible different nodes generate different coinbaseHash for same blocks.
*
*/
*/
//old_name_was doGenerateCoinbaseBlock
pub fn do_generate_coinbase_block(
    cycle: &str,
    mode: &str,
    version: &str) -> (bool, Block)
{
    dlog(
        &format!("do GenerateCoinbaseBlock cycle({}) mode({})", cycle, mode),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let now_ = application().get_now();
    let (
        _cycle_stamp,
        from,
        to,
        _from_hour,
        _to_hour) = application().get_coinbase_info(&now_, cycle);

    println!("mmmmmmmm ressss: {}, {}, {}, {}, {}",
             _cycle_stamp,
             from,
             to,
             _from_hour,
             _to_hour);

    let (status, mut block) = create_coinbase_core(cycle, mode, version);
    if !status {
        dlog(
            &format!("Failed in create CB Core cycle({cycle}),  mode({mode}),  version({version})"),
            constants::Modules::CB,
            constants::SecLevel::Error);
        return (false, block);
    }

// connecting to existed leaves as ancestors
    let leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&from);
    let mut leaves_hashes: Vec<String> = leaves.keys().cloned().collect::<Vec<String>>();
    leaves_hashes.sort();
    dlog(
        &format!("do GenerateCoinbaseBlock retrieved cbInfo: from_({}) to_({})", from, to),
        constants::Modules::CB,
        constants::SecLevel::Info);
    dlog(
        &format!(
            "do GenerateCoinbaseBlock retrieved leaves from kv: cycle({}) leaves_hashes({}) leaves({})",
            cycle, leaves_hashes.join(", "), serde_json::to_string(&leaves).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Info);

    println!("GBC ----- 1");
    let now_ = application().get_now();
    let (_confidence, block_hashes, _backers) = aggrigate_floating_signatures(&now_);
    println!("GBC ----- 2");
    leaves_hashes = cutils::array_add(&leaves_hashes, &block_hashes);
    leaves_hashes.sort();
    leaves_hashes.dedup();

// if requested cycle is current cycle and machine hasn't fresh leaves, so can not generate a CB block
    let now_ = application().get_now();
    if (mode == constants::stages::CREATING) &&
        (leaves_hashes.len() == 0) &&
        (cycle == application().get_coinbase_cycle_stamp(&now_))
    {
        if mode == constants::stages::CREATING
        {
            dlog(
                &format!("generating new CB in generating mode failed!! leaves({})", leaves_hashes.join(",")),
                constants::Modules::CB,
                constants::SecLevel::Info);
        } else {
            dlog(
                &format!("strange error generating new CB failed!! mode({}) mode({}) ", mode, leaves_hashes.join(",")),
                constants::Modules::CB,
                constants::SecLevel::Error);
        }
        return (false, block);
    }

    block.m_block_ancestors = leaves_hashes.clone();
    dlog(
        &format!("do GenerateCoinbaseBlock block.ancestors: {}", leaves_hashes.join(",")),
        constants::Modules::CB,
        constants::SecLevel::Info);


    // if the backer is also a shareholder (probably with large amount of shares),
    // would be more usefull if she also signs the dividends by his private key as a shareholder
    // and sends also her corresponding publick key
    // this signature wil be used for 2 reson
    // 1. anti rebuild DAG by adverseries
    // 2. prevent fake sus-blocks to apply to network
    // signing block by backer private key (TODO: or delegated private key for the sake of security)


    return (true, block);
}

// it seems we do not need the big number module at all
//old_name_was calcPotentialMicroPaiPerOneCycle
pub fn calc_potential_micro_pai_per_one_cycle(year_: &String) -> CMPAIValueT
{
    let mut year = year_.to_string();
    if year == ""
    {
        year = application().get_current_year();
    }

    let year = year.parse::<u16>().unwrap();
    let halving_cycle_number = cutils::c_floor(((year - constants::LAUNCH_YEAR) / constants::HALVING_PERIOD) as f64);
    let one_cycle_max_bili_pais: CMPAIValueT = 2_i32.pow((constants::COIN_ISSUING_INIT_EXPONENT as i64 - halving_cycle_number) as u32) as CMPAIValueT;
    return one_cycle_max_bili_pais * constants::ONE_BILLION;
}

//old_name_was calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore
pub fn calc_definite_releasable_micro_pai_per_one_cycle_now_or_before(
    c_date: &CDateT) -> (CMPAIValueT, SharesCountT, HashMap<String, SharesCountT>)
{
    let one_cycle_issued: CMPAIValueT = calc_potential_micro_pai_per_one_cycle(&c_date.split("-").collect::<Vec<&str>>()[0].to_string());
    let (
        total_shares,
        share_amount_per_holder,
        _holders_order_by_shares) = get_shares_info(c_date);

    return (one_cycle_issued, total_shares, share_amount_per_holder);
}
/*

std::tuple<CMPAIValueT, CMPAIValueT, DNAShareCountT> CoinbaseIssuer::predictReleaseableMicroPAIsPerOneCycle(
const uint32_t& annualContributeGrowthRate,
const DNAShareCountT& current_total_sahres,
const String& prevDue,
CDateT due,
CDateT c_date)
{
if (c_date == "")
c_date = application().get_now();

if (due == "")
due = application().get_now();

if (c_date > application().get_now())
{
CLog::log("for now the formule does not support future contribute calculation. TODO: implement it", "app", "error");
return {0, 0, 0};
}

auto[one_cycle_issued, sum_shares, share_amount_per_holder_] = calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore(c_date);
Q_UNUSED(share_amount_per_holder_);

if (current_total_sahres != 0)
{
sum_shares = current_total_sahres;
//  } else {
//    sum_shares = sum_shares;
}

if (due > prevDue)
{
// add potentially contributes in next days
uint64_t futureDays = cutils::time_diff(prevDue, due).asDays;
DNAShareCountT newShares = sum_shares * (((annualContributeGrowthRate) / 100) / 365) * futureDays;
sum_shares += newShares;
uint8_t releseablePercentage = calculateReleasableCoinsBasedOnContributesVolume(sum_shares) / 100;
one_cycle_issued = cutils::CFloor(releseablePercentage * one_cycle_max_coins);
}


return {
one_cycle_max_coins,
one_cycle_issued,
sum_shares
};
}

/**
*
* @param {*} args
* return approxmatly incomes in next n years, based on your contribution and epotesic contributions growth
* {definiteIncomes, reserveIncomes, , monthly_incomes, firstIncomeDate, lastIncomeDate}
*/
FutureIncomes CoinbaseIssuer::predictFutureIncomes(
const uint32_t& the_contribute,  // contributeHours * contributeLevel
CDateT c_date, // contribute creation date (supposing aproved date of proposal)
uint32_t months,
DNAShareCountT current_total_sahres,
uint32_t annualContributeGrowthRate)
{
if (c_date == "")
c_date = application().get_now();

CMPAIValueT total_incomes = 0;
std::vector<MonthCoinsReport> monthly_incomes {};
CMPAIValueT one_cycle_income, income_per_month;
CDateT due;
CDateT prevDue = c_date;
for (uint32_t month = 0; month < months; month++)
{
due = cutils::minutesAfter((60 * 24) * (365 / 12) * month, c_date);   //TODO change it to more accurate on month starting day

auto[one_cycle_max_coins, one_cycle_issued, sum_shares] = predictReleaseableMicroPAIsPerOneCycle(
annualContributeGrowthRate,
current_total_sahres,
prevDue,
due,
c_date);
one_cycle_income = cutils::CFloor((one_cycle_issued * the_contribute) / sum_shares);  // one cycle income
income_per_month = cutils::CFloor((one_cycle_issued * the_contribute * (2 * 30)) / sum_shares);  // one month income almost = one cycle income * 2 perDay * 30 perMonth
CLog::log("predict Future Incomes sum_shares " + cutils::dumpIt(sum_shares) + " one_cycle_income("+cutils::microPAIToPAI6(one_cycle_income)+") income_per_month("+cutils::microPAIToPAI6(income_per_month)+")", "app", "trace");

MonthCoinsReport a_month {
one_cycle_max_coins,
one_cycle_issued,
one_cycle_income,
income_per_month,
sum_shares,
due.split(" ")[0]};
monthly_incomes.push(a_month);

//    monthly_incomes.push({
//      one_cycle_max_coins,
//      one_cycle_issued,
//      one_cycle_income,
//      due: due.split(' ')[0],
//      income_per_month,
//      sum_shares: Math.floor(sum_shares)
//    });
total_incomes += income_per_month;

prevDue = due;
current_total_sahres = sum_shares;
}

return {
total_incomes,
total_incomes * 3,   // 1 block released immidiately wheras 3 copy of that will be releaseable in next 3 months by voting of shareholders of block on creation time of block
//    cutils::CFloor(total_incomes / 1000000),  //TODO treasuryIncomes should be calcullated in a more smart way :)
monthly_incomes,
c_date,
due};
}

/**
* theorically the coinbase block can be created by any one,
* and the root hash of the block could be different(because of adifferent ancesters).
*/

void CoinbaseIssuer::loopMaybeIssueACoinbaseBlock()
{
String thread_prefix = "maybe_issue_coinbase_";
String thread_code = String::number((quint64)QThread::currentThread(), 16);

while (CMachine::shouldLoopThreads())
{
CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
maybeCreateCoinbaseBlock();

CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
std::this_thread::sleep_for(std::chrono::seconds(CMachine::getBlockInvokeGap()));
}

CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop maybe coinbase issuance");
}

*/

//old_name_was doesDAGHasMoreConfidenceCB
pub fn does_dag_has_more_confidence_cb() -> bool
{
    let now_ = application().get_now();
    let current_cycle_range_from: CDateT = application().get_coinbase_range(&now_).from;

    let already_recorded_coinbase_blocks: QVDRecordsT = search_in_dag(
        vec![
            simple_eq_clause("b_type", &constants::block_types::COINBASE.to_string()),
            ModelClause {
                m_field_name: "b_creation_date",
                m_field_single_str_value: &current_cycle_range_from as &(dyn ToSql + Sync),
                m_clause_operand: ">=",
                m_field_multi_values: vec![],
            },
        ],
        vec!["b_hash", "b_confidence", "b_ancestors"],
        vec![],
        0,
        false,
    );
    dlog(
        &format!("Already recorded coinbase blocks: {}", dump_hashmap_of_qvd_records(&already_recorded_coinbase_blocks)),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    if already_recorded_coinbase_blocks.len() == 0 {
        return false;
    }

    let mut already_recorded_confidents: Vec<f64> = vec![];
    let mut already_recorded_ancestors: Vec<String> = vec![];
    for block_record in already_recorded_coinbase_blocks {
        already_recorded_confidents.push(cutils::i_floor_float(block_record["b_confidence"].parse().unwrap()));
        already_recorded_ancestors = cutils::array_add(
            &already_recorded_ancestors,
            &cutils::convert_json_array_to_string_vector(
                &cutils::parse_to_json_array(&block_record["b_ancestors"])
            ),
        );
    }
    already_recorded_ancestors = cutils::array_unique(&already_recorded_ancestors);
    dlog(
        &format!("already Recorded Confidents from({}) {:?}", current_cycle_range_from, already_recorded_confidents),
        constants::Modules::CB,
        constants::SecLevel::Info);
    dlog(
        &format!("already Recorded Ancestors from({}) {:?}", current_cycle_range_from, already_recorded_ancestors),
        constants::Modules::CB,
        constants::SecLevel::Info);

    if (already_recorded_confidents.len() == 0) || (already_recorded_ancestors.len() == 0) {
        return false;
    }

    already_recorded_confidents.sort_by(f64::total_cmp);
    already_recorded_confidents.dedup();
    // already_recorded_confidents.erase(last, already_recorded_confidents.end());
    already_recorded_confidents.reverse();

    let max_recorded_confident: f64 = already_recorded_confidents[0];

    let now_ = application().get_now();
    let (the_confidence, block_hashes, _backers) = aggrigate_floating_signatures(&now_);
    let not_recorded_blocks: Vec<String> = cutils::array_diff(&block_hashes, &already_recorded_ancestors);
    if (the_confidence > max_recorded_confident) || (not_recorded_blocks.len() > 0) {
        return true;
    }

    return false;
}

//old_name_was makeEmailHashDict
pub fn make_email_hash_dictionary() -> (String, String, String, QSDicT)
{
    let mut emails_hash_dict: QSDicT = HashMap::new();
    let now_ = application().get_now();
    let cycle: CDateT = application().get_coinbase_cycle_stamp(&now_);
    let machine_email: String = machine().get_pub_email_info().m_address.clone();
    let machine_key: String = ccrypto::keccak256(&(cycle.clone() + "::" + &*machine_email));
    emails_hash_dict.insert(machine_key.clone(), machine_email.clone());

    let neightbors: QVDRecordsT = get_neighbors(
        "",
        constants::YES,
        "",
        0,
        "");
    dlog(
        &format!("neightbors in makeEmail Hash Dict: {}", dump_hashmap_of_qvd_records(&neightbors)),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    for neighbor in neightbors
    {
        let key: String = ccrypto::keccak256(&(cycle.clone() + "::" + &neighbor["n_email"]));
        emails_hash_dict.insert(key, neighbor["n_email"].to_string());
    }
    return (cycle.to_string(), machine_email.clone(), machine_key.to_string(), emails_hash_dict);
}

//old_name_was haveIFirstHashedEmail
pub fn if_i_have_first_hashed_email(order: &str) -> bool
{
    let (cycle, machine_email, machine_key, emails_hash_dict) = make_email_hash_dictionary();
    let mut keys: Vec<String> = emails_hash_dict.keys().cloned().collect::<Vec<String>>();

    if order == "asc" {
        keys.sort();
    } else {
        // reverse it
        keys.sort();
        keys.reverse();
    }
    dlog(
        &format!("Ordered emails_hash_dict {:?}", keys),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    for i in 0..keys.len() {
        dlog(
            &format!("{}. candidate email for issueing CB {} {} ", i + 1, emails_hash_dict[&keys[i]], cutils::hash8c(&keys[i])),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    }
    let machine_index: i32 = keys.iter().position(|r| r == &machine_key).unwrap() as i32; // keys.indexOf(machine_key)
    if machine_index == 0 {
        // the machin has minimum hash, so can generate the coinbase
        dlog(
            &format!("Machine has the lowest/heighest hash: {} {} ", machine_email, machine_key),
            constants::Modules::CB,
            constants::SecLevel::Info);
        return true;
    }

    // if the machine email hash is not the smalest,
    // control it if based on time passed from coinbase-cycle can create the coinbase?
    let now_ = application().get_now();
    let (
        _backer_address,
        _shares,
        mut percentage) = get_machine_shares(&now_);

    println!("kkkkkkkk 2");
    percentage = (percentage / 5.0) + 1.0;
    let mut sub_cycle = 12.0 + percentage;
    if application().cycle_length() != 1 {
        sub_cycle = 6.0 + percentage; // who has more shares should try more times to create a coinbase block
    }

    let now_ = application().get_now();
    let mut cb_email_counter = application().get_coinbase_age_by_seconds(&now_);
    println!("jjjjj sub_cycle: {}", sub_cycle);
    let sub2 =  application().get_cycle_by_seconds() / sub_cycle as TimeBySecT;
    println!("jjjjj kk: {}", sub2);
    cb_email_counter = cb_email_counter / sub2;
    println!("kkkkkkkk 2a {}", cb_email_counter);
    dlog(
        &format!("coinbase email counter cycle {} {} > {}", cb_email_counter, cb_email_counter, machine_index),
        constants::Modules::CB,
        constants::SecLevel::Info);
    println!("kkkkkkkk 3");
    if cb_email_counter > machine_index as TimeBySecT {
        // it is already passed time and if still no one create the block it is my turn to create it
        dlog(
            &format!("It already passed {} of 10 dividend of a cycle and now it's my turn {} to issue coinbase!", cb_email_counter, machine_email),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        return true;
    }
    println!("kkkkkkkk 4");
    dlog(
        &format!("Machine has to wait To Create Coinbase Block! (if does not receive the fresh CBB) keys({}::{})", cycle, machine_email),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);
    return false;
}


//old_name_was controlCoinbaseIssuanceCriteria
pub fn control_coinbase_issuance_criteria() -> bool
{
    let now_ = application().get_now();
    let current_cycle_range = application().get_coinbase_range(&now_);
    dlog(
        &format!("Coinbase check Range (from {} to {} ", current_cycle_range.from, current_cycle_range.to),
        constants::Modules::CB,
        constants::SecLevel::Info);


    if !has_fresh_leaves()
    {
        // younger than 2 cycle
        dlog(
            &format!("Machine hasn't fresh leaves!"),
            constants::Modules::CB,
            constants::SecLevel::Info);
        set_maybe_ask_for_latest_blocks_flag(constants::YES);
        return false;
    }
    // control if already exist in DAG a more confidence Coinbase Block than what machine can create?
    let dag_has_more_confidence_cb: bool = does_dag_has_more_confidence_cb();
    if dag_has_more_confidence_cb {
        dlog(
            &format!("Machine already has more confidente Coinbase blocks in DAG"),
            constants::Modules::CB,
            constants::SecLevel::Info);
        return false;
    }


    // // some control to be sure the coinbase block for current 12 hour cycle didn't create still,
    // // or at least I haven't it in my local machine
    // let latestCoinbase = cbBufferHandler.getMostConfidenceFromBuffer(currntCoinbaseTimestamp);
    // if (latestCoinbase.cycle == currntCoinbaseTimestamp) {
    //     // check if after sending cb to other , the local machine has added any new block to DAG? and machine still doesn't receive confirmed cb block?
    //     // so must create nd send new coinbase block
    //     msg = `At least one CB exists on local machine: ${latestCoinbase} (${iutils.getCoinbaseRange().from.split(' ')[1]} - ${iutils.getCoinbaseRange().to.split(' ')[1]})`
    //     clog.cb.info(msg);
    //     res.msg = msg
    //     res.atLeastOneCBExists = true;
    // }

    // postpond coinbase-generating if machine missed some blocks
    let missed_blocks: Vec<String> = get_missed_blocks_to_invoke(0);
    println!("mmmmmmm missed_blocks {:?}", missed_blocks);

    if missed_blocks.len() > 0 {

        // // FIXME: if an adversory sends a bunch of blocks which have ancestors, in machine will finished with a long list of
        // // missed blocks. so machine(or entire network machines) can not issue new coinbase block!
        // if (missed_blocks.length > iConsts.MAX_TOLERATED_MISS_BLOCKS) {
        //     msg = `Machine missed ${missed_blocks.length} blocks and touched the limitation(${iConsts.MAX_TOLERATED_MISS_BLOCKS}), so can not issue a coinbase block`
        //     clog.cb.info(msg);
        //     res.canGenCB = false;
        //     res.msg = msg
        //     return res;
        // }

        let now_ = application().get_now();
        let latenancy_factor: f64 = ((missed_blocks.len() + 1) as f64 / (constants::MAX_TOLERATED_MISS_BLOCKS as f64)) * application().get_coinbase_age_by_seconds(&now_) as f64;
        let are_we_in_4_of_5: bool = application().get_coinbase_age_by_seconds(&now_) < (application().get_cycle_by_seconds() * 4 / 5);
        if are_we_in_4_of_5 && (application().get_coinbase_age_by_seconds(&now_) < latenancy_factor as TimeBySecT)
        {
            dlog(
                &format!("Because of {} missed blocks, machine can not create a CB before {} second age of cycle or atleast 4/5 of cycle age passed", missed_blocks.len(), latenancy_factor),
                constants::Modules::CB,
                constants::SecLevel::Info);
            return false;
        }
    }

    // a psudo random mechanisem
    println!("mmmmmmm 65 76 ");
    let am_i_qualified_to_issue_coinbase: bool = if_i_have_first_hashed_email("asc");
    if !am_i_qualified_to_issue_coinbase {
        dlog(
            &format!("It is not machine turn To Create Coinbase Block!"),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        return false;
    }
    return true;
}

// if passed 1/4 of a cycle time and still the coinbase block is not created, so broadcast one of them
//old_name_was passedCertainTimeOfCycleToRecordInDAG
pub fn if_passed_certain_time_of_cycle_to_record_in_dag(c_date: &CDateT) -> bool
{
    let (_cycle, _machine_email, machine_key, emails_hash_dict) = make_email_hash_dictionary();
    let mut keys: Vec<String> = emails_hash_dict.keys().cloned().collect::<Vec<String>>();
    keys.sort();
    let machine_index: i32 = keys.iter().position(|r| r == &machine_key).unwrap() as i32 + 2; // keys.indexOf(machine_key)
    dlog(
        &format!("psudo-random CB creation machine_index: {}", machine_index),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let mut cycle_by_minutes: TimeByMinutesT = constants::STANDARD_CYCLE_BY_MINUTES as TimeByMinutesT;
    if application().cycle_length() != 1 {
        cycle_by_minutes = application().cycle_length() as TimeByMinutesT;
    }
    let from_t_ = application().get_coinbase_range(c_date).from;
    let to_t_ = application().get_now();
    let res: bool = application().time_diff(from_t_, to_t_).as_seconds
        >= (cycle_by_minutes as f64 * 60.0 * constants::COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG * (1 + (machine_index.pow(7) / 131)) as f64) as u64;
    dlog(
        &format!("passed CertainTimeOfCycleToRecordInDAG? {}", res),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    return res;
}

//old_name_was maybeCreateCoinbaseBlock
pub fn maybe_create_coinbase_block() -> bool
{
    let can_issue_new_cb = control_coinbase_issuance_criteria();
    println!("kkkkkkkk 66 can_issue_new_cb? {}", can_issue_new_cb);
    if !can_issue_new_cb {
        return true;
    }
    return try_create_coinbase_block();
}

//old_name_was tryCreateCoinbaseBlock
pub fn try_create_coinbase_block() -> bool
{
    let now_ = application().get_now();
    let (
        _coinbase_cycle_stamp,
        coinbase_from,
        coinbase_to,
        _coinbase_from_hour,
        _coinbase_to_hour) = application().get_coinbase_info(&now_, "");

    dlog(
        &format!("Try to Create Coinbase for Range ({}, {})", coinbase_from, coinbase_to),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let cycle_stamp = application().get_coinbase_cycle_stamp(&now_);
    let (status, mut block) = do_generate_coinbase_block(
        &cycle_stamp,
        constants::stages::CREATING,
        "0.0.1");

    if !status {
        dlog(
            &format!("Due to an error, can not create a coinbase block for range ({}, {})", coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Fatal);
        return false;
    }


    dlog(
        &format!("Serialized locally created cb block. before objecting1 {}", serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    block.m_block_length = serde_json::to_string(&block).unwrap().len() as BlockLenT;
    dlog(
        &format!("Serialized locally created cb block. before objecting2 {}", serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    block.set_block_hash(&block.calc_block_hash());
    dlog(
        &format!("Serialized locally created cb block. after objecting {}", serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let tmp_local_confidence: f64 = block.m_block_confidence as f64;

// if local machine can create a coinbase block with more confidence or ancestors, broadcast it
    let now_ = application().get_now();
    let (atleast_one_coinbase_block_exist, most_confidence_in_dag) = get_most_confidence_coinbase_block_from_dag(&now_);

    let mut tmp_dag_confidence: f64 = 0.0;
    let mut tmp_dag_ancestors: Vec<String> = vec![];
    if !atleast_one_coinbase_block_exist
    {
        dlog(
            &format!("DAG hasn't coinbase for cycle range ({}, {})", coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    } else {
        dlog(
            &format!("The most_confidence_in_DAG for cycle range ({}, {}) is: {}", coinbase_from, coinbase_to, dump_it(&most_confidence_in_dag)),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);

        tmp_dag_confidence = most_confidence_in_dag["b_confidence"].parse::<f64>().unwrap();
        tmp_dag_ancestors = cutils::convert_comma_separated_to_array(&most_confidence_in_dag["b_ancestors"].to_string(), &",".to_string());
    }

    let locally_created_coinbase_block_has_more_confidence_than_dag: bool = tmp_dag_confidence < tmp_local_confidence;
    // locally_created_coinbase_block_has_more_confidence_than_dag = false;// FIXME implement remote block confidence calcuilation
    if locally_created_coinbase_block_has_more_confidence_than_dag
    {
        dlog(
            &format!("More confidence: local coinbase({}) has more confidence({}) than DAG({}) in cycle range ({}, {})",
                     cutils::hash8c(&block.m_block_hash), tmp_local_confidence.to_string(), tmp_dag_confidence.to_string(), coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    }

    let mut ancestors_diff: Vec<String> = cutils::array_diff(
        &block.m_block_ancestors,
        &tmp_dag_ancestors);
    if ancestors_diff.len() > 0
    {
        // try to remove repayBack blocks
        let empty_string = "".to_string();
        let mut c1 = ModelClause {
            m_field_name: "b_hash",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        for an_anc in &ancestors_diff {
            c1.m_field_multi_values.push(an_anc as &(dyn ToSql + Sync));
        }
        let existed_repay_blocks: QVDRecordsT = search_in_dag(
            vec![
                simple_eq_clause("b_type", &constants::block_types::REPAYMENT_BLOCK.to_string()),
                c1,
            ],
            vec!["b_hash"],
            vec![],
            0,
            true);
        if existed_repay_blocks.len() > 0
        {
            let mut tmp: Vec<String> = vec![];
            for record in existed_repay_blocks
            {
                tmp.push(record["b_hash"].to_string());
            }
            ancestors_diff = cutils::array_diff(&ancestors_diff, &tmp);
        }
    }
    let locally_created_coinbase_block_has_more_ancestors_than_dag: bool = ancestors_diff.len() > 0;
    if locally_created_coinbase_block_has_more_ancestors_than_dag
    {
        dlog(
            &format!("More ancestors: local coinbase({}) has more ancestors({:?} than DAG({}) in cycle range ({}, {})",
                     cutils::hash8c(&block.m_block_hash.to_string()), block.m_block_ancestors, dump_vec_of_str(&tmp_dag_ancestors), coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
    }

    dlog(
        &format!("Is about to issuing coinbase block in cycle range ({}, {}) the block: {}",
                 coinbase_from, coinbase_to, serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    let missed_blocks: Vec<String> = get_missed_blocks_to_invoke(0);
    if missed_blocks.len() > 0
    {
        dlog(
            &format!("BTW machine has some missed blocks: {}", dump_it(&missed_blocks)),
            constants::Modules::CB,
            constants::SecLevel::Warning);
    }

// FIXME: it is a way to avoid creating too many coinbases which have a little difference because of the ancestors.
// could it be a security issue? when an adversory in last minutes(before midnight or mid-day) starts to spam network by blocks
// and most of nodes can not be synched, so too many coinbase blocks creating
//
    if (locally_created_coinbase_block_has_more_confidence_than_dag
        || locally_created_coinbase_block_has_more_ancestors_than_dag)
        && (get_missed_blocks_to_invoke(0).len() < 1)
    {

        // broadcast coin base
        if application().is_in_current_cycle(&block.m_block_creation_date.to_string())
        {
            dlog(
                &format!("pushing coinbase block to network: {}", block.m_block_hash),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            let mut block_body = serde_json::to_string(&block).unwrap();
            block_body = ccrypto::b64_encode(&block_body);
            let _ancestors: Vec<String> = block.m_block_ancestors.clone();

            let (_code, body) = make_a_packet(
                vec![
                    json!({
                "cdType": constants::block_types::COINBASE,
                "cdVer": constants::DEFAULT_CARD_VERSION,
                "bHash": block.m_block_hash.clone(),
                // "ancestors": ancestors,
                "block": block_body,
            }),
                ],
                constants::DEFAULT_PACKET_TYPE,
                constants::DEFAULT_PACKET_VERSION,
                application().get_now(),
            );
            dlog(
                &format!("prepared coinbase packet, before insert into DB code({}) {}", block.m_block_hash, body),
                constants::Modules::App,
                constants::SecLevel::Info);

            let status = push_into_sending_q(
                constants::block_types::COINBASE,
                block.m_block_hash.as_str(),
                &body,
                &format!("Coinbase block ({}) issued by me", cutils::hash6c(&block.m_block_hash)),
                &vec![],
                &vec![],
                false,
            );

            // let push_res: bool = push_into_sending_q(
            //     constants::block_types::COINBASE,
            //     &*block.m_block_hash,
            //     &*serde_json::to_string(&block).unwrap(),
            //     &format!("Broadcasting coinbase block CB({}) issued by({} for cycle range({}, {})",
            //              &cutils::hash8c(&block.m_block_hash),
            //              machine().get_pub_email_info().m_address,
            //              coinbase_from,
            //              coinbase_to),
            //     &vec![],
            //     &vec![],
            //     false,
            // );

            dlog(
                &format!("coinbase push1 res({})", status),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            dlog(
                &format!("Coinbase issued because of clause 1 CB({}) issued by({} for cycle range({}, {})",
                         cutils::hash8c(&block.m_block_hash.to_string()), machine().get_pub_email_info().m_address, coinbase_from, coinbase_to),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);

            return status;
        }
    } else if
    if_passed_certain_time_of_cycle_to_record_in_dag(&now_) && !atleast_one_coinbase_block_exist
    {
        // another psudo random emulatore
        // if already passed more than 1/4 of cycle and still no coinbase block recorded in DAG,
        // so the machine has to create one
        if if_i_have_first_hashed_email("desc") {
            let push_res: bool = push_into_sending_q(
                constants::block_types::COINBASE,
                &*block.m_block_hash,
                &serde_json::to_string(&block).unwrap(),
                &("Broadcasting coinbase block CB(".to_owned() + &cutils::hash8c(&block.m_block_hash) + ") issued by(" + &machine().get_pub_email_info().m_address + " for cycle range(" + &coinbase_from + ", " + &coinbase_to + ")"),
                &vec![],
                &vec![],
                false);

            dlog(
                &format!("coinbase push2 res({})", dump_it(push_res)),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);
            dlog(
                &format!("Coinbase issued because of clause 2 CB({}) issued by({} for cycle range({}, {})",
                         cutils::hash8c(&block.m_block_hash), &machine().get_pub_email_info().m_address, &coinbase_from, &coinbase_to),
                constants::Modules::CB,
                constants::SecLevel::TmpDebug);
            return true;
        }
    } else {
        dlog(
            &format!("Coinbase can be issued by clause 3 but local hasn't neither more confidence nor more ancestors and still not riched to 1/4 of cycle time. CB({}) issued by({} for cycle range({}, {})",
                     cutils::hash8c(&block.m_block_hash),
                     &machine().get_pub_email_info().m_address,
                     coinbase_from,
                     coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        return true;
    }
    true
}

