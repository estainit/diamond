/*
#include "stable.h"

class Document;

#include "lib/ccrypto.h"
#include "lib/utils/cmerkle.h"
#include "lib/dag/leaves_handler.h"
#include "lib/dag/missed_blocks_handler.h"
#include "lib/block/document_types/document.h"
#include "lib/services/treasury/treasury_handler.h"
#include "lib/sending_q_handler/sending_q_handler.h"
#include "lib/block/document_types/document_factory.h"
#include "lib/messaging_protocol/dag_message_handler.h"
#include "lib/block/block_types/block_floating_signature/floating_signature_block.h"

#include "coinbase_issuer.h"

static QJsonObject coinbaseBlockVersion0 {

    // coud be imagine test net (it) or main net (im)
  {"net", "im"},

  {"bVer", "0.0.0"},
  {"bType", CConsts::BLOCK_TYPES::Coinbase},
    // coud be imagine test net (it) or main net (im)
  {"bLen", "0000000"}, // seialized block size by byte (char). this number is also a part of block root hash

    // miner trx-fee address is implicitly is in coinbase transaction, the email address of node or it's public-key I am not sure it is usefull or not
    // or even compromise some un-necessary informations for adversary
    // miners publick key, has many uses such as open-pgp comunicating to other miners via email to broadcasting the block
    // minerKey: "02e90f689391564243f22082996b75e41ba832733fb25810c576d0cf1cb83f1b54", // dab5b531d6366b298d143175bf47b2ddf671813bd493c9b32dc235102a77a596

    // root hash of all doc hashes
    // it is maked based on merkle tree root of transactions, segwits, wikis, SNSs, SSCs, DVCs, ...
  {"bHash", ""},

    // the structure in which contain signature of shareholders(which are backers too) and
    // by their sign, they confirm the value of shares&  DAG screen-shoot on that time.
    // later this confirmation will be used in validating the existance of a "sus-block" at the time of that cycle of coinbase

    // a list of ancestors blocks, these ancestor's hash also recorded in block root hash
    // if a block linked to an ancestors block, it must not liks to the father of that block
    // a <--- b <---- c
    // if the block linked to b, it must not link to a
    // the new block must be linked to as many possible as previous blocks (leave blocks)
    // maybe some discount depends on how many block you linked as an ancester!
    // when a new block linke t another block, it MUST not linked to that's block ancester

  {"ancestors", QJsonArray {}},

    // a list of descendent blocks. this list can not be exist when the node receive block.
    // the node update these information whenever receive new childs blocks. and in sequence node send this information ( if exist)

    // creation time timestamp, it is a part of block root-hash
    // it also used to calculate spendabality of an output. each output is not spendable befor passing 12 hours of creation time.
    // creation date must be greater than all it's ancesters
  {"bCDate", ""}, // 'yyyy-mm-dd hh:mm:ss'

    // the time in which block is received in node, this time is valid only on local node and not published
    // receiveDate: null,


  {"docsRootHash", ""}, //the hash root of merkle tree of transactions
  {"docs", QJsonArray {}}

};

CoinbaseIssuer::CoinbaseIssuer()
{

}

QJsonObject CoinbaseIssuer::getCoinbaseTemplateObject()
{
  QJsonObject Jdoc {
    {"dHash", ""},
    {"dType", CConsts::DOC_TYPES::Coinbase},
    {"dVer", "0.0.0"},
    {"cycle", ""}, // 'yyyy-mm-dd am' / 'yyyy-mm-dd pm'
    {"treasuryFrom", ""}, // incomes from date
    {"treasuryTo", ""}, // incomes to date
    {"treasuryIncomes", 0}, // incomes value
    {"mintedCoins", 0},
    {"outputs", QJsonArray{}}};
  return Jdoc;
}

/**
 *
 * @param {the time for which cycle is calculated} cycle
 *
 * coinbase core is only shares and dividends,
 * so MUST BE SAME IN EVERY NODES.
 * the inputs are newly minted coins and treasury incomes
 *
 */
