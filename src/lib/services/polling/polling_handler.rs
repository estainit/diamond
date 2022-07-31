/*

const QString PollingHandler::stbl_pollings = "c_pollings";
const QStringList PollingHandler::stbl_pollings_fields = {"pll_id", "pll_hash", "pll_creator", "pll_type", "pll_class", "pll_ref", "pll_ref_type", "pll_ref_class", "pll_start_date", "pll_end_date", "pll_timeframe", "pll_version", "pll_comment", "pll_y_count", "pll_y_shares", "pll_y_gain", "pll_y_value", "pll_n_count", "pll_n_shares", "pll_n_gain", "pll_n_value", "pll_a_count", "pll_a_shares", "pll_a_gain", "pll_a_value", "pll_status", "pll_ct_done"};

const QString PollingHandler::stbl_ballots = "c_ballots";
const QStringList PollingHandler::stbl_ballots_fields = {"ba_hash", "ba_pll_hash", "ba_creation_date", "ba_receive_date", "ba_voter", "ba_voter_shares", "ba_vote", "ba_comment", "ba_vote_c_diff", "ba_vote_r_diff"};

const QString PollingHandler::stbl_polling_profiles = "c_polling_profiles";
const QStringList PollingHandler::stbl_polling_profiles_fields = {"ppr_name", "ppr_activated", "ppr_perform_type", "ppr_amendment_allowed", "ppr_votes_counting_method", "ppr_version"};

QHash<QString, QJsonObject> PollingHandler::POLLING_PROFILE_CLASSES {
    {
      "Basic",
      QJsonObject
      {
        {"ppName", "Basic"},
        {"activated", CConsts::YES},
        {"performType", POLL_PERFORMANCE_TYPES::Transparent},
        {"voteAmendmentAllowed", CConsts::NO},
        {"votesCountingMethod", VOTE_COUNTING_METHODS::Plurality}
      }
    },

    {
      "ZKP",
      QJsonObject
      {
        {"ppName", "ZKP"},
        {"activated",  CConsts::NO},
        {"performType",  POLL_PERFORMANCE_TYPES::ZKP},
        {"voteAmendmentAllowed",  CConsts::NO},
        {"votesCountingMethod",  VOTE_COUNTING_METHODS::Plurality}
      }
    }
};

bool PollingHandler::autoCreatePollingForProposal(QJsonObject& params)
{

  if (VersionHandler::isOlderThan(params.value("dVer").toString(), "0.0.8") == 1)
    params["dVer"] = "0.0.0";

  QString custom_stringified = stringifyAutoGeneratedProposalPolling(params);
  QString dExtHash = CCrypto::keccak256(custom_stringified);
  CLog::log("custom Stringified Auto-generated polling (dExtHash: " + dExtHash + "): " + custom_stringified);
  params.insert("dExtHash", dExtHash);

  QString dLen = CUtils::paddingLengthValue(custom_stringified.length() + dExtHash.length());
  params.insert("dLen", dLen);

  Document* doc = DocumentFactory::create(params);
  QString pll_hash = doc->calcDocHash(); //old name was calcHashDPolling
  delete doc;

  double voting_timeframe = params.value("pTimeframe").toDouble();
  TimeByMinutesT abs_no_timeframe_by_minutes = static_cast<int64_t>(voting_timeframe * 60.0 * 1.5); // 90 = 60 minute + 30 minute => 1.5 * yesTime
  CLog::log("abs_no_timeframe_by_minutes = " + QString::number(abs_no_timeframe_by_minutes), "app", "trace");
  CDateT end_date = CUtils::minutesAfter(
    abs_no_timeframe_by_minutes,
    params.value("startDate").toString());

  uint32_t ZERO = 0;
  QVDicT values {
    {"pll_creator", params.value("dCreator").toString()},
    {"pll_type", params.value("dType").toString()},
    {"pll_class", params.value("dClass").toString()},
    {"pll_ref", params.value("dRef").toString()},  // referenced voting subject(could be proposalHash,..
    {"pll_ref_type", params.value("dRefType").toString()}, // referenced voting subject(could be proposalHash,..
    {"pll_ref_class", params.value("dRefClass").toString()},
    {"pll_start_date", params.value("startDate").toString()},
    {"pll_end_date", end_date}, // end date is the deadline for absatin/no voting
    {"pll_timeframe", QVariant::fromValue(voting_timeframe)},
    {"pll_version", params.value("dVer").toString()},
    {"pll_comment", params.value("dComment").toString()},
    {"pll_y_count", ZERO},
    {"pll_y_shares", ZERO},
    {"pll_y_gain", ZERO},
    {"pll_y_value", ZERO},
    {"pll_n_count", ZERO},
    {"pll_n_shares", ZERO},
    {"pll_n_gain", ZERO},
    {"pll_n_value", ZERO},
    {"pll_a_count", ZERO},
    {"pll_a_shares", ZERO},
    {"pll_a_gain", ZERO},
    {"pll_a_value", ZERO},
    {"pll_status", params.value("status").toString()},
    {"pll_hash", pll_hash}
  };
  CLog::log("Insert a auto-generated polling for proposa() values: " + CUtils::dumpIt(values), "app", "trace");
  bool res = DbModel::insert(
    PollingHandler::stbl_pollings,
    values,
    true
  );

  return res;
}

QString PollingHandler::stringifyAutoGeneratedProposalPolling(QJsonObject& params)
{
  // stringifieng auto-created polling for proposal
  QString out = "{";
  out += "\"dCDate\":\"" + params.value("startDate").toString() + "\",";
  out += "\"dComment\":\"" + params.value("dComment").toString() + "\",";
  out += "\"dClass\":\"" + params.value("dClass").toString() + "\",";  // CConsts.POLLING_PROFILE_CLASSES.Basic.ppName
  out += "\"dRef\":\"" + params.value("dRef").toString() + "\",";
  out += "\"dRefClass\":\"" + params.value("dRefClass").toString() + "\",";
  out += "\"dRefType\":\"" + params.value("dRefType").toString() + "\",";
  out += "\"dType\":\"" + params.value("dType").toString() + "\",";
  out += "\"dVer\":\"" + params.value("dVer").toString() + "\",";
  out += "\"dCreator\":\"" + params.value("dCreator").toString() + "\",";
  out += "\"pTimeframe\":" + QString::number(params.value("pTimeframe").toDouble()) + "";
  out += "}";
  return out;
}

GenRes PollingHandler::updatePolling(
    const QVDicT& values,
    const ClausesT& clauses,
    const bool& is_transactional)
{
  DbModel::update(
    PollingHandler::stbl_pollings,
    values, // update values
    clauses,  // update clauses
    is_transactional
    );

  return { true };
}

void PollingHandler::initPollingProfiles()
{

//  QString rrr = PollingHandler::POLLING_PROFILE_CLASSES.value("Basic").value("zzz").toString();
//  auto sss3 = PollingHandler::POLLING_PROFILE_CLASSES["Basic"]["ppName"].toString();
//  auto sy3 = PollingHandler::POLLING_PROFILE_CLASSES["Basic"]["voteAmendmentAllowed"].toString();
//  auto sss = PollingHandler::POLLING_PROFILE_CLASSES["ZKP"]["ppName"].toString();
//  auto ss2 = PollingHandler::POLLING_PROFILE_CLASSES["ZKP"]["votesCountingMethod"].toString();


  QueryRes dbl = DbModel::select(
    PollingHandler::stbl_polling_profiles,
    {"ppr_name"},
    {ModelClause ("ppr_name", PollingHandler::POLLING_PROFILE_CLASSES["Basic"]["ppName"].toString())}
  );

  if (dbl.records.size() > 0)
      return;



  for (QString a_polling_profile_key : PollingHandler::POLLING_PROFILE_CLASSES.keys())
  {
    DbModel::insert(
      PollingHandler::stbl_polling_profiles,
      QVDicT
      {
        {"ppr_name", PollingHandler::POLLING_PROFILE_CLASSES[a_polling_profile_key]["ppName"].toString()},
        {"ppr_activated", PollingHandler::POLLING_PROFILE_CLASSES[a_polling_profile_key]["activated"].toString()},
        {"ppr_perform_type", PollingHandler::POLLING_PROFILE_CLASSES[a_polling_profile_key]["performType"].toString()},
        {"ppr_amendment_allowed", PollingHandler::POLLING_PROFILE_CLASSES[a_polling_profile_key]["voteAmendmentAllowed"].toString()},
        {"ppr_votes_counting_method", PollingHandler::POLLING_PROFILE_CLASSES[a_polling_profile_key]["votesCountingMethod"].toString()},
        {"ppr_version", "0.0.8"},
      }
    );
  }

}


TimeByHoursT PollingHandler::normalizeVotingTimeframe(TimeByHoursT voting_timeframe)
{
  if (CConsts::TIME_GAIN == 1)
     return static_cast<uint64_t>(voting_timeframe);
  // because of test ambient the longivity can be float and less than 1 hour
  return CUtils::customFloorFloat(static_cast<double>(voting_timeframe), 2);
  // return CUtils::CFloor(voting_timeframe * 100.0) / 100.0;
}

TimeByHoursT PollingHandler::getMinVotingTimeframe()
{
  TimeByHoursT voting_timeframe = (CMachine::getCycleByMinutes() * 2.0) / 60.0;   // at least 2 cycle for voting
  return normalizeVotingTimeframe(voting_timeframe);
}

bool PollingHandler::removePollingByRelatedProposal(const CDocHashT& proposal_hash)
{
  //sceptical test
  QueryRes exist = DbModel::select(
    stbl_pollings,
    {"pll_ref_type"},
    {{"pll_ref_type", CConsts::POLLING_REF_TYPE::Proposal},
    {"pll_ref", proposal_hash}});
  if (exist.records.size() != 1)
  {
    CLog::log("Try to delete polling strange result!  " + CUtils::dumpIt(exist.records), "sec", "error");
    return false;
  }

  DbModel::dDelete(
    stbl_pollings,
    {{"pll_ref_type", CConsts::POLLING_REF_TYPE::Proposal},
        {"pll_ref", proposal_hash}});

  return true;
}


bool PollingHandler::removePollingG(const CDocHashT& polling_hash)
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
std::tuple<double, double, double> PollingHandler::calculateVoteGain(
  double voteCreationTimeByMinute,
  double voteReceiveingTimeByMinute,
  TimeByMinutesT voting_timeframe_by_minute)
{
  if (voting_timeframe_by_minute == 0)
    voting_timeframe_by_minute = CMachine::getCycleByMinutes() * 2;

  TimeByMinutesT offset = CMachine::getCycleByMinutes() / 4; // 3 hour(1/4 cycle) will be enough to entire network to be synched
  TimeByMinutesT minRange = CMachine::getCycleByMinutes() * 2; // 24 hour(2 cycle)
  double latenancyFactor = log(voteReceiveingTimeByMinute - voteCreationTimeByMinute + minRange) / log(minRange);

  double gainYes = 0;
  if (voteCreationTimeByMinute < voting_timeframe_by_minute)
  {
    gainYes = (log(voting_timeframe_by_minute - voteCreationTimeByMinute + 1) / log(voting_timeframe_by_minute + offset)) / latenancyFactor;
    gainYes = CUtils::customFloorFloat(gainYes, 2);
  }

  double gainNoAbstain = 0;
  if (voteCreationTimeByMinute < (voting_timeframe_by_minute * 1.5))
  {
    gainNoAbstain = (log((voting_timeframe_by_minute * 1.5) - voteCreationTimeByMinute + 1) / log((voting_timeframe_by_minute * 1.5) + offset)) / latenancyFactor;
    gainNoAbstain = CUtils::customFloorFloat(gainNoAbstain, 2);
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
bool PollingHandler::maybeUpdatePollingStat(const CDocHashT& polling_hash)
{
  QVDRecordsT pollings = searchInPollings({{"pll_hash", polling_hash}});
  if (pollings.size()== 0)
  {
    CLog::log("Polling does not exist in DAG! polling(" + CUtils::hash8c(polling_hash) + ")", "app", "debug");
    return false;
  }
  QVDicT polling = pollings[0];
  CLog::log("retrieve polling(" + CUtils::hash8c(polling_hash) + ") info: " + CUtils::dumpIt(polling), "app", "trace");

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
      "CreationDif: " + QString::number(aBall.value("ba_vote_c_diff").toDouble()) +
      ", receiveDiff: " + QString::number(aBall.value("ba_vote_r_diff").toDouble()) +
      ", yesPeriod: " + QString::number(yesPeriod) + ") " +
      "gainYes(" + QString::number(gainYes) + ") " +
      "gainNoAbstain(" + QString::number(gainNoAbstain) + ") " +
      "latenancy_(" + QString::number(latenancy_) + ") ");

    DNAShareCountT shares = aBall.value("ba_voter_shares").toDouble();

    int16_t ba_vote = aBall.value("ba_vote").toInt();
    if (ba_vote > 0) {
        vStatistics.m_yes_count++;
        vStatistics.m_yes_shares += shares;
        vStatistics.m_yes_gain += CUtils::CFloor(shares * gainYes);
        vStatistics.m_yes_value += CUtils::CFloor(vStatistics.m_yes_gain * ba_vote);

    } else if (ba_vote == 0) {
        vStatistics.m_abstain_count++;
        vStatistics.m_abstain_shares += shares;
        vStatistics.m_abstain_gain += CUtils::CFloor(shares * gainNoAbstain);
        vStatistics.m_abstain_value += 0;

    } else if (ba_vote < 0) {
        vStatistics.m_no_count++;
        vStatistics.m_no_shares += shares;
        vStatistics.m_no_gain += CUtils::CFloor(shares * gainNoAbstain);
        vStatistics.m_no_value += CUtils::CFloor(vStatistics.m_no_gain * abs(ba_vote));

    }
  }

  vStatistics.m_polling_status = (polling.value("pll_end_date").toString() < CUtils::getNow()) ? CConsts::CLOSE : CConsts::OPEN;

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

bool PollingHandler::treatPollingWon(
  const QVDicT& polling,
  const CDateT& approve_date)
{
  /**
  * depends on pll_ref_type needs different treatment. there are a bunch of reserved pollings(e.g. Proposal, AdmPollings,...)
  */
  if (polling.value("pll_ref_type").toString() == CConsts::POLLING_REF_TYPE::Proposal)
  {
    ProposalHandler::transformApprovedProposalToDNAShares(
      polling,
      approve_date);

  }else if (polling.value("pll_ref_type").toString() == CConsts::POLLING_REF_TYPE::AdmPolling) {
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

void PollingHandler::treatPollingMissed(
  const QVDicT& polling,
  const CDateT& approve_date)
{
  QString pll_ref_type = polling.value("pll_ref_type").toString();

  /**
  * depends on pll_ref_type needs different treatment. there are a bunch of reserved pollings(e.g. Proposal, AdmPollings,...)
  */
  if (pll_ref_type == CConsts::POLLING_REF_TYPE::Proposal)
  {
    ProposalHandler::concludeProposal(
      polling,
      approve_date);

  }else if (pll_ref_type == CConsts::POLLING_REF_TYPE::AdmPolling)
  {
    SocietyRules::concludeAdmPolling(
      polling,
      approve_date);

  }

}

void PollingHandler::doOnePollingConcludeTreatment(
  QVDicT aPolling,
  const CDateT& c_date)
{
//  CDateT polling_end_date = CUtils::minutesAfter(aPolling.value("pll_timeframe").toDouble() * 60 * 1.5, aPolling.value("pll_start_date").toString());
  CDateT polling_end_date = aPolling.value("pll_end_date").toString();
  CDateT approve_date_ = CUtils::minutesAfter(CMachine::getCycleByMinutes() * 2, polling_end_date);
  CDateT approve_date = CUtils::getCoinbaseRange(approve_date_).from;
  CLog::log("retrive for conclude Treatment polling(" + CUtils::hash8c(aPolling.value("pll_hash").toString()) + ") polling_end_date(" + polling_end_date + ")", "app", "info");

  // generaly Close the pollings which are finished the voting time
  if (polling_end_date < CUtils::getNow())
  {
    maybeUpdatePollingStat(aPolling.value("pll_hash").toString());

    // retrieve (maybe)updated info
    QVDRecordsT recs = searchInPollings({{"pll_hash", aPolling.value("pll_hash").toString()}});
    if( recs.size() > 0)
      aPolling = recs[0];
  }

  // compare_date is the start date of last cycle, so ONLY pollings with close date before compare date can be consider and treat the conclude treatments
  CDateT compare_date = CUtils::getACycleRange(c_date, 1).from;
  CLog::log("retrive for conclude Treatment polling(" + CUtils::hash8c(aPolling.value("pll_hash").toString()) + ") compare_date(" + compare_date + ")");

  /**
  * TODO: implement a way to run a customized contract as a conclude treatment, at least as a plugin on some nodes.
  * somethig like PledgeP conclude by Arbiters
  */
  CLog::log("controll if polling(" + CUtils::hash8c(aPolling.value("pll_hash").toString()) + ") end date(" + polling_end_date + ") is before the previous cycle end(" + compare_date + ")");
  QVDicT latest_block_in_DAG = DAG::getLatestRecordedBlcok();
  if (polling_end_date < latest_block_in_DAG.value("b_creation_date").toString())
  {
    CLog::log("comparing pllYValue(" + QString::number(aPolling.value("pll_y_value").toDouble()) + " & pllNValue(" + QString::number(aPolling.value("pll_n_value").toDouble()) + ")");
    if (aPolling.value("pll_y_value").toDouble() >= aPolling.value("pll_n_value").toDouble())
    {

//      {
//        // remove this code block after tests.  if (polling_end_date < last block inserted in DAG)
//        if (aPolling.value("pll_hash").toString() == "e8e406b6362521637b31fa308f6879d4aee00d23e34ceda614dcdb9d6f181a8a")
//        {
//          QVDicT latest_block_in_DAG = DAG::getLatestRecordedBlcok();
//          if (polling_end_date < latest_block_in_DAG.value("b_creation_date").toString())
//          {
//            CLog::log("nnnnnnnnnnnooooooooooooo even new method doesnot work" + CUtils::dumpIt(aPolling));

//          }else{
//            CLog::log("yyyyyyyes it works" + CUtils::dumpIt(aPolling));

//          }
//          CLog::log("ooooooooooooo" + CUtils::dumpIt(aPolling));
//        }
//      }

      CLog::log("conclude Winer polling(" + CUtils::hash8c(aPolling.value("pll_hash").toString()) + ") aPolling: " + CUtils::dumpIt(aPolling));
      treatPollingWon(
        aPolling,
        approve_date);

    } else {
      CLog::log("Conclude Missed polling(" + CUtils::hash8c(aPolling.value("pll_hash").toString()) + ") Polling info: " + CUtils::dumpIt(aPolling));
      treatPollingMissed(
        aPolling,
        approve_date);

    }

    // mark polling as conclude treatment done
    DbModel::update(
      stbl_pollings,
      {{"pll_ct_done", CConsts::YES }},
      {{"pll_hash", aPolling.value("pll_hash").toString()}});
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
bool PollingHandler::loopConcludeTreatment(
  const CDateT& c_date,
  bool force_to_update)
{
  QString thread_prefix = "conclude_treatment_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    doConcludeTreatment(c_date, force_to_update);

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getConcludeTreatmentGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Conclude Treatment");
  return true;
}
*/

use crate::lib::custom_types::CDateT;

// js name was doPollingConcludeTreatment
//old_name_was doConcludeTreatment
pub fn do_conclude_treatment(
    c_date: &CDateT,
    force_to_update: bool) -> bool
{
    /*

      if (CMachine::isInSyncProcess())
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
        if (CMachine::isInSyncProcess())
        {
          auto[status_, lastWBLock] = DAG::getLatestBlockRecord();
          Q_UNUSED(status_);
          CDateT oldestDate = CUtils::getACycleRange(lastWBLock.m_creation_date, 10).from;
          clauses.push_back({"pll_end_date", oldestDate, ">="});
        }

      } else {
        clauses.push_back({"pll_ct_done", CConsts::NO});
      }
      QVDRecordsT pollings = PollingHandler::searchInPollings(clauses);
      for (QVDicT a_polling: pollings)
      {
        doOnePollingConcludeTreatment(a_polling, c_date);
      }
    */
    return true;
}

/*
bool PollingHandler::recordPollingInDB(
  const Block& block,
  const PollingDocument* polling,
  const QString& pll_status)
{
  CLog::log("record A general polling document: " + CUtils::dumpIt(polling), "app", "trace");

  QVDicT values {
    {"pll_hash", polling->m_doc_hash},
    {"pll_creator", polling->m_polling_creator},
    {"pll_type", polling->m_doc_type},
    {"pll_class", polling->m_doc_class}, // it is equal to pollingDoc.dClass
    {"pll_ref", polling->m_polling_ref},  // referenced voting subject(could be proposalHash,...)
    {"pll_ref_type", polling->m_polling_ref_type},  // referenced voting subject(could be proposalHash,...)
    {"pll_ref_class", polling->m_polling_ref_class},  // referenced voting subject(could be proposalHash,...)
    {"pll_start_date", block.m_block_creation_date}, // TODO: improve to user can set different start date(at least one cycle later than block.creation Date)
    {"pll_end_date", CUtils::minutesAfter(polling->m_voting_timeframe * 90, block.m_block_creation_date)}, // TODO: improve to user can set different start date(at least one cycle later than block.creation Date)
    {"pll_timeframe", polling->m_voting_timeframe},
    {"pll_version", polling->m_doc_version},
    {"pll_comment", polling->m_polling_comment},
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
  CLog::log("record A general polling values: " + CUtils::dumpIt(values), "app", "trace");

  DbModel::insert(
    stbl_pollings,
    values);
  return true;
}



bool PollingHandler::removePollingByRelatedAdmPolling(const CDocHashT& adm_polling_doc_hash)
{
  //sceptical test
  QueryRes exist = DbModel::select(
    stbl_pollings,
    {"pll_ref"},
    {{"pll_ref_type", CConsts::POLLING_REF_TYPE::AdmPolling},
    {"pll_ref", adm_polling_doc_hash}});
  if (exist.records.size() != 1)
  {
    CLog::log("Try to delete polling(" + adm_polling_doc_hash + ") strange result! records" + CUtils::dumpIt(exist.records), "sec", "warning");
    return false;
  }

  DbModel::dDelete(
    stbl_pollings,
    {{"pll_ref_type", CConsts::POLLING_REF_TYPE::AdmPolling},
    {"pll_ref", adm_polling_doc_hash}});

  return true;
}

void PollingHandler::maybeUpdateOpenPollingsStat()
{

  ClausesT clauses {};
  /**
   * this query is because in synching process there is moment in which
   * a polling is closed before all ballots beeing considered
   * so we re-processing ALL pollings
   */
  if (!CMachine::isInSyncProcess())
    clauses.push_back({"pll_status", CConsts::OPEN});
  QueryRes open_pollings = DbModel::select(
    stbl_pollings,
    {"pll_hash"},
    clauses);
  CLog::log("maybe Update open_pollingsStat: " + CUtils::dumpIt(open_pollings.records), "app", "trace");
  for (QVDicT a_polling: open_pollings.records)
    maybeUpdatePollingStat(a_polling.value("pll_hash").toString());
}

QVDRecordsT PollingHandler::searchInPollingProfiles(
  const ClausesT& clauses,
  const QStringList& fields,
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

QVDRecordsT PollingHandler::searchInPollings(
  const ClausesT& clauses,
  const QStringList& fields,
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

GenRes PollingHandler::retrievePollingByProposalHash(const CDocHashT& proposal_hash)
{
  QString msg;
  QVDRecordsT pollings = searchInPollings({{"pll_ref", proposal_hash}});
  if (pollings.size() != 1)
  {
    msg = "Failed in retrieve polling for proposal(" + CUtils::hash8c(proposal_hash) + ")";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CDocHashT polling_hash = pollings[0].value("pll_hash").toString();
  return {true, polling_hash};
}

GenRes PollingHandler::retrievePollingByAdmPollingHash(const CDocHashT& adm_polling_hash)
{
  QString msg;
  QVDRecordsT pollings = searchInPollings({{"pll_ref", adm_polling_hash}});
  if (pollings.size() != 1)
  {
    msg = "Failed in retrieve polling for adm-polling(" + CUtils::hash8c(adm_polling_hash) + ")";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  CDocHashT polling_hash = pollings[0].value("pll_hash").toString();
  return {true, polling_hash};
}

GenRes PollingHandler::maybeUpdatePollingStatByProposalHash(const CDocHashT& proposal_hash)
{
  GenRes polling_res = retrievePollingByProposalHash(proposal_hash);
  if (!polling_res.status)
    return polling_res;

  PollingHandler::maybeUpdatePollingStat(polling_res.msg);
  CGUI::signalUpdateContributes();
  return {true, ""};
}

GenRes PollingHandler::maybeUpdatePollingStatByAdmPollingHash(const CDocHashT& adm_polling_hash)
{
  CLog::log("maybe Update Polling Stat By Adm Polling Hash " + adm_polling_hash);
  GenRes polling_res = retrievePollingByAdmPollingHash(adm_polling_hash);
  if (!polling_res.status)
    return polling_res;

  PollingHandler::maybeUpdatePollingStat(polling_res.msg);
  CGUI::signalUpdateContributes();
  return {true, ""};
}


std::tuple<bool, QString, PollingDocument*> PollingHandler::prepareNewPolling(
  const CAddressT& document_creator,
  TimeByHoursT voting_timeframe,
  const CDocHashT& documnet_ref,
  const CDocHashT& documnet_ref_type,
  const CDocHashT& documnet_ref_class,
  const uint64_t voters_count,
  const QString& document_type,
  const QString& document_class,
  const QString& documnet_comment,
  const CDateT& creation_date)
{
  // TODO: add optional startDate(a date in future & superiore than one cycle) in which votting will be started,
  // currently voting start date is equal the block in which placed request
  QJsonObject polling_json {
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
  CLog::log("polling_json: " + CUtils::serializeJson(polling_json), "app", "info");
  PollingDocument* polling_doc = new PollingDocument(polling_json);
  polling_doc->m_potential_voters_count = voters_count;

  QString sign_message = polling_doc->getDocSignMsg();
  auto[sign_status, res_msg, sign_signatures, sign_unlock_set] = Wallet::signByAnAddress(
    document_creator,
    sign_message);
  if (!sign_status)
  {
    CLog::log(res_msg, "app", "error");
    return {false, res_msg, nullptr};
  }

  polling_doc->m_doc_ext_info = SignatureStructureHandler::compactUnlockersArray(QJsonArray {QJsonObject {
    {"uSet", sign_unlock_set},
    {"signatures", CUtils::convertQStringListToJSonArray(sign_signatures)}}});
  polling_doc->setDExtHash();
  polling_doc->setDocLength();
  polling_doc->setDocHash();

  CLog::log("prepared polling request: " + polling_doc->safeStringifyDoc(true), "app", "info");

  return {true, "", polling_doc};
}

std::tuple<bool, QString> PollingHandler::makeReqForAdmPolling(
  const QString& polling_subject,
  TimeByHoursT voting_timeframe,
  double the_value,
  uint64_t voters_count,
  QString doc_comment,
  const CDateT& c_date,
  const CAddressT& the_creator,
  const QString& polling_doc_type,
  const QString& polling_doc_class)
{
  QString msg;

  if (voting_timeframe < getMinVotingTimeframe())
  {
    msg = "Invalid polling time frame " + QString::number(voting_timeframe);
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  if (doc_comment == "")
  {
    doc_comment = "Request For Administrative polling to refine(" + polling_subject + ") to (" + QString::number(the_value) + ")";
  }

  QJsonObject polling_values {
    {"pTimeframe", voting_timeframe},
  };

  if (QStringList{POLLING_TYPES::RFRfPLedgePrice,
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

  } else if (QStringList{POLLING_TYPES::RFRfMinS2Wk,
    POLLING_TYPES::RFRfMinS2DA,
    POLLING_TYPES::RFRfMinS2V,
    POLLING_TYPES::RFRfMinFSign,
    POLLING_TYPES::RFRfMinFVote}.contains(polling_subject))
  {
    polling_values["pShare"] = the_value;

  }


  // 1. create a adm polling js doc
  QJsonObject adm_polling_json {
    {"dType", polling_doc_type},
    {"dClass", polling_doc_class},
    {"dVer", "0.0.9"},
    {"dComment", doc_comment},
    {"pSubject", polling_subject},
    {"pValues", polling_values},
    {"dCDate", c_date},
    {"dCreator", the_creator}};    // the bech32 address of shareholde}r

  AdministrativePollingDocument* adm_polling_doc = new AdministrativePollingDocument(adm_polling_json);

  QString sign_message = adm_polling_doc->getDocSignMsg();
  auto[sign_status, res_msg, sign_signatures, sign_unlock_set] = Wallet::signByAnAddress(
    the_creator,
    sign_message);
  if (!sign_status)
  {
    CLog::log(res_msg, "app", "error");
    return {false, res_msg};
  }

  adm_polling_doc->m_doc_ext_info = SignatureStructureHandler::compactUnlockersArray(QJsonArray {QJsonObject {
    {"uSet", sign_unlock_set},
    {"signatures", CUtils::convertQStringListToJSonArray(sign_signatures)}}});
  adm_polling_doc->setDExtHash();

  adm_polling_doc->setDocLength();
  adm_polling_doc->setDocHash();

  CLog::log("prepared adm-polling request: " + adm_polling_doc->safeStringifyDoc(true), "app", "info");

  // 2. create a administrative-polling for block release
  auto [polling_status, polling_msg, polling_doc] = prepareNewPolling(
    the_creator,
    voting_timeframe,
    adm_polling_doc->getDocHash(),
    adm_polling_doc->m_doc_type,
    adm_polling_doc->m_doc_class,
    voters_count,
    CConsts::DOC_TYPES::Polling,
    POLLING_PROFILE_CLASSES["Basic"]["ppName"].toString(),
    doc_comment,
    c_date);

  // 3. create trx to pay ReqForRelRes doc cost
  // calculate ballot cost
  auto[adm_cost_status, adm_dp_cost] = adm_polling_doc->calcDocDataAndProcessCost(
    CConsts::STAGES::Creating,
    CUtils::getNow());
  if (!adm_cost_status)
  {
    msg = "Failed in Adm-polling calculation for: " + adm_polling_doc->m_doc_comment;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto[coins_status1, coins_msg1, spendable_coins1, spendable_amount1] = Wallet::getSomeCoins(
    CUtils::CFloor(adm_dp_cost * 1.3),  // an small portion bigger to support DPCosts
    CConsts::COIN_SELECTING_METHOD::PRECISE);
  if (!coins_status1)
  {
    msg = "Failed in finding coin to spend for Adm polling (" + adm_polling_doc->m_doc_comment + ") " + coins_msg1;
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
  std::vector<TOutput> outputs1 {
    TOutput{change_back_address1, 1, CConsts::OUTPUT_CHANGEBACK},
    TOutput{"TP_ADM_POLLING", adm_dp_cost, CConsts::OUTPUT_TREASURY}};

  auto trx_template1 = BasicTransactionTemplate {
    spendable_coins1,
    outputs1,
    static_cast<CMPAIValueT>(adm_dp_cost * 1.3),  // max DPCost
    0,    // pre calculated dDPCost
    "Payed for adm-polling cost",
    adm_polling_doc->getDocHash()};
  auto[tx_status, res_msg1, adm_polling_payer_trx, dp_cost1] = BasicTransactionHandler::makeATransaction(trx_template1);
  if (!tx_status)
    return {false, res_msg1};

  CLog::log("Signed trx for adm-polling cost: " + adm_polling_payer_trx->safeStringifyDoc(true), "app", "info");

  // mark UTXOs as used in local machine
  Wallet::locallyMarkUTXOAsUsed(adm_polling_payer_trx);

  // 4. create trx to pay real polling costs
  auto[polling_cost_status, polling_dp_cost] = polling_doc->calcDocDataAndProcessCost(
    CConsts::STAGES::Creating,
    CUtils::getNow());
  if (!polling_cost_status)
  {
    msg = "Failed in polling calculation for: " + adm_polling_doc->m_doc_comment;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto[coins_status2, coins_msg2, spendable_coins2, spendable_amount2] = Wallet::getSomeCoins(
    CUtils::CFloor(polling_dp_cost * 1.3),  // an small portion bigger to support DPCosts
    CConsts::COIN_SELECTING_METHOD::PRECISE);
  if (!coins_status2)
  {
    msg = "Failed in finding coin to spend for Adm polling (" + adm_polling_doc->m_doc_comment + ")";
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
  std::vector<TOutput> outputs2 {
    TOutput{change_back_address2, 1, CConsts::OUTPUT_CHANGEBACK},
    TOutput{"TP_POLLING", polling_dp_cost, CConsts::OUTPUT_TREASURY}};

  auto trx_template2 = BasicTransactionTemplate {
    spendable_coins2,
    outputs2,
    static_cast<CMPAIValueT>(polling_dp_cost * 1.3),  // max DPCost
    0,    // pre calculated dDPCost
    "Payed for polling cost",
    polling_doc->getDocHash()};
  auto[tx_status2, res_msg2, polling_payer_trx, dp_cost2] = BasicTransactionHandler::makeATransaction(trx_template2);
  if (!tx_status2)
    return {false, res_msg2};

  CLog::log("Signed trx for polling cost: " + polling_payer_trx->safeStringifyDoc(true), "app", "info");

  // mark UTXOs as used in local machine
  Wallet::locallyMarkUTXOAsUsed(polling_payer_trx);


  auto[push_res1, push_msg1] = CMachine::pushToBlockBuffer(adm_polling_doc, adm_dp_cost);
  if (!push_res1)
  {
    msg = "Failed in push to block buffer adm-polling(" + adm_polling_doc->m_doc_comment + ") " + push_msg1;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  auto[push_res2, push_msg2] = CMachine::pushToBlockBuffer(adm_polling_payer_trx, adm_polling_payer_trx->getDocCosts());
  if (!push_res2)
  {
    msg = "Failed in push to block buffer trx2 (" + adm_polling_payer_trx->getDocHash() + ") " + push_msg2;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  auto[push_res3, push_msg3] = CMachine::pushToBlockBuffer(polling_doc, polling_dp_cost);
  if (!push_res3)
  {
    msg = "Failed in push to block buffer polling(" + adm_polling_doc->m_doc_comment + ") " + push_msg3;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  auto[push_res4, push_msg4] = CMachine::pushToBlockBuffer(polling_payer_trx, polling_payer_trx->getDocCosts());
  if (!push_res4)
  {
    msg = "Failed in push to block buffer trx4 (" + polling_payer_trx->getDocHash() + ") " + push_msg4;
    CLog::log(msg, "app", "error");
    return {false, msg};
  }

  return {true, "Your administrative polling request is pushed to Block buffer"};
}

 */