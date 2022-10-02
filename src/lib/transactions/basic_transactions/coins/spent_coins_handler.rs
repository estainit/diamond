use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CInputIndexT, ClausesT, COutputIndexT, GRecordsT, LimitT, OrderT, QVDicT, QVDRecordsT, VString};
use crate::lib::dag::dag::get_wrap_blocks_by_doc_hash;
use crate::lib::database::abs_psql::{ModelClause, OrderModifier, q_select};
use crate::lib::database::tables::{C_TRX_SPEND, C_TRX_SPEND_FIELDS};

#[derive(Clone, Debug)]
pub struct SpendCoinInfo
{
    pub m_spend_date: String,
    pub m_spend_block: String,
    pub m_spend_document: String,
    pub m_spend_order: COutputIndexT,
}

impl SpendCoinInfo
{
    #[allow(unused, dead_code)]
    pub fn new() -> Self
    {
        Self {
            m_spend_date: "".to_string(),
            m_spend_block: "".to_string(),
            m_spend_document: "".to_string(),
            m_spend_order: 0,
        }
    }
}

#[derive(Debug)]
pub struct SpendCoinsList
{
    pub m_coins_dict: HashMap<String, Vec<SpendCoinInfo>>,
}

impl SpendCoinsList
{
    pub fn new() -> Self
    {
        Self {
            m_coins_dict: HashMap::new()
        }
    }
}


pub struct SpentCoinsHandler
{}

