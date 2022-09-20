use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, CMachine, constants, dlog};
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{ClausesT, CMPAIValueT, LimitT, OrderT, QVDRecordsT};
use crate::lib::database::abs_psql::{q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::C_MACHINE_BLOCK_BUFFER;

impl CMachine {
    // js name was pushToBlockBuffer
    pub fn push_to_block_buffer(
        &self,
        doc: &Document,
        dp_cost: CMPAIValueT,
        mp_code: &String) -> (bool, String)
    {
        let msg: String;
        //listener.doCallAsync('APSH_before_push_doc_to_buffer_async', args);

        let dbl_chk = self.search_buffered_docs(
            vec![
                simple_eq_clause("bd_mp_code", mp_code),
                simple_eq_clause("bd_doc_hash", &doc.get_doc_hash()),
            ],
            vec!["bd_doc_hash"],
            vec![],
            0,
        );
        if dbl_chk.len() > 0
        {
            msg = format!("Tried to insert in buffer duplicated {}", doc.get_doc_identifier());
            dlog(
                &msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, msg);
        }

        let payload: String = doc.safe_stringify_doc(true);

        let doc_hash = doc.get_doc_hash();
        let doc_type = doc.get_doc_type();
        let doc_class = doc.get_doc_class();
        let dp_cost_i64 = dp_cost as i64;
        let payload_len_i32 = payload.len() as i32;
        let now_ = application().now();
        let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("bd_mp_code", &mp_code as &(dyn ToSql + Sync)),
            ("bd_insert_date", &now_ as &(dyn ToSql + Sync)),
            ("bd_doc_hash", &doc_hash as &(dyn ToSql + Sync)),
            ("bd_doc_type", &doc_type as &(dyn ToSql + Sync)),
            ("bd_doc_class", &doc_class as &(dyn ToSql + Sync)),
            ("bd_payload", &payload as &(dyn ToSql + Sync)),
            ("bd_dp_cost", &dp_cost_i64 as &(dyn ToSql + Sync)),
            ("bd_doc_len", &payload_len_i32 as &(dyn ToSql + Sync)),
        ]);
        let status: bool = q_insert(
            C_MACHINE_BLOCK_BUFFER,
            &values,
            true);
        dlog(
            &format!("Insert a document in block buffer, values: {:#?}", values),
            constants::Modules::App,
            constants::SecLevel::Info);
        if status
        {
            return (
                true,
                format!("The document have been pushed into buffer. {}", doc.get_doc_identifier())
            );
        }

        dlog(
            &format!("Failed in push doc to block buffer, values: {:#?}", values),
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, "Failed in push doc to block buffer".to_string());
    }

    //old_name_was searchBufferedDocs
    pub fn search_buffered_docs(
        &self,
        clauses: ClausesT,
        fields: Vec<&str>,
        order: OrderT,
        limit: LimitT) -> QVDRecordsT
    {
        let (_status, records) = q_select(
            C_MACHINE_BLOCK_BUFFER,
            fields,
            clauses,
            order,
            limit,
            false);
        return records;
    }
    /*


    std::tuple<bool, String> CMachine::broadcastBlock(
      const String& cost_pay_mode,
      const String& create_date_type)
    {
      Block* block;
      bool should_reset_block_buffer;
      CDateT cheating_creation_date = "";
      StringList cheating_ancestors = {};
      if (create_date_type == "cheat")
      {
        cheating_creation_date = KVHandler::getValue("cheating_creation_date");
        cheating_ancestors = KVHandler::getValue("cheating_ancestors").split(",");
      }

      if (cost_pay_mode == "byPoW")
      {
        // TODO: implement it (if we really need POW payment block types)
        //res = await POWblockHandler.createAPOWBlock({
        //  creationDate: cheating_creation_date,
        //  ancestors: (!utils._nilEmptyFalse(cheating_ancestors)) ? utils.parse(cheating_ancestors) : null
        //});

      } else {
          auto[status, block_, should_reset_block_buffer_, msg] = NormalBlcokHandler::createANormalBlock(
            cheating_ancestors,
            cheating_creation_date,
            (cheating_ancestors.len() > 0));
          should_reset_block_buffer = should_reset_block_buffer_;
          if (!status)
          {
            CLog::log("failed in generating normal block! " + msg, "app", "error");
            return {false, "failed in generating normal block! " + msg};
          }
          block = block_;
      }


      // write file on hard output/send email
      String block_str = block->safeStringifyBlock(false);
      CLog::log("About to sending a normal block to network(" + cutils::hash8c(block->getBlockHash())+ "): " + block_str);

      bool push_res = SendingQHandler::pushIntoSendingQ(
        block->m_block_type,
        block->getBlockHash(),
        block_str,
        "Broadcasting the created normal block(" + cutils::hash8c(block->getBlockHash()) + ") " + cutils::getNow());

      String msg = "Normal block generated & pushed to sending Q. push res(" + cutils::dumpIt(push_res) + ") block(" + cutils::hash8c(block->getBlockHash()) + ") " + cutils::getNow();
      CLog::log(msg);

      // remove from buffer
      if (should_reset_block_buffer)
        CMachine::removeFromBuffer(
          {{"bd_mp_code", CMachine::getSelectedMProfile()},
          {"bd_doc_hash", block->getDocumentsHashes(), "IN"}});

      if (block != nullptr)
        delete block;

      return {true, msg};
    }

    /**
     * @brief CMachine::fetchBufferedTransactions
     * @param block
     * @return {creating block status, should empty buffer, msg}
     */
    std::tuple<bool, bool, String> CMachine::fetchBufferedTransactions(
      Block* block,
      TransientBlockInfo& transient_block_info)
    {
      String msg;

      QVDRecordsT buffered_trxs = search_buffered_docs(
        {{"bd_doc_type", constants::DOC_TYPES::BasicTx}},
        stb_machine_block_buffer_fields,
        {{"bd_dp_cost", "DESC"},
        {"bd_doc_class", "ASC"},
        {"bd_insert_date", "ASC"}});

      // TODO: currently naivly the query select most payer transaction first.
      // the algorythm must be enhanced. specially to deal with block size, and beeing sure
      // if prerequsities doc(e.g payer transaction and referenced doscument & somtimes documents) are all placed in same block!

      CLog::log("The NORMAL block will contain " + String::number(buffered_trxs.len()) + " transactions");

      if (buffered_trxs.len() == 0 )
        return {
          false,
          false,
          "There is no transaction to append to block!"};

      StringList supported_P4P {};// extracting P4P (if exist)
      for (QVDicT serializedTrx: buffered_trxs)
      {
        QJsonObject Jtrx = cutils::parseToJsonObj(serializedTrx.value("bd_payload").to_string());
        Document* trx = DocumentFactory::create(Jtrx);
        Block* tmp_block = new Block(QJsonObject {
          {"bCDate", cutils::getNow()},
          {"bType", "futureBlockTrx"},
          {"bHash", "futureHashTrx"}});
        auto[status, msg] = dynamic_cast<BasicTxDocument*>(trx)->customValidateDoc(tmp_block);
        if (!status)
        {
          msg = "error in validate Doc. transaction(" + cutils::hash8c(trx->getDocHash()) +") block(" + cutils::hash8c(block->getBlockHash()) +")!";
          CLog::log(msg, "trx", "error");
          return {false, false, msg};
        }

        if (trx->m_doc_class == constants::TRX_CLASSES::P4P)
          supported_P4P.push(trx->m_doc_ref);

        for (auto an_output: trx->getOutputs())
          transient_block_info.m_block_total_output += an_output->m_amount;

        block->m_documents.push(trx);
      }

      CMPAIValueT block_total_dp_cost = 0;
      for (auto trx: block->m_documents)
      {
        // collect backer fees
        CMPAISValueT DPCost = 0;
        if (supported_P4P.contains(trx->getDocHash()))
        {
          CLog::log("Block(" + cutils::hash8c(block->getBlockHash()) +") trx(" + cutils::hash8c(trx->getDocHash()) + ") is supported by p4p trx, so this trx must not pay trx-fee", "trx", "info");

        } else {
          // find the backer output
          // check if trx is clone, in which client pays for more than one backer in a transaction
          // in order to ensure more backers put trx in DAG
          for (auto aDPIndex: trx->getDPIs())
            if (trx->getOutputs()[aDPIndex]->m_address == block->m_block_backer)
              DPCost = trx->getOutputs()[aDPIndex]->m_amount;

          if (DPCost == 0)
          {
            msg = "can not create block, because at least one trx hasn't backer fee! transaction(" + cutils::hash8c(trx->getDocHash()) + ") in Block(" + cutils::hash8c(block->getBlockHash()) +")";
            CLog::log(msg, "trx", "error");
            return {false, false, msg};
          }
        }

        block_total_dp_cost += DPCost;
        transient_block_info.m_block_documents_hashes.push(trx->getDocHash());
        transient_block_info.m_block_ext_infos_hashes.push(trx->m_doc_ext_hash);
        //block->m_block_ext_info.push(trx.value("dExtInfo"));
      }

      // create treasury payment
      CMPAISValueT block_fix_cost = SocietyRules::getBlockFixCost(block->m_block_creation_date);
      CMPAISValueT backer_net_fee = cutils::CFloor((block_total_dp_cost * constants::BACKER_PERCENT_OF_BLOCK_FEE) / 100) - block_fix_cost;
      if (backer_net_fee < 0)
      {
        msg = "The block can not cover broadcasting costs! \nblock Total DPCost(" + cutils::microPAIToPAI6(block_total_dp_cost) + "\nbacker Net Fee(" + cutils::microPAIToPAI6(backer_net_fee) + ")";
        CLog::log(msg, "trx", "error");
        return {false, false, msg};
      }

      CMPAISValueT treasury = block_total_dp_cost - backer_net_fee;
      QJsonArray Joutputs {
        {QJsonArray {"TP_DP", QVariant::fromValue(treasury).toDouble()}},
        {QJsonArray {CMachine::getBackerAddress(), QVariant::fromValue(backer_net_fee).toDouble()}}
      };
      Document* DPCostTrx = DocumentFactory::create(QJsonObject {
        {"dType", constants::DOC_TYPES::DPCostPay},
        {"dCDate", block->m_block_creation_date},
        {"outputs", Joutputs}});
      DPCostTrx->setDocHash();

      transient_block_info.m_block_documents_hashes.push_front(DPCostTrx->getDocHash());

      std::vector<Document *> tmp_documents {};
      tmp_documents.push(DPCostTrx);
      for (auto a_doc: block->m_documents)
        tmp_documents.push(a_doc);
      block->m_documents = tmp_documents;

      // block->m_block_ext_info.push_front(QJsonArray{});   // althougt it is empty but must be exits, in order to having right index in block ext Infos array
      // transient_block_info.m_block_ext_infos_hashes.push_front("-");   // althougt it is empty but must be exits, in order to having right index in block ext Infos array

      return {
        true,
        true,
        "Sucessfully appended transactions to block"
      };
    }

    /**
     * @brief CMachine::retrieveAndGroupBufferedDocuments
     * @param block
     * @param transient_block_info
     * @return {status, should clera buffer, err_msg}
     */
    std::tuple<bool, bool, String> CMachine::retrieveAndGroupBufferedDocuments(
      Block* block,
      TransientBlockInfo& transient_block_info)
    {
      String msg;

      QVDRecordsT buffered_docs = search_buffered_docs(
        {},
        stb_machine_block_buffer_fields,
        {{"bd_dp_cost", "DESC"},
        {"bd_doc_class", "ASC"},
        {"bd_insert_date", "ASC"}});
      if (buffered_docs.len() == 0)
        return {true, true, "There is no doc to append!"};

      for (QVDicT serialized_doc: buffered_docs)
      {
        BlockLenT roughly_block_size = block->safeStringifyBlock(true).len();
        // size control TODO: needs a little tuneing
        if (roughly_block_size > ((constants::MAX_BLOCK_LENGTH_BY_CHAR * 80) / 100))
          continue;

        // TODO: it is too important in a unique block exit both trx and it's reffered Doc(if is reffered to a doc),
        // in some case there are even4 document which must exist together in same block
        // add some controll to be sure about it.
        // now it is not the case until reaching buffer  total saize bigger than a single block(almost 10 Mega Byte)

        QJsonObject a_js_doc = cutils::parseToJsonObj(serialized_doc.value("bd_payload").to_string());
        Document* a_document = DocumentFactory::create(a_js_doc);

        if (!cutils::isValidVersionNumber(a_document->m_doc_version))
        {
          msg = "invalid dVer for in retrieve And Group Buffered Documents doc(" + cutils::hash8c(a_document->m_doc_hash) + ")";
          CLog::log(msg, "app", "error");
          return {false, false, msg};
        }

        if (!cutils::isValidDateForamt(a_document->m_doc_creation_date))
        {
         msg = "Invalide date format block-creationDate(" + block->m_block_creation_date + ")!";
         CLog::log(msg, "app", "error");
         return {false, false, msg};
        }

        if (a_document->m_doc_creation_date > block->m_block_creation_date)
        {
         msg = "Creating new block, document creationdate(" + a_document->m_doc_creation_date + ") is after block-creationDate(" + block->m_block_creation_date + ")!";
         CLog::log(msg, "app", "error");
         return {false, false, msg};
        }

        if (a_document->m_doc_creation_date > cutils::getNow())
        {
         msg = "Creating new block, documents is created in future(" + a_document->m_doc_creation_date + ")!";
         CLog::log(msg, "app", "error");
         return {false, false, msg};
        }

    //  // length control
    //  if (parseInt(a_document.dLen) != utils.stringify(a_document).length) {
    //     msg = `create: The doc(${a_document.dType} / ${utils.hash6c(a_document->m_doc_hash)}), stated dLen(${a_document.dLen}), III. is not same as real length(${utils.stringify(a_document).length})!`
    //     clog.sec.error(msg);
    //     return { err: true, msg }
    //  }

      if (!transient_block_info.m_groupped_documents.keys().contains(a_document->m_doc_type))
        transient_block_info.m_groupped_documents[a_document->m_doc_type] = std::vector<Document*> {};

      transient_block_info.m_groupped_documents[a_document->m_doc_type].push(a_document);

        if (a_document->m_doc_ref != "")
        {
         if (Document::canBeACostPayerDoc(a_document->m_doc_type))
         {
           transient_block_info.m_transactions_dict[a_document->m_doc_hash] = a_document;
           transient_block_info.m_map_trx_hash_to_trx_ref[a_document->m_doc_hash] = a_document->m_doc_ref;
           transient_block_info.m_map_trx_ref_to_trx_hash[a_document->m_doc_ref] = a_document->m_doc_hash;
         } else {
           transient_block_info.m_map_referencer_to_referenced[a_document->m_doc_hash] = a_document->m_doc_ref;
           transient_block_info.m_map_referenced_to_referencer[a_document->m_doc_ref] = a_document->m_doc_hash;
         }
        }

        transient_block_info.m_doc_by_hash[a_document->m_doc_hash] = a_document;
      }

      if (transient_block_info.m_map_trx_ref_to_trx_hash.keys().len() != transient_block_info.m_map_trx_hash_to_trx_ref.keys().len())
      {
        msg = "Creating new block, create: transaction count and ref count are different! map trx ref to trx hash: " + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash) + " map trx hash to trx ref: " + cutils::dumpIt(transient_block_info.m_map_trx_hash_to_trx_ref);
        CLog::log(msg, "app", "error");
        return {false, false, msg};
      }

      for (CDocHashT a_reference: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
      {
        if (!transient_block_info.m_transactions_dict.keys().contains(transient_block_info.m_map_trx_ref_to_trx_hash[a_reference]))
        {
          msg = "Creating new block, missed some3 transaction to support referenced documents. transactions dict: " + cutils::dumpIt(transient_block_info.m_transactions_dict) + " map trx ref to trx hash: " + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash);
          CLog::log(msg, "app", "error");
          return {false, false, msg};
        }
      }

      if (cutils::arrayDiff(transient_block_info.m_map_trx_hash_to_trx_ref.keys(), transient_block_info.m_transactions_dict.keys()).len() != 0)
      {
        msg = "Creating new block, missed some2 transaction to support referenced documents. transactions dict: " + cutils::dumpIt(transient_block_info.m_transactions_dict) + " map trx ref to trx hash: " + cutils::dumpIt(transient_block_info.m_map_trx_ref_to_trx_hash);
        CLog::log(msg, "app", "error");
        return {false, false, msg};
      }

      for (String groupCode: transient_block_info.m_groupped_documents.keys())
      {
        msg = "Creating new block, extracted " + String::number(transient_block_info.m_groupped_documents[groupCode].len()) + " documents of type(" + groupCode + ")";
        CLog::log(msg, "app", "info");
      }

      for (CDocHashT a_reference: transient_block_info.m_map_trx_ref_to_trx_hash.keys())
      {
        if (!transient_block_info.m_doc_by_hash.keys().contains(a_reference))
        {
          msg = "Creating new block, missed referenced document, which is supported by trx.ref(" + cutils::hash8c(a_reference) + ")";
          CLog::log(msg, "app", "error");
          return {false, false, msg};
        }
      }

      CLog::log("Transient block info: " + transient_block_info.dumpMe(), "app", "trace");

      return {true, true, "Successfully grouped"};
    }


    bool CMachine::removeFromBuffer(const ClausesT& clauses)
    {
      DbModel::dDelete(
        stb_machine_block_buffer,
        clauses);

      return true;
    }

    */
}
