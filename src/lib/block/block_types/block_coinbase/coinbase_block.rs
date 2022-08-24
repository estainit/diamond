use crate::lib::custom_types::JSonObject;
use serde::{Serialize, Deserialize};
use crate::{ccrypto, constants, dlog};
use crate::lib::block::block_types::block::Block;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CoinbaseBlock
{
    pub m_cycle: String,
}

impl CoinbaseBlock {
    pub fn new() -> CoinbaseBlock {
        CoinbaseBlock {
            m_cycle: "".to_string()
        }
    }

    pub fn set_by_json_obj(&mut self, _obj: &JSonObject) -> bool
    {
        // maybe drived class assignings
        return true;
    }

    /*


    /**
    * @brief CoinbaseBlock::handleReceivedBlock(
    * @return <status, should_purge_record>
    */
    std::tuple<bool, bool> CoinbaseBlock::handleReceivedBlock() const
    {
      auto[status, should_purge_record] = validateCoinbaseBlock();
      CLog::log("Received validate CoinbaseBlock result: status(" + cutils::dumpIt(status) + ") should_purge_record(" + cutils::dumpIt(should_purge_record) + ")", "cb", "trace");
      if (!status) {
        // do something (e.g. bad reputation log for sender neighbor)
        return {false, true};
      }

      CLog::log("dummy log pre add to DAG a CoinbaseBlock: " + cutils::serializeJson(export_block_to_json()), "cb", "trace");

      addBlockToDAG();

      postAddBlockToDAG();


      // broadcast block to neighbors
      if (cutils::isInCurrentCycle(m_block_creation_date))
      {
        bool push_res = SendingQHandler::pushIntoSendingQ(
          m_block_type,
          m_block_hash,
          safe_stringify_block(false),
          "Broadcasting the confirmed coinbase block(" + cutils::hash8c(m_block_hash) + ") in current cycle(" + m_cycle + ")");

        CLog::log("coinbase push_res(" + cutils::dumpIt(push_res) + ")");
      }

      return {true, true};
    }


    /**
    * @brief CoinbaseBlock::validateCoinbaseBlock
    * @return <status, should_purge_record>
    */
    std::tuple<bool, bool> CoinbaseBlock::validateCoinbaseBlock() const
    {

      auto[cycleStamp, from_, to_, fromHour, toHour] = cutils::getCoinbaseInfo("", m_cycle);
      Q_UNUSED(cycleStamp);
      Q_UNUSED(fromHour);
      Q_UNUSED(toHour);
      CLog::log("\nValidate Coinbase Block(" + cutils::hash8c(m_block_hash) + ") cycle:(" + m_cycle + ") from:(" + from_ + ") to:(" + to_ + ")", "cb", "info");
    //  let res = { err: true, shouldPurgeMessage: true, msg: '', sender: sender };


      // in case of synching, we force machine to (maybe)conclude the open pollings
      // this code should be somewhere else, because conceptually it has nothing with coinbase flow!
      if (CMachine::is_in_sync_process())
        PollingHandler::doConcludeTreatment(m_block_creation_date);

      auto[status, local_regenerated_coinbase] = CoinbaseIssuer::doGenerateCoinbaseBlock(
        m_cycle,
        constants::STAGES::Regenerating,
        m_block_version);
      Q_UNUSED(status);

      // re-write remote values on local values
      local_regenerated_coinbase["ancestors"] = cutils::convertStringListToJSonArray(m_ancestors);
      local_regenerated_coinbase["bHash"] = constants::HASH_PROP_PLACEHOLDER;
      local_regenerated_coinbase["bLen"] = cutils::padding_length_value(cutils::serializeJson(local_regenerated_coinbase).length());

      Block* tmp_block = BlockFactory::create(local_regenerated_coinbase);
      tmp_block->setBlockHash(tmp_block->calcBlockHash());
      local_regenerated_coinbase["bHash"] = tmp_block->getBlockHash();
      JSonObject tmp_Jblock = tmp_block->export_block_to_json();
    //  tmp_Jblock["bLen"] = cutils::padding_length_value(cutils::serializeJson(tmp_Jblock).length());
      CLog::log("dummy dumping after calculating it's length(serialized): " + cutils::serializeJson(tmp_Jblock) , "cb", "info");
      delete tmp_block;

      CLog::log("dummy dumping local_regenerated_coinbase before calculating it's length(serialized): " + cutils::serializeJson(local_regenerated_coinbase) , "cb", "info");
    //  CLog::log("dummy dumping local_regenerated_coinbase beofr calculating it's length(object): " + cutils::dumpIt(local_regenerated_coinbase) , "cb", "info");

      if (tmp_Jblock.value("bDocsRootHash").to_string() != m_documents_root_hash)
      {
        String msg = "Discrepancy in bDocsRootHash locally created coinbase bDocsRootHash(" + cutils::hash8c(tmp_Jblock.value("bDocsRootHash").to_string());
        msg += ") and remote-bDocsRootHash(" + cutils::hash8c(m_documents_root_hash) + ") Block(" + cutils::hash8c(m_block_hash) + ") ";
        CLog::log(msg, "cb", "error");
        CLog::log("Remote block" + dumpBlock(), "cb", "error");
        CLog::log("Local regenrated block" + cutils::serializeJson(tmp_Jblock), "cb", "error");

        {
          // TODO: remove this block of code after all tests
          auto[status, local_regenerated_coinbase] = CoinbaseIssuer::doGenerateCoinbaseBlock(
            m_cycle,
            constants::STAGES::Regenerating,
            m_block_version);
          Q_UNUSED(status);

          Block* tmp_block = BlockFactory::create(local_regenerated_coinbase);
          auto dummyHash = tmp_block->calcBlockHash();
        }

        return {false, true};
      }

      if (tmp_Jblock.value("bHash").to_string() == "")
      {
        CLog::log("big failllllllll 1 . Regenerated coinbase in local has no hash! " + cutils::dumpIt(tmp_Jblock), "cb", "error");
        return {false, true};
      }

      if (tmp_Jblock.value("bHash").to_string() != m_block_hash)
      {
        CLog::log("big failllllllll2 in Regenerating coinbase in local, has differetnt hash! " + cutils::dumpIt(tmp_Jblock), "cb", "error");
        return {false, true};
      }

      CLog::log("remoteConfidence(" + String::number(m_block_confidence)+ ")", "cb", "info");
      CLog::log("Valid Coinbase block has received. Block(" + m_block_hash + ")", "cb", "info");
      return {true, true};
    }

*/

