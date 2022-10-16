use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, ccrypto, constants, cutils, dlog};
use crate::cutils::{remove_quotes};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CDateT, ClausesT, JSonObject, TimeByMinutesT};
use crate::lib::database::abs_psql::{q_insert, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::{C_POLLING_PROFILES, C_POLLINGS};
use crate::lib::utils::version_handler::is_older_than;

/*
pub const POLLING_PROFILE_CLASSES: HashMap<String, JSonT> = json!({

      "Basic":
            json!({
                "ppName": "Basic",
                "activated": constants::YES,
                "performType": POLL_PERFORMANCE_TYPES::Transparent,
                "voteAmendmentAllowed": constants::NO,
                "votesCountingMethod": VOTE_COUNTING_METHODS::Plurality
              }),


      "ZKP":
            json!({
                "ppName": "ZKP",
                "activated":  constants::NO,
                "performType":  POLL_PERFORMANCE_TYPES::ZKP,
                "voteAmendmentAllowed":  constants::NO,
                "votesCountingMethod":  VOTE_COUNTING_METHODS::Plurality
              })

    });

*/

pub mod poll_performance_types
{
    pub const TRANSPARENT: &str = "Transparent";
    // Transparent
    #[allow(dead_code, unused)]
    pub const ZKP: &str = "ZKP"; // Zero-Knowladge Proof
}


pub mod vote_counting_methods {
    #[allow(dead_code, unused)]
    pub const PLURALITY: &str = "Plurality";
    // Plurality
    #[allow(dead_code, unused)]
    pub const PLURALITY_LOG: &str = "PluralityLog";
    // Plurality logarythmic and (x )minutes for Y/N/A (x+1/2) Minutes for N/A
    #[allow(dead_code, unused)]
    pub const TWO_ROUND_RUN_OFF: &str = "TwoRRo";
    //"Two-Round Runoff"
    #[allow(dead_code, unused)]
    pub const INSTANT_RUN_OFF: &str = "InstantRo";
    //"Instant Runoff"
    #[allow(dead_code, unused)]
    pub const BORDA_COUNT: &str = "BordaCount"; //"Borda Count"
}

//old_name_was autoCreatePollingForProposal
pub fn auto_create_polling_for_proposal(params: &mut JSonObject, block: &Block) -> bool
{
    if is_older_than(remove_quotes(&params["dVer"]), "0.0.8".to_string()) == 1 {
        params["dVer"] = "0.0.0".into();
    }

    let custom_stringified = stringify_an_auto_generated_proposal_polling(params);
    let doc_ext_hash = ccrypto::keccak256(&custom_stringified);
    dlog(
        &format!("custom Stringifyed Auto-generated polling (dExtHash: {}): {}",
                 doc_ext_hash, custom_stringified),
        constants::Modules::App,
        constants::SecLevel::Debug);
    params["dExtHash"] = doc_ext_hash.clone().into();

    let doc_length = cutils::padding_length_value(
        (custom_stringified.len() + doc_ext_hash.to_string().len()).to_string(),
        constants::LEN_PROP_LENGTH);
    params["dLen"] = doc_length.into();

    let (status, doc) = Document::load_document(params, block, 0);
    if !status
    {
        dlog(
            &format!("Failed in load pooling document {} ",
                     &params),
            constants::Modules::App,
            constants::SecLevel::Error);
        return false;
    }
    let pll_hash = doc.m_if_polling_doc.calc_doc_hash(&doc); //old name was calcHashDPolling

    let voting_timeframe: f64 = match remove_quotes(&params["pTimeframe"])
        .parse::<f64>() {
        Ok(t) => t,
        Err(e) => {
            println!("Failed in pooling Time frame(pTimeframe) value {} {}",
                     &params["pTimeframe"].to_string(), e);
            dlog(
                &format!("Failed in pooling Time frame(pTimeframe) value {} {}",
                         &params["pTimeframe"].to_string(), e),
                constants::Modules::App,
                constants::SecLevel::Fatal);
            0.0
        }
    };
    // let voting_timeframe: f64 = remove_quotes(&params["pTimeframe"].to_string()).parse::<f64>().unwrap();
    let abs_no_timeframe_by_minutes: TimeByMinutesT = (voting_timeframe.clone() * 60.0 * 1.5) as u64; // 90 = 60 minute + 30 minute => 1.5 * yesTime
    dlog(
        &format!("Abs and No timeframe by minutes = {}", abs_no_timeframe_by_minutes),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let pll_end_date: CDateT = application().minutes_after(
        abs_no_timeframe_by_minutes,
        &remove_quotes(&params["startDate"]));

    let pll_creator = remove_quotes(&params["dCreator"]);
    let pll_type = remove_quotes(&params["dType"]);
    let pll_class = remove_quotes(&params["dClass"]);
    let pll_ref = remove_quotes(&params["dRef"]);
    let pll_ref_type = remove_quotes(&params["dRefType"]);
    let pll_ref_class = remove_quotes(&params["dRefClass"]);
    let pll_start_date = remove_quotes(&params["startDate"]);
    let pll_timeframe = voting_timeframe;
    let pll_version = remove_quotes(&params["dVer"]);
    let pll_comment = remove_quotes(&params["dComment"]);
    let zero_i64: i64 = 0;
    let pll_status = remove_quotes(&params["status"]);
    let pll_ct_done = constants::NO;
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("pll_creator", &pll_creator as &(dyn ToSql + Sync)),
        ("pll_type", &pll_type as &(dyn ToSql + Sync)),
        ("pll_class", &pll_class as &(dyn ToSql + Sync)),
        ("pll_ref", &pll_ref as &(dyn ToSql + Sync)),  // referenced voting subject(could be proposalHash,..
        ("pll_ref_type", &pll_ref_type as &(dyn ToSql + Sync)), // referenced voting subject(could be proposalHash,..
        ("pll_ref_class", &pll_ref_class as &(dyn ToSql + Sync)),
        ("pll_start_date", &pll_start_date as &(dyn ToSql + Sync)),
        ("pll_end_date", &pll_end_date as &(dyn ToSql + Sync)), // end date is the deadline for Absatin/No voting
        ("pll_timeframe", &pll_timeframe as &(dyn ToSql + Sync)),
        ("pll_version", &pll_version as &(dyn ToSql + Sync)),
        ("pll_comment", &pll_comment as &(dyn ToSql + Sync)),
        ("pll_y_count", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_y_shares", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_y_gain", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_y_value", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_n_count", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_n_shares", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_n_gain", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_n_value", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_a_count", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_a_shares", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_a_gain", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_a_value", &zero_i64 as &(dyn ToSql + Sync)),
        ("pll_status", &pll_status as &(dyn ToSql + Sync)),
        ("pll_ct_done", &pll_ct_done as &(dyn ToSql + Sync)),
        ("pll_hash", &pll_hash as &(dyn ToSql + Sync))
    ]);
    dlog(
        &format!("Insert an auto-generated polling for proposal values: {:?}", &values),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    return q_insert(
        C_POLLINGS,
        &values,
        true,
    );
}

//old_name_was stringifyAutoGeneratedProposalPolling
pub fn stringify_an_auto_generated_proposal_polling(params: &JSonObject) -> String
{
    // stringifies auto-created polling for proposal
    let doc_hahsables: String = format!(
        "dCDate:{},dClass:{},dComment:{},dRef:{},dRefClass:{},dRefType:{},dType:{},dVer:{},dCreator:{},pTimeframe:{}",
        params["startDate"],
        params["dClass"],
        params["dComment"],
        params["dRef"],
        params["dRefClass"],
        params["dRefType"],
        params["dType"],
        params["dVer"],
        params["dCreator"],
        params["pTimeframe"]
    );
    return doc_hahsables;
}

//old_name_was updatePolling
pub fn update_polling(
    upd_values: &HashMap<&str, &(dyn ToSql + Sync)>,
    clauses: ClausesT,
    is_transactional: bool) -> (bool, String)
{
    q_update(
        C_POLLINGS,
        upd_values, // update values
        clauses,  // update clauses
        is_transactional,
    );

    return (true, "".to_string());
}

//old_name_was initPollingProfiles
pub fn init_polling_profiles()
{

    //  String rrr = POLLING_PROFILE_CLASSES.value("Basic").value("zzz").to_string();
    //  auto sss3 = POLLING_PROFILE_CLASSES["Basic"]["ppName"].to_string();
    //  auto sy3 = POLLING_PROFILE_CLASSES["Basic"]["voteAmendmentAllowed"].to_string();
    //  auto sss = POLLING_PROFILE_CLASSES["ZKP"]["ppName"].to_string();
    //  auto ss2 = POLLING_PROFILE_CLASSES["ZKP"]["votesCountingMethod"].to_string();


    let (_status, records) = q_select(
        C_POLLING_PROFILES,
        vec!["ppr_name"],
        vec![
            simple_eq_clause("ppr_name", &"Basic".to_string()), // POLLING_PROFILE_CLASSES["Basic"]["ppName"].to_string(),
        ],
        vec![],
        1,
        false,
    );

    if records.len() > 0
    { return; }


    // for a_polling_profile_key in POLLING_PROFILE_CLASSES.keys()
    // {
    let ppr_name = "Basic".to_string();
    let ppr_activated = constants::YES.to_string();
    let ppr_perform_type = poll_performance_types::TRANSPARENT.to_string();
    let ppr_amendment_allowed = constants::NO.to_string();
    let ppr_votes_counting_method = vote_counting_methods::PLURALITY.to_string();
    let ppr_version = "0.0.8".to_string();
    let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
        ("ppr_name", &ppr_name as &(dyn ToSql + Sync)),//&*POLLING_PROFILE_CLASSES[a_polling_profile_key]["ppName"].to_string()
        ("ppr_activated", &ppr_activated as &(dyn ToSql + Sync)),//&*POLLING_PROFILE_CLASSES[a_polling_profile_key]["activated"].to_string()),
        ("ppr_perform_type", &ppr_perform_type as &(dyn ToSql + Sync)),//&*POLLING_PROFILE_CLASSES[a_polling_profile_key]["performType"].to_string()),
        ("ppr_amendment_allowed", &ppr_amendment_allowed as &(dyn ToSql + Sync)),//&*POLLING_PROFILE_CLASSES[a_polling_profile_key]["voteAmendmentAllowed"].to_string()),
        ("ppr_votes_counting_method", &ppr_votes_counting_method as &(dyn ToSql + Sync)),//&*POLLING_PROFILE_CLASSES[a_polling_profile_key]["votesCountingMethod"].to_string()),
        ("ppr_version", &ppr_version as &(dyn ToSql + Sync)),
    ]);
    q_insert(
        C_POLLING_PROFILES,
        &values,
        false,
    );
    // }
}

