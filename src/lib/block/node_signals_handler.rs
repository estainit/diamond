use serde_json::json;
use crate::constants;
use crate::lib::block::block_types::block::Block;
use crate::lib::custom_types::JSonObject;

//old_name_was logSignals
pub fn log_signals(_block: &Block)
{
    /*
  for (QJsonValue aSignal_ : block.m_signals)
  {
    JSonObject aSignal = aSignal_.toObject();
    String sigKey = aSignal.value("sig").to_string();
    String sigVal;
    if (aSignal.keys().contains("val"))
    {
      auto tmpSigVal = aSignal.value("val");
      if (tmpSigVal.isObject())
      {
        sigVal = cutils::serializeJson(tmpSigVal.toObject());

      }
      else if (tmpSigVal.isString())
      {
        sigVal = tmpSigVal.to_string();

      }
    }
    else
    {
      // backward compatibility
      sigVal = aSignal.value("ver").to_string();
    }

    QVDicT values{
      {"sig_signaler", block.m_block_backer},
      {"sig_block_hash", block.m_block_hash},
      {"sig_key", sigKey},
      {"sig_value", sigVal},
      {"sig_creation_date", block.m_block_creation_date}};

    DbModel::insert(
      NodeSignalsHandler::stbl_signals,     // table
      values, // values to insert
      true
    );
  }
     */
}

/*
QVDRecordsT NodeSignalsHandler::searchInSignals(
  const ClausesT& clauses,
  const VString& fields,
  const OrderT& order,
  const int& limit)
{
  QueryRes res = DbModel::select(
    stbl_signals,
    fields,
    clauses,
    order,
    limit);

  return res.records;
}


*/

//old_name_was getMachineSignals
pub fn get_machine_signals() -> Vec<JSonObject>
{
    let signals: JSonObject = json!({
      "nodeInfo": json! ({
        "spec": "Rust",
        "ver": constants::CLIENT_VERSION
      }),
      "P4Psupport": constants::NO
    });

    //signals = listener.doCallSync('SASH_signals', { signals_ });
    return vec![signals];
}


