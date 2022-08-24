use crate::{ccrypto, constants, dlog};
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::TimeByHoursT;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct PollingDocument
{
    pub m_voting_timeframe: TimeByHoursT,
    pub m_polling_ref: String,
    // reference to document for which is running this polling
    pub m_polling_ref_type: String,
    // refType
    pub m_polling_ref_class: String,
    pub m_polling_comment: String,
    pub m_polling_creator: String,
    pub m_polling_start_date: String,
    pub m_polling_status: String,

    pub m_potential_voters_count: u64,
}

impl PollingDocument
{
    pub fn new()->PollingDocument{
        PollingDocument{
            m_voting_timeframe: 0.0,
            m_polling_ref: "".to_string(),
            m_polling_ref_type: "".to_string(),
            m_polling_ref_class: "".to_string(),
            m_polling_comment: "".to_string(),
            m_polling_creator: "".to_string(),
            m_polling_start_date: "".to_string(),
            m_polling_status: "".to_string(),
            m_potential_voters_count: 0
        }
    }
    /*

    PollingDocument::PollingDocument(const JSonObject& obj)
    {
      set_by_json_obj(obj);
    }

    bool PollingDocument::set_by_json_obj(const JSonObject& obj)
    {
      Document::set_by_json_obj(obj);

      if (obj.value("pTimeframe").toDouble() != 0)
        m_voting_timeframe = obj.value("pTimeframe").toDouble();

      if (obj.value("dRef").to_string() != "")
        m_polling_ref = obj.value("dRef").to_string();

      if (obj.value("dRefType").to_string() != "")
        m_polling_ref_type = obj.value("dRefType").to_string();

      if (obj.value("dRefClass").to_string() != "")
        m_polling_ref_class = obj.value("dRefClass").to_string();

      if (obj.value("dComment").to_string() != "")
        m_polling_comment = obj.value("dComment").to_string();

      if (obj.value("dCreator").to_string() != "")
        m_polling_creator = obj.value("dCreator").to_string();

      if (obj.value("startDate").to_string() != "")
        m_polling_start_date = obj.value("startDate").to_string();

      if (obj.value("status").to_string() != "")
        m_polling_status = obj.value("status").to_string();

      return true;
    }

    JSonObject PollingDocument::export_doc_to_json(const bool ext_info_in_document) const
    {
      JSonObject document = Document::export_doc_to_json(ext_info_in_document);

      document["dCreator"] = m_polling_creator;
      document["dRefType"] = m_polling_ref_type;
      document["dRefClass"] = m_polling_ref_class;
      document["pTimeframe"] = m_voting_timeframe;

      return document;
    }

    String PollingDocument::safe_stringify_doc(const bool ext_info_in_document) const
    {
      JSonObject document = export_doc_to_json(ext_info_in_document);

      // recaluculate block final length
      document["dLen"] = cutils::padding_length_value(cutils::serializeJson(document).length());

      CLog::log("11 safe Sringify Doc(" + cutils::hash8c(m_doc_hash) + "): " + m_doc_type + " / " + m_doc_class + " length:" + String::number(cutils::serializeJson(document).length()) + " serialized document: " + cutils::serializeJson(document), "app", "trace");

      return cutils::serializeJson(document);
    }

    */

    //old_name_was getDocHashableString
    pub fn get_doc_hashable_string(&self, doc: &Document) -> String
    {
        let hahsables: String = format!(
            "dExtHash:{},dLen:{},",
            doc.m_doc_ext_hash,
            doc.m_doc_length
        );
        return hahsables;
        // //FIXME: probably we need more complete hashables. e.g. like transaction documents
        // let mut hahsables = "{";
        // hahsables += "\"dExtHash\":\"" + doc.m_doc_ext_hash.clone() + "\",";
        // hahsables += "\"dLen\":\"" + cutils::padding_length_value(doc.m_doc_length.to_string(), constants::LEN_PROP_LENGTH) + "\"}";
        // return hahsables.to_string();
    }