/*

TimeByHoursT normalizeVotingTimeframe(TimeByHoursT voting_timeframe)
{
  if (application().cycle() == 1)
     return static_cast<uint64_t>(voting_timeframe);
  // because of test ambient the longivity can be float and less than 1 hour
  return cutils::customFloorFloat(static_cast<double>(voting_timeframe), 2);
  // return cutils::CFloor(voting_timeframe * 100.0) / 100.0;
}

TimeByHoursT getMinVotingTimeframe()
{
  TimeByHoursT voting_timeframe = (cutils::get_cycle_by_minutes() * 2.0) / 60.0;   // at least 2 cycle for voting
  return normalizeVotingTimeframe(voting_timeframe);
}

bool removePollingByRelatedProposal(const CDocHashT& proposal_hash)
{
  //sceptical test
  QueryRes exist = DbModel::select(
    stbl_pollings,
    {"pll_ref_type"},
    {{"pll_ref_type", constants::POLLING_REF_TYPE::Proposal},
    {"pll_ref", proposal_hash}});
  if (exist.records.len() != 1)
  {
    CLog::log("Try to delete polling strange result!  " + cutils::dumpIt(exist.records), "sec", "error");
    return false;
  }

  DbModel::dDelete(
    stbl_pollings,
    {{"pll_ref_type", constants::POLLING_REF_TYPE::Proposal},
        {"pll_ref", proposal_hash}});

  return true;
}


bool removePollingG(const CDocHashT& polling_hash)
{
  DbModel::dDelete(
    stbl_pollings,
    {{"pll_hash", polling_hash}});

  return true;
}

/**
*
* @param {*} voteCreationTimeByMinute      : timeDiff by minutes (VotingDateRecordedInBlock.creation Date - dateOfStartVoting)
* @param {*} voteReceiveingTimeByMinute    : timeDiff by minutes (receivingDate - dateOfStartVoting)
* @param {*} voting_timeframe_by_minute
*/
std::tuple<double, double, double> calculateVoteGain(
  double voteCreationTimeByMinute,
  double voteReceiveingTimeByMinute,
  TimeByMinutesT voting_timeframe_by_minute)
{
  if (voting_timeframe_by_minute == 0)
    voting_timeframe_by_minute = cutils::get_cycle_by_minutes() * 2;

  TimeByMinutesT offset = cutils::get_cycle_by_minutes() / 4; // 3 hour(1/4 cycle) will be enough to entire network to be synched
  TimeByMinutesT minRange = cutils::get_cycle_by_minutes() * 2; // 24 hour(2 cycle)
  double latenancyFactor = log(voteReceiveingTimeByMinute - voteCreationTimeByMinute + minRange) / log(minRange);

  double gainYes = 0;
  if (voteCreationTimeByMinute < voting_timeframe_by_minute)
  {
    gainYes = (log(voting_timeframe_by_minute - voteCreationTimeByMinute + 1) / log(voting_timeframe_by_minute + offset)) / latenancyFactor;
    gainYes = cutils::customFloorFloat(gainYes, 2);
  }

  double gainNoAbstain = 0;
  if (voteCreationTimeByMinute < (voting_timeframe_by_minute * 1.5))
  {
    gainNoAbstain = (log((voting_timeframe_by_minute * 1.5) - voteCreationTimeByMinute + 1) / log((voting_timeframe_by_minute * 1.5) + offset)) / latenancyFactor;
    gainNoAbstain = cutils::customFloorFloat(gainNoAbstain, 2);
  }

  return {gainYes, gainNoAbstain, latenancyFactor};

  // primitive Sigmoidal Membership Function implementaition
  // TODO: improvement needed, based on network delay propogation and ...
}


