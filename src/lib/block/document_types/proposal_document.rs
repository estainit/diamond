use std::collections::HashMap;
use postgres::types::ToSql;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use crate::{application, ccrypto, constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CAddressT, CDateT, CDocHashT, ClausesT, JSonObject, TimeByHoursT};
use crate::lib::database::abs_psql::{q_insert, q_select, q_update, simple_eq_clause};
use crate::lib::database::tables::C_PROPOSALS;
use crate::lib::services::polling::polling_handler::auto_create_polling_for_proposal;

#[derive(Clone, Serialize, Deserialize)]
pub struct ProposalDocument
{
    pub m_project_hash: CDocHashT,
    // "";//58be4875eaa3736f60622e26bda746fb81812e8d7ecad50a2c3f97f0605a662c
    pub m_approval_date: CDateT,
    pub m_contributor_account: CAddressT,
    pub m_help_hours: i32,
    pub m_help_level: i32,
    pub m_shares: i64,
    pub m_votes_yes: i64,
    pub m_votes_abstain: i64,
    pub m_votes_no: i64,
    pub m_voting_timeframe: TimeByHoursT,
    pub m_polling_profile: String,
    pub m_polling_version: String,
}

impl ProposalDocument {
    pub fn new() -> ProposalDocument {
        ProposalDocument {
            m_project_hash: "".to_string(),
            m_approval_date: "".to_string(),
            m_contributor_account: "".to_string(),
            m_help_hours: 0,
            m_help_level: 0,
            m_shares: 0,
            m_votes_yes: 0,
            m_votes_abstain: 0,
            m_votes_no: 0,
            m_voting_timeframe: 0.0,
            m_polling_profile: "Basic".to_string(),
            m_polling_version: "0.0.0".to_string(),
        }
    }

    pub fn set_by_json_doc(&mut self, obj: &JSonObject) -> bool
    {
        panic!("gggggg {}" , obj);
        self.m_help_hours = obj["helpHours"].to_string().parse::<i32>().unwrap();
        self.m_help_level = obj["helpLevel"].to_string().parse::<i32>().unwrap();
        self.m_project_hash = obj["projectHash"].to_string();
        self.m_contributor_account = obj["contributor"].to_string();
        self.m_polling_profile = obj["pollingProfile"].to_string();
        self.m_polling_version = obj["pollingVersion"].to_string();

        self.m_voting_timeframe = obj["pTimeframe"].to_string().parse::<TimeByHoursT>().unwrap();
        if application().cycle_length() == 1
        {
            self.m_voting_timeframe = self.m_voting_timeframe as TimeByHoursT;
        }     // because of test ambient the longivity can be float and les than 1 hour

        return true;
    }

    pub fn export_doc_to_json(&self, doc: &Document, ext_info_in_document: bool) -> JSonObject {
        let mut document: JSonObject = doc.export_doc_to_json_inner(ext_info_in_document);

        document["projectHash"] = self.m_project_hash.clone().into();
        document["helpHours"] = self.m_help_hours.clone().into();
        document["helpLevel"] = self.m_help_level.clone().into();
        document["contributor"] = self.m_contributor_account.clone().into();
        document["pTimeframe"] = cutils::convert_float_to_string(self.m_voting_timeframe, constants::FLOAT_LENGTH).into();
        document["pollingProfile"] = self.m_polling_profile.clone().into();
        document["pollingVersion"] = self.m_polling_version.clone().into();

        return document;
    }