QJsonObject CoinbaseIssuer::createCBCore(
  QString cycle,
  const QString& mode,
  const QString& version)
{
  CLog::log("create CBCore cycle(" + cycle +") mode(" + mode +")", "cb", "info");
  CDateT cDate = "";
  QString mintingYear = "";
  if (cycle == "")
  {
    cDate = CUtils::getNow();
    cycle = CUtils::getCoinbaseCycleStamp();
    mintingYear = cDate.split(" ")[0].split("-")[0];

  } else {
    if (CConsts::TIME_GAIN == 1)
    {
      // normally the cycle time is 12 hours
      cDate = cycle;

    } else {
      // here is for test net in which the cycle time is accelerated to have a faster block generator(even 2 minutes)
      uint64_t minutes = cycle.split(" ")[1].toUInt() * CMachine::getCycleByMinutes();
      QString minutes_ = CUtils::convertMinutesToHHMM(minutes);
      cDate = cycle.split(" ")[0] + " " + minutes_ + ":00";
    }
    mintingYear = cycle.midRef(0, 4).toString();
  }

  CDateT block_creation_date = CUtils::getCoinbaseRangeByCycleStamp(cycle).from;
  QJsonObject coinbase_document = getCoinbaseTemplateObject();

  coinbase_document["cycle"] = cycle;

  auto[fromDate, toDate, incomes] = TreasuryHandler::calcTreasuryIncomes(cDate);
  CLog::log("The treasury incomes for coinbase cDate(" + cDate + ") treasury incomes(" + CUtils::sepNum(incomes) + ") micro PAIs from Date(" + fromDate + ") toDate(" + toDate + ")", "cb", "info");

  coinbase_document["treasuryIncomes"] = QVariant::fromValue(incomes).toJsonValue();
  coinbase_document["treasuryFrom"] = fromDate;
  coinbase_document["treasuryTo"] = toDate;

  // create coinbase outputs
  QHash<CMPAIValueT, QHash<QString, TmpHolder> > tmpOutDict = {};
  QStringList holders = {};
  std::vector<CMPAIValueT> dividends {};

  /**
  *
  minted: 2,251,799,813.685248
  burned:
  share1:    21,110,874.142979
  share2:       985,017.380061
  share3:       422,068.382528
  share4:            38.230606
  */
  auto[one_cycle_max_mili_PAIs, one_cycle_issued, total_shares, share_amount_per_holder] = calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore(
    block_creation_date);

  CLog::log("share_amount_per_holder: " + CUtils::dumpIt(share_amount_per_holder));
  coinbase_document["mintedCoins"] = QVariant::fromValue(one_cycle_issued).toJsonValue();

  CMPAIValueT cycleCoins = coinbase_document["treasuryIncomes"].toDouble() + coinbase_document["mintedCoins"].toDouble();
  CLog::log("DNA cycle sum minted coins+treasury(" + CUtils::sepNum(cycleCoins) + " micro PAIs) . Cycle Max Coins(" + CUtils::sepNum(one_cycle_max_mili_PAIs) + " micro PAIs)", "cb", "info");

  for(CAddressT aHolder: share_amount_per_holder.keys())
  {
    DNAShareCountT aShare = share_amount_per_holder[aHolder];
    CMPAIValueT dividend = CUtils::CFloor((CUtils::iFloorFloat(aShare/total_shares) * cycleCoins)); // ((aShare * cycleCoins)/ total_shares);
    // let dividend = utils.floor((utils.iFloorFloat(aShare / sumShares) * cycleCoins));

    holders.append(aHolder);
    dividends.push_back(dividend);
    if (!tmpOutDict.keys().contains(dividend))
      tmpOutDict[dividend] = QHash<QString, TmpHolder> {};
    tmpOutDict[dividend][aHolder] = TmpHolder {aHolder, dividend};

  };
  // in order to have unique hash for coinbase block (even created by different backers) sort it by sahres desc, addresses asc
  std::sort(dividends.begin(), dividends.end());
  auto last = std::unique(dividends.begin(), dividends.end());
  dividends.erase(last, dividends.end());
  std::reverse(dividends.begin(), dividends.end());

  holders.sort();
  std::reverse(holders.begin(), holders.end());

  QJsonArray outputs {};
  for (CMPAIValueT dividend: dividends)
  {
    for (QString holder: holders)
    {
      if ((tmpOutDict.keys().contains(dividend)) && (tmpOutDict[dividend].keys().contains(holder)))
      {
        QJsonArray output_arr {
          tmpOutDict[dividend][holder].holder,
          QVariant::fromValue(tmpOutDict[dividend][holder].dividend).toDouble()
        };
        outputs.append(output_arr);
      }
    }
  }
  coinbase_document["outputs"] = outputs;
  CLog::log("Coinbase recalculated outputs on Cycle(" + cycle + "): details:" + CUtils::serializeJson(outputs), "cb", "trace");

  Document* doc = DocumentFactory::create(coinbase_document);
  coinbase_document["dHash"] = doc->calcDocHash(); // trxHashHandler.doHashTransaction(trx)
  delete doc;

  QJsonObject Jblock = coinbaseBlockVersion0;
  Jblock["bVer"] = version;

  if (version > "0.0.0")
    Jblock.remove("descriptions");

  Jblock["cycle"] = cycle;
  Jblock["docs"] = QJsonArray {coinbase_document};
  auto[root, verifies, merkle_version, levels, leaves] = CMerkle::generate({coinbase_document["dHash"].toString()});
  Q_UNUSED(verifies);
  Q_UNUSED(merkle_version);
  Q_UNUSED(levels);
  Q_UNUSED(leaves);
  Jblock["docsRootHash"] = root;
  Jblock["bCDate"] = block_creation_date;

  return Jblock;
}

