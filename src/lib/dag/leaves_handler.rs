use serde_json::json;
use crate::lib::custom_types::{CDateT, JSonT};
use crate::lib::k_v_handler::get_value;

/*

const QString LeavesHandler::STBL_KVALUE = "c_kvalue";

std::tuple<bool, QString> LeavesHandler::removeFromLeaveBlocks(const QStringList& leaves)
{
  QJsonObject current = getLeaveBlocks();
  QJsonObject newLeaves {};
  QStringList keys = current.keys();
  for (QString a_key : keys)
  {
    if (std::find(leaves.begin(), leaves.end(), a_key) == leaves.end())
    {
      // push it to new vector
      newLeaves.insert(a_key, current.value(a_key));
    }
  }

  // update db
  QString newLeavesSer = CUtils::serializeJson(newLeaves);
  QVDicT values;
  values["kv_value"] = newLeavesSer;
  values["kv_last_modified"] = CUtils::getNow();

  DbModel::upsert(
    LeavesHandler::STBL_KVALUE,
    "kv_key",
    "DAG_LEAVE_BLOCKS",
    values,
    true);

  return {true, ""};
}
*/

//old_name_was getLeaveBlocks
pub fn get_leave_blocks(only_before_date:&CDateT)->JSonT
{
  let value:String = get_value(&"DAG_LEAVE_BLOCKS".to_string());
    /*
  if (value == "")
    return QJsonObject {};

//  QueryRes currentLeaves = DbModel::select(
//    LeavesHandler::STBL_KVALUE,
//    QStringList {"kv_value"},
//    {ModelClause("kv_key", QString::fromStdString("DAG_LEAVE_BLOCKS"))},
//    {},   // order
//    1);   // limit
//  if (currentLeaves.records.size() == 0) {
//    QJsonObject json_obj {};
//    return json_obj;
//  }

  QJsonObject json_obj = CUtils::parseToJsonObj(value);
  if (only_before_date == "")
  {
    return json_obj;
  }

  // filter older leaves  FIXME: complete it
  QJsonObject filterd_json_obj {};
  for (QString a_key: json_obj.keys())
  {
    QJsonObject aLeave = json_obj.value(a_key).toObject();
    if (
       (aLeave.value("bType").toString() == CConsts::BLOCK_TYPES::Genesis) ||
       (aLeave.value("bCDate").toString() < only_before_date)
       )
    {
      filterd_json_obj.insert(a_key, json_obj.value(a_key));
    }
  }

  return filterd_json_obj;
    */
        return json!({});
}

/*
std::tuple<bool, QString> LeavesHandler::addToLeaveBlocks(
  const CBlockHashT& block_hash,
  const CDateT& creation_date,
  const QString& bType)
{
  QJsonObject current = getLeaveBlocks();
  QJsonObject jsonObj
  {
      {"bType", bType},
      {"bCDate", creation_date},
  };
  current.insert(block_hash, jsonObj);

  QVDicT values;
  values["kv_value"] = CUtils::serializeJson(current);
  values["kv_last_modified"] = CUtils::getNow();

  DbModel::upsert(
    LeavesHandler::STBL_KVALUE,
    "kv_key",
    "DAG_LEAVE_BLOCKS",
    values,
    true);

  return {true, ""};
}
*/

//old_name_was getFreshLeaves
 pub fn get_fresh_leaves()->JSonT
{
  // the leaves younger than two cylce (24 hours) old
  let mut leaves:JSonT = get_leave_blocks(&"".to_string());

/*
  CLog::log("current leaves: " + CUtils::serializeJson(leaves));

  if (leaves.keys().size() == 0)
    return leaves;

  QString now = CUtils::getNow();
  QJsonObject refreshes = {};
  for (QString a_key : leaves.keys())
  {
    QJsonObject aLeave = leaves.value(a_key).toObject();
    uint64_t leaveAge = CUtils::timeDiff(aLeave.value("bCDate").toString(), now).asMinutes;
    QString msg = "leave("+CUtils::hash8c(a_key)+") age (" + QString("%1").arg(leaveAge) + ") minutes is " +
          ((leaveAge < CMachine::getCycleByMinutes() * 2) ? "younger" : "older") +
        " than 2 cycles";
    CLog::log(msg);
    if (leaveAge < CMachine::getCycleByMinutes() * 2)
    {
      refreshes.insert(a_key, leaves[a_key]);
    }
  }

  return refreshes;
 */
    return json!({});
}


//old_name_was hasFreshLeaves
pub fn has_fresh_leaves()->bool
{
    true
  // let freshLeaves = get_fresh_leaves();
  // return (freshLeaves.keys().size() > 0);
}