/**
 *
 * @param {*} polling_hash
 * it re-calculate all Ballots and refreshes the polling final results
 */
bool maybeUpdatePollingStat(const CDocHashT& polling_hash)
{
  QVDRecordsT pollings = searchInPollings({{"pll_hash", polling_hash}});
  if (pollings.len()== 0)
  {
    CLog::log("Polling does not exist in DAG! polling(" + cutils::hash8c(polling_hash) + ")", "app", "debug");
    return false;
  }
  QVDicT polling = pollings[0];
  CLog::log("retrieve polling(" + cutils::hash8c(polling_hash) + ") info: " + cutils::dumpIt(polling), "app", "trace");

  // this control is commented because in synching process there is moment in which a polling is closed before all ballots beeing considered
  // TODO: FIXIT ASAP
  // // control if polling still is open & active
  // if (polling.pllStatus == iConsts.CONSTS.CLOSE)
  //     return { err: false, msg: `polling(${utils.hash6c(polling_hash)}) already closed` };


  QueryRes ballots = DbModel::select(
    stbl_ballots,
    stbl_ballots_fields,
    {{"ba_pll_hash", polling_hash}});

  double yesPeriod = polling.value("pll_timeframe").toDouble() * 60;

  PollingStatistics vStatistics = {};
  for (QVDicT aBall: ballots.records)
  {

    auto[gainYes, gainNoAbstain, latenancy_] = calculateVoteGain(
      aBall.value("ba_vote_c_diff").toDouble(),
      aBall.value("ba_vote_r_diff").toDouble(),
      yesPeriod);
    CLog::log(
      "CreationDif: " + String::number(aBall.value("ba_vote_c_diff").toDouble()) +
      ", receiveDiff: " + String::number(aBall.value("ba_vote_r_diff").toDouble()) +
      ", yesPeriod: " + String::number(yesPeriod) + ") " +
      "gainYes(" + String::number(gainYes) + ") " +
      "gainNoAbstain(" + String::number(gainNoAbstain) + ") " +
      "latenancy_(" + String::number(latenancy_) + ") ");

    DNAShareCountT shares = aBall.value("ba_voter_shares").toDouble();

    int16_t ba_vote = aBall.value("ba_vote").toInt();
    if (ba_vote > 0) {
        vStatistics.m_yes_count++;
        vStatistics.m_yes_shares += shares;
        vStatistics.m_yes_gain += cutils::CFloor(shares * gainYes);
        vStatistics.m_yes_value += cutils::CFloor(vStatistics.m_yes_gain * ba_vote);

    } else if (ba_vote == 0) {
        vStatistics.m_abstain_count++;
        vStatistics.m_abstain_shares += shares;
        vStatistics.m_abstain_gain += cutils::CFloor(shares * gainNoAbstain);
        vStatistics.m_abstain_value += 0;

    } else if (ba_vote < 0) {
        vStatistics.m_no_count++;
        vStatistics.m_no_shares += shares;
        vStatistics.m_no_gain += cutils::CFloor(shares * gainNoAbstain);
        vStatistics.m_no_value += cutils::CFloor(vStatistics.m_no_gain * abs(ba_vote));

    }
  }

  vStatistics.m_polling_status = (polling.value("pll_end_date").to_string() < application().now()) ? constants::CLOSE : constants::OPEN;

  CLog::log("vStatistics: " + vStatistics.dumpMe(), "app", "trace");

  // update polling info
  DbModel::update(
    stbl_pollings,
    {{"pll_status", vStatistics.m_polling_status},
    {"pll_y_count", vStatistics.m_yes_count},
    {"pll_y_shares", vStatistics.m_yes_shares},
    {"pll_y_gain", vStatistics.m_yes_gain},
    {"pll_y_value", vStatistics.m_yes_value},
    {"pll_n_count", vStatistics.m_no_count},
    {"pll_n_shares", vStatistics.m_no_shares},
    {"pll_n_gain", vStatistics.m_no_gain},
    {"pll_n_value", vStatistics.m_no_value},
    {"pll_a_count", vStatistics.m_abstain_count},
    {"pll_a_shares", vStatistics.m_abstain_shares},
    {"pll_a_gain", vStatistics.m_abstain_gain},
    {"pll_a_value", vStatistics.m_abstain_value}},
    {{"pll_hash", polling_hash}});

  return true;
}

