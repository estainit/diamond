/*



class FSignExtInfo {
public:
  UnlockSet m_unlock_set;
  Vec<String> m_signatures;
};


//   -  -  -  FloatingSignatureBlock
class FloatingSignatureBlock : public Block
{

  FSignExtInfo m_fsign_ext_info = {};


};

#endif // FLOATINGSIGNATUREBLOCK_H



#include "stable.h"

#include "lib/dag/dag.h"
#include "lib/ccrypto.h"
#include "lib/transactions/basic_transactions/signature_structure_handler/general_structure.h"
#include "lib/sending_q_handler/sending_q_handler.h"
#include "floating_signature_block.h"

FloatingSignatureBlock::FloatingSignatureBlock(const JSonObject& obj)
{
  set_by_json_obj(obj);
}

bool FloatingSignatureBlock::set_by_json_obj(const JSonObject& obj)
{
  Block::set_by_json_obj(obj);

  // drived class assignings
  m_fsign_ext_info = FSignExtInfo {
    SignatureStructureHandler::convertJsonUSetToStruct(obj["bExtInfo"].toObject()["uSet"].toObject()),
    cutils::convertJSonArrayToStringVector(obj["bExtInfo"].toObject()["signatures"].toArray())};

  return true;
}


bool FloatingSignatureBlock::validateFSBlock() const
{
  String msg;
  // control shares/confidence
  CLog::log("validate FSBlock block Creation Date(" + m_block_creation_date + ") backer(" + m_block_backer  + ") ");
  auto[shares_, issuer_shares_percentage] = get_an_address_shares(m_block_backer, m_block_creation_date);
  Q_UNUSED(shares_);
  if (m_block_confidence != issuer_shares_percentage)
  {
    msg = "FSBlock(" + cutils::hash8c(m_block_hash) + ") was rejected because of wrong! confidence(" + String("%1").arg(m_block_confidence) + ")";
    msg += "!=local(" + String("%1").arg(issuer_shares_percentage) + ")";
    CLog::log(msg, "app", "error");
    return false;
  }

  // control signature
  if (
    (m_fsign_ext_info.m_unlock_set.m_signature_sets.len() == 0) ||
    (m_fsign_ext_info.m_signatures.len() == 0)
  )
  {
    msg = "Rejected FSBlock because of missed bExtInfo FSBlock(" + cutils::hash8c(m_block_hash) + ") ";
    CLog::log(msg, "app", "error");
    return false;
  }


  bool isValidUnlock = validate_sig_struct(
    m_fsign_ext_info.m_unlock_set,
    m_block_backer);
  if (isValidUnlock != true) {
    msg = "Invalid given uSet structure for FSBlock(" + cutils::hash8c(m_block_hash) + ")";
    CLog::log(msg, "app", "error");
    return false;
  }

  String ancestors_str = "[\"" + m_ancestors[0] + "\"]";
//  String ancestors_str = m_ancestors[0];
  String signMsg = ccrypto::keccak256(ancestors_str).midRef(0, constants::SIGN_MSG_LENGTH).to_string();
//  let signMsg = crypto.convertToSignMsg(block.ancestors)

  for (int singatureInx = 0; singatureInx < m_fsign_ext_info.m_unlock_set.m_signature_sets.len(); singatureInx++)
  {
    bool verifyRes = ccrypto::ECDSAVerifysignature(
      m_fsign_ext_info.m_unlock_set.m_signature_sets[singatureInx].m_signature_key,
      signMsg,
      m_fsign_ext_info.m_signatures[singatureInx]);

    if (verifyRes != true)
    {
      msg = "Invalid given signature for FSBlock(" + cutils::hash8c(m_block_hash) + ")";
      CLog::log(msg, "app", "error");
      return false;
    }
  }

  msg = "received FSBlock(" + cutils::hash8c(m_block_hash) + ") is valid";
  CLog::log(msg, "app", "info");
  return true;
}

std::tuple<bool, bool> FloatingSignatureBlock::handle_received_block() const
{

  bool is_valid = validateFSBlock();
  if (!is_valid)
  {
    // do something
    return {false, true};
  }

  // record in dag

  CLog::log("add a valid FSBlock(" + cutils::hash8c(m_block_hash) + ") to DAG", "app", "info");
  addBlockToDAG();
  postAddBlockToDAG();

  // broadcast to neighbors
  if (cutils::isInCurrentCycle(m_block_creation_date))
  {
    bool pushRes = SendingQHandler::push_into_sending_q(
      m_block_type,
      m_block_hash,
      safe_stringify_block(false),
      "Broadcasting the confirmed FS block(" + cutils::hash8c(m_block_hash) + ") in current cycle(" + m_cycle + ")");

    CLog::log("FS pushRes(" + cutils::dumpIt(pushRes) + ")");

  }

  return {true, true};

}

JSonObject FloatingSignatureBlock::export_block_to_json(const bool ext_info_in_document) const
{
  JSonObject block = Block::export_block_to_json(ext_info_in_document);

  block.remove("bDocs");
  block.remove("fVotes");
  block.remove("bDocsRootHash");

  block["confidence"] = m_block_confidence;

  block["bLen"] = cutils::padding_length_value(calcBlockLength(block));

  return block;
}

/**
*
* @param {time} cDate
* the functions accepts a time and searchs for all floating signatures which are signed the prev coinbase block(either linked or not linked blocks)
*/
*/
use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog};
use crate::lib::custom_types::{CDateT, DoubleDicT, QVDRecordsT};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, simple_eq_clause};
use crate::lib::utils::dumper::dump_hashmap_of_qvd_records;