/**
 *
 * @param {*} cycle
 *
 * although the coinbase core is only shares (which are equal in entire nodes)
 * but the final coinbase block consists of also ancestors links (which are participating in block hash)
 * so it could be possible different nodes generate different coinbaseHash for same blocks.
 *
 */
std::tuple<bool, QJsonObject> CoinbaseIssuer::doGenerateCoinbaseBlock(
  const QString& cycle,
  const QString& mode,
  const QString& version)
{
  CLog::log("do GenerateCoinbaseBlock cycle(" + cycle + ") mode(" + mode + ")", "cb", "info");
  auto[cycleStamp, from_, to_, fromHour, toHour] = CUtils::getCoinbaseInfo("", cycle);
  Q_UNUSED(cycleStamp);
  Q_UNUSED(fromHour);
  Q_UNUSED(toHour);

  QJsonObject Jblock = createCBCore(cycle, mode, version);

  // connecting to existed leaves as ancestors
  QJsonObject leaves = LeavesHandler::getLeaveBlocks(from_);
  QStringList leaves_hashes = leaves.keys();
  leaves_hashes.sort();
  CLog::log("do GenerateCoinbaseBlock retrieved cbInfo: from_(" + from_ +") to_(" + to_ +")", "cb", "info");
  CLog::log("do GenerateCoinbaseBlock retrieved leaves from kv: cycle(" + cycle +") leaves_hashes(" + leaves_hashes.join(", ") +") leaves(" + CUtils::dumpIt(leaves) +")", "cb", "info");

  auto[confidence_, block_hashes_, backers_] = FloatingSignatureBlock::aggrigateFloatingSignatures();
  Q_UNUSED(backers_);
//  clog.cb.info(`locally created block's confidence: ${JSON.stringify(floatingSignatures)}`);
  leaves_hashes = CUtils::arrayUnique(CUtils::arrayAdd(leaves_hashes, block_hashes_));

  // if requested cycle is current cycle and machine hasn't fresh leaves, so can not generate a CB block
  if (
    (mode == CConsts::STAGES::Creating) &&
    (leaves_hashes.size() == 0) &&
    (cycle == CUtils::getCoinbaseCycleStamp()))
  {
    if (mode == CConsts::STAGES::Creating)
    {
      CLog::log("generating new CB in generating mode failed!! leaves(" + leaves_hashes.join(",") + ")", "cb", "info");
    } else {
      CLog::log("strange error generating new CB failed!! mode(" + mode + ") mode(" + leaves_hashes.join(",") + ") ", "cb", "error");
    }
    return {false, Jblock};
  }

  Jblock["ancestors"] = QVariant(leaves_hashes).toJsonArray();
  CLog::log("do GenerateCoinbaseBlock block.ancestors: " + leaves_hashes.join(","), "cb", "info");


  // if the backer is also a shareholder (probably with large amount of shares),
  // would be more usefull if she also signs the dividends by his private key as a shareholder
  // and sends also her corresponding publick key
  // this signature wil be used for 2 reson
  // 1. anti rebuild DAG by adverseries
  // 2. prevent fake sus-blocks to apply to network
  // signing block by backer private key (TODO: or delegated private key for the sake of security)



  // clog.app.info(blockPresenter(block));
  return {true, Jblock};
}


uint8_t CoinbaseIssuer::calculateReleasableCoinsBasedOnContributesVolume(const uint64_t& total_shares)
{
  uint64_t contributes = CUtils::CFloor((static_cast<double>(total_shares) * 100) / CConsts::WORLD_POPULATION);
  uint8_t releseable_percentage = 1;
  for (uint8_t target: CConsts::MAP_CONTRIBUTE_AMOUNT_TO_MINTING_PERCENTAGE.keys())
  {
    if (contributes >= target)
    {
      releseable_percentage = CConsts::MAP_CONTRIBUTE_AMOUNT_TO_MINTING_PERCENTAGE[target];
    }
  }

  if (releseable_percentage > 100)
    releseable_percentage = 100;
  return releseable_percentage;
}


