/*


#ifndef TRANSACTIONSINRELATEDBLOCK_H
#define TRANSACTIONSINRELATEDBLOCK_H

class BlockOverview
{
public:
  bool m_status = false;
  String m_msg = "";

  StringList m_supported_P4P {};
  StringList m_block_used_coins {};
  QSDicT m_map_coin_to_spender_doc {};
  QV2DicT m_used_coins_dict {};
  StringList m_block_not_matured_coins {};
};

class TransactionsInRelatedBlock
{
public:
  TransactionsInRelatedBlock();

  static BlockOverview prepareBlockOverview(
    const Block *block);

  static std::tuple<bool, bool, String, SpendCoinsList*> validateTransactions(
    const Block* block,
    const String& stage);


  static std::tuple<bool, QV2DicT, QV2DicT, bool, SpendCoinsList*> considerInvalidCoins(
    const String& blockHash,
    const String& blockCreationDate,
    const StringList& block_used_coins,
    QV2DicT used_coins_dict,
    StringList maybe_invalid_coins,
    const QSDicT& map_coin_to_spender_doc);

  static std::tuple<bool, bool, String> appendTransactions(
    Block* block,
    TransientBlockInfo& transient_block_info);
};


TransactionsInRelatedBlock::TransactionsInRelatedBlock()
{

}

BlockOverview TransactionsInRelatedBlock::prepareBlockOverview(
  const Block *block)
{
  String msg = "";
  StringList supported_P4P = {};
  StringList trx_uniqueness = {};
  StringList inputs_doc_hashes = {};
  StringList block_used_coins = {};
  QSDicT map_coin_to_spender_doc = {};

  BlockOverview block_overview = {};
  for (CDocIndexT doc_inx = 0; doc_inx < block->getDocuments().size(); doc_inx++)
  {
    Document* a_doc = block->getDocuments()[doc_inx];

    if (a_doc->m_doc_creation_date > block->m_block_creation_date)
    {
      msg = "The trx(" + CUtils::hash8c(a_doc->getDocHash()) + ") is after block(" + CUtils::hash8c(block->getBlockHash()) + ") creation-date!";
      CLog::log(msg, "trx", "error");
      block_overview.m_msg = msg;
      return block_overview;
    }

    trx_uniqueness.append(a_doc->getDocHash());

    // extracting P4P (if exist)
    if ((a_doc->m_doc_type == constants::DOC_TYPES::BasicTx) && (a_doc->m_doc_class == constants::TRX_CLASSES::P4P))
    {
      if (!constants::SUPPORTS_P4P_TRANSACTION)
      {
        msg = "Network still doen't support P4P transactions. (" + CUtils::hash8c(a_doc->getDocHash()) + ") in block(" + CUtils::hash8c(block->getBlockHash()) + ")!";
        CLog::log(msg, "trx", "error");
        block_overview.m_msg = msg;
        return block_overview;
      }
      if (a_doc->getRef() != "")
        supported_P4P.append(a_doc->getRef());
    }

    if (a_doc->trxHasInput() && !a_doc->trxHasNotInput())
    {
      for (TInput* input: a_doc->getInputs())
      {
        inputs_doc_hashes.append(input->m_transaction_hash);
        String a_coin = input->getCoinCode();
        block_used_coins.append(a_coin);
        map_coin_to_spender_doc[a_coin] = a_doc->getDocHash();
      }
    }
  }

  // uniquness test
  if (trx_uniqueness.size() != CUtils::arrayUnique(trx_uniqueness).size())
  {
    msg = "Duplicating same trx in block body. block(" + CUtils::hash8c(block->getBlockHash()) + ")!";
    CLog::log(msg, "trx", "error");
    block_overview.m_msg = msg;
    return block_overview;
  }

  // control for using of rejected Transactions refLocs
  // in fact a refLoc can exist in table trx_utxo or not. if not, it doesn't matter whether exist in rejected trx or not.
  // and this controll of rejected trx is not necessary but it is a fastest way to discover a double-spend
  QVDRecordsT rejected_transactions = RejectedTransactionsHandler::searchInRejectedTrx({{"rt_doc_hash", inputs_doc_hashes, "IN"}});
  if (rejected_transactions.size() > 0)
  {
    msg = "Useing rejected transaction's outputs in block(" + CUtils::hash8c(block->getBlockHash()) + ")! rejected transactions:(" + CUtils::dumpIt(rejected_transactions) + ")!";
    CLog::log(msg, "trx", "error");
    block_overview.m_msg = msg;
    return block_overview;
  }

  // control double spending in a block
  // because malisciuos user can use one ref in multiple transaction in same block
  if (block_used_coins.size() != CUtils::arrayUnique(block_used_coins).size())
  {
    msg = "Double spending same refs in a block(" + CUtils::hash8c(block->getBlockHash()) + ")! ";
    CLog::log(msg, "trx", "error");
    block_overview.m_msg = msg;
    return block_overview;
  }
  CLog::log("Block(" + CUtils::hash8c(block->getBlockHash()) + ") has " + String::number(block_used_coins.size())+ " inputs ", "trx", "trace");

  // it is a dictionary for all inputs either valid or invalid
  // it has 3 keys/values (ut_coin, ut_o_address, ut_o_value)
  QV2DicT used_coins_dict = {};
  // all inputs must be maturated, maturated means it passed at least 12 hours of creeating the outputs and now they are presented in table trx_utxos adn are spendable
  StringList spendable_coins = {};
  if (block_used_coins.size() > 0)
  {
    // check if the coins exist in UTXOs?
    // implementing spendable coins chache to reduce DB load
    QVDRecordsT coins_info = UTXOHandler::searchInSpendableCoinsCache(block_used_coins);

//    remve top line and uncoment this lines after solving block database problem
//    QVDRecordsT coins_info = UTXOHandler::searchInSpendableCoins(
//      {{"ut_coin", block_used_coins, "IN"}},
//      {"ut_ref_creation_date"});

    if (coins_info.size() > 0)
    {
      for (QVDicT a_coin: coins_info)
      {
        CCoinCodeT ut_coin = a_coin.value("ut_coin").toString();
        spendable_coins.append(ut_coin);
        used_coins_dict[ut_coin] = a_coin;
        // the block creation Date MUST be at least 12 hours after the creation date of reflocs
        if (block->m_block_creation_date < CUtils::minutesAfter(CMachine::getCycleByMinutes(), a_coin.value("ut_ref_creation_date").toString()))
        {
          msg = "The creation of coin(" + CUtils::shortCoinRef(ut_coin) + ") is after usage in Block(" + CUtils::hash8c(block->getBlockHash()) + ")! ";
          CLog::log(msg, "trx", "error");
          block_overview.m_msg = msg;
          return block_overview;
        }
      }
    }
  }

  CLog::log("Block(" + CUtils::hash8c(block->getBlockHash()) + ") has " + String::number(spendable_coins.size()) + " maturated Inputs: " + spendable_coins.join(", "), "trx", "trace");

  // all inputs which are not in spendable coins, potentialy can be invalid
  StringList block_not_matured_coins = CUtils::arrayDiff(block_used_coins, spendable_coins);
  if (block_not_matured_coins.size() > 0)
    CLog::log("Missed matured coins in table trx_utxo at " + CUtils::getNowSSS() + " block(" + block->getBlockHash() + ")  missed(" + block_not_matured_coins.join(", ") + ") inputs! probably is cloned transaction", "sec", "error");

  block_overview.m_status = true;
  block_overview.m_supported_P4P = supported_P4P;
  block_overview.m_block_used_coins = block_used_coins;
  block_overview.m_map_coin_to_spender_doc = map_coin_to_spender_doc;
  block_overview.m_used_coins_dict = used_coins_dict;
  block_overview.m_block_not_matured_coins = block_not_matured_coins;
  return block_overview;
}

std::tuple<bool, QV2DicT, QV2DicT, bool, SpendCoinsList*> TransactionsInRelatedBlock::considerInvalidCoins(
  const String& blockHash,
  const String& blockCreationDate,
  const StringList& block_used_coins,
  QV2DicT used_coins_dict,
  StringList maybe_invalid_coins,
  const QSDicT& map_coin_to_spender_doc)
{

  QV2DicT invalid_coins_dict {};  // it contains invalid coins historical creation info

  // retrieve all spent coins in last 5 days
  SpendCoinsList* coinsInSpentTable = SpentCoinsHandler::makeSpentCoinsDict(block_used_coins);
  if (coinsInSpentTable->m_coins_dict.keys().size()> 0)
  {
    // the inputs which are already spended are invalid coins too
    maybe_invalid_coins = CUtils::arrayAdd(maybe_invalid_coins, coinsInSpentTable->m_coins_dict.keys());
    maybe_invalid_coins = CUtils::arrayUnique(maybe_invalid_coins);
  }

  if (maybe_invalid_coins.size() > 0)
  {
    CLog::log("maybe Invalid coins (either because of not matured or already spend): " + CUtils::dumpIt(maybe_invalid_coins), "trx", "error");
    invalid_coins_dict = DAG::getCoinsGenerationInfoViaSQL(maybe_invalid_coins);
    CLog::log("invalid Coins Dict: " + CUtils::dumpIt(invalid_coins_dict), "trx", "trace");

    // controll if all potentially invalid coins, have coin creation record in DAG history
    if (invalid_coins_dict.keys().size() != maybe_invalid_coins.size())
    {
      CLog::log("The block uses some un-existed inputs. may be machine is not synched. block(" + CUtils::hash8c(blockHash) + ")", "trx", "error");
      return {false, invalid_coins_dict, used_coins_dict, false, coinsInSpentTable};
    }

    /**
     * control if invalidity is because of using really unmatured outputs(which will be matured in next hours)?
     * if yes drop block
     */
    for (String aCoin: invalid_coins_dict.keys())
    {
      bool is_matured = CUtils::isMatured(
        invalid_coins_dict[aCoin]["coinGenDocType"].toString(),
        invalid_coins_dict[aCoin]["coinGenCreationDate"].toString());
      if (!is_matured)
      {
        CLog::log("The block uses at least one unmaturated input: block(" + CUtils::hash8c(blockHash) + ") coin(" + aCoin +")", "trx", "error");
        return {false, invalid_coins_dict, used_coins_dict, false, coinsInSpentTable};
      }
    }
  }


  for (CCoinCodeT an_invalid_coin_code: invalid_coins_dict.keys())
  {
    // append also invalid refs to used coins dict
    used_coins_dict[an_invalid_coin_code] = QVDicT{
      {"ut_coin", an_invalid_coin_code},
      {"ut_o_address", invalid_coins_dict[an_invalid_coin_code]["coinGenOutputAddress"]},
      {"ut_o_value", invalid_coins_dict[an_invalid_coin_code]["coinGenOutputValue"]},
      {"ut_ref_creation_date", invalid_coins_dict[an_invalid_coin_code]["coinGenCreationDate"]}};

    /**
     * adding to spend-input-dictionary the invalid coins in current block too
     * in order to having a complete history & order of entire spent coins of the block
     */
    if (!coinsInSpentTable->m_coins_dict.keys().contains(an_invalid_coin_code))
      coinsInSpentTable->m_coins_dict[an_invalid_coin_code] = std::vector<SpendCoinInfo*> {};

    coinsInSpentTable->m_coins_dict[an_invalid_coin_code].emplace_back(
      new SpendCoinInfo {
      blockCreationDate,
      blockHash,
      map_coin_to_spender_doc[an_invalid_coin_code]});

  }


  // all spent_loc must exist in invalid_coins_dict
  StringList tmp1 = invalid_coins_dict.keys();
  StringList tmp2 = coinsInSpentTable->m_coins_dict.keys();
  if ((tmp1.size() != tmp2.size()) ||
    (CUtils::arrayDiff(tmp1, tmp2).size() > 0) ||
    (CUtils::arrayDiff(tmp2, tmp1).size()> 0))
  {
    String msg = "finding invalidations messed up block(" + CUtils::hash8c(blockHash) + ") maybe Invalid Inputs: ";
    msg += "invalid Coins Dict: " + CUtils::dumpIt(invalid_coins_dict) + " coins In Spent Table: " + CUtils::dumpIt(coinsInSpentTable);
    CLog::log(msg, "sec", "error");
    return {false, invalid_coins_dict, used_coins_dict, false, coinsInSpentTable};
  }

  bool is_sus_block = false;
  if (invalid_coins_dict.keys().size() > 0)
  {
    String msg = "Some transaction inputs in block(" + CUtils::hash8c(blockHash) + ") are not valid";
    msg += "these are duplicated inputs: " + CUtils::dumpIt(invalid_coins_dict);
    CLog::log(msg, "trx", "error");
    is_sus_block = true;
  }

  // apllying machine-POV-order to coinsInSpentTable as an order-attr
  for (String aCoin: coinsInSpentTable->m_coins_dict.keys())
  {
    //looping on orders
    for (uint32_t inx = 0; inx < coinsInSpentTable->m_coins_dict[aCoin].size(); inx++)
      coinsInSpentTable->m_coins_dict[aCoin][inx]->m_spend_order = inx;
  }


  return {
    true,
    invalid_coins_dict,
    used_coins_dict,
    is_sus_block,
    coinsInSpentTable};

}

/**
 * @brief TransactionsInRelatedBlock::validateTransactions
 * @param block
 * @param stage
 * @return <status, is_sus_block, double_spends>
 */
std::tuple<bool, bool, String, SpendCoinsList*> TransactionsInRelatedBlock::validateTransactions(
  const Block *block,
  const String& stage)
{
  String msg;

  if (block->m_block_ext_info.size() == 0)
  {
    msg = "Missed ext Info for Block(CUtils::hash8c(" + block->getBlockHash() + ")!";
    CLog::log(msg, "trx", "error");
    return {false, false, msg, nullptr};
  }

  BlockOverview block_overview = prepareBlockOverview(block);
  if (!block_overview.m_status)
    return {false, false, block_overview.m_msg, nullptr};

  StringList maybe_invalid_coins = block_overview.m_block_not_matured_coins;

  CMPAIValueT sum_remotes = 0;
  CMPAIValueT treasury_incomes, backer_incomes = 0;

  // let remoteBlockDPCostBacker = 0;
  for (CDocIndexT doc_inx = 0; doc_inx < block->getDocuments().size(); doc_inx++)
  {
    Document* a_doc = block->getDocuments()[doc_inx];

    // do validate only transactions
    if (
      !Document::isBasicTransaction(a_doc->m_doc_type) &&
      !Document::isDPCostPayment(a_doc->m_doc_type)
    )
      continue;


    // DPCOst payment control
    if (a_doc->m_doc_type == constants::DOC_TYPES::DPCostPay)
    {
      auto [status, treasury_incomes_, backer_incomes_] = BlockUtils::retrieveDPCostInfo(
        a_doc,
        block->m_block_backer);
      if (!status)
        return {false, false, "Failed in calculation of retrieve-DPCost-Info", nullptr};
      treasury_incomes = treasury_incomes_;
      backer_incomes = backer_incomes_;
    }

    CMPAIValueT trx_stated_dp_cost = 0;
    if (block_overview.m_supported_P4P.contains(a_doc->getDocHash()))
    {
      CLog::log("The trx is supported by p4p trx. Block(" + CUtils::hash8c(block->getBlockHash()) + ") trx(" + CUtils::hash8c(a_doc->getDocHash()) + ") ", "trx", "info");
      // so we do not need to controll trx fee, because it is already payed

    }
    else if (StringList {constants::DOC_TYPES::DPCostPay}.contains(a_doc->m_doc_type))
    {
      // this kind of documents do not need to have trx-fee

    } else {
      if (!constants::SUPPORTS_CLONED_TRANSACTION && (a_doc->getDPIs().size() > 1))
      {
        msg = "The network still do not accept Cloned transactions!";
        CLog::log(msg, "trx", "error");
        return {false, false, msg, nullptr};
      }

      for (DPIIndexT a_dpi_index: a_doc->getDPIs())
      {
        if (a_doc->getOutputs()[a_dpi_index]->m_address == block->m_block_backer)
        {
          trx_stated_dp_cost = a_doc->getOutputs()[a_dpi_index]->m_amount;
        }
      }
      if (trx_stated_dp_cost == 0)
      {
        msg = "At least one trx hasn't backer fee! Block(" + CUtils::hash8c(block->getBlockHash()) + ") trx(" + CUtils::hash8c(a_doc->getDocHash()) + ")";
        CLog::log(msg, "trx", "error");
        return {false, false, msg, nullptr};
      }

      if (trx_stated_dp_cost < SocietyRules::getTransactionMinimumFee(block->m_block_creation_date))
      {
        msg = "The backer fee is less than Minimum acceptable fee!! Block(" + CUtils::hash8c(block->getBlockHash()) + ") trx(" + CUtils::hash8c(a_doc->getDocHash()) + ") trx_stated_dp_cost(" + String::number(trx_stated_dp_cost)+ ") < minimum fee(" + SocietyRules::getTransactionMinimumFee(block->m_block_creation_date) + ")";
        CLog::log(msg, "trx", "error");
        return {false, false, msg, nullptr};
      }

      auto[status, locally_recalculate_trx_dp_cost] = a_doc->calcDocDataAndProcessCost(
        stage,
        block->m_block_creation_date);
      if (!status)
        return {false, false, "Failed in calc-Doc-Data-And-Process-Cost", nullptr};

      CLog::log(
        "compare costs(remote: " + CUtils::sepNum(trx_stated_dp_cost) + " local: " + CUtils::sepNum(locally_recalculate_trx_dp_cost) + " ) doc(" +
        a_doc->m_doc_type + " / " + CUtils::hash8c(a_doc->getDocHash()) +")  Block(" + CUtils::hash8c(block->getBlockHash()) + ")", "trx", "trace");

      if (trx_stated_dp_cost < locally_recalculate_trx_dp_cost)
      {
        msg = "Miss-calculated documet length: " + a_doc->safeStringifyDoc(true);
        msg += "The backer fee is less than network values! Block(" +
          CUtils::hash8c(block->getBlockHash()) + ") trx(" + CUtils::hash8c(a_doc->getDocHash()) +
          ") trx_stated_dp_cost(" + CUtils::sepNum(trx_stated_dp_cost)+ ") < network minimum fee(" +
              CUtils::sepNum(locally_recalculate_trx_dp_cost) + ") mcPAIs",
        CLog::log(msg, "trx", "error");
        return {false, false, msg, nullptr};
      }
    }

    sum_remotes += trx_stated_dp_cost;
  }
  CLog::log("Backer Fees Sum = " + CUtils::sepNum(sum_remotes) + " PAIs for Block(" + CUtils::hash8c(block->getBlockHash()) + ") ", "app", "info");

  // control if block total trx fees are valid
  auto block_fix_cost = SocietyRules::getBlockFixCost(block->m_block_creation_date);
  auto befor_block_tax = (sum_remotes * constants::BACKER_PERCENT_OF_BLOCK_FEE) / 100;
  CMPAIValueT recalc_remote_backer_fee = befor_block_tax - block_fix_cost;
  if (recalc_remote_backer_fee != backer_incomes)
  {
    msg = "The locally calculated backer fee is not what remote is! local(" + CUtils::sepNum(recalc_remote_backer_fee)+
      ") remote(" + CUtils::sepNum(backer_incomes) + ") mcPAIs, Block(" +
      CUtils::hash8c(block->getBlockHash()) + ")";
    CLog::log(msg, "trx", "error");

    return {false, false, msg, nullptr};
  }

  CMPAIValueT locally_recalculate_block_treasury_income = sum_remotes - recalc_remote_backer_fee;
  if (locally_recalculate_block_treasury_income != treasury_incomes)
  {
      msg = "The locally calculated treasury is not what remote is! Block(" +
      CUtils::hash8c(block->getBlockHash()) + ") locally_recalculate_block_treasury_income(" + CUtils::sepNum(locally_recalculate_block_treasury_income)+ ") treasury_incomes(" +
      CUtils::sepNum(treasury_incomes) + ") mcPAIs";
      CLog::log(msg, "trx", "error");
    return {false, false, msg, nullptr};
  }

  GRecordsT SCUDS {};
  if (constants::SUPER_CONTROL_UTXO_DOUBLE_SPENDING)
  {
    /**
    * after being sure about secure and proper functionality of code, we can cut this controll in next monthes
    * finding the block(s) which are used these coins and already are registerg in DAG
    */
    auto[status, SCUDS] = SpentCoinsHandler::findCoinsSpendLocations(block_overview.m_block_used_coins);
    if (!status)
      return {false, false, "Failed in find-Coins-Spend-Locations", nullptr};

    if (SCUDS.keys().size() > 0)
    {
      msg = "SCUDS: SUPER_CONTROL_UTXO_DOUBLE_SPENDING found some double-spending with block(" + CUtils::hash8c(block->getBlockHash()) + ") SCUDS.spendsDict: " + CUtils::dumpIt(SCUDS);
      CLog::log(msg, "sec", "error");
      return {false, false, msg, nullptr};
    }
  }

  if (constants::SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING)
  {
    /**
    * most paranoidic and pesimistic control of input validation
    * for now I put this double-controll to also quality controll of the previuos-controls.
    * this control is too costly, so it must be removed or optimized ASAP
    */
    auto[validate_status, validate_msg, coins_track] = SuperControlTilCoinbaseMinting::trackingBackTheCoins(
      block,
      {},
      {});
    if (!validate_status)
    {
      msg = "SuperValidate, block(" + CUtils::hash8c(block->getBlockHash()) + ") error message: " + validate_msg;
      CLog::log(msg, "trx", "error");
      return {false, false, msg, nullptr};
    } else {
      CLog::log("SuperValidate, block(${CUtils::hash8c(block->getBlockHash())})'s inputs have confirmed path going back to coinbase", "trx", "info");
    }
  }

  auto[status2, invalid_coins_dict, used_coins_dict_, is_sus_block, double_spends] = considerInvalidCoins(
    block->getBlockHash(),
    block->m_block_creation_date,
    block_overview.m_block_used_coins,
    block_overview.m_used_coins_dict,
    maybe_invalid_coins,
    block_overview.m_map_coin_to_spender_doc);
  if (!status2)
    return {false, false, "Failed in consider-Invalid-Coins", double_spends};
  block_overview.m_used_coins_dict = used_coins_dict_;


  bool equation_control_res = EquationsControls::validateEquation(
    block,
    block_overview.m_used_coins_dict,
    invalid_coins_dict);
  if (!equation_control_res)
    return {false, false, "Failed in validate-Equation", double_spends};


  /**
  * control UTXO visibility in DAG history by going back throught ancestors
  * since the block can contains only UTXOs which are already took palce in her hsitory
  * in oder words, they are in block's sibility
  */
  if (!is_sus_block && !constants::SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING)
  {
    bool is_visible = CoinsVisibilityHandler::controlCoinsVisibilityInGraphHistory(
      block_overview.m_block_used_coins,
      block->m_ancestors,
      block->getBlockHash());
    if (!is_visible)
      return {false, false, "Failed in control-Coins-Visibility-In-Graph-History", double_spends};
  }


  return {true, is_sus_block, "valid", double_spends};
}

/**
 * @brief TransactionsInRelatedBlock::appendTransactions
 * @param block
 * @param transient_block_info
 * @return {creating block status, should empty buffer, msg}
 */
std::tuple<bool, bool, String> TransactionsInRelatedBlock::appendTransactions(
  Block* block,
  TransientBlockInfo& transient_block_info)
{
  return CMachine::fetchBufferedTransactions(block, transient_block_info);
}

*/