//old_name_was aggrigateFloatingSignatures
pub fn aggrigate_floating_signatures(c_date: &CDateT) -> (f64, Vec<String>, Vec<String>)
{
    // retrieve prev cycle info
    let launch_date = application().launch_date();
    let to_= application().get_coinbase_range(&launch_date).to;
    if application().now() > to_
    {
        let (
            cycle_stamp,
            from_t,
            _to,
            _from_hour,
            _to_hour) = application().get_prev_coinbase_info(c_date);

        // retrieve prev cycle coinbases
        let prv_coinbase_blocks: QVDRecordsT = search_in_dag(
            vec![
                simple_eq_clause("b_type", &constants::block_types::COINBASE.to_string()),
                simple_eq_clause("b_cycle", &cycle_stamp),
                ModelClause {
                    m_field_name: "b_creation_date",
                    m_field_single_str_value: &from_t as &(dyn ToSql + Sync),
                    m_clause_operand: ">=",
                    m_field_multi_values: vec![],
                },
            ],
            vec!["b_hash"],
            vec![
                &OrderModifier { m_field: "b_confidence", m_order: "DESC" },
                &OrderModifier { m_field: "b_hash", m_order: "ASC" },
            ],
            0,
            true,
        );
        dlog(
            &format!("prvCoinbaseBlocks: {}", dump_hashmap_of_qvd_records(&prv_coinbase_blocks)),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        let mut prv_coinbase_blocks_: Vec<String> = vec![];
        for a_row in prv_coinbase_blocks
        {
            prv_coinbase_blocks_.push(a_row["b_hash"].to_string());
        }

        // retrieve all floating signature blocks which are created in prev cycle
        dlog(
            &format!("retrieve floating signatures for cycle({}) from({}) ", cycle_stamp, from_t),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);

        let f_s_w_blocks: QVDRecordsT = search_in_dag(
            vec![
                simple_eq_clause("b_type", &constants::block_types::FLOATING_SIGNATURE.to_string()),
                simple_eq_clause("b_cycle", &cycle_stamp),
                ModelClause {
                    m_field_name: "b_creation_date",
                    m_field_single_str_value: &from_t as &(dyn ToSql + Sync),
                    m_clause_operand: ">=",
                    m_field_multi_values: vec![],
                }], // TODO add a max Date to reduce results
            vec!["b_hash", "b_ancestors", "b_confidence", "b_backer"],
            vec![
                &OrderModifier { m_field: "b_confidence", m_order: "DESC" },
                &OrderModifier { m_field: "b_hash", m_order: "ASC" },
            ],
            0,
            true);

        let mut block_hashes: Vec<String> = vec![];
        let mut backers: DoubleDicT = HashMap::new();
        for a_fs_w_block in f_s_w_blocks
        {
            // drop float if it is not linked to proper coinbase block
            let mut is_linked_to_propoer_cb: bool = false;
            let tmp_ancestors: Vec<String> = a_fs_w_block["b_ncestors"]
                .to_string()
                .split(",")
                .collect::<Vec<&str>>()
                .iter()
                .map(|&x| x.to_string())
                .collect::<Vec<String>>();

            for an_ancestor in tmp_ancestors
            {
                if prv_coinbase_blocks_.contains(&an_ancestor)
                {
                    is_linked_to_propoer_cb = true;
                }
            }
            if !is_linked_to_propoer_cb
            {
                continue;
            }

            backers.insert(a_fs_w_block["b_backer"].to_string(), a_fs_w_block["b_confidence"].parse::<f64>().unwrap());
            block_hashes.push(a_fs_w_block["b_hash"].to_string());
        }
        let mut confidence: f64 = 0.0;
        for (_bckr, v) in &backers
        {
            confidence += v;
        }
        let confidence = cutils::i_floor_float(confidence);

        return (
            confidence,
            block_hashes,
            backers.keys().cloned().collect::<Vec<String>>()
        );
    } else {
        // machine is in init cycle, so there is no floating signature
        let genesis: QVDRecordsT = search_in_dag(
            vec![simple_eq_clause("b_type", &constants::block_types::GENESIS.to_string())],
            vec!["b_hash", "b_ancestors", "b_confidence", "b_backer"],
            vec![
                &OrderModifier { m_field: "b_confidence", m_order: "DESC" },
                &OrderModifier { m_field: "b_hash", m_order: "ASC" },
            ],
            0,
            true);

        return (
            100.00,
            vec![genesis[0]["b_hash"].to_string()], // only the genesis block hash
            vec![]
        );
    }
}
/*

String FloatingSignatureBlock::safe_stringify_block(const bool ext_info_in_document) const
{
  JSonObject block = export_block_to_json(ext_info_in_document);

  // maybe remove add some item in object
  if (m_block_descriptions == "")
    block["descriptions"] = constants::JS_FAKSE_NULL;

  // recaluculate block final length
  String tmp_stringified = cutils::serializeJson(block);
  block["bLen"] = cutils::padding_length_value(tmp_stringified.len());

  String out = cutils::serializeJson(block);
  CLog::log("Safe sringified block(floating signature) Block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(out.len()) + ") the block: " + out, "app", "trace");

  return out;
}

String FloatingSignatureBlock::stringify_block_ext_info() const
{
  JSonArray block_ext_info {m_block_ext_info[0].toObject()};
  String out = cutils::serializeJson(block_ext_info);
  return out;
}

*/