bool treatPollingWon(
  const QVDicT& polling,
  const CDateT& approve_date)
{
  /**
  * depends on pll_ref_type needs different treatment. there are a bunch of reserved pollings(e.g. Proposal, AdmPollings,...)
  */
  if (polling.value("pll_ref_type").to_string() == constants::POLLING_REF_TYPE::Proposal)
  {
    ProposalHandler::transformApprovedProposalToDNAShares(
      polling,
      approve_date);

  }else if (polling.value("pll_ref_type").to_string() == constants::POLLING_REF_TYPE::AdmPolling) {
    SocietyRules::treatPollingWon(
      polling,
      approve_date);

  };


    // case iConsts.POLLING_REF_TYPE.ReqForRelRes:
    // TODO move it to up "admPollingsHandler.onChain.treatPollingWon"
    //     const reservedHandler = require('../../dag/coinbase/reserved-coins-handler');
    //     clog.app.info(`controll if ReqForRelRes polling(${utils.hash6c(polling.pllHash)}) end date(${polling_end_date}) is before the previous cycle end(${compare_date})`);
    //     if (polling_end_date < compare_date) {
    //         let cTRes = reservedHandler.doReqRelConcludeTreatment({ polling: polling, polling_end_date, compare_date });
    //     }

  return true;
}

void treatPollingMissed(
  const QVDicT& polling,
  const CDateT& approve_date)
{
  String pll_ref_type = polling.value("pll_ref_type").to_string();

  /**
  * depends on pll_ref_type needs different treatment. there are a bunch of reserved pollings(e.g. Proposal, AdmPollings,...)
  */
  if (pll_ref_type == constants::POLLING_REF_TYPE::Proposal)
  {
    ProposalHandler::concludeProposal(
      polling,
      approve_date);

  }else if (pll_ref_type == constants::POLLING_REF_TYPE::AdmPolling)
  {
    SocietyRules::concludeAdmPolling(
      polling,
      approve_date);

  }

}

