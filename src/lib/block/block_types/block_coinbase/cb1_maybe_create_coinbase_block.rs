use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, cutils};
use crate::lib::block::block_types::block_coinbase::cb3_control_coinbase_issuance_criteria::control_coinbase_issuance_criteria;
use crate::lib::block::block_types::block_coinbase::cb2_random_ordering_neighbors::make_email_hash_dictionary;
use crate::lib::block::block_types::block_coinbase::cb4_try_create_coinbase_block::try_create_coinbase_block;
use crate::lib::block::block_types::block_floating_signature::floating_signature_block::aggrigate_floating_signatures;
use crate::lib::constants;
use crate::lib::custom_types::{CDateT, CMPAIValueT, SharesCountT, QVDRecordsT, TimeByMinutesT};
use crate::lib::dag::dag::{search_in_dag};
use crate::lib::database::abs_psql::{ModelClause, simple_eq_clause};
use crate::lib::dlog::dlog;
use crate::lib::services::dna::dna_handler::{get_shares_info};
use crate::lib::utils::dumper::dump_hashmap_of_qvd_records;

#[derive(Clone)]
pub struct TmpHolder {
    pub(crate) holder: String,
    pub(crate) dividend: CMPAIValueT,
}

//old_name_was maybeCreateCoinbaseBlock
pub fn maybe_create_coinbase_block() -> bool
{
    let can_issue_new_cb = control_coinbase_issuance_criteria();
    if !can_issue_new_cb {
        return true;
    }
    return try_create_coinbase_block();
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
    return one_cycle_max_bili_pais * constants::MONEY_MAX_DIVISION;
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
c_date = application().now();

if (due == "")
due = application().now();

if (c_date > application().now())
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
c_date = application().now();

CMPAIValueT total_incomes = 0;
Vec<MonthCoinsReport> monthly_incomes {};
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
    let now_ = application().now();
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
        let ancestors = cutils::convert_comma_separated_string_to_string_vector(&block_record["b_ancestors"]);
        already_recorded_ancestors = cutils::array_add(
            &already_recorded_ancestors,
            &ancestors);
    }
    already_recorded_ancestors = cutils::array_unique(&already_recorded_ancestors);
    dlog(
        &format!("already Recorded Confidence from({}) {:#?}", current_cycle_range_from, already_recorded_confidents),
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

    let now_ = application().now();
    let (the_confidence, block_hashes, _backers) = aggrigate_floating_signatures(&now_);
    let not_recorded_blocks: Vec<String> = cutils::array_diff(&block_hashes, &already_recorded_ancestors);
    if (the_confidence > max_recorded_confident) || (not_recorded_blocks.len() > 0) {
        return true;
    }

    return false;
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
    let to_t_ = application().now();
    let res: bool = application().time_diff(from_t_, to_t_).as_seconds
        >= (cycle_by_minutes as f64 * 60.0 * constants::COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG * (1 + (machine_index.pow(7) / 131)) as f64) as u64;
    dlog(
        &format!("passed CertainTimeOfCycleToRecordInDAG? {}", res),
        constants::Modules::CB,
        constants::SecLevel::TmpDebug);

    return res;
}