// it seems we do not need the big number module at all
CMPAIValueT CoinbaseIssuer::calcPotentialMicroPaiPerOneCycle(QString year_)
{
  if (year_ == "")
    year_ = CUtils::getCurrentYear();

  auto year = year_.toUInt();
  uint halving_cycle_number = CUtils::CFloor((year - CConsts::LAUNCH_YEAR) / CConsts::HALVING_PERIOD);
  CMPAIValueT one_cycle_max_mili_PAIs = pow(2, (CConsts::COIN_ISSUING_INIT_EXPONENT - halving_cycle_number));
  return one_cycle_max_mili_PAIs;
}

std::tuple<CMPAIValueT, CMPAIValueT, DNAShareCountT, QHash<QString, DNAShareCountT> > CoinbaseIssuer::calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore(
  const CDateT& cDate)
{
  CMPAIValueT one_cycle_max_mili_PAIs = calcPotentialMicroPaiPerOneCycle(cDate.split("-")[0]);
  auto[total_shares, share_amount_per_holder, holdersOrderByShares_] = DNAHandler::getSharesInfo(cDate);
  Q_UNUSED(holdersOrderByShares_);

  CMPAIValueT one_cycle_issued = CUtils::CFloor((calculateReleasableCoinsBasedOnContributesVolume(total_shares) * one_cycle_max_mili_PAIs) / 100);
  return { one_cycle_max_mili_PAIs, one_cycle_issued, total_shares, share_amount_per_holder };
}

std::tuple<CMPAIValueT, CMPAIValueT, DNAShareCountT> CoinbaseIssuer::predictReleaseableMicroPAIsPerOneCycle(
  const uint32_t& annualContributeGrowthRate,
  const DNAShareCountT& current_total_sahres,
  const QString& prevDue,
  CDateT due,
  CDateT cDate)
{
  if (cDate == "")
    cDate = CUtils::getNow();

  if (due == "")
    due = CUtils::getNow();

  if (cDate > CUtils::getNow())
  {
    CLog::log("for now the formule does not support future contribute calculation. TODO: implement it", "app", "error");
    return {0, 0, 0};
  }

  auto[one_cycle_max_coins, one_cycle_issued, sum_shares, share_amount_per_holder_] = calcDefiniteReleaseableMicroPaiPerOneCycleNowOrBefore(cDate);
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
    uint64_t futureDays = CUtils::timeDiff(prevDue, due).asDays;
    DNAShareCountT newShares = sum_shares * (((annualContributeGrowthRate) / 100) / 365) * futureDays;
    sum_shares += newShares;
    uint8_t releseablePercentage = calculateReleasableCoinsBasedOnContributesVolume(sum_shares) / 100;
    one_cycle_issued = CUtils::CFloor(releseablePercentage * one_cycle_max_coins);
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
  CDateT cDate, // contribute creation date (supposing aproved date of proposal)
  uint32_t months,
  DNAShareCountT current_total_sahres,
  uint32_t annualContributeGrowthRate)
{
  if (cDate == "")
    cDate = CUtils::getNow();

  CMPAIValueT total_incomes = 0;
  std::vector<MonthCoinsReport> monthly_incomes {};
  CMPAIValueT one_cycle_income, income_per_month;
  CDateT due;
  CDateT prevDue = cDate;
  for (uint32_t month = 0; month < months; month++)
  {
    due = CUtils::minutesAfter((60 * 24) * (365 / 12) * month, cDate);   //TODO change it to more accurate on month starting day

    auto[one_cycle_max_coins, one_cycle_issued, sum_shares] = predictReleaseableMicroPAIsPerOneCycle(
      annualContributeGrowthRate,
      current_total_sahres,
      prevDue,
      due,
      cDate);
    one_cycle_income = CUtils::CFloor((one_cycle_issued * the_contribute) / sum_shares);  // one cycle income
    income_per_month = CUtils::CFloor((one_cycle_issued * the_contribute * (2 * 30)) / sum_shares);  // one month income almost = one cycle income * 2 perDay * 30 perMonth
    CLog::log("predict Future Incomes sum_shares " + CUtils::dumpIt(sum_shares) + " one_cycle_income("+CUtils::microPAIToPAI6(one_cycle_income)+") income_per_month("+CUtils::microPAIToPAI6(income_per_month)+")", "app", "trace");

    MonthCoinsReport a_month {
      one_cycle_max_coins,
      one_cycle_issued,
      one_cycle_income,
      income_per_month,
      sum_shares,
      due.split(" ")[0]};
    monthly_incomes.push_back(a_month);

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
//    CUtils::CFloor(total_incomes / 1000000),  //TODO treasuryIncomes should be calcullated in a more smart way :)
    monthly_incomes,
    cDate,
    due};
}

/**
* theorically the coinbase block can be created by any one,
* and the root hash of the block could be different(because of adifferent ancesters).
*/