void doOnePollingConcludeTreatment(
  QVDicT aPolling,
  const CDateT& c_date)
{
//  CDateT polling_end_date = cutils::minutesAfter(aPolling.value("pll_timeframe").toDouble() * 60 * 1.5, aPolling.value("pll_start_date").to_string());
  CDateT polling_end_date = aPolling.value("pll_end_date").to_string();
  CDateT approve_date_ = cutils::minutesAfter(cutils::get_cycle_by_minutes() * 2, polling_end_date);
  CDateT approve_date = cutils::get_coinbase_range(approve_date_).from;
  CLog::log("retrive for conclude Treatment polling(" + cutils::hash8c(aPolling.value("pll_hash").to_string()) + ") polling_end_date(" + polling_end_date + ")", "app", "info");

  // generaly Close the pollings which are finished the voting time
  if (polling_end_date < application().now())
  {
    maybeUpdatePollingStat(aPolling.value("pll_hash").to_string());

    // retrieve (maybe)updated info
    QVDRecordsT recs = searchInPollings({{"pll_hash", aPolling.value("pll_hash").to_string()}});
    if( recs.len() > 0)
      aPolling = recs[0];
  }

  // compare_date is the start date of last cycle, so ONLY pollings with close date before compare date can be consider and treat the conclude treatments
  CDateT compare_date = cutils::getACycleRange(c_date, 1).from;
  CLog::log("retrive for conclude Treatment polling(" + cutils::hash8c(aPolling.value("pll_hash").to_string()) + ") compare_date(" + compare_date + ")");

  /**
  * TODO: implement a way to run a customized contract as a conclude treatment, at least as a plugin on some nodes.
  * somethig like PledgeP conclude by Arbiters
  */
  CLog::log("controll if polling(" + cutils::hash8c(aPolling.value("pll_hash").to_string()) + ") end date(" + polling_end_date + ") is before the previous cycle end(" + compare_date + ")");
  QVDicT latest_block_in_DAG = DAG::getLatestRecordedBlcok();
  if (polling_end_date < latest_block_in_DAG.value("b_creation_date").to_string())
  {
    CLog::log("comparing pllYValue(" + String::number(aPolling.value("pll_y_value").toDouble()) + " & pllNValue(" + String::number(aPolling.value("pll_n_value").toDouble()) + ")");
    if (aPolling.value("pll_y_value").toDouble() >= aPolling.value("pll_n_value").toDouble())
    {

//      {
//        // remove this code block after tests.  if (polling_end_date < last block inserted in DAG)
//        if (aPolling.value("pll_hash").to_string() == "e8e406b6362521637b31fa308f6879d4aee00d23e34ceda614dcdb9d6f181a8a")
//        {
//          QVDicT latest_block_in_DAG = DAG::getLatestRecordedBlcok();
//          if (polling_end_date < latest_block_in_DAG.value("b_creation_date").to_string())
//          {
//            CLog::log("nnnnnnnnnnnooooooooooooo even new method doesnot work" + cutils::dumpIt(aPolling));

//          }else{
//            CLog::log("yyyyyyyes it works" + cutils::dumpIt(aPolling));

//          }
//          CLog::log("ooooooooooooo" + cutils::dumpIt(aPolling));
//        }
//      }

      CLog::log("conclude Winer polling(" + cutils::hash8c(aPolling.value("pll_hash").to_string()) + ") aPolling: " + cutils::dumpIt(aPolling));
      treatPollingWon(
        aPolling,
        approve_date);

    } else {
      CLog::log("Conclude Missed polling(" + cutils::hash8c(aPolling.value("pll_hash").to_string()) + ") Polling info: " + cutils::dumpIt(aPolling));
      treatPollingMissed(
        aPolling,
        approve_date);

    }

    // mark polling as conclude treatment done
    DbModel::update(
      stbl_pollings,
      {{"pll_ct_done", constants::YES }},
      {{"pll_hash", aPolling.value("pll_hash").to_string()}});
  }

  // custome polling-conclude-treatments
  if (polling_end_date < compare_date)
  {
    if (aPolling.value("pll_y_value").toDouble() >= aPolling.value("pll_n_value").toDouble())
    {
      // listener.doCallSync('SASH_custom_polling_winer', { polling: aPolling });
    } else {
      // listener.doCallSync('SASH_custom_polling_missed', { polling: aPolling });
    }
    // listener.doCallSync('SASH_custom_polling_conclude_treatment_done', { polling: aPolling });
  }
}

// js name was doPollingConcludeTreatment
bool loopConcludeTreatment(
  const CDateT& c_date,
  bool force_to_update)
{
  String thread_prefix = "conclude_treatment_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    doConcludeTreatment(c_date, force_to_update);

    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getConcludeTreatmentGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Conclude Treatment");
  return true;
}
*/


// js name was doPollingConcludeTreatment
//old_name_was doConcludeTreatment
pub fn do_conclude_treatment(
    _c_date: &CDateT,
    _force_to_update: bool) -> bool
{
    /*

      if (CMachine::is_in_sync_process())
        force_to_update = true; // in sync mode it is possible the ballots receiving after concluding the polling, so we always update the pollings. FIXME: it has CPU over head on sync process, improve it ASAP

      // select the pollings which conclude treatment = No, and are closed in previous cycle
      ClausesT clauses {};
      if (force_to_update)
      {
        /**
         * Add an start date limitation filter to not fetching all pollings
         * But only after than 60 cycles (almost one month) ago
         * TODO: fixit for polling which duration is longer than one month and in synching/booting a node maybe makes problem
         **/
        if (CMachine::is_in_sync_process())
        {
          auto[status_, lastWBLock] = DAG::getLatestBlockRecord();
          Q_UNUSED(status_);
          CDateT oldestDate = cutils::getACycleRange(lastWBLocHashMap<&str, &(dyn ToSql + Sync)>, 10).from;
          clauses.push({"pll_end_date", oldestDate, ">="});
        }

      } else {
        clauses.push({"pll_ct_done", constants::NO});
      }
      QVDRecordsT pollings = searchInPollings(clauses);
      for (QVDicT a_polling: pollings)
      {
        doOnePollingConcludeTreatment(a_polling, c_date);
      }
    */
    return true;
}

