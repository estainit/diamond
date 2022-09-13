/*

bool EquationsControls::validateEquation(
  const Block* block,
  const QV2DicT& used_coins_dict,
  const QV2DicT& invalid_coins_dict)
{
  QString msg;
  bool validate_res;

  // transaction details check
  QHash<CDocHashT, CMPAIValueT> inputs_amounts_dict {};
  QHash<CDocHashT, CMPAIValueT> outputs_amounts_dict {};
  for (CDocIndexT doc_inx = 0; doc_inx < block->m_documents.size(); doc_inx++)
  {
    if (Document::isBasicTransaction(block->m_documents[doc_inx]->m_doc_type))
    {
      BasicTxDocument* doc = dynamic_cast<BasicTxDocument*>(block->m_documents[doc_inx]);
      if (doc->m_doc_ext_info.size() == 0)
        doc->maybeAssignDocExtInfo(block, doc_inx);

      // signatures control
      // validate_tx_signatures
      validate_res = doc->validateSignatures(
        used_coins_dict,
        {},
        block->getBlockHash());
      if (!validate_res)
        return false;

      // control transaction hash
      validate_res = doc->validateGeneralRulsForTransaction();
      if (!validate_res)
        return false;

      // equation control
      auto[equation_check_res, equation_msg, total_inputs_amounts, total_outputs_amounts] = doc->equation_check(
        used_coins_dict,
        invalid_coins_dict,
        block->getBlockHash());
      if(!equation_check_res)
      {
        CLog::log(equation_msg, "sec", "error");
        return false;
      }

      inputs_amounts_dict[doc->getDocHash()] = total_inputs_amounts;
      outputs_amounts_dict[doc->getDocHash()] = total_outputs_amounts;

      if (doc->m_inputs.size() > 0)
      {
        for (TInput* input: doc->m_inputs)
        {
          CCoinCodeT a_coin_code = input->getCoinCode();
          if (used_coins_dict.keys().contains(a_coin_code))
          {
            if (used_coins_dict[a_coin_code].value("ut_o_value").toDouble() >= MAX_COIN_VALUE)
            {
              msg = "The transaction has input bigger than MAX_SAFE_INTEGER! trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")  value: " + CUtils::microPAIToPAI6(used_coins_dict[a_coin_code].value("ut_o_value").toDouble());
              CLog::log(msg, "sec", "error");
              return false;
            }
            inputs_amounts_dict[doc->getDocHash()] += used_coins_dict[a_coin_code].value("ut_o_value").toDouble();

          } else {
            /**
            * trx uses already spent outputs! so try invalid_coins_dict
            * probably it is a double-spend case, which will be decided after 12 hours, in importing step
            * BTW ALL trx must have balanced equation (even duoble-spendeds)
            */
            if (invalid_coins_dict.keys().contains(a_coin_code))
            {
              if (invalid_coins_dict[a_coin_code].value("coinGenOutputValue").toDouble() >= MAX_COIN_VALUE)
              {
                msg = "The transaction has inv-input bigger than MAX_SAFE_INTEGER! trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")  value: " + CUtils::microPAIToPAI6(invalid_coins_dict[a_coin_code].value("coinGenOutputValue").toDouble());
                CLog::log(msg, "sec", "error");
                return false;
              }
              inputs_amounts_dict[doc->getDocHash()] += invalid_coins_dict[a_coin_code].value("coinGenOutputValue").toDouble();

            } else {
              msg = "The input absolutely missed! not in tables neither in DAG! coin(" + a_coin_code + ") trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")";
              CLog::log(msg, "sec", "error");
              return false;
            }
          }
        }
      }

      if (doc->m_outputs.size() > 0)
      {
        for (TOutput* output: doc->m_outputs)
        {
          if (output->m_address != CUtils::stripOutputAddress(output->m_address))
          {
            msg = "The transaction has not digit charecters! trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")";
            CLog::log(msg, "sec", "error");
            return false;
          }

          if (output->m_amount == 0)
          {
            msg = "The transaction has zero output! trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")";
            CLog::log(msg, "sec", "error");
            return false;
          }

          if (output->m_amount < 0)
          {
            msg = "The transaction has negative output! trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")";
            CLog::log(msg, "sec", "error");
            return false;
          }

          if (output->m_amount >= MAX_COIN_VALUE)
          {
            msg = "The transaction has output bigger than MAX_SAFE_INTEGER! trx(" + doc->m_doc_type + " / " + CUtils::hash8c(doc->m_doc_hash) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ")";
            CLog::log(msg, "sec", "error");
            return false;
          }

          outputs_amounts_dict[doc->getDocHash()] += output->m_amount;
        }
      }

      msg = "The inputs_amounts_dict must be equal outputs. input(" + CUtils::microPAIToPAI6(inputs_amounts_dict[doc->getDocHash()]) + ") == (" + CUtils::microPAIToPAI6(outputs_amounts_dict[doc->getDocHash()]) + " output)";
      CLog::log(msg, "trx", "trace");
      if (inputs_amounts_dict[doc->getDocHash()] != outputs_amounts_dict[doc->getDocHash()])
      {
        msg = "The transaction is not balanced! doc(" + CUtils::hash8c(doc->getDocHash()) + ") Block(" + CUtils::hash8c(block->getBlockHash()) + ") input(" + CUtils::microPAIToPAI6(inputs_amounts_dict[doc->getDocHash()]) + ") != ( output " + CUtils::microPAIToPAI6(outputs_amounts_dict[doc->getDocHash()]) + ")";
        CLog::log(msg, "sec", "error");
        return false;
      }
    }
  }

  // calculate block total inputs & outputs
  CMPAIValueT total_amount_block_inputs = 0;
  CMPAIValueT total_amount_block_outputs = 0;
  for(CDocHashT a_doc_hash: inputs_amounts_dict.keys())
  {
    total_amount_block_inputs += inputs_amounts_dict[a_doc_hash];
    total_amount_block_outputs += outputs_amounts_dict[a_doc_hash];
  }
  if (total_amount_block_inputs != total_amount_block_outputs)
  {
    msg = "unbalanced total in/out of Block(" + CUtils::hash8c(block->getBlockHash()) + ") ";
    CLog::log(msg, "sec", "error");
    return false;
  }

  msg = "valid in/out equation Block(" + CUtils::hash8c(block->getBlockHash()) + ") value(" + CUtils::microPAIToPAI6(total_amount_block_inputs) + ")";
  CLog::log(msg, "trx", "trace");

  return true;
}

*/