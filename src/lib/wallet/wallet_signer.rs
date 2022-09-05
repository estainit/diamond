use crate::{cutils, machine};
use crate::lib::custom_types::{ClausesT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::q_select;
use crate::lib::database::tables::C_MACHINE_USED_COINS;
use crate::lib::wallet::wallet_coins::delete_from_funds;

/*



void Wallet::locallyMarkUTXOAsUsed(const BasicTxDocument* trx)
{
  QString mp_code = CMachine::getSelectedMProfile();
  for (CInputIndexT i = 0; i < trx->m_inputs.size(); i++)
  {
    CCoinCodeT refLoc = trx->m_inputs[i]->getCoinCode();
    QVDicT values {
      {"lu_mp_code", mp_code},
      {"lu_coin", refLoc},
      {"lu_spend_loc", trx->getDocHash()},
      {"lu_insert_date", cutils::getNow()}};
    CLog::log("Mark the coin as used coin: " + cutils::dumpIt(values), "app", "info");
    DbModel::insert(
      stbl_machine_used_coins,
      values);
  }
}
*/

//old_name_was excludeLocallyUsedCoins
pub fn exclude_locally_used_coins() -> bool
{
    let (_status, locally_used_coins) = q_select(
        C_MACHINE_USED_COINS,
        vec!["lu_coin"],
        vec![],
        vec![],
        0,
        false);
    let mp_code = machine().get_selected_m_profile();
    for a_coin in locally_used_coins
    {
        let (doc_hash, output_index) =
            cutils::unpack_coin_code(&a_coin["lu_coin"]);
        delete_from_funds(
            &doc_hash,
            output_index,
            &mp_code,
        );
    }
    return true;
}


//old_name_was searchLocallyMarkedUTXOs
#[allow(unused, dead_code)]
pub fn search_in_locally_marked_coins(
    clauses: ClausesT,
    fields: Vec<&str>,
    order: OrderT,
    limit: LimitT) -> QVDRecordsT
{
    let (_status, records) = q_select(
        C_MACHINE_USED_COINS,
        fields,
        clauses,
        order,
        limit,
        false);

    return records;
}

/*
std::tuple<bool, QString> Wallet::walletSigner(
  QStringList coins,
  CMPAIValueT sending_amount,
  CMPAIValueT desired_trx_fee,
  CAddressT recipient,
  CAddressT change_back_address,
  CMPAIValueT output_bill_size,  // 0 means one output for entire sending coins
  QString comment)
{
  QString msg = "";

  int unlocker_index = 0; // if client didn't mention, chose the first unlock struct FIXME =0

  if (coins.size() == 0)
  {
    msg = "No coin selected to spend!";
    CLog::log(msg, "app", "warning");
    return {false, msg};
  }

  if (recipient == "")
  {
    msg = "The recipient was missed!";
    CLog::log(msg, "app", "warning");
    return {false, msg};
  }

  if (sending_amount == 0)
  {
    msg = "Missed sending amount!";
    CLog::log(msg, "app", "warning");
    return {false, msg};
  }

  if (desired_trx_fee == 0)
  {
    msg = "Missed transaction fee!";
    CLog::log(msg, "app", "warning");
    return {false, msg};
  }

  //TODO: double-controll if mentioned coins are spendable and avoid double spending

  QVDRecordsT coins_records = UTXOHandler::searchInSpendableCoins(
    {{"ut_coin", coins, "IN"}},
    {"ut_coin", "ut_o_value", "ut_o_address"});

  QStringList envolved_spending_addresses = {};
  CMPAIValueT spendable_amount = 0;
  QHash<CCoinCodeT, TInput> inputs {};
  for (QVDicT a_coin: coins_records)
  {
    CAddressT coin_owner = a_coin["ut_o_address"].toString();
    CMPAIValueT coin_value = a_coin["ut_o_value"].toDouble();

    envolved_spending_addresses.append(coin_owner);
    spendable_amount += coin_value;

    QStringList a_coin_segments = a_coin["ut_coin"].toString().split(":");
    inputs[a_coin["ut_coin"].toString()] = TInput {
      a_coin_segments[0],
      static_cast<CInputIndexT>(QVariant::fromValue(a_coin_segments[1]).toUInt()),
      coin_owner,
      coin_value};

  }

  CMPAISValueT change_back_amount = spendable_amount - sending_amount - desired_trx_fee;
  if (change_back_amount < 0)
  {
    msg = "Output more than inut fund! " + cutils::sep_num_3(change_back_amount);
    CLog::log(msg, "app", "warning");
    return {false, msg};
  }

  std::vector<TOutput> outputs = {};
  if ( change_back_address == "")
    change_back_address = CMachine::getBackerAddress();

  outputs.emplace_back(TOutput{
    change_back_address,
    static_cast<CMPAIValueT>(change_back_amount),
    constants::OUTPUT_CHANGEBACK});

  if (output_bill_size == 0) {
      outputs.emplace_back(TOutput{
        recipient,
        sending_amount,
        constants::OUTPUT_NORMAL});

  } else {
      while (sending_amount >= output_bill_size)
      {
        outputs.emplace_back(TOutput{
          recipient,
          output_bill_size,
          constants::OUTPUT_NORMAL});
        sending_amount -= output_bill_size;
      }
      if (sending_amount > 0)
        outputs.emplace_back(TOutput {
          recipient,
          sending_amount,
          constants::OUTPUT_NORMAL});
  }


  QVDRecordsT addresses_records = getAddressesInfo(envolved_spending_addresses);
  QV2DicT addresses_dict = {};
  for(QVDicT an_address: addresses_records)
    addresses_dict[an_address["wa_address"].toString()] = an_address;

  // find the unlockers
  for (QString a_coin: inputs.keys())
  {

    QJsonObject address_details = cutils::parseToJsonObj(addresses_dict[inputs[a_coin].m_owner]["wa_detail"].toString());
    QJsonObject uSet = address_details["uSets"].toArray()[unlocker_index].toObject();
    inputs[a_coin].m_unlock_set = uSet;
    QString salt = uSet["salt"].toString();
    QJsonArray private_keys = address_details["the_private_keys"].toObject()[salt].toArray();
    inputs[a_coin].m_private_keys = cutils::convertJSonArrayToQStringList(private_keys);

    CLog::log("\nCoin to be sepnt: " + inputs[a_coin].dumpMe(), "trx", "info");
  }

  auto trx_template = BasicTransactionTemplate {
    inputs,
    outputs,
    0,  // max trx fee
    desired_trx_fee,    // dDPCost
    comment};

  auto[res_status, res_msg, trx, dp_cost] = BasicTransactionHandler::makeATransaction(trx_template);
  if (!res_status)
    return {false, res_msg};

  CLog::log("In wallet signer the signed baic transaction: " + trx->safeStringifyDoc(), "trx", "info");

  // push transaction to Block buffer
  auto[buffer_push_res, buffer_push_msg] = CMachine::pushToBlockBuffer(trx, dp_cost);
  if (!buffer_push_res)
    return {false, buffer_push_msg};

  // remove from wallet funds
  deleteFromFunds(trx);

  locallyMarkUTXOAsUsed(trx);

  msg = "Transaction(" + cutils::hash8c(trx->getDocHash())+ ") have been created and pushed to block buffer";
  CLog::log(msg, "trx", "info");

  delete trx;

  return {true, msg};
}

std::tuple<bool, QString, QStringList, QJsonObject> Wallet::signByAnAddress(
  const CAddressT& signer_address,
  const QString& sign_message,
  CSigIndexT unlocker_index)
{
  QString msg;

  QVDRecordsT addresses_details = Wallet::getAddressesInfo({signer_address});
  if (addresses_details.size() != 1)
  {
    msg = "The address " + cutils::shortBech16(signer_address) + " is not controlled by current wallet/profile!";
    CLog::log(msg, "app", "error");
    return {false, msg, {}, {}};
  }

  QJsonObject addrDtl = cutils::parseToJsonObj(addresses_details[0]["wa_detail"].toString());
  QStringList signatures {};
  QJsonObject dExtInfo {};
  QJsonObject unlock_set = addrDtl["uSets"].toArray()[unlocker_index].toObject();
  QJsonArray sSets = unlock_set["sSets"].toArray();

  for (CSigIndexT inx = 0; inx < sSets.size(); inx++)
  {
    auto[signing_res, signature_hex, signature] = CCrypto::ECDSAsignMessage(
      addrDtl["the_private_keys"].toObject()[unlock_set["salt"].toString()].toArray()[inx].toString(),
      sign_message);
    if (!signing_res)
    {
      msg = "Failed in sign of Salt(" + unlock_set["salt"].toString() + ")";
      CLog::log(msg, "app", "error");
      return {false, msg, {}, {}};
    }
    signatures.append(QString::fromStdString(signature_hex));
  }
  if (signatures.size() == 0)
  {
    msg = "The message couldn't be signed";
    CLog::log(msg, "app", "error");
    return {false, msg, {}, {}};
  }

  return {true, "", signatures, unlock_set};
}

*/