    /*
    //JSonObject ProposalDocument::ZexportJson() const
    //{
    //  JSonObject params = JSonObject
    //  {
    //    {"m_doc_hash", m_doc_hash},
    //    {"m_doc_tags", m_doc_tags},
    //    {"m_project_hash", m_project_hash},
    //    {"m_doc_title", m_doc_title},
    //    {"m_doc_comment", m_doc_comment},
    //    {"m_shareholder", m_contributor_account}, // creator  contributor
    //    {"m_help_hours", String::number(m_help_hours)},
    //    {"m_help_level", String::number(m_help_level)},

    //    {"m_votes_yes", String::number(m_votes_yes)},
    //    {"m_votes_abstain", String::number(m_votes_abstain)},
    //    {"m_votes_no", String::number(m_votes_no)},
    //    {"m_voting_timeframe", String::number(m_voting_timeframe)},
    //    {"m_polling_profile", m_polling_profile},
    //    {"m_doc_creation_date", m_doc_creation_date},
    //    {"m_block_creation_date", m_block_creation_date},

    //    {"m_shares", String::number(m_shares)},
    //    {"m_approval_date", m_approval_date},

    //  //    {"comment", QJsonValue(String::fromStdString("Polling for proposal(" + cutils::hash6c(proposal.get_doc_hash()) + "), " + proposal.m_doc_title + " "))},
    //  //    {"status", QJsonValue(String::fromStdString(constants::OPEN))}
    //    };
    //  return params;
    //}

    */
    pub fn safe_stringify_doc(&self, doc: &Document, ext_info_in_document: bool) -> String
    {
        let mut j_doc: JSonObject = self.export_doc_to_json(doc, ext_info_in_document);

        // recaluculate block final length
        j_doc["dLen"] = cutils::padding_length_value(
            cutils::serialize_json(&j_doc).len().to_string(),
            constants::LEN_PROP_LENGTH).into();
        dlog(
            &format!("do safe Sringify Doc({}): {} / {} length: {} serialized document: {}",
                     cutils::hash8c(&doc.m_doc_hash),
                     &doc.m_doc_type,
                     &doc.m_doc_class,
                     cutils::serialize_json(&j_doc).len(),
                     cutils::serialize_json(&j_doc)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return cutils::serialize_json(&j_doc);
    }
    /*


    // old name was calcProposalDocumetnCost
    std::tuple<bool, CMPAIValueT> ProposalDocument::calcDocDataAndProcessCost(
      const String& stage,
      String cDate,
      const uint32_t& extraLength) const
    {
      Q_UNUSED(extraLength);
      if (cDate == "")
        cDate =application().get_now();

      DocLenT dLen = m_doc_length;

      CMPAIValueT the_cost =
          dLen *
          SocietyRules::getBasePricePerChar(cDate) *
          SocietyRules::getDocExpense(m_doc_type, dLen, m_doc_class, cDate);

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


    String ProposalDocument::calcDocExtInfoHash() const
    {
      return constants::NO_EXT_HASH;
    }

    bool ProposalDocument::hasSignable() const
    {
      return false;
    }

    //// old name was
    //String ProposalDocument::getDocSignMsg() const
    //{
    //  String signables = "{";
    ////  signables += "\"creation Date\":\"" + m_doc_creation_date + "\"," ;
    ////  signables += "\"dClass\":\"" + m_doc_class + "\"," ;
    ////  signables += "\"dType\":\"" + m_doc_type + "\"," ;
    ////  signables += "\"dVer\":\"" + m_doc_version + "\"," ;
    ////  signables += "\"ref\":\"" + m_doc_ref + "\"," ;
    ////  signables += "\"vote\":" + String::number(m_vote) + "," ;
    ////  signables += "\"voter\":\"" + m_voter + "\"}" ;

    ////  String sign_message = ccrypto::keccak256(signables);
    ////  sign_message = sign_message.midRef(0, constants::SIGN_MSG_LENGTH).to_string();
    ////  CLog::log("Ballot sign_message(" + sign_message + ") signables: " + signables + " ", "app", "trace");
    ////  return sign_message;
    //}

    //bool ProposalDocument::veridfyDocSignature() const
    //{
    //  JSonObject dExtInfo = m_doc_ext_info;
    ////  auto unlockSet = dExtInfo.value("uSet").toObject();

    ////  bool is_valid_unlock = SignatureStructureHandler::validateSigStruct(
    ////    unlockSet,
    ////    m_voter);

    ////  if (!is_valid_unlock)
    ////  {
    ////    CLog::log("Invalid creator signature structure on Ballot(" + m_doc_hash + ")! ", "sec", "error");
    ////    return false;
    ////  }

    ////  // ballot signature & permission validate check
    ////  String sign_message = getDocSignMsg();
    ////  JSonArray signatures = m_doc_ext_info[0].toObject().value("signatures").toArray();
    ////  for (CSigIndexT signature_index = 0; signature_index < signatures.len(); signature_index++)
    ////  {
    ////    String a_signature = signatures[signature_index].to_string();
    ////    try {
    ////      bool verifyRes = ccrypto::ECDSAVerifysignature(
    ////        unlockSet.value("sSets").toArray()[signature_index].toObject().value("sKey").to_string(),
    ////        sign_message,
    ////        a_signature);
    ////      if (!verifyRes)
    ////      {
    ////        CLog::log("Invalid creator signature sign on ballot(" + m_doc_hash + ")! ", "sec", "error");
    ////        return false;
    ////      }
    ////    } catch (std::exception) {
    ////      CLog::log("Exception! Invalid creator signature sign on ballot(" + m_doc_hash + ")! ", "sec", "error");
    ////      return false;
    ////    }
    ////  }
    //  return false;
    //}

    */

    // js name was extractHashableParts
    //old_name_was getDocHashableString
    pub fn get_doc_hashable_string(&self, doc: &Document) -> String
    {
        let doc_hahsables: String = format!(
            "dCDate:{},dClass:{},dComment:{},dLen:{},dTags:{},dTitle:{},dType:{},dVer:{},contributor:{},helpHours:{},helpLevel:{},pollingProfile:{},pollingVersion:{},projectHash:{},pTimeframe:{}",
            doc.m_doc_creation_date,
            doc.m_doc_class,
            doc.m_doc_comment,
            cutils::padding_length_value(doc.m_doc_length.to_string(), constants::LEN_PROP_LENGTH),
            doc.m_doc_tags,
            doc.m_doc_title,
            doc.m_doc_type,
            doc.m_doc_version,
            self.m_contributor_account,
            self.m_help_hours,
            self.m_help_level,
            self.m_polling_profile,
            self.m_polling_version,
            self.m_project_hash,
            self.m_voting_timeframe
        );
        return doc_hahsables;
    }

    //old_name_was doHashProposal
    //old_name_was calcDocHash
    pub fn calc_doc_hash(&self, doc: &Document) -> String
    {
        dlog(
            &format!("calc proposal Hash: {}", self.safe_stringify_doc(doc, true)),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        let hashables: String = self.get_doc_hashable_string(doc);

        let the_hash = ccrypto::keccak256(&hashables);
        dlog(
            &format!("Hashable string for dna proposal doc({} / {}) hash of ({})",
                     doc.m_doc_type, doc.m_doc_class, hashables),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return the_hash;
    }
    /*

    std::tuple<bool, JSonArray> ProposalDocument::exportInputsToJson() const
    {
      return {false, JSonArray {}};
    }

    std::tuple<bool, JSonArray> ProposalDocument::exportOutputsToJson() const
    {
      return {false, JSonArray {}};
    }

    std::vector<TInput*> ProposalDocument::get_inputs() const
    {
      return {};
    }

    std::vector<TOutput*> ProposalDocument::get_outputs() const
    {
      return {};
    }

    void ProposalDocument::importCostsToTreasury(
      const Block* block,
      UTXOImportDataContainer* block_inspect_container)
    {

      QHash<CDocHashT, CostPaymentStatus> cost_payment_status {};

      if (block_inspect_container.m_block_alter_treasury_incomes.contains("TP_PROPOSAL"))
      {
        CLog::log("Try to import TP_PROPOSAL for block(" + cutils::hash8c(block->getBlockHash()) + "): " + UTXOImportDataContainer::dumpMe(block_inspect_container.m_block_alter_treasury_incomes["TP_PROPOSAL"]), "trx", "trace");

        QSDicT mapPledgeHashToPayerTrxHash {};
        QSDicT mapProposalToPledgedContract {};
        for (Document* aDoc: block->getDocuments())
        {
          // retrieve proposals & pledge mappings
          if (
            (aDoc.m_doc_type == constants::DOC_TYPES::Pledge) &&
            (aDoc.m_doc_class == constants::PLEDGE_CLASSES::PledgeP) &&
            (aDoc->getProposalRef() != ""))
          {
            if(aDoc->getPayerTrxLinkBack() != "")
              mapPledgeHashToPayerTrxHash[aDoc->get_doc_hash()] = aDoc->getPayerTrxLinkBack();
            mapProposalToPledgedContract[aDoc->getProposalRef()] = aDoc->get_doc_hash();
          }
        }


        for (BlockAlterTreasuryIncome a_treasury_entry: block_inspect_container.m_block_alter_treasury_incomes["TP_PROPOSAL"])
        {
          // if proposal costs was payed by Pledging & a lending transaction
          bool doc_cost_is_payed = true;

          if (block_inspect_container.m_rejected_transactions.keys().contains(a_treasury_entry.m_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[a_treasury_entry.m_trx_hash] = {"Supporter transaction(" + cutils::hash8c(a_treasury_entry.m_trx_hash) + ") for Proposal is rejected because of doublespending", false};
          }

          if (!block_inspect_container.m_map_U_trx_hash_to_trx_ref.keys().contains(a_treasury_entry.m_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[a_treasury_entry.m_trx_hash] = {"The proposal costs is not supported by any trx!", false};
          }

          CDocHashT proposal_hash = block_inspect_container.m_map_U_trx_hash_to_trx_ref[a_treasury_entry.m_trx_hash];

          auto[_index, proposal_doc] = block->getDocumentByHash(proposal_hash);
          Q_UNUSED(_index);

          if (proposal_doc.m_doc_class != constants::PROPOSAL_CLASESS::General)
            cost_payment_status[proposal_hash] = {"Proposal dClass(" + proposal_doc.m_doc_class + ") is not supported yet", false};


          String income_title = "";
          CDocHashT payer_trx_hash = block_inspect_container.m_map_U_trx_ref_to_trx_hash[proposal_hash];

          CDocHashT supporter_pledge = "";
          if (mapProposalToPledgedContract.keys().contains(proposal_hash))
          {
            // the proposer, in order to pay proposal costs pledged its future incomes
            supporter_pledge = mapProposalToPledgedContract[proposal_hash];
            if (!mapPledgeHashToPayerTrxHash.keys().contains(supporter_pledge))
            {
              doc_cost_is_payed = false;
              cost_payment_status[proposal_hash] = {"Proposal is not payed by a transaction", false};
            }


            CDocHashT payer_trx_hash2 = mapPledgeHashToPayerTrxHash[supporter_pledge];
            if (!block_inspect_container.m_map_U_trx_hash_to_trx_ref.keys().contains(payer_trx_hash2) ||
                (block_inspect_container.m_map_U_trx_hash_to_trx_ref[payer_trx_hash2] != proposal_hash))
            {
              doc_cost_is_payed = false;
              cost_payment_status[proposal_hash] = {"Supporter transaction is referred to different doc", false};
            }


            if (payer_trx_hash != payer_trx_hash2)
            {
              doc_cost_is_payed = false;
              cost_payment_status[proposal_hash] = {"Not same payer_trx_hash! " + cutils::hash8c(payer_trx_hash) + "!=" + cutils::hash8c(payer_trx_hash2) + " !!", false};
            }


            income_title = "TP_PROPOSAL Proposal(" + cutils::hash8c(proposal_hash) + ") Pledge(" + cutils::hash8c(supporter_pledge) + ") Trx(" + cutils::hash8c(payer_trx_hash) + ")";

          } else {
            // probably the proposal costs payed by a transaction directly signed by proposer
            income_title = "TP_PROPOSAL Proposal(" + cutils::hash8c(proposal_hash) + ") Trx(" + cutils::hash8c(payer_trx_hash) + ")";

          }

          if (block_inspect_container.m_rejected_transactions.keys().contains(payer_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[proposal_hash] = {"Supporter transaction is rejected because of doublespending", false};
          }

          if (!block_inspect_container.m_map_U_trx_ref_to_trx_hash.keys().contains(proposal_hash) ||
              (block_inspect_container.m_map_U_trx_ref_to_trx_hash[proposal_hash] != payer_trx_hash))
          {
            doc_cost_is_payed = false;
            cost_payment_status[proposal_hash] = {"supporter transaction ref is defferent than proposal hash", false};
          }

          cost_payment_status[proposal_hash].m_is_payed = doc_cost_is_payed;
          if (doc_cost_is_payed)
          {
            CLog::log("Successfully TP_PROPOSAL Block(" + cutils::hash8c(block->getBlockHash()) + ") Coin(" + cutils::shortCoinRef(a_treasury_entry.m_coin) + ") importing(TP_PROPOSAL)", "app", "trace");

            cost_payment_status[proposal_hash].m_message = "Proposal Cost imported to treasury succsessfully.";
            TreasuryHandler::insertIncome(
              income_title,
              "TP_PROPOSAL",
              income_title,
              block.m_block_creation_date,
              a_treasury_entry.m_value,
              block->getBlockHash(),
              a_treasury_entry.m_coin);

          } else {
              CLog::log("Failed TP_... Block(" + cutils::hash8c(block->getBlockHash()) + ") Coin(" + cutils::shortCoinRef(a_treasury_entry.m_coin) + ") importing(TP_PROPOSAL)", "sec", "error");
              CLog::log("Failed cost_payment_status: Proposal(" + cutils::hash8c(proposal_hash) + ")" + UTXOImportDataContainer::dumpMe(cost_payment_status[proposal_hash]), "sec", "error");

              // since by adding block to DAG, the proposals(and related pollings) also were added to DAG, and after 12 hours we found the payment
              // for that particulare proposal is failed, so we must remove both (proposal & polling) from data base
              // also removing pledge!(if exist)
              ProposalHandler::removeProposal(proposal_hash);

              PollingHandler::removePollingByRelatedProposal(proposal_hash);

              if (supporter_pledge != "")
                GeneralPledgeHandler::removePledgeBecauseOfPaymentsFail(supporter_pledge);

          }
        }
      }

      block_inspect_container.m_cost_payment_status["TP_PROPOSAL"] = cost_payment_status;
    }

     */


    pub fn update_proposal(
        upd_values: &HashMap<&str, &(dyn ToSql + Sync)>,
        clauses: ClausesT,
        is_transactional: bool) -> (bool, String)
    {
        q_update(
            C_PROPOSALS,
            upd_values, // update values
            clauses,  // update clauses
            is_transactional);

        return (true, "".to_string());
    }


    // js name was activatePollingForProposal
    //old_name_was applyDocFirstImpact
    pub fn apply_doc_first_impact(
        &self,
        doc: &Document,
        block: &Block) -> bool
    {
        // recording block in DAG means also starting voting period for proposals inside block(if exist)
        // it implicitly leads to create a new polling and activate it in order to collect shareholders opinion about proposal
        dlog(
            &format!("Try to activate Proposal Polling({}) Block({})",
                     cutils::hash8c(&doc.m_doc_hash),
                     cutils::hash8c(&block.m_block_hash)),
            constants::Modules::App,
            constants::SecLevel::Debug);


        // record in c_proposals (i_proposal)
        let (_status, records) = q_select(
            C_PROPOSALS,
            vec!["pr_hash"],
            vec![simple_eq_clause("pr_hash", &doc.m_doc_hash)],
            vec![],
            0,
            true,
        );
        if records.len() > 0 {
            dlog(
                &format!("try to double insert existed proposal {}",
                         cutils::hash8c(&doc.m_doc_hash)),
                constants::Modules::Sec,
                constants::SecLevel::Error);
        }

        let pr_help_level = doc.m_if_proposal_doc.m_help_level;
        let pr_help_hours=doc.m_if_proposal_doc.m_help_hours;
        let pr_voting_timeframe=doc.m_if_proposal_doc.m_voting_timeframe;
        let pr_conclude_date="".to_string();
        let pr_approved=constants::NO.to_string();
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("pr_hash", &doc.m_doc_hash as &(dyn ToSql + Sync)),
            ("pr_type", &doc.m_doc_type as &(dyn ToSql + Sync)),
            ("pr_class", &doc.m_doc_class as &(dyn ToSql + Sync)),
            ("pr_version", &doc.m_doc_version as &(dyn ToSql + Sync)),
            ("pr_title", &doc.m_doc_title as &(dyn ToSql + Sync)),
            ("pr_descriptions", &doc.m_doc_comment as &(dyn ToSql + Sync)),
            ("pr_tags", &doc.m_doc_tags as &(dyn ToSql + Sync)),
            ("pr_project_id", &doc.m_if_proposal_doc.m_project_hash as &(dyn ToSql + Sync)),
            ("pr_help_hours", &pr_help_hours as &(dyn ToSql + Sync)),
            ("pr_help_level", &pr_help_level as &(dyn ToSql + Sync)),
            ("pr_voting_timeframe", &pr_voting_timeframe as &(dyn ToSql + Sync)),
            ("pr_polling_profile", &doc.m_if_proposal_doc.m_polling_profile as &(dyn ToSql + Sync)),
            ("pr_contributor_account", &doc.m_if_proposal_doc.m_contributor_account as &(dyn ToSql + Sync)),
            ("pr_start_voting_date", &block.m_block_creation_date as &(dyn ToSql + Sync)),
            ("pr_conclude_date", &pr_conclude_date as &(dyn ToSql + Sync)),
            ("pr_approved", &pr_approved as &(dyn ToSql + Sync))
        ]);

        q_insert(
            C_PROPOSALS,
            &values,
            true);


        // create a new polling
        let mut params: JSonObject = json!({
            "dType": constants::document_types::POLLING,
            "dClass": doc.m_if_proposal_doc.m_polling_profile , // default is iConsts.POLLING_PROFILE_CLASSES.Basic.ppNae
            "dVer": doc.m_doc_version ,
            "dRef": doc.m_doc_hash ,
            "dRefType": constants::polling_ref_types::PROPOSAL,
            "dRefClass": constants::pledge_classes::PLEDGE_P,
            "startDate": block.m_block_creation_date ,
            "pTimeframe": cutils::convert_float_to_string(self.m_voting_timeframe, constants::FLOAT_LENGTH),
            "dCreator": doc.m_if_proposal_doc.m_contributor_account ,
            "related_proposal": doc.m_doc_hash ,
            "dComment": "Polling for proposal(".to_owned() + &cutils::hash6c(&doc.m_doc_hash) + "), " + &doc.m_doc_title + " " ,
            "status": constants::OPEN});
        let res = auto_create_polling_for_proposal(&mut params, block);
        return res;
    }
}