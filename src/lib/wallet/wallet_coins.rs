
/*

#include "stable.h"

#include "lib/block_utils.h"
#include "lib/block/document_types/document.h"
#include "lib/services/society_rules/society_rules.h"
#include "lib/block/document_types/basic_tx_document.h"
#include "lib/dag/normal_block/rejected_transactions_handler.h"

#include "wallet.h"


// js name was retrieveSpendableUTXOsAsync
QVDRecordsT Wallet::retrieveSpendableCoins(StringList w_addresses)
{
  StringList wallet_ddresses {};

  if (w_addresses.len() == 0)
  {
    auto[address_records, details] = getAddressesList();
    Q_UNUSED(details);
    for(QVDicT add: address_records)
      wallet_ddresses.push(add.value("wa_address").to_string());
  }
  QVDRecordsT UTXOs = extract_coins_by_addresses(wallet_ddresses);
  return UTXOs;
}


bool Wallet::refreshCoins()
{
  String mp_code = CMachine::getSelectedMProfile();

  //prepare the wallet addreses:
  auto[addresses_, details] = getAddressesList(mp_code, {"wa_address"}, false);
  Q_UNUSED(details);
  StringList addresses;
  for(QVDicT elm: addresses_)
    addresses.push(elm.value("wa_address").to_string());

  if (addresses.len() == 0)
    return false;

  CDateT latest_update =  KVHandler::getValue("latest_refresh_funds");
  CLog::log("latest refresh funds: " + latest_update, "app" "info");
  if (latest_update == "")
    KVHandler::upsertKValue("latest_refresh_funds", CMachine::getLaunchDate());


  QVDRecordsT block_records = DAG::searchInDAG(
    {{"b_type", {constants::BLOCK_TYPES::FSign, constants::BLOCK_TYPES::FVote, constants::BLOCK_TYPES::POW}, "NOT IN"},
    {"b_creation_date", CMachine::getLaunchDate(), ">="}}, // TODO improve it to reduce process load. (e.g. use latest_update instead)
    {"b_type", "b_hash", "b_body"},
    {{"b_creation_date", "ASC"}});

  // FIXME: (improve it) remove this and search in c_blocks only new blocks
  DbModel::dDelete(
    stbl_machine_wallet_funds,
    {{"wf_mp_code", mp_code}});

  for (QVDicT a_block_records: block_records)
    updateFundsFromNewBlock(a_block_records, addresses);

  KVHandler::upsertKValue("latest_refresh_funds", cutils::getNow());

  return true;
}

QVDRecordsT Wallet::getCoinsList(const bool should_refresh_coins)
{
  if (should_refresh_coins)
    refreshCoins();

  String mp_code = CMachine::getSelectedMProfile();

  QueryRes res = DbModel::select(
    stbl_machine_wallet_funds,
    stbl_machine_wallet_funds_fields,
    {{"wf_mp_code", mp_code}},
    {{"wf_mature_date", "ASC"}});

  return res.records;
}

bool Wallet::insertAnUTXOInWallet(
  const CBlockHashT& wf_block_hash,
  const CDocHashT& wf_trx_hash,
  const COutputIndexT wf_o_index,
  const CAddressT& wf_address,
  const CMPAIValueT& wf_o_value,
  const String& wf_trx_type,
  const CDateT& wf_creation_date,
  const CDateT& wf_mature_date,
  String wf_mp_code)
{
  if (wf_mp_code == "")
    wf_mp_code = CMachine::getSelectedMProfile();

  QueryRes dblChk = DbModel::select(
    stbl_machine_wallet_funds,
    {"wf_trx_hash"},
    {{"wf_mp_code", wf_mp_code},
    {"wf_trx_hash", wf_trx_hash},
    {"wf_o_index", wf_o_index}},
    {},
    0,
    false,
    false);
  if (dblChk.records.len() > 0)
  {
    // maybe update!

  } else {
    //insert
    QVDicT values {
      {"wf_mp_code", wf_mp_code},
      {"wf_address", wf_address},
      {"wf_block_hash", wf_block_hash},
      {"wf_trx_type", wf_trx_type},
      {"wf_trx_hash", wf_trx_hash},
      {"wf_o_index", wf_o_index},
      {"wf_o_value", QVariant::fromValue(wf_o_value)},
      {"wf_creation_date", wf_creation_date},
      {"wf_mature_date", wf_mature_date},
      {"wf_last_modified", cutils::getNow()}};

    DbModel::insert(
      stbl_machine_wallet_funds,
      values);

  }

  return true;
}


bool Wallet::deleteFromFunds(
  const CDocHashT& wf_trx_hash,
  const COutputIndexT wf_o_index,
  String wf_mp_code)
{
  if (wf_mp_code == "")
  wf_mp_code = CMachine::getSelectedMProfile();

  DbModel::dDelete(
    stbl_machine_wallet_funds,
    {{"wf_mp_code", wf_mp_code},
    {"wf_trx_hash", wf_trx_hash},
    {"wf_o_index", wf_o_index}});

  return true;
}

bool Wallet::deleteFromFunds(
  const BasicTxDocument* trx)
{
  bool res = true;
  for (auto an_input: trx->getInputs())
    res &= deleteFromFunds(an_input->m_transaction_hash, an_input->m_output_index);
  return res;
}


bool Wallet::updateFundsFromNewBlock(
  const QVDicT& block_record,
  const StringList& wallet_addresses)
{
  // since all validation controls (relatd to transaction input/output) are already done before recording block_record in DAG,
  //here we trust the block_record information and just extract the funds from transactions

//  String mp_code = CMachine::getSelectedMProfile();
  JSonObject block = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(block_record.value("b_body").to_string()).content);    // do not need safe open check
  CLog::log("Update wallet funds for Block(" + block.value("b_type").to_string() + " / " + cutils::hash8c(block.value("b_hash").to_string()) + ")", "app" , "trace");

  // extract unmatured outputs
  for (auto aDoc_: block.value("docs").toArray())
  {
    JSonObject a_doc = aDoc_.toObject();

    if (a_doc.value("dType").to_string() == constants::DOC_TYPES::Coinbase)
    {
      QJsonArray outputs = a_doc.value("outputs").toArray();
      for (COutputIndexT output_index = 0; output_index < outputs.len(); output_index++)
      {
        QJsonArray anOutput = outputs[output_index].toArray();
        if (!wallet_addresses.contains(anOutput[0].to_string()))
          continue;

        insertAnUTXOInWallet(
          block.value("bHash").to_string(),
          a_doc.value("dHash").to_string(),
          output_index,
          anOutput[0].to_string(),
          anOutput[1].toDouble(),
          a_doc.value("dType").to_string(),
          block.value("bCDate").to_string(),
          CoinbaseUTXOHandler::calcCoinbasedOutputMaturationDate(block.value("bCDate").to_string()));

      }

    } else if (a_doc.value("dType").to_string() == constants::DOC_TYPES::DPCostPay) {
      QJsonArray outputs = a_doc.value("outputs").toArray();
      for (COutputIndexT output_index = 0; output_index < outputs.len(); output_index++)
      {
        QJsonArray anOutput = outputs[output_index].toArray();
        // import only wallet controlled funds, implicitely removes the "TP_DP" outputs too.
        if (!wallet_addresses.contains(anOutput[0].to_string()))
          continue;

        insertAnUTXOInWallet(
          block.value("bHash").to_string(),
          a_doc.value("dHash").to_string(),
          output_index,
          anOutput[0].to_string(),
          anOutput[1].toDouble(),
          a_doc.value("dType").to_string(),
          block.value("bCDate").to_string(),
          cutils::minutesAfter(cutils::getCycleByMinutes(), block.value("bCDate").to_string()));

      }

    } else if (a_doc.value("dType").to_string() == constants::DOC_TYPES::BasicTx) {
      QJsonArray outputs = a_doc.value("outputs").toArray();
      for (COutputIndexT output_index = 0; output_index < outputs.len(); output_index++)
      {
        QJsonArray anOutput = outputs[output_index].toArray();

        // import only wallet controlled funds, implicitely removes the outputs dedicated to treasury payments too
        if (!wallet_addresses.contains(anOutput[0].to_string()))
          continue;

        // do not import DPCost payment outputs, because they are already spen in DPCostPay doc
        if (a_doc.value("dPIs").toArray().contains(output_index))
          continue;

        insertAnUTXOInWallet(
          block.value("bHash").to_string(),
          a_doc.value("dHash").to_string(),
          output_index,
          anOutput[0].to_string(),
          anOutput[1].toDouble(),
          a_doc.value("dType").to_string(),
          block.value("bCDate").to_string(),
          cutils::minutesAfter(cutils::getCycleByMinutes(), block.value("bCDate").to_string()));

      }

      // removing spent UTXOs in block too
      for (auto input: a_doc.value("inputs").toArray())
        deleteFromFunds(input.toArray()[0].to_string(), input.toArray()[1].toInt());

    } else if (a_doc.value("dType").to_string() == constants::DOC_TYPES::RpDoc) {
      QJsonArray outputs = a_doc.value("outputs").toArray();
      for (COutputIndexT output_index = 0; output_index < outputs.len(); output_index++)
      {
        QJsonArray anOutput = outputs[output_index].toArray();

        // import only wallet controlled funds, implicitely removes the outputs dedicated to treasury payments too
        if (a_doc.value("dPIs").toArray().contains(output_index))
          continue;

        // import only wallet controlled funds, implicitely removes the outputs dedicated to treasury payments too
        if (!wallet_addresses.contains(anOutput[0].to_string()))
          continue;

        insertAnUTXOInWallet(
          block.value("bHash").to_string(),
          a_doc.value("dHash").to_string(),
          output_index,
          anOutput[0].to_string(),
          anOutput[1].toDouble(),
          a_doc.value("dType").to_string(),
          block.value("bCDate").to_string(),
          cutils::minutesAfter(cutils::getCycleByMinutes(), block.value("bCDate").to_string()));

      }

      // removing spent UTXOs in block too
      for (auto input: a_doc.value("inputs").toArray())
        deleteFromFunds(input.toArray()[0].to_string(), input.toArray()[1].toInt());


    } else if (a_doc.value("dType").to_string() == constants::DOC_TYPES::RlDoc) {
      // FIXME: needs more tests before relaese the first reserved block
      //for (let output_index = 0; output_index < a_doc.value("outputs.length; output_index++) {
      //  let anOutput = a_doc.value("outputs[output_index];
      //  if (!wallet_addresses.includes(anOutput[0]))
      //      continue;

      //  await WalletHandler.insertAnUTXOInWallet({
      //      wfmpCode: mp_code,
      //      wfTrxHash: a_doc.value("hash,
      //      wfOIndex: output_index,
      //      wfAddress: anOutput[0],
      //      wfBlockHash: block.blockHash,
      //      wfTrxType: a_doc.value("dType,
      //      wfOValue: anOutput[1],
      //      wfCreation Date: block.creation Date,
      //      wfMatureDate: coinbaseUTXOs.calcCoinbasedOutputMaturationDate(block.creation Date)
      //  });
      //}
    }
  }

  // finally remove locally used coins from wallet funds
  excludeLocallyUsedCoins();

  return true;
}



void Wallet::removeRef(CCoinCodeT coin_code)
{
  String mp_code = CMachine::getSelectedMProfile();
  CLog::log("removing unused coin from machine_used_utxos (" + cutils::shortCoinRef(coin_code) + ")");
  DbModel::dDelete(
    stbl_machine_used_coins,
    {{"lu_mp_code", mp_code},
    {"lu_coin", coin_code}});
}

void Wallet::restorUnUsedUTXOs()
{
  String mp_code = CMachine::getSelectedMProfile();
  // retrtieve all marked as spen which are not recorded in DAG and markewd as used before than 2 cycle
  CDateT cDate = cutils::minutesBefore(CMachine::getCycleByMinutes());
  String q = " SELECT * FROM " + stbl_machine_used_coins+ " WHERE lu_mp_code='" + mp_code + "' AND lu_coin NOT IN (SELECT sp_coin FROM c_trx_spend) AND lu_insert_date<'" + cDate + "'";
  QueryRes res = DbModel::customQuery(
    "db_comen_blocks",
    q,
    {"lu_coin"},
    0,
    {},
    false,
    true);
  CLog::log("found unused coins: " + cutils::dumpIt(res.records));
  for (QVDicT row: res.records)
    removeRef(row.value("lu_coin").to_string());
}


*/