void CoinbaseIssuer::loopMaybeIssueACoinbaseBlock()
{
  QString thread_prefix = "maybe_issue_coinbase_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    maybeCreateCoinbaseBlock();

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getBlockInvokeGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop maybe coinbase issuance");
}


bool CoinbaseIssuer::doesDAGHasMoreConfidenceCB()
{
  CDateT current_cycle_range_from = CUtils::getCoinbaseRange().from;
  QVDRecordsT already_recorded_coinbase_blocks = DAG::searchInDAG(
    {{"b_type", CConsts::BLOCK_TYPES::Coinbase},
    {"b_creation_date", current_cycle_range_from, ">="}},
    {"b_hash", "b_confidence", "b_ancestors"});
  CLog::log("Already recorded coinbase blocks: " + CUtils::dumpIt(already_recorded_coinbase_blocks), "cb", "trace");
  if (already_recorded_coinbase_blocks.size() == 0)
    return false;

  std::vector<double> already_recorded_confidents {};
  QStringList already_recorded_ancestors {};
  for (QVDicT block_record: already_recorded_coinbase_blocks)
  {
    already_recorded_confidents.push_back(CUtils::customFloorFloat(block_record.value("b_confidence").toDouble()));
    already_recorded_ancestors = CUtils::arrayAdd(already_recorded_ancestors, CUtils::convertJSonArrayToQStringList(CUtils::parseToJsonArr(block_record.value("b_ancestors").toString())));
  }
  already_recorded_ancestors = CUtils::arrayUnique(already_recorded_ancestors);
  CLog::log("already Recorded Confidents from(" + current_cycle_range_from + ")" + CUtils::dumpIt(already_recorded_confidents), "cb", "info");
  CLog::log("already Recorded Ancestors from(" + current_cycle_range_from + ")" + CUtils::dumpIt(already_recorded_ancestors), "cb", "info");

  if ((already_recorded_confidents.size() == 0) || (already_recorded_ancestors.size() == 0))
    return false;

  std::sort(already_recorded_confidents.begin(), already_recorded_confidents.end());
  auto last = std::unique(already_recorded_confidents.begin(), already_recorded_confidents.end());
  already_recorded_confidents.erase(last, already_recorded_confidents.end());
  std::reverse(already_recorded_confidents.begin(), already_recorded_confidents.end());

  double max_recorded_confident = already_recorded_confidents[0];

  auto[the_confidence, block_hashes, backers] = FloatingSignatureBlock::aggrigateFloatingSignatures();
  QStringList not_recorded_blocks = CUtils::arrayDiff(block_hashes, already_recorded_ancestors);
  if ((the_confidence > max_recorded_confident) || (not_recorded_blocks.size() > 0))
    return true;

  return false;
}

std::tuple<QString, QString, QString, QSDicT> CoinbaseIssuer::makeEmailHashDict()
{
  QSDicT emails_hash_dict {};
  CDateT cycle = CUtils::getCoinbaseCycleStamp();
  QString machine_email = CMachine::getPubEmailInfo().m_address;
  QString machine_key = CCrypto::keccak256(cycle + "::" + machine_email);
  emails_hash_dict[machine_key] = machine_email;

  QVDRecordsT neightbors = CMachine::getNeighbors("", CConsts::YES);
  CLog::log("neightbors in makeEmail Hash Dict: " + CUtils::dumpIt(neightbors), "cb", "trace");
  for(QVDicT neighbor: neightbors)
  {
    QString key = CCrypto::keccak256(cycle + "::" + neighbor.value("n_email").toString());
    emails_hash_dict[key] = neighbor.value("n_email").toString();
  }
  return {cycle, machine_email, machine_key, emails_hash_dict};
}


bool CoinbaseIssuer::haveIFirstHashedEmail(const QString order)
{
  auto[cycle, machine_email, machine_key, emails_hash_dict] = makeEmailHashDict();
  QStringList keys = emails_hash_dict.keys();
  if (order == "asc")
  {
    keys.sort();
  } else {
    // reverse it
    for(int k = 0; k < (keys.size()/2); k++)
      keys.swapItemsAt(k, keys.size()-(1+k));
  }
  CLog::log("Ordered emails_hash_dict " + CUtils::dumpIt(keys), "cb", "trace");
  for (int i = 0; i < keys.size(); i++)
  {
    QString key = keys[i];
    CLog::log(QString::number(i + 1) + ". candidate email for issueing CB " + emails_hash_dict[key] + " " + CUtils::hash8c(key) + " ", "cb", "trace");
  }
  int inx = keys.lastIndexOf(machine_key);
  if (inx == 0)
  {
    // the machin has minimum hash, so can generate the coinbase
    CLog::log("Machine has the lowest/heighest hash: " + machine_email + " " + machine_key + " ", "cb", "trace");
    return true;
  }

  // if the machine email hash is not the smalest,
  // control it if based on time passed from coinbase-cycle can create the coinbase?
  auto[backer_address_, shares_, percentage] = DNAHandler::getMachineShares();
  Q_UNUSED(backer_address_);
  Q_UNUSED(shares_);
  percentage = (percentage / 5) + 1;
  int sub_cycle = (CConsts::TIME_GAIN == 1) ? 12 + percentage : 6 + percentage; // who has more shares should try more times to create a coinbase block
  int cb_email_counter = CUtils::getCoinbaseAgeBySecond() / (CUtils::getCycleBySeconds() / sub_cycle);
  CLog::log("cb_email_counter cycle " + QString::number(cb_email_counter) + " " + QString::number(cb_email_counter) + " > " + QString::number(inx), "cb", "trace");
  if (cb_email_counter > inx)
  {
    // it is already passed time and if still no one create the block it is my turn to create it
    CLog::log("It already passed " + QString::number(cb_email_counter) + " of 10 dividend of a cycle and now it's my turn " + machine_email + " to issue coinbase!", "cb", "trace");
    return true;
  }

  CLog::log("Machine has to wait To Create Coinbase Block! (if does not receive the fresh CBB) keys(" + cycle + "::" + machine_email + ")", "cb", "trace");
  return false;
}

bool CoinbaseIssuer::controlCoinbaseIssuanceCriteria()
{
  auto current_cycle_range = CUtils::getCoinbaseRange();
  //let res = {};
  CLog::log("Coinbase check Range (from " + current_cycle_range.from + " to " + current_cycle_range.to + ")", "cb", "info");

  if (!LeavesHandler::hasFreshLeaves())
  {
    // younger than 2 cycle
    CLog::log("Machine hasn't fresh leaves!", "cb", "info");
    DAGMessageHandler::setMaybeAskForLatestBlocksFlag(CConsts::YES);
    return false;
  }

  // control if already exist in DAG a more confidence Coinbase Block than what machine can create?
  bool DAG_has_more_confidence_CB = doesDAGHasMoreConfidenceCB();
  if (DAG_has_more_confidence_CB)
  {
    CLog::log("Machine already has more confidente Coinbase blocks in DAG", "cb", "info");
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
  QStringList missed_blocks = MissedBlocksHandler::getMissedBlocksToInvoke();
  if (missed_blocks.size() > 0)
  {

    // // FIXME: if an adversory sends a bunch of blocks which have ancestors, in machine will finished with a long list of
    // // missed blocks. so machine(or entire network machines) can not issue new coinbase block!
    // if (missed_blocks.length > iConsts.MAX_TOLERATED_MISS_BLOCKS) {
    //     msg = `Machine missed ${missed_blocks.length} blocks and touched the limitation(${iConsts.MAX_TOLERATED_MISS_BLOCKS}), so can not issue a coinbase block`
    //     clog.cb.info(msg);
    //     res.canGenCB = false;
    //     res.msg = msg
    //     return res;
    // }

    double latenancy_factor = CUtils::CFloor(((log(missed_blocks.size() + 1) / log(CConsts::MAX_TOLERATED_MISS_BLOCKS)) * CUtils::getCoinbaseAgeBySecond()));
    bool are_we_in_4_of_5 = (CUtils::getCoinbaseAgeBySecond() < (CUtils::getCycleBySeconds() * 4 / 5));
    if (are_we_in_4_of_5 && (CUtils::getCoinbaseAgeBySecond() < latenancy_factor))
    {
      CLog::log("Because of " + QString::number(missed_blocks.size()) + " missed blocks, machine can not create a CB before " + QString::number(latenancy_factor) + " second age of cycle or atleast 4/5 of cycle age passed", "cb", "info");
      return false;
    }
  }

  // a psudo random mechanisem
  bool am_i_qualified_to_issue_coinbase = haveIFirstHashedEmail();
  if (!am_i_qualified_to_issue_coinbase)
  {
    CLog::log("It is not machine turn To Create Coinbase Block!", "cb", "trace");
    return false;
  }

  return true;
}

// if passed 1/4 of a cycle time and still the coinbase block is not created, so broadcast one of them
bool CoinbaseIssuer::passedCertainTimeOfCycleToRecordInDAG(const CDateT& cDate)
{
  auto[cycle, machine_email, machine_key, emails_hash_dict] = makeEmailHashDict();
  QStringList keys = emails_hash_dict.keys();
  keys.sort();
  int machine_index = keys.indexOf(machine_key) + 2;
  CLog::log("psudo-random CB creation machine_index: " + QString::number(machine_index), "cb", "trace");

  TimeByMinutesT cycle_by_minutes = (CConsts::TIME_GAIN == 1) ? CConsts::STANDARD_CYCLE_BY_MINUTES : CConsts::TIME_GAIN;
  bool res = CUtils::timeDiff(CUtils::getCoinbaseRange(cDate).from).asSeconds >= (cycle_by_minutes * 60 * CConsts::COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG * (1 + (pow(machine_index, 7) / 131)));
  CLog::log("passed CertainTimeOfCycleToRecordInDAG? " + CUtils::dumpIt(res), "cb", "trace");
  return res;
}
*/

//old_name_was maybeCreateCoinbaseBlock
pub fn maybe_create_coinbase_block()
{
    /*
  bool can_issue_new_cb = controlCoinbaseIssuanceCriteria();
  if (!can_issue_new_cb)
    return;

  tryCreateCoinbaseBlock();

     */
}

/*

void CoinbaseIssuer::tryCreateCoinbaseBlock()
{
  auto[coinbase_cycle_stamp, coinbase_from, coinbase_to, coinbase_from_hour, coinbase_to_hour] = CUtils::getCoinbaseInfo(CUtils::getNow());
  // listener.doCallAsync('APSH_create_coinbase_block', { cbInfo });

  CLog::log("Try to Create Coinbase for Range (" + coinbase_from + ", " + coinbase_to + ")", "cb", "trace");
  auto[status, Jblock] = doGenerateCoinbaseBlock(
    CUtils::getCoinbaseCycleStamp(),
    CConsts::STAGES::Creating,
    "0.0.1");
  if ((Jblock.keys().size() == 0) || (!status))
  {
    CLog::log("Due to an error, can not create a coinbase block for range (" + coinbase_from + ", " + coinbase_to + ")", "cb", "fatal");
    return;
  }

  Jblock["bHash"] = "0000000000000000000000000000000000000000000000000000000000000000";
  CLog::log("Serialized locally created cb block. before objecting1 " + CUtils::serializeJson(Jblock), "cb", "trace");
  Jblock["bLen"] = CUtils::paddingLengthValue(CUtils::serializeJson(Jblock).length());
  CLog::log("Serialized locally created cb block. before objecting2 " + CUtils::serializeJson(Jblock), "cb", "trace");

  Block* tmp_block = BlockFactory::create(Jblock);
  tmp_block->setBlockHash(tmp_block->calcBlockHash());
  Jblock["bHash"] = tmp_block->getBlockHash();
  QJsonObject final_json_block = tmp_block->exportBlockToJSon();
  CLog::log("Serialized locally created cb block. after objecting " + CUtils::serializeJson(final_json_block), "cb", "trace");
  delete tmp_block;

  double tmp_local_confidence = final_json_block.value("confidence").toDouble();

  // if local machine can create a coinbase block with more confidence or ancestors, broadcast it
  auto [atleast_one_coinbase_block_exist, most_confidence_in_DAG] = DAG::getMostConfidenceCoinbaseBlockFromDAG();

  double tmp_DAG_confidence = 0.0;
  QStringList tmp_DAG_ancestors {};
  if (!atleast_one_coinbase_block_exist)
  {
    CLog::log("DAG hasn't coinbase for cycle range (" + coinbase_from + ", " + coinbase_to + ")", "cb", "trace");
  } else {
    CLog::log("The most_confidence_in_DAG for cycle range (" + coinbase_from + ", " + coinbase_to + ") is: " + CUtils::dumpIt(most_confidence_in_DAG), "cb", "trace");
    tmp_DAG_confidence = most_confidence_in_DAG.value("b_confidence").toDouble();
    QStringList tmp_DAG_ancestors = CUtils::convertCommaSeperatedToArray(most_confidence_in_DAG.value("b_ancestors").toString());
  }

  bool locally_created_coinbase_block_has_more_confidence_than_DAG = (tmp_DAG_confidence < tmp_local_confidence);
  locally_created_coinbase_block_has_more_confidence_than_DAG = false;// FIXME implement remote block confidence calcuilation
  if (locally_created_coinbase_block_has_more_confidence_than_DAG)
    CLog::log("More confidence: local coinbase(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") has more confidence(" + QString::number(tmp_local_confidence) + " than DAG(" + QString::number(tmp_DAG_confidence) + ") in cycle range (" + coinbase_from + ", " + coinbase_to + ")" , "cb", "trace");

  QStringList ancestors_diff = CUtils::arrayDiff(CUtils::convertJSonArrayToQStringList(final_json_block.value("ancestors").toArray()), tmp_DAG_ancestors);
  if (ancestors_diff.size() > 0)
  {
    // try to remove repayBack blocks
    QVDRecordsT existed_RpBlocks = DAG::searchInDAG(
      {{"b_type", CConsts::BLOCK_TYPES::RpBlock},
      {"b_hash", ancestors_diff, "IN"}},
      {"b_hash"});
    if (existed_RpBlocks.size() > 0)
    {
      QStringList tmp {};
      for (QVDicT record: existed_RpBlocks)
        tmp.append(record.value("b_hash").toString());
      ancestors_diff = CUtils::arrayDiff(ancestors_diff, tmp);
    }
  }
  bool locally_created_coinbase_block_has_more_ancestors_than_DAG = (ancestors_diff.size() > 0);
  if (locally_created_coinbase_block_has_more_ancestors_than_DAG)
    CLog::log("More ancestors: local coinbase(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") has more ancestors(" + CUtils::dumpIt(final_json_block.value("ancestors").toArray()) + " than DAG(" + CUtils::dumpIt(tmp_DAG_ancestors) + ") in cycle range (" + coinbase_from + ", " + coinbase_to + ")" , "cb", "trace");

  CLog::log("Is about to issuing coinbase block in cycle range (" + coinbase_from + ", " + coinbase_to + ") the block: " + CUtils::serializeJson(final_json_block)  , "cb", "trace");

  QStringList missedBlocks = MissedBlocksHandler::getMissedBlocksToInvoke();
  if (missedBlocks.size() > 0)
    CLog::log("BTW machine has some missed blocks: " + CUtils::dumpIt(missedBlocks)  , "cb", "warning");

  // FIXME: it is a way to evoid creating too many coinbases which have a little difference because of the ancestors.
  // could it be a security issue? when an adversory in last minutes(before midnight or mid-day) starts to spam network by blocks
  // and most of nodes can not be synched, so too many coinbase blocks creating
  //
  if (
    (locally_created_coinbase_block_has_more_confidence_than_DAG || locally_created_coinbase_block_has_more_ancestors_than_DAG)&&
    (MissedBlocksHandler::getMissedBlocksToInvoke().size() < 1))
  {

    // broadcast coin base
    if (CUtils::isInCurrentCycle(final_json_block.value("bCDate").toString()))
    {
      bool push_res = SendingQHandler::pushIntoSendingQ(
        CConsts::BLOCK_TYPES::Coinbase,
        final_json_block.value("bHash").toString(),
        CUtils::serializeJson(final_json_block),
        "Broadcasting coinbase block CB(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") issued by(" + CMachine::getPubEmailInfo().m_address + " for cycle range(" + coinbase_from + ", " + coinbase_to + ")");

      CLog::log("coinbase push1 res(" + CUtils::dumpIt(push_res) + ")", "cb", "trace");
      CLog::log("Coinbase issued because of clause 1 CB(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") issued by(" + CMachine::getPubEmailInfo().m_address + " for cycle range(" + coinbase_from + ", " + coinbase_to + ")", "cb", "trace");
      return;
    }

  }
  else if (passedCertainTimeOfCycleToRecordInDAG() && !atleast_one_coinbase_block_exist)
  {
    // another psudo random emulatore
    // if already passed more than 1/4 of cycle and still no coinbase block recorded in DAG,
    // so the machine has to create one
    if (haveIFirstHashedEmail("desc"))
    {
      bool push_res = SendingQHandler::pushIntoSendingQ(
        CConsts::BLOCK_TYPES::Coinbase,
        final_json_block.value("bHash").toString(),
        CUtils::serializeJson(final_json_block),
        "Broadcasting coinbase block CB(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") issued by(" + CMachine::getPubEmailInfo().m_address + " for cycle range(" + coinbase_from + ", " + coinbase_to + ")");

      CLog::log("coinbase push2 res(" + CUtils::dumpIt(push_res) + ")", "cb", "trace");
      CLog::log("Coinbase issued because of clause 2 CB(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") issued by(" + CMachine::getPubEmailInfo().m_address + " for cycle range(" + coinbase_from + ", " + coinbase_to + ")", "cb", "trace");
      return;
    }

  }
  else
  {
    CLog::log("Coinbase can be issued by clause 3 but local hasn't neither more confidence nor more ancestors and still not riched to 1/4 of cycle time. CB(" + CUtils::hash8c(final_json_block.value("bHash").toString()) + ") issued by(" + CMachine::getPubEmailInfo().m_address + " for cycle range(" + coinbase_from + ", " + coinbase_to + ")", "cb", "trace");
    return;
  }

}

 */