    //old_name_was calcDocHash
    pub fn calc_doc_hash(&self, doc: &Document) -> String // old name was calcHashDPolling
    {
        // as always alphabetical sort
        let hashables: String = self.get_doc_hashable_string(doc);
        let hashed: String = ccrypto::keccak256(&hashables);
        dlog(
            &format!("Hashable string for Polling doc{} doc({}/{}) hash({})", hashables, doc.m_doc_type, doc.m_doc_class, hashed),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);
        return hashed;
    }
    /*
    // old name was calcPollingCost
    std::tuple<bool, CMPAIValueT> PollingDocument::calcDocDataAndProcessCost(
      const String& stage,
      String cDate,
      const uint32_t& extraLength) const
    {
      Q_UNUSED(extraLength);
      if (cDate == "")
        cDate =cutils::getNow();

      DocLenT dLen = m_doc_length;

      CMPAIValueT the_cost =
          m_potential_voters_count *
          dLen *
          SocietyRules::getBasePricePerChar(cDate) *
          SocietyRules::getDocExpense(m_doc_type, dLen, m_doc_class, cDate);
    //      cnfHandler.getDocExpense({ cDate, dType: polling.dType, dClass: polling.dClass, dLen });

      if (stage == constants::STAGES::Creating)
      {
        the_cost = the_cost * CMachine::getMachineServiceInterests(
          m_doc_type,
          m_doc_class,
          dLen);
        CLog::log("calc cutom post the_cost + machine interest(" + cutils::sepNum(the_cost) +" micro PAIs) type/class(" + m_doc_type + "/" + m_doc_class + ") Doc(" + cutils::hash8c(m_doc_hash) + ")", "app", "trace");
      }

      return {true, cutils::CFloor(the_cost)};
    }

    std::tuple<bool, JSonArray> PollingDocument::exportInputsToJson() const
    {
      return {false, JSonArray {}};
    }

    std::tuple<bool, JSonArray> PollingDocument::exportOutputsToJson() const
    {
      return {false, JSonArray {}};
    }

    // js name was recordAPolling & recordPollingInDB
    bool PollingDocument::applyDocFirstImpact(const Block& block) const
    {
      return PollingHandler::recordPollingInDB(
        block,
        this);
    }

    String PollingDocument::get_ref() const
    {
      return m_polling_ref;
    }

    String PollingDocument::getRefType() const
    {
      return m_polling_ref_type;
    }

    String PollingDocument::getRefClass() const
    {
      return m_polling_ref_class;
    }

    String PollingDocument::getDocToBeSignedHash() const
    {
      String signables = "{";
      signables += "\"dCDate\":\"" + m_doc_creation_date + "\",";
      signables += "\"dClass\":\"" + m_doc_class + "\",";
      signables += "\"dComment\":\"" + m_polling_comment + "\",";
      signables += "\"dCreator\":\"" + m_polling_creator + "\",";
      signables += "\"dRef\":\"" + get_ref() + "\",";
      signables += "\"dRefClass\":\"" + m_polling_ref_class + "\",";
      signables += "\"dRefType\":\"" + m_polling_ref_type + "\",";
      signables += "\"dType\":\"" + m_doc_type + "\",";
      signables += "\"dVer\":\"" + m_doc_version + "\",";
      signables += "\"pTimeframe\":" + String::number(m_voting_timeframe) + "}";

      String to_be_signed_hash = ccrypto::keccak256(signables);
      CLog::log("Polling to_be_signed_hash(" + to_be_signed_hash + ") signables: " + signables + " ", "app", "trace");
      return to_be_signed_hash;
    }

    String PollingDocument::calcDocExtInfoHash() const //calcTrxExtRootHash()
    {
      String hash, hashables = "";

      hashables += "{\"signatures\":" + cutils::serializeJson(m_doc_ext_info[0].toObject().value("signatures").toVariant().toJsonArray()) + ",";
      hashables += "\"signedHash\":\"" + getDocToBeSignedHash() + "\",";
      hashables += "\"uSet\":" + SignatureStructureHandler::safeStringifyUnlockSet(m_doc_ext_info[0].toObject().value("uSet").toObject()) + "}";

      hash = ccrypto::keccak256(hashables);
      CLog::log("Ext Hash Hashables polling(" + m_doc_hash + ") Regenrated Ext hash: " + hash + " hashables: " + hashables, "app", "trace");
      return hash;
    }

    // old name was importPollingsCost
    void PollingDocument::importCostsToTreasury(
      const Block* block,
      CoinImportDataContainer* block_inspect_container)
    {
      QHash<CDocHashT, CostPaymentStatus> cost_payment_status {};

      if (block_inspect_container.m_block_alter_treasury_incomes.keys().contains( "TP_POLLING"))
      {
        for (BlockAlterTreasuryIncome a_treasury_entry: block_inspect_container.m_block_alter_treasury_incomes["TP_POLLING"])
        {
            // if polling costs was payed by a valid trx
          bool doc_cost_is_payed = true;

          if (block_inspect_container.m_rejected_transactions.keys().contains(a_treasury_entry.m_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[a_treasury_entry.m_trx_hash] = {"supporter transaction(" + cutils::hash8c(a_treasury_entry.m_trx_hash) + ") for Polling is rejected because of doublespending", false};
          }

          if (!block_inspect_container.m_map_U_trx_hash_to_trx_ref.keys().contains(a_treasury_entry.m_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[a_treasury_entry.m_trx_hash] = {"The Polling costs is not supported by any trx", false};
          }
          CDocHashT polling_hash = block_inspect_container.m_map_U_trx_hash_to_trx_ref[a_treasury_entry.m_trx_hash];
          auto[inx_, polling_doc] = block->getDocumentByHash(polling_hash);
          Q_UNUSED(inx_);

          if (polling_doc.m_doc_class != PollingHandler::POLLING_PROFILE_CLASSES["Basic"]["ppName"].to_string())
          {
            doc_cost_is_payed = false;
            cost_payment_status[polling_hash] = {"Polling dClass(" + polling_doc.m_doc_class + ") is not supported", false};
          }

          CDocHashT supporter_trx = block_inspect_container.m_map_U_trx_ref_to_trx_hash[polling_hash];
          StringList rejected_transactions =block_inspect_container.m_rejected_transactions.keys();
          if (rejected_transactions.contains(supporter_trx) || rejected_transactions.contains(a_treasury_entry.m_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[polling_hash] = {"supporter transaction is rejected because of doublespending", false};
          }


          if (doc_cost_is_payed)
          {
            CLog::log("Successfully TP_POLLING Block(" + cutils::hash8c(block->getBlockHash()) + ") Coin(" + cutils::shortCoinRef(a_treasury_entry.m_coin) + ") importing(TP_POLLING)", "app", "trace");

            cost_payment_status[polling_hash].m_message = "Ballot Cost imported to treasury succsessfully.";
            String title = "TP_POLLING Polling(" + cutils::hash6c(polling_hash) + ")";
            TreasuryHandler::insertIncome(
              title,
              "TP_POLLING",
              title,
              block.m_block_creation_date,
              a_treasury_entry.m_value,
              block->getBlockHash(),
              a_treasury_entry.m_coin);

            } else {
              CLog::log("Failed TP_... Block(" + cutils::hash8c(block->getBlockHash()) + ") Coin(" + cutils::shortCoinRef(a_treasury_entry.m_coin) + ") importing(TP_POLLING)", "sec", "error");
              CLog::log("cost_payment_status not payed: " + CoinImportDataContainer::dumpMe(cost_payment_status[polling_hash]), "sec", "error");

              PollingHandler::removePollingG(polling_hash);

              // remove referenced & related doc for which there is a polling
              String ref_type = polling_doc->getRefType();
              if (ref_type == constants::DOC_TYPES::ReqForRelRes)
              {
                ResevedCoinsHandler::removeReqRelRes(polling_doc->get_ref());

              }else{

              }
            }
        }
      }
      block_inspect_container.m_cost_payment_status["TP_POLLING"] = cost_payment_status;

    }

    */
}