/*
bool recordPollingInDB(
  const Block& block,
  const PollingDocument* polling,
  const String& pll_status)
{
  CLog::log("record A general polling document: " + cutils::dumpIt(polling), "app", "trace");

  QVDicT values {
    {"pll_hash", polling.m_doc_hash},
    {"pll_creator", polling.m_polling_creator},
    {"pll_type", polling.m_doc_type},
    {"pll_class", polling.m_doc_class}, // it is equal to pollingDoc.dClass
    {"pll_ref", polling.m_polling_ref},  // referenced voting subject(could be proposalHash,...)
    {"pll_ref_type", polling.m_polling_ref_type},  // referenced voting subject(could be proposalHash,...)
    {"pll_ref_class", polling.m_polling_ref_class},  // referenced voting subject(could be proposalHash,...)
    {"pll_start_date", block.m_block_creation_date}, // TODO: improve to user can set different start date(at least one cycle later than block.creation Date)
    {"pll_end_date", cutils::minutesAfter(polling.m_voting_timeframe * 90, block.m_block_creation_date)}, // TODO: improve to user can set different start date(at least one cycle later than block.creation Date)
    {"pll_timeframe", polling.m_voting_timeframe},
    {"pll_version", polling.m_doc_version},
    {"pll_comment", polling.m_polling_comment},
    {"pll_y_count", 0},
    {"pll_y_shares", 0},
    {"pll_y_gain", 0},
    {"pll_y_value", 0},
    {"pll_n_count", 0},
    {"pll_n_shares", 0},
    {"pll_n_gain", 0},
    {"pll_n_value", 0},
    {"pll_a_count", 0},
    {"pll_a_shares", 0},
    {"pll_a_gain", 0},
    {"pll_a_value", 0},
    {"pll_status", pll_status}
  };
  CLog::log("record A general polling values: " + cutils::dumpIt(values), "app", "trace");

  DbModel::insert(
    stbl_pollings,
    values);
  return true;
}



bool removePollingByRelatedAdmPolling(const CDocHashT& adm_polling_doc_hash)
{
  //sceptical test
  QueryRes exist = DbModel::select(
    stbl_pollings,
    {"pll_ref"},
    {{"pll_ref_type", constants::POLLING_REF_TYPE::AdmPolling},
    {"pll_ref", adm_polling_doc_hash}});
  if (exist.records.len() != 1)
  {
    CLog::log("Try to delete polling(" + adm_polling_doc_hash + ") strange result! records" + cutils::dumpIt(exist.records), "sec", "warning");
    return false;
  }

  DbModel::dDelete(
    stbl_pollings,
    {{"pll_ref_type", constants::POLLING_REF_TYPE::AdmPolling},
    {"pll_ref", adm_polling_doc_hash}});

  return true;
}

void maybeUpdateOpenPollingsStat()
{

  ClausesT clauses {};
  /**
   * this query is because in synching process there is moment in which
   * a polling is closed before all ballots beeing considered
   * so we re-processing ALL pollings
   */
  if (!CMachine::is_in_sync_process())
    clauses.push({"pll_status", constants::OPEN});
  QueryRes open_pollings = DbModel::select(
    stbl_pollings,
    {"pll_hash"},
    clauses);
  CLog::log("maybe Update open_pollingsStat: " + cutils::dumpIt(open_pollings.records), "app", "trace");
  for (QVDicT a_polling: open_pollings.records)
    maybeUpdatePollingStat(a_polling.value("pll_hash").to_string());
}

QVDRecordsT searchInPollingProfiles(
  const ClausesT& clauses,
  const VString& fields,
  const OrderT order,
  const uint64_t limit)
{
  QueryRes res = DbModel::select(
    stbl_polling_profiles,
    fields,
    clauses,
    order,
    limit);
  return res.records;
}

QVDRecordsT searchInPollings(
  const ClausesT& clauses,
  const VString& fields,
  const OrderT order,
  const uint64_t limit)
{
  QueryRes res = DbModel::select(
    stbl_pollings,
    fields,
    clauses,
    order,
    limit);

  return res.records;
}

