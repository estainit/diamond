use std::collections::HashMap;
use serde_json::json;
use crate::cmerkle::generate_m;
use crate::{ccrypto, cutils, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_factory::load_block;
use crate::lib::block::block_types::block_floating_signature::floating_signature_block::aggrigateFloatingSignatures;
use crate::lib::block::document_types::document::Document;
use crate::lib::constants;
use crate::lib::custom_types::{CDateT, CMPAIValueT, DNAShareCountT, QVDRecordsT, TimeByMinutesT, QSDicT, TimeBySecT, BlockLenT};
use crate::lib::dag::dag::{getMostConfidenceCoinbaseBlockFromDAG, searchInDAG};
use crate::lib::dag::leaves_handler::{get_leave_blocks, has_fresh_leaves, LeaveBlock};
use crate::lib::dag::missed_blocks_handler::getMissedBlocksToInvoke;
use crate::lib::database::abs_psql::{ModelClause, simple_eq_clause};
use crate::lib::dlog::dlog;
use crate::lib::messaging_protocol::dag_message_handler::setMaybeAskForLatestBlocksFlag;
use crate::lib::sending_q_handler::sending_q_handler::pushIntoSendingQ;
use crate::lib::services::dna::dna_handler::{getMachineShares, getSharesInfo};
use crate::lib::services::treasury::treasury_handler::calcTreasuryIncomes;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use crate::lib::utils::dumper::{dump_hashmap_of_QVDRecordsT, dump_hashmap_of_string_f64, dump_it, dump_vec_of_str, dump_vec_of_t_output};

#[derive(Clone)]
struct TmpHolder {
    holder: String,
    dividend: CMPAIValueT,
}

// pub fn get_coinbase_block_template() -> Block {
//     let mut block: Block = Block::new();
//     block.m_block_type = constants::block_types::Coinbase.to_string();
//
//     return coinbaseBlockVersion0;
// }

// pub fn get_coinbase_doc_template_object() -> JSonObject
// {
//     let j_doc: JSonObject = json!({
//         "dHash": "",
//         "dType": constants::doc_types::Coinbase,
//         "dVer": "0.0.0",
//         "cycle": "", // 'yyyy-mm-dd am' / 'yyyy-mm-dd pm'
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

pub fn createCBCore(
    cycle_: &str,
    mode: &str,
    version: &str) -> Block
{
    dlog(
        &format!("create CBCore cycle({cycle_}) mode({mode})"),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let c_date: CDateT;

    let mut cycle = cycle_.to_string();

    if cycle == ""
    {
        c_date = cutils::get_now();
        cycle = cutils::get_coinbase_cycle_stamp(&cutils::get_now());
    } else {
        if constants::TIME_GAIN == 1
        {
            // normally the cycle time is 12 hours
            c_date = cycle.clone();
        } else {
            // here is for test net in which the cycle time is accelerated to have a faster block generator(even 2 minutes)
            let minutes = cycle.split(" ").collect::<Vec<&str>>()[1].to_string().parse::<TimeByMinutesT>().unwrap() * cutils::get_cycle_by_minutes();
            let minutes_ = cutils::convert_minutes_to_hhmm(minutes);
            c_date = cycle.split(" ").collect::<Vec<&str>>()[0].to_string().clone() + " " + &minutes_ + ":00";
        }
    }

    let block_creation_date: CDateT = cutils::get_coinbase_range_by_cycle_stamp(&cycle).from;
    let mut doc: Document = Document::new();
    doc.m_doc_type = constants::doc_types::Coinbase.to_string();
    // doc.m_if_coinbase = get_coinbase_doc_template_object();

    doc.m_if_coinbase.m_doc_cycle = cycle.clone();

    let (from_date, to_date, incomes) = calcTreasuryIncomes(&c_date);
    dlog(
        &format!("The treasury incomes for coinbase c_date({}) treasury incomes({}) micro PAIs from Date({}) toDate({})",
                 c_date, cutils::sepNum(incomes as i64), from_date, to_date),
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
    let (one_cycle_issued, total_shares, share_amount_per_holder) =
        calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore(
            &block_creation_date);

    dlog(
        &format!("share_amount_per_holder: {}", dump_hashmap_of_string_f64(&share_amount_per_holder)),
        constants::Modules::CB,
        constants::SecLevel::Info);

    doc.m_if_coinbase.m_minted_coins = one_cycle_issued;

    let cycleCoins: CMPAIValueT = doc.m_if_coinbase.m_treasury_incomes as CMPAIValueT + doc.m_if_coinbase.m_minted_coins as CMPAIValueT;
    dlog(
        &format!("DNA cycle sum minted coins+treasury({} bicro PAIs)",
                 cutils::sepNum(cycleCoins as i64)),
        constants::Modules::CB,
        constants::SecLevel::Info);

    for (a_holder, a_share) in share_amount_per_holder
    {
        let dividend: CMPAIValueT = ((a_share * cycleCoins as f64) / total_shares) as CMPAIValueT;
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
        constants::SecLevel::Trace);

    // let doc: Document = load_document(&doc, &Block::new(), 0);
    doc.m_doc_hash = doc.calc_doc_hash(); // trxHashHandler.doHashTransaction(trx)

    let mut block: Block = load_block(&json!({
        "net": constants::SOCIETY_NAME.to_string(),
        "bType": constants::block_types::Coinbase.to_string()
    }));
    block.m_block_version = version.to_string();

    block.m_if_coinbase_block.m_cycle = cycle.clone();

    let (root, _verifies, _merkle_version, _levels, _leaves) =
        generate_m(vec![doc.m_doc_hash.clone()], &"hashed".to_string(), &"".to_string(), &"".to_string());
    block.m_documents = vec![doc];
    block.m_documents_root_hash = root;
    block.m_block_creation_date = block_creation_date;

    return block;
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
pub fn doGenerateCoinbaseBlock(
    cycle: &str,
    mode: &str,
    version: &str) -> (bool, Block)
{
    dlog(
        &format!("do GenerateCoinbaseBlock cycle({}) mode({})", cycle, mode),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let (_cycleStamp, from, to, _from_hour, _to_hour) =
        cutils::get_coinbase_info(&cutils::get_now(), cycle);

    let mut block: Block = createCBCore(cycle, mode, version);

// connecting to existed leaves as ancestors
    let leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&from);
    let mut leaves_hashes: Vec<String> = leaves.keys().cloned().collect::<Vec<String>>();
    leaves_hashes.sort();
    dlog(
        &format!("do GenerateCoinbaseBlock retrieved cbInfo: from_({}) to_({})", from, to),
        constants::Modules::CB,
        constants::SecLevel::Info);
    dlog(
        &format!("do GenerateCoinbaseBlock retrieved leaves from kv: cycle({}) leaves_hashes({}) leaves({})",
                 cycle, leaves_hashes.join(", "), serde_json::to_string(&leaves).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let (_confidence, block_hashes, _backers) = aggrigateFloatingSignatures(&cutils::get_now());
    leaves_hashes = cutils::arrayAdd(&leaves_hashes, &block_hashes);
    leaves_hashes.sort();
    leaves_hashes.dedup();

// if requested cycle is current cycle and machine hasn't fresh leaves, so can not generate a CB block
    if (mode == constants::stages::Creating) &&
        (leaves_hashes.len() == 0) &&
        (cycle == cutils::get_coinbase_cycle_stamp(&cutils::get_now()))
    {
        if mode == constants::stages::Creating
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

    block.m_ancestors = leaves_hashes.clone();
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
pub fn calcPotentialMicroPaiPerOneCycle(year_: &String) -> CMPAIValueT
{
    let mut year = year_.to_string();
    if year == ""
    {
        year = cutils::get_current_year();
    }

    let year = year.parse::<u8>().unwrap();
    let halving_cycle_number = cutils::CFloor(((year - constants::LAUNCH_YEAR) / constants::HALVING_PERIOD) as f64);
    let one_cycle_max_bili_pais: CMPAIValueT = 2_i32.pow((constants::COIN_ISSUING_INIT_EXPONENT as i64 - halving_cycle_number) as u32) as CMPAIValueT;
    return one_cycle_max_bili_pais * constants::ONE_BILLION;
}

pub fn calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore(
    c_date: &CDateT) -> (CMPAIValueT, DNAShareCountT, HashMap<String, DNAShareCountT>)
{
    let one_cycle_issued: CMPAIValueT = calcPotentialMicroPaiPerOneCycle(&c_date.split("-").collect::<Vec<&str>>()[0].to_string());
    let (total_shares, share_amount_per_holder, _holdersOrderByShares) = getSharesInfo(c_date);

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
c_date = cutils::get_now();

if (due == "")
due = cutils::get_now();

if (c_date > cutils::get_now())
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
c_date = cutils::get_now();

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

pub fn doesDAGHasMoreConfidenceCB() -> bool
{
    let current_cycle_range_from: CDateT = cutils::get_coinbase_range(&cutils::get_now()).from;

    let already_recorded_coinbase_blocks: QVDRecordsT = searchInDAG(
        &vec![
            &simple_eq_clause("b_type", constants::block_types::Coinbase),
            &ModelClause {
                m_field_name: "b_creation_date",
                m_field_single_str_value: &*current_cycle_range_from,
                m_clause_operand: ">=",
                m_field_multi_values: vec![],
            },
        ],
        &vec!["b_hash", "b_confidence", "b_ancestors"],
        &vec![],
        0,
        false,
    );
    dlog(
        &format!("Already recorded coinbase blocks: {}", dump_hashmap_of_QVDRecordsT(&already_recorded_coinbase_blocks)),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    if already_recorded_coinbase_blocks.len() == 0 {
        return false;
    }

    let mut already_recorded_confidents: Vec<f64> = vec![];
    let mut already_recorded_ancestors: Vec<String> = vec![];
    for block_record in already_recorded_coinbase_blocks {
        already_recorded_confidents.push(cutils::customFloorFloat(block_record["b_confidence"].parse().unwrap(), 11));
        already_recorded_ancestors = cutils::arrayAdd(
            &already_recorded_ancestors,
            &cutils::convertJSonArrayToStringVector(
                &cutils::parseToJsonArr(&block_record["b_ancestors"])
            ),
        );
    }
    already_recorded_ancestors = cutils::arrayUnique(&already_recorded_ancestors);
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

    let (the_confidence, block_hashes, _backers) = aggrigateFloatingSignatures(&cutils::get_now());
    let not_recorded_blocks: Vec<String> = cutils::arrayDiff(&block_hashes, &already_recorded_ancestors);
    if (the_confidence > max_recorded_confident) || (not_recorded_blocks.len() > 0) {
        return true;
    }

    return false;
}

pub fn makeEmailHashDict() -> (String, String, String, QSDicT)
{
    let mut emails_hash_dict: QSDicT = HashMap::new();
    let cycle: CDateT = cutils::get_coinbase_cycle_stamp(&cutils::get_now());
    let machine_email: String = machine().getPubEmailInfo().m_address.clone();
    let machine_key: String = ccrypto::keccak256(&(cycle.clone() + "::" + &*machine_email));
    emails_hash_dict.insert(machine_key.clone(), machine_email.clone());

    let neightbors: QVDRecordsT = machine().getNeighbors(
        "",
        constants::YES,
        "",
        "",
        "");
    dlog(
        &format!("neightbors in makeEmail Hash Dict: {}", dump_hashmap_of_QVDRecordsT(&neightbors)),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    for neighbor in neightbors
    {
        let key: String = ccrypto::keccak256(&(cycle.clone() + "::" + &neighbor["n_email"]));
        emails_hash_dict.insert(key, neighbor["n_email"].to_string());
    }
    return (cycle.to_string(), machine_email.clone(), machine_key.to_string(), emails_hash_dict);
}

pub fn haveIFirstHashedEmail(order: &str) -> bool
{
    let (cycle, machine_email, machine_key, emails_hash_dict) = makeEmailHashDict();
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
        constants::SecLevel::Trace);

    for i in 0..keys.len() {
        dlog(
            &format!("{}. candidate email for issueing CB {} {} ", i + 1, emails_hash_dict[&keys[i]], cutils::hash8c(&keys[i])),
            constants::Modules::CB,
            constants::SecLevel::Trace);
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
    let (_backer_address, _shares, mut percentage) = getMachineShares(&cutils::get_now());
    percentage = (percentage / 5.0) + 1.0;
    let mut sub_cycle = 12.0 + percentage;
    if constants::TIME_GAIN != 1 {
        sub_cycle = 6.0 + percentage; // who has more shares should try more times to create a coinbase block
    }
    let cb_email_counter = cutils::getCoinbaseAgeBySecond(&cutils::get_now()) / (cutils::get_cycle_by_seconds() / sub_cycle as TimeBySecT);
    dlog(
        &format!("cb_email_counter cycle {} {} > {}", cb_email_counter, cb_email_counter, machine_index),
        constants::Modules::CB,
        constants::SecLevel::Info);
    if cb_email_counter > machine_index as TimeBySecT {
        // it is already passed time and if still no one create the block it is my turn to create it
        dlog(
            &format!("It already passed {} of 10 dividend of a cycle and now it's my turn {} to issue coinbase!", cb_email_counter, machine_email),
            constants::Modules::CB,
            constants::SecLevel::Trace);
        return true;
    }
    dlog(
        &format!("Machine has to wait To Create Coinbase Block! (if does not receive the fresh CBB) keys({}::{})", cycle, machine_email),
        constants::Modules::CB,
        constants::SecLevel::Trace);
    return false;
}


//old_name_was controlCoinbaseIssuanceCriteria
pub fn control_coinbase_issuance_criteria() -> bool
{
    let current_cycle_range = cutils::get_coinbase_range(&cutils::get_now());
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
        setMaybeAskForLatestBlocksFlag(&constants::YES.to_string());
        return false;
    }
    // control if already exist in DAG a more confidence Coinbase Block than what machine can create?
    let dag_has_more_confidence_cb: bool = doesDAGHasMoreConfidenceCB();
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
    let missed_blocks: Vec<String> = getMissedBlocksToInvoke(0);
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

        let latenancy_factor: f64 = ((missed_blocks.len() + 1) as f64 / (constants::MAX_TOLERATED_MISS_BLOCKS as f64)) * cutils::getCoinbaseAgeBySecond(&cutils::get_now()) as f64;
        let are_we_in_4_of_5: bool = cutils::getCoinbaseAgeBySecond(&cutils::get_now()) < (cutils::get_cycle_by_seconds() * 4 / 5);
        if are_we_in_4_of_5 && (cutils::getCoinbaseAgeBySecond(&cutils::get_now()) < latenancy_factor as TimeBySecT)
        {
            dlog(
                &format!("Because of {} missed blocks, machine can not create a CB before {} second age of cycle or atleast 4/5 of cycle age passed", missed_blocks.len(), latenancy_factor),
                constants::Modules::CB,
                constants::SecLevel::Info);
            return false;
        }
    }

    // a psudo random mechanisem
    let am_i_qualified_to_issue_coinbase: bool = haveIFirstHashedEmail("asc");
    if !am_i_qualified_to_issue_coinbase {
        dlog(
            &format!("It is not machine turn To Create Coinbase Block!"),
            constants::Modules::CB,
            constants::SecLevel::Trace);
        return false;
    }
    return true;
}

// if passed 1/4 of a cycle time and still the coinbase block is not created, so broadcast one of them
pub fn passedCertainTimeOfCycleToRecordInDAG(c_date: &CDateT) -> bool
{
    let (_cycle, _machine_email, machine_key, emails_hash_dict) = makeEmailHashDict();
    let mut keys: Vec<String> = emails_hash_dict.keys().cloned().collect::<Vec<String>>();
    keys.sort();
    let machine_index: i32 = keys.iter().position(|r| r == &machine_key).unwrap() as i32 + 2; // keys.indexOf(machine_key)
    dlog(
        &format!("psudo-random CB creation machine_index: {}", machine_index),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    let mut cycle_by_minutes: TimeByMinutesT = constants::STANDARD_CYCLE_BY_MINUTES as TimeByMinutesT;
    if constants::TIME_GAIN != 1 {
        cycle_by_minutes = constants::TIME_GAIN as TimeByMinutesT;
    }
    let res: bool = cutils::time_diff(
        cutils::get_coinbase_range(c_date).from,
        cutils::get_now()).as_seconds
        >= (cycle_by_minutes as f64 * 60.0 * constants::COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG * (1 + (machine_index.pow(7) / 131)) as f64) as u64;
    dlog(
        &format!("passed CertainTimeOfCycleToRecordInDAG? {}", res),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    return res;
}

//old_name_was maybeCreateCoinbaseBlock
pub fn maybe_create_coinbase_block()
{
    let can_issue_new_cb = control_coinbase_issuance_criteria();
    if !can_issue_new_cb {
        return;
    }
    tryCreateCoinbaseBlock();
}


pub fn tryCreateCoinbaseBlock()
{
    let (_coinbase_cycle_stamp, coinbase_from, coinbase_to, _coinbase_from_hour, _coinbase_to_hour) =
        cutils::get_coinbase_info(&cutils::get_now(), "");
// listener.doCallAsync('APSH_create_coinbase_block', { cbInfo });

    dlog(
        &format!("Try to Create Coinbase for Range ({}, {})", coinbase_from, coinbase_to),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    let (status, mut block) = doGenerateCoinbaseBlock(
        &cutils::get_coinbase_cycle_stamp(&cutils::get_now()),
        constants::stages::Creating,
        "0.0.1");
    if !status {
        dlog(
            &format!("Due to an error, can not create a coinbase block for range ({}, {})", coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Fatal);
        return;
    }


    dlog(
        &format!("Serialized locally created cb block. before objecting1 {}", serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    block.m_block_length = serde_json::to_string(&block).unwrap().len() as BlockLenT;
    dlog(
        &format!("Serialized locally created cb block. before objecting2 {}", serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    block.setBlockHash(&block.calc_block_hash());
    dlog(
        &format!("Serialized locally created cb block. after objecting {}", serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    let tmp_local_confidence: f64 = block.m_block_confidence as f64;

// if local machine can create a coinbase block with more confidence or ancestors, broadcast it
    let (atleast_one_coinbase_block_exist, most_confidence_in_dag) = getMostConfidenceCoinbaseBlockFromDAG(&cutils::get_now());

    let mut tmp_dag_confidence: f64 = 0.0;
    let mut tmp_dag_ancestors: Vec<String> = vec![];
    if !atleast_one_coinbase_block_exist
    {
        dlog(
            &format!("DAG hasn't coinbase for cycle range ({}, {})", coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Trace);
    } else {
        dlog(
            &format!("The most_confidence_in_DAG for cycle range ({}, {}) is: {}", coinbase_from, coinbase_to, dump_it(&most_confidence_in_dag)),
            constants::Modules::CB,
            constants::SecLevel::Trace);

        tmp_dag_confidence = most_confidence_in_dag["b_confidence"].parse::<f64>().unwrap();
        tmp_dag_ancestors = cutils::convert_comma_separated_to_array(&most_confidence_in_dag["b_ancestors"].to_string(), &",".to_string());
    }

    let mut locally_created_coinbase_block_has_more_confidence_than_dag: bool = tmp_dag_confidence < tmp_local_confidence;
    // locally_created_coinbase_block_has_more_confidence_than_dag = false;// FIXME implement remote block confidence calcuilation
    if locally_created_coinbase_block_has_more_confidence_than_dag
    {
        dlog(
            &format!("More confidence: local coinbase({}) has more confidence({}) than DAG({}) in cycle range ({}, {})",
                     cutils::hash8c(&block.m_block_hash), tmp_local_confidence.to_string(), tmp_dag_confidence.to_string(), coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Trace);
    }

    let mut ancestors_diff: Vec<String> = cutils::arrayDiff(
        &block.m_ancestors,
        &tmp_dag_ancestors);
    if ancestors_diff.len() > 0
    {
        // try to remove repayBack blocks
        let existed_RpBlocks: QVDRecordsT = searchInDAG(
            &vec![
                &simple_eq_clause("b_type", constants::block_types::RpBlock),
                &ModelClause {
                    m_field_name: "b_hash",
                    m_field_single_str_value: "",
                    m_clause_operand: "IN",
                    m_field_multi_values: ancestors_diff.iter().map(|x| x.as_str()).collect::<Vec<&str>>(),
                },
            ],
            &vec!["b_hash"],
            &vec![],
            0,
            true);
        if existed_RpBlocks.len() > 0
        {
            let mut tmp: Vec<String> = vec![];
            for record in existed_RpBlocks
            {
                tmp.push(record["b_hash"].to_string());
            }
            ancestors_diff = cutils::arrayDiff(&ancestors_diff, &tmp);
        }
    }
    let locally_created_coinbase_block_has_more_ancestors_than_dag: bool = ancestors_diff.len() > 0;
    if locally_created_coinbase_block_has_more_ancestors_than_dag
    {
        dlog(
            &format!("More ancestors: local coinbase({}) has more ancestors({:?} than DAG({}) in cycle range ({}, {})",
                     cutils::hash8c(&block.m_block_hash.to_string()), block.m_ancestors, dump_vec_of_str(&tmp_dag_ancestors), coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Trace);
    }

    dlog(
        &format!("Is about to issuing coinbase block in cycle range ({}, {}) the block: {}",
                 coinbase_from, coinbase_to, serde_json::to_string(&block).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Trace);

    let missedBlocks: Vec<String> = getMissedBlocksToInvoke(0);
    if missedBlocks.len() > 0
    {
        dlog(
            &format!("BTW machine has some missed blocks: {}", dump_it(&missedBlocks)),
            constants::Modules::CB,
            constants::SecLevel::Warning);
    }

// FIXME: it is a way to avoid creating too many coinbases which have a little difference because of the ancestors.
// could it be a security issue? when an adversory in last minutes(before midnight or mid-day) starts to spam network by blocks
// and most of nodes can not be synched, so too many coinbase blocks creating
//
    if (locally_created_coinbase_block_has_more_confidence_than_dag || locally_created_coinbase_block_has_more_ancestors_than_dag)
        && (getMissedBlocksToInvoke(0).len() < 1)
    {

        // broadcast coin base
        if cutils::isInCurrentCycle(&block.m_block_creation_date.to_string())
        {
            let push_res: bool = pushIntoSendingQ(
                constants::block_types::Coinbase,
                &*block.m_block_hash,
                &*serde_json::to_string(&block).unwrap(),
                &format!("Broadcasting coinbase block CB({}) issued by({} for cycle range({}, {})",
                         &cutils::hash8c(&block.m_block_hash),
                         machine().getPubEmailInfo().m_address,
                         coinbase_from,
                         coinbase_to),
                &vec![],
                &vec![],
                false,
            );

            dlog(
                &format!("coinbase push1 res({})", dump_it(push_res)),
                constants::Modules::CB,
                constants::SecLevel::Trace);

            dlog(
                &format!("Coinbase issued because of clause 1 CB({}) issued by({} for cycle range({}, {})",
                         cutils::hash8c(&block.m_block_hash.to_string()), machine().getPubEmailInfo().m_address, coinbase_from, coinbase_to),
                constants::Modules::CB,
                constants::SecLevel::Trace);

            return;
        }
    } else if passedCertainTimeOfCycleToRecordInDAG(&cutils::get_now()) && !atleast_one_coinbase_block_exist
    {
        // another psudo random emulatore
        // if already passed more than 1/4 of cycle and still no coinbase block recorded in DAG,
        // so the machine has to create one
        if haveIFirstHashedEmail("desc") {
            let push_res: bool = pushIntoSendingQ(
                constants::block_types::Coinbase,
                &*block.m_block_hash,
                &serde_json::to_string(&block).unwrap(),
                &("Broadcasting coinbase block CB(".to_owned() + &cutils::hash8c(&block.m_block_hash) + ") issued by(" + &machine().getPubEmailInfo().m_address + " for cycle range(" + &coinbase_from + ", " + &coinbase_to + ")"),
                &vec![],
                &vec![],
                false);

            dlog(
                &format!("coinbase push2 res({})", dump_it(push_res)),
                constants::Modules::CB,
                constants::SecLevel::Trace);
            dlog(
                &format!("Coinbase issued because of clause 2 CB({}) issued by({} for cycle range({}, {})",
                         cutils::hash8c(&block.m_block_hash), &machine().getPubEmailInfo().m_address, &coinbase_from, &coinbase_to),
                constants::Modules::CB,
                constants::SecLevel::Trace);
            return;
        }
    } else {
        dlog(
            &format!("Coinbase can be issued by clause 3 but local hasn't neither more confidence nor more ancestors and still not riched to 1/4 of cycle time. CB({}) issued by({} for cycle range({}, {})",
                     cutils::hash8c(&block.m_block_hash), &machine().getPubEmailInfo().m_address, coinbase_from, coinbase_to),
            constants::Modules::CB,
            constants::SecLevel::Trace);
        return;
    }
}