    pub fn get_block_hashable_string(&self, block: &Block) -> String
    {
        // in order to have almost same hash! we sort the attribiutes alphabeticaly
        let block_hashables: String = format!(
            "bAncestors:{},bCDate:{},bDocsRootHash:{},bLen:{},bType:{},bVer:{},cycle:{},net:{}",
            serde_json::to_string(&block.m_block_ancestors).unwrap(),
            block.m_block_creation_date,
            block.m_block_documents_root_hash, // note that we do not put the docsHash directly in block hash, instead using docsHash-merkle-root-hash
            block.m_block_length,
            block.m_block_type,
            block.m_block_version,
            self.m_cycle,
            block.m_block_net
        );
        return block_hashables;
    }

    pub fn calc_block_hash(&self, block: &Block) -> String
    {
        let hashable_block: String = self.get_block_hashable_string(block);

        // clonedTransactionsRootHash: block.clonedTransactionsRootHash,
        // note that we do not put the clonedTransactions directly in block hash,
        // instead using clonedTransactions-merkle-root-hash

        dlog(
            &format!("The Coinbase! block hashable: {}", hashable_block),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return ccrypto::keccak256(&hashable_block);
    }
    /*

    JSonObject CoinbaseBlock::export_block_to_json(const bool ext_info_in_document) const
    {
      JSonObject block = Block::export_block_to_json(ext_info_in_document);

      // maybe remove add some item in object
      block.remove("bExtInfo");

      if (block.keys().contains("bExtHash"))
        block.remove("bExtHash");

      if (m_block_descriptions == "")
        block["descriptions"] = constants::JS_FAKSE_NULL;

      if (block["bVer"].to_string() > "0.0.0")
        block.remove("descriptions");

      if (block.keys().contains("fVotes"))
        block.remove("fVotes");

      if (block.keys().contains("signals"))
        block.remove("signals");

      if (block.keys().contains("backer"))
        block.remove("backer");

      block.insert("bCycle", m_cycle);
      block["bLen"] = cutils::padding_length_value(calcBlockLength(block));
      return block;
    }

    String CoinbaseBlock::safe_stringify_block(const bool ext_info_in_document) const
    {
      JSonObject block = export_block_to_json(ext_info_in_document);

      // recaluculate block final length
      String tmp_stringified = cutils::serializeJson(block);
      block["bLen"] = cutils::padding_length_value(tmp_stringified.length());

      String out = cutils::serializeJson(block);
      CLog::log("Safe sringified block(Coinbase) Block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(out.length()) + ") the block: " + out, "app", "trace");

      return out;
    }

    BlockLenT CoinbaseBlock::calcBlockLength(const JSonObject& block_obj) const
    {
      return cutils::serializeJson(block_obj).length();
    }

    bool CoinbaseBlock::controlBlockLength() const
    {
      String stringyfied_block = safe_stringify_block(false);
      if (
          (static_cast<BlockLenT>(stringyfied_block.length()) != m_block_length) &&
          (static_cast<BlockLenT>(stringyfied_block.length()) != m_block_length + 136) // legacy JS coinbase created blocks have mis-calculated block length
      )
      {
        CLog::log("Mismatch coinbase block length Block(" + cutils::hash8c(m_block_hash) + ") local length(" + String::number(stringyfied_block.length()) + ") remote length(" + String::number(m_block_length) + ") stringyfied_block:" + stringyfied_block, "sec", "error");
        return false;
      }
      return true;
    }

    String CoinbaseBlock::stringify_block_ext_info() const
    {
      return "";
    }

    */
}