GenRes retrievePollingByProposalHash(const CDocHashT& proposal_hash)
{
  String msg;
  QVDRecordsT pollings = searchInPollings({{"pll_ref", proposal_hash}});
  if (pollings.len() != 1)
  {
    msg = "Failed in retrieve polling for proposal(" + cutils::hash8c(proposal_hash) + ")";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CDocHashT polling_hash = pollings[0].value("pll_hash").to_string();
  return {true, polling_hash};
}

GenRes retrievePollingByAdmPollingHash(const CDocHashT& adm_polling_hash)
{
  String msg;
  QVDRecordsT pollings = searchInPollings({{"pll_ref", adm_polling_hash}});
  if (pollings.len() != 1)
  {
    msg = "Failed in retrieve polling for adm-polling(" + cutils::hash8c(adm_polling_hash) + ")";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CDocHashT polling_hash = pollings[0].value("pll_hash").to_string();
  return {true, polling_hash};
}

GenRes maybeUpdatePollingStatByProposalHash(const CDocHashT& proposal_hash)
{
  GenRes polling_res = retrievePollingByProposalHash(proposal_hash);
  if (!polling_res.status)
    return polling_res;

  maybeUpdatePollingStat(polling_res.msg);
  CGUI::signalUpdateContributes();
  return {true, ""};
}

GenRes maybeUpdatePollingStatByAdmPollingHash(const CDocHashT& adm_polling_hash)
{
  CLog::log("maybe Update Polling Stat By Adm Polling Hash " + adm_polling_hash);
  GenRes polling_res = retrievePollingByAdmPollingHash(adm_polling_hash);
  if (!polling_res.status)
    return polling_res;

  maybeUpdatePollingStat(polling_res.msg);
  CGUI::signalUpdateContributes();
  return {true, ""};
}


std::tuple<bool, String, PollingDocument*> prepareNewPolling(
  const CAddressT& document_creator,
  TimeByHoursT voting_timeframe,
  const CDocHashT& documnet_ref,
  const CDocHashT& documnet_ref_type,
  const CDocHashT& documnet_ref_class,
  const uint64_t voters_count,
  const String& document_type,
  const String& document_class,
  const String& documnet_comment,
  const CDateT& creation_date)
{
  // TODO: add optional startDate(a date in future & superiore than one cycle) in which votting will be started,
  // currently voting start date is equal the block in which placed request
  JSonObject polling_json {
    {"dType", document_type},
    {"dClass", document_class},
    {"dVer", "0.0.9"},
    {"dComment", documnet_comment},
    {"dRef", documnet_ref},    // reference pollingHash
    {"dRefType", documnet_ref_type},    // reference pollingHash
    {"dRefClass", documnet_ref_class},    // reference pollingHash
    {"dCDate", creation_date}, // creation date
    {"pTimeframe", normalizeVotingTimeframe(voting_timeframe)},
    {"dCreator", document_creator}    // the bech32 address of shareholder
  };
  CLog::log("polling_json: " + cutils::serializeJson(polling_json), "app", "info");
  PollingDocument* polling_doc = new PollingDocument(polling_json);
  polling_doc.m_potential_voters_count = voters_count;

  String sign_message = polling_doc->getDocSignMsg();
  auto[sign_status, res_msg, sign_signatures, sign_unlock_set] = sign_by_an_address(
    document_creator,
    sign_message);
  if (!sign_status)
  {
    CLog::log(res_msg, "app", "error");
    return {false, res_msg, nullptr};
  }

  polling_doc.m_doc_ext_info = SignatureStructureHandler::compactUnlockersArray(JSonArray {JSonObject {
    {"uSet", sign_unlock_set},
    {"signatures", cutils::convertStringListToJSonArray(sign_signatures)}}});
  polling_doc.set_doc_ext_hash();
  polling_doc.set_doc_length();
  polling_doc->setDocHash();

  CLog::log("prepared polling request: " + polling_doc->safe_stringify_doc(true), "app", "info");

  return {true, "", polling_doc};
}

std::tuple<bool, String> makeReqForAdmPolling(
  const String& polling_subject,
  TimeByHoursT voting_timeframe,
  double the_value,
  uint64_t voters_count,
  String doc_comment,
  const CDateT& c_date,
  const CAddressT& the_creator,
  const String& polling_doc_type,
  const String& polling_doc_class)
{
  String msg;

  if (voting_timeframe < getMinVotingTimeframe())
  {
    msg = "Invalid polling time frame " + String::number(voting_timeframe);
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  if (doc_comment == "")
  {
    doc_comment = "Request For Administrative polling to refine(" + polling_subject + ") to (" + String::number(the_value) + ")";
  }

  JSonObject polling_values {
    {"pTimeframe", voting_timeframe},
  };

  if (VString{POLLING_TYPES::RFRfPLedgePrice,
  POLLING_TYPES::RFRfPollingPrice,
  POLLING_TYPES::RFRfTxBPrice,
  POLLING_TYPES::RFRfClPLedgePrice,
  POLLING_TYPES::RFRfDNAPropPrice,
  POLLING_TYPES::RFRfBallotPrice,
  POLLING_TYPES::RFRfINameRegPrice,
  POLLING_TYPES::RFRfINameBndPGPPrice,
  POLLING_TYPES::RFRfINameMsgPrice,
  POLLING_TYPES::RFRfFPostPrice,
  POLLING_TYPES::RFRfBasePrice,
  POLLING_TYPES::RFRfBlockFixCost}.contains(polling_subject))
  {
    polling_values["pFee"] = the_value;

  } else if (VString{POLLING_TYPES::RFRfMinS2Wk,
    POLLING_TYPES::RFRfMinS2DA,
    POLLING_TYPES::RFRfMinS2V,
    POLLING_TYPES::RFRfMinFSign,
    POLLING_TYPES::RFRfMinFVote}.contains(polling_subject))
  {
    polling_values["pShare"] = the_value;

  }


  // 1. create a adm polling js doc
  JSonObject adm_polling_json {
    {"dType", polling_doc_type},
    {"dClass", polling_doc_class},
    {"dVer", "0.0.9"},
    {"dComment", doc_comment},
    {"pSubject", polling_subject},
    {"pValues", polling_values},
    {"dCDate", c_date},
    {"dCreator", the_creator}};    // the bech32 address of shareholde}r

  AdministrativePollingDocument* adm_polling_doc = new AdministrativePollingDocument(adm_polling_json);

  String sign_message = adm_polling_doc->getDocSignMsg();
  auto[sign_status, res_msg, sign_signatures, sign_unlock_set] = sign_by_an_address(
    the_creator,
    sign_message);
  if (!sign_status)
  {
    CLog::log(res_msg, "app", "error");
    return {false, res_msg};
  }

  adm_polling_doc.m_doc_ext_info = SignatureStructureHandler::compactUnlockersArray(JSonArray {JSonObject {
    {"uSet", sign_unlock_set},
    {"signatures", cutils::convertStringListToJSonArray(sign_signatures)}}});
  adm_polling_doc.set_doc_ext_hash();

  adm_polling_docset_doc_length();
  adm_polling_doc->setDocHash();

  CLog::log("prepared adm-polling request: " + adm_polling_doc->safe_stringify_doc(true), "app", "info");

  // 2. create a administrative-polling for block release
  auto [polling_status, polling_msg, polling_doc] = prepareNewPolling(
    the_creator,
    voting_timeframe,
    adm_polling_doc->get_doc_hash(),
    adm_polling_doc.m_doc_type,
    adm_polling_doc.m_doc_class,
    voters_count,
    constants::document_types::Polling,
    POLLING_PROFILE_CLASSES["Basic"]["ppName"].to_string(),
    doc_comment,
    c_date);

  // 3. create trx to pay ReqForRelRes doc cost
  // calculate ballot cost
  auto[adm_cost_status, adm_dp_cost] = adm_polling_doc.calc_doc_data_and_process_cost(
    constants::stages::Creating,
    application().now());
  if (!adm_cost_status)
  {
    msg = "Failed in Adm-polling calculation for: " + adm_polling_doc.m_doc_comment;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto[coins_status1, coins_msg1, spendable_coins1, spendable_amount1] = Wallet::getSomeCoins(
    cutils::CFloor(adm_dp_cost * 1.3),  // an small portion bigger to support DPCosts
    constants::COIN_SELECTING_METHOD::PRECISE);
  if (!coins_status1)
  {
    msg = "Failed in finding coin to spend for Adm polling (" + adm_polling_doc.m_doc_comment + ") " + coins_msg1;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  // create a transaction for payment
  auto changeback_res1 = Wallet::getAnOutputAddress(true);
  if (!changeback_res1.status)
  {
    msg = "Failed in create changeback address for Adm polling!";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CAddressT change_back_address1 = changeback_res1.msg;
  Vec<TOutput> outputs1 {
    TOutput{change_back_address1, 1, constants::OUTPUT_CHANGE_BACK},
    TOutput{"TP_ADM_POLLING", adm_dp_cost, constants::OUTPUT_TREASURY}};

  auto trx_template1 = BasicTransactionTemplate {
    spendable_coins1,
    outputs1,
    static_cast<CMPAIValueT>(adm_dp_cost * 1.3),  // max DPCost
    0,    // pre calculated dDPCost
    "Payed for adm-polling cost",
    adm_polling_doc->get_doc_hash()};
  auto[tx_status, res_msg1, adm_polling_payer_trx, dp_cost1] = make_a_transaction(trx_template1);
  if (!tx_status)
    return {false, res_msg1};

  CLog::log("Signed trx for adm-polling cost: " + adm_polling_payer_trx->safe_stringify_doc(true), "app", "info");

  // mark UTXOs as used in local machine
  locally_mark_coin_as_used(adm_polling_payer_trx);

  // 4. create trx to pay real polling costs
  auto[polling_cost_status, polling_dp_cost] = polling_doc.calc_doc_data_and_process_cost(
    constants::stages::Creating,
    application().now());
  if (!polling_cost_status)
  {
    msg = "Failed in polling calculation for: " + adm_polling_doc.m_doc_comment;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto[coins_status2, coins_msg2, spendable_coins2, spendable_amount2] = Wallet::getSomeCoins(
    cutils::CFloor(polling_dp_cost * 1.3),  // an small portion bigger to support DPCosts
    constants::COIN_SELECTING_METHOD::PRECISE);
  if (!coins_status2)
  {
    msg = "Failed in finding coin to spend for Adm polling (" + adm_polling_doc.m_doc_comment + ")";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  // create a transaction for payment
  auto changeback_res2 = Wallet::getAnOutputAddress(true);
  if (!changeback_res2.status)
  {
    msg = "Failed in create changeback address for Adm polling!";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CAddressT change_back_address2 = changeback_res2.msg;
  Vec<TOutput> outputs2 {
    TOutput{change_back_address2, 1, constants::OUTPUT_CHANGE_BACK},
    TOutput{"TP_POLLING", polling_dp_cost, constants::OUTPUT_TREASURY}};

  auto trx_template2 = BasicTransactionTemplate {
    spendable_coins2,
    outputs2,
    static_cast<CMPAIValueT>(polling_dp_cost * 1.3),  // max DPCost
    0,    // pre calculated dDPCost
    "Payed for polling cost",
    polling_doc->get_doc_hash()};
  auto[tx_status2, res_msg2, polling_payer_trx, dp_cost2] = make_a_transaction(trx_template2);
  if (!tx_status2)
    return {false, res_msg2};

  CLog::log("Signed trx for polling cost: " + polling_payer_trx->safe_stringify_doc(true), "app", "info");

  // mark UTXOs as used in local machine
  locally_mark_coin_as_used(polling_payer_trx);


  auto[push_res1, push_msg1] = push_to_block_buffer(adm_polling_doc, adm_dp_cost);
  if (!push_res1)
  {
    msg = "Failed in push to block buffer adm-polling(" + adm_polling_doc.m_doc_comment + ") " + push_msg1;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  auto[push_res2, push_msg2] = push_to_block_buffer(adm_polling_payer_trx, adm_polling_payer_trx->getDocCosts());
  if (!push_res2)
  {
    msg = "Failed in push to block buffer trx2 (" + adm_polling_payer_trx->get_doc_hash() + ") " + push_msg2;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto[push_res3, push_msg3] = push_to_block_buffer(polling_doc, polling_dp_cost);
  if (!push_res3)
  {
    msg = "Failed in push to block buffer polling(" + adm_polling_doc.m_doc_comment + ") " + push_msg3;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  auto[push_res4, push_msg4] = push_to_block_buffer(polling_payer_trx, polling_payer_trx->getDocCosts());
  if (!push_res4)
  {
    msg = "Failed in push to block buffer trx4 (" + polling_payer_trx->get_doc_hash() + ") " + push_msg4;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  return {true, "Your administrative polling request is pushed to Block buffer"};
}

 */