impl SpentCoinsHandler
{
    // old name was findCoinsSpendLocations
    pub fn find_coins_spend_locations(coins: &VString) -> (bool, GRecordsT)
    {
        // finding the block(s) which are used these coins and already are registerg in DAG(if they did)
        // this function just writes some logs and have not effect on block validation accept/denay
        let mut recorded_spend_in_dag: GRecordsT = HashMap::new();
        for a_coin in coins
        {
            dlog(
                &format!("SCUDS: looking for a Coin({})", cutils::short_coin_code(a_coin)),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);

            let (doc_hash, _inx) = cutils::unpack_coin_code(a_coin);

            // find broadly already recorded block(s) which used(or referenced) this input-doc_hash
            let (w_blocks, _map) = get_wrap_blocks_by_doc_hash(
                &vec![doc_hash.clone()],
                &constants::COMPLETE.to_string());

            dlog(
                &format!(
                    "SCUDS: looking for doc ({}) returned ({})  potentially blocks",
                    cutils::hash8c(&doc_hash),
                    w_blocks.len()),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);
            if w_blocks.len() > 0
            {
                for w_block in w_blocks
                {
                    let (_status, _sf_ver, content) = unwrap_safed_content_for_db(&w_block["b_body"]);
                    let (_status, ref_block) = cutils::controlled_str_to_json(&content);

                    let block_hash: CBlockHashT = remove_quotes(&ref_block["bHash"]);
                    if block_hash == ""
                    { continue; }

                    dlog(
                        &format!("SCUDS: controlling block({})", cutils::hash8c(&block_hash)),
                        constants::Modules::Trx,
                        constants::SecLevel::TmpDebug);

                    if !ref_block["bDocs"].is_null()
                        && (ref_block["bDocs"].as_array().unwrap().len() > 0)
                    {
                        dlog(
                            &format!("SCUDS: block({}) has {} docs",
                                     cutils::hash8c(&block_hash),
                                     ref_block["bDocs"].as_array().unwrap().len()
                            ),
                            constants::Modules::Trx,
                            constants::SecLevel::Info);

                        for doc in ref_block["bDocs"].as_array().unwrap()
                        {
                            if !doc["inputs"].is_null()
                            {
                                let the_inputs = doc["inputs"].as_array().unwrap();
                                dlog(
                                    &format!("SCUDS: doc has {} inputs", the_inputs.len()),
                                    constants::Modules::Trx,
                                    constants::SecLevel::Info);
                                for input_index in 0..the_inputs.len() as CInputIndexT
                                {
                                    let trx_input = the_inputs[input_index as usize].as_array().unwrap();
                                    // if the doc_hash is referenced as an input index, select id
                                    if trx_input[0].to_string() == doc_hash
                                    {
                                        let tmp_coin: CCoinCodeT = cutils::pack_coin_code(
                                            &trx_input[0].to_string(),
                                            trx_input[1].to_string().parse::<COutputIndexT>().unwrap());
                                        dlog(
                                            &format!("SCUDS: controlling input({})", cutils::short_coin_code(&tmp_coin)),
                                            constants::Modules::Trx,
                                            constants::SecLevel::Info);
                                        if coins.contains(&tmp_coin)
                                        {
                                            if !recorded_spend_in_dag.contains_key(&tmp_coin)
                                            { recorded_spend_in_dag.insert(tmp_coin.clone(), vec![]); }
                                            let mut tmp_v = recorded_spend_in_dag[&tmp_coin].clone();
                                            let tmp_qv: QVDicT = HashMap::from([
                                                ("block_hash".to_string(), ref_block["bHash"].to_string()),
                                                ("doc_hash".to_string(), doc_hash.clone()),
                                                ("input_index".to_string(), input_index.to_string())]);
                                            tmp_v.push(tmp_qv);

                                            recorded_spend_in_dag.insert(tmp_coin, tmp_v);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        return (true, recorded_spend_in_dag);
    }

    // old name was searchInSpentCoins
    pub fn search_in_spent_coins(
        clauses: ClausesT,
        fields: Vec<&str>,
        order: OrderT,
        limit: LimitT) -> QVDRecordsT
    {
        let (_status, records) = q_select(
            C_TRX_SPEND,
            fields,
            clauses,
            order,
            limit,
            false);
        return records;
    }

    //* accepts given coins and prepares an ordered history of coins spent
    // old name was makeSpentCoinsDict
    pub fn make_spent_coins_dict(coins: &VString) -> SpendCoinsList
    {
        // TODO: maybe optimize it via bloom filters for daily(cyclic) spent coins
        let mut spend_coins: SpendCoinsList = SpendCoinsList::new();
        let empty_string = "".to_string();
        let mut c1 = ModelClause {
            m_field_name: "sp_coin",
            m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
            m_clause_operand: "IN",
            m_field_multi_values: vec![],
        };
        for a_coin in coins
        {
            c1.m_field_multi_values.push(a_coin as &(dyn ToSql + Sync));
        }
        let in_dag_recorded_coins: QVDRecordsT = SpentCoinsHandler::search_in_spent_coins(
            vec![c1],
            Vec::from(C_TRX_SPEND_FIELDS),
            vec![&OrderModifier { m_field: "sp_spend_date", m_order: "ASC" }],
            0);
        if in_dag_recorded_coins.len() > 0
        {
            for sp in in_dag_recorded_coins
            {
                let the_coin = sp["sp_coin"].to_string();
                if !spend_coins.m_coins_dict.contains_key(&the_coin)
                {
                    spend_coins.m_coins_dict.insert(the_coin.clone(), vec![]);
                }

                let (block_hash_, doc_hash_) = cutils::unpack_coin_spend_loc(&sp["sp_spend_loc"].to_string());

                let tmp: SpendCoinInfo = SpendCoinInfo {
                    m_spend_date: sp["sp_spend_date"].to_string(),
                    m_spend_block: block_hash_,
                    m_spend_document: doc_hash_,
                    m_spend_order: 0,
                };

                let mut tmp2 = spend_coins.m_coins_dict[&the_coin].clone();
                tmp2.push(tmp);
                spend_coins.m_coins_dict.insert(the_coin, tmp2);

                // * making a dictionary of history of spending each unique coin,
                // * and ordering by spent time in machine's point of view.
                // * later it will be used to vote about transactions priority.
                // * it is totally possible in this step machine can not retrieve very old spending
                // * (because the spent table periodicly truncated), in this case machine will vote a doublespended doc as a first document
                // * later in "doGroupByCoinAndVoter" method we add second index(vote date) to securing the spend order
                // if (!_.has(spendsOrder, the_coin))
                //     spendsOrder[the_coin] = [];
                // spendsOrder[the_coin].push({
                //     date: sp.spSpendDate,
                //     blockHash: spendInfo.blockHash,
                //     docHash: spendInfo.docHash,
                // });
            }
            //    CLog::log("already Spent And Recorded Inputs Dict: " + cutils::dumpIt(spend_coins), "trx", "error");
        }
        return spend_coins;
    }

    /*
        bool SpentCoinsHandler::markSpentAnInput(
          const String& the_coin,
          const String& spend_loc,
          const String& spend_date,
          const String& cDate)
        {
          /**
           *
           * remove old records
           * TODO: infact we should not remove history(at least for some resonably percent of nodes)
           * they have to keep ALL data, specially for long-term loans, to un-pledge the account, nodes needed this information.
           * wherase repayments took place in RpBlocks.RpDocs and most of time are sufficient to close a pledge,
           * but sometimes pledger can pay all the loan in one transaction to get ride of loan
           * or pays a big part of loan to reduce interest rate.
           * in all of these case, the nodes which are engaged in loan(e.g. the arbiters nodes) must have these information.
           * although all trx info are reachable via blocks, but this table provides a faster & easier crawling
           * so in pledge time, the pledgee (can or must) add a bunch of long-term-data-backers(LTDB or LoTeDB)
           * as evidence of repayments.
           * in such a way, to unpledge an account either pledgee signature or these LoTeDB signatures will be sufficient,
           * so, there is not obligue to mantain all data by all nodes
           * obviously these LoteDbs will be payed by pledge contract based on repayments longth
           * TODO: must be implemented
           */
          DbModel::dDelete(
            stbl_trx_spend,
            {{"sp_spend_date", cutils::minutesBefore(CMachine::getCycleByMinutes() * CConsts::KEEP_SPENT_COINS_BY_CYCLE, cDate),"<"}},
            true,
            false);

          DbModel::insert(
            stbl_trx_spend,
            {{"sp_coin", the_coin},
            {"sp_spend_loc", spend_loc},
            {"sp_spend_date", spend_date}},
            true,
            false);

          return true;
        }

        bool SpentCoinsHandler::markAsSpentAllBlockInputs(
          const Block* block,
          const String& cDate)
        {
          for (Document* doc: block->getDocuments())
          {
            // TODO FIXME: discover cloned transactions and mark them too
            for (TInput* input: doc->getInputs())
            {
              markSpentAnInput(
                input->getCoinCode(),  // the spent coin
                cutils::packCoinSpendLoc(block->getBlockHash(), doc->getDocHash()),  // spending location
                block->m_block_creation_date,
                cDate);
            }
          }

          return true;
        }

        JSonObject SpentCoinsHandler::convertSpendsToJsonObject(const SpendCoinsList* sp)
        {
          JSonObject Jout {};
          for (String a_group_key: sp->m_coins_dict.keys())
          {
            QJsonArray Jgroup {};
            for (SpendCoinInfo* a_row: sp->m_coins_dict[a_group_key])
            {
              JSonObject Jrow{
                {"spendDate", a_row->m_spend_date},
                {"spendBlockHash", a_row->m_spend_block},
                {"spendDocHash", a_row->m_spend_document},
                {"spendOrder", QVariant::fromValue(a_row->m_spend_order).toDouble()},
              };
              Jgroup.push(Jrow);
            }
            Jout.insert(a_group_key, Jgroup);
          }
          return Jout;
        }

        */
}