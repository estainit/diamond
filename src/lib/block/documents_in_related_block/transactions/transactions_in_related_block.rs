use std::collections::HashMap;
use postgres::types::ToSql;
use crate::lib::block::block_types::block::{Block, TransientBlockInfo};
use crate::{application, constants, cutils, dlog};
use crate::lib::block::document_types::document::Document;
use crate::lib::block::documents_in_related_block::transactions::coins_visibility_handler::control_coins_visibility_in_graph_history;
use crate::lib::block::documents_in_related_block::transactions::equations_controls::validate_equation;
use crate::lib::block_utils::retrieve_dp_cost_info;
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CDateT, CDocIndexT, CMPAISValueT, CMPAIValueT, COutputIndexT, GRecordsT, QSDicT, QV2DicT, QVDRecordsT, VString};
use crate::lib::dag::dag::get_coins_generation_info_via_sql;
use crate::lib::dag::normal_block::rejected_transactions_handler::search_in_rejected_trx;
use crate::lib::dag::super_control_til_coinbase_minting::tracking_back_the_coins;
use crate::lib::database::abs_psql::ModelClause;
use crate::lib::machine::machine_buffer::fetch_buffered_transactions::fetch_buffered_transactions;
use crate::lib::services::society_rules::society_rules::{get_block_fix_cost, get_transaction_minimum_fee};
use crate::lib::transactions::basic_transactions::coins::coins_handler::{Coin, search_in_spendable_coins_cache};
use crate::lib::transactions::basic_transactions::coins::spent_coins_handler::{SpendCoinInfo, SpendCoinsList, SpentCoinsHandler};

pub struct BlockOverview
{
    pub m_status: bool,
    pub m_msg: String,

    pub m_supported_p4p: VString,
    pub m_block_used_coins: VString,
    pub m_map_coin_to_spender_doc: QSDicT,
    pub m_used_coins_dict: HashMap<CCoinCodeT, Coin>,
    pub m_block_not_matured_coins: VString,
}

impl BlockOverview
{
    pub fn new() -> Self
    {
        Self {
            m_status: false,
            m_msg: "".to_string(),
            m_supported_p4p: vec![],
            m_block_used_coins: vec![],
            m_map_coin_to_spender_doc: Default::default(),
            m_used_coins_dict: Default::default(),
            m_block_not_matured_coins: vec![],
        }
    }
}

//old_name_was prepareBlockOverview
pub fn prepare_block_overview(
    block: &Block) -> BlockOverview
{
    let msg: String;
    let mut supported_p4p: VString = vec![];
    let mut trx_uniqueness: VString = vec![];
    let mut inputs_doc_hashes: VString = vec![];
    let mut block_used_coins: VString = vec![];
    let mut map_coin_to_spender_doc: QSDicT = HashMap::new();

    let mut block_overview: BlockOverview = BlockOverview::new();
    for doc_inx in 0..block.get_docs_count() as CDocIndexT
    {
        let a_doc: &Document = &block.get_documents()[doc_inx as usize];

        if a_doc.m_doc_creation_date > block.m_block_creation_date
        {
            msg = format!(
                "The trx creation-date is after block! {} {}!",
                a_doc.get_doc_identifier(),
                block.get_block_identifier()
            );
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Error);


            block_overview.m_msg = msg;
            return block_overview;
        }

        trx_uniqueness.push(a_doc.get_doc_hash());

        // extracting P4P (if exist)
        if (a_doc.get_doc_type() == constants::document_types::BASIC_TX)
            && (a_doc.get_doc_class() == constants::trx_classes::P4P)
        {
            if constants::SUPPORTS_P4P_TRANSACTION
            {
                msg = format!(
                    "Network still don't support P4P transactions. {} in block {} !",
                    a_doc.get_doc_identifier(),
                    block.get_block_identifier()
                );
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                block_overview.m_msg = msg;
                return block_overview;
            }
            if a_doc.get_doc_ref() != ""
            { supported_p4p.push(a_doc.get_doc_ref()); }
        }

        if a_doc.trx_has_input() && !a_doc.trx_has_not_input()
        {
            for input in a_doc.get_inputs()
            {
                inputs_doc_hashes.push(input.m_transaction_hash.clone());
                let a_coin: String = input.get_coin_code();
                block_used_coins.push(a_coin.clone());
                map_coin_to_spender_doc.insert(a_coin, a_doc.get_doc_hash());
            }
        }
    }

    // uniqueness test
    if trx_uniqueness.len() != cutils::array_unique(&trx_uniqueness).len()
    {
        msg = format!(
            "Duplicating same trx in block body. block {}!",
            block.get_block_identifier()
        );
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        block_overview.m_msg = msg;
        return block_overview;
    }

    // control for using of rejected Transactions coins
    // in fact a refLoc can exist in table trx_utxo or not. if not, it doesn't matter whether exist in rejected trx or not.
    // and this control of rejected trx is not necessary but it is a fastest way to discover a double-spend
    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "rt_doc_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &inputs_doc_hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let rejected_transactions: QVDRecordsT = search_in_rejected_trx(
        vec![c1],
        vec![],
        vec![],
        0);
    if rejected_transactions.len() > 0
    {
        msg = format!(
            "Using rejected transaction's outputs in block {}! rejected transactions: {:?}",
            block.get_block_identifier(),
            rejected_transactions
        );
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        block_overview.m_msg = msg;
        return block_overview;
    }

    // control double spending in a block
    // because malisciuos user can use one ref in multiple transaction in same block
    if block_used_coins.len() != cutils::array_unique(&block_used_coins).len()
    {
        msg = format!(
            "Double spending same refs in a block {}",
            block.get_block_identifier()
        );
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        block_overview.m_msg = msg;
        return block_overview;
    }
    dlog(
        &format!(
            "Block has {} inputs {}",
            block_used_coins.len(), block.get_block_identifier()
        ),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    // it is a dictionary for all inputs either valid or invalid
    // it has 3 keys/values (ut_coin, ut_o_address, ut_o_value)
    let mut used_coins_dict: HashMap<CCoinCodeT, Coin> = HashMap::new();

    // all inputs must be maturated, maturated means it passed at least 12 hours of creeating the outputs and now they are presented in table trx_utxos adn are spendable
    let mut spendable_coins: VString = vec![];
    if block_used_coins.len() > 0
    {
        // check if the coins exist in cache?
        // implementing spendable coins cache to reduce DB load
        let coins_info: QVDRecordsT = search_in_spendable_coins_cache(&block_used_coins);

//    remove top line and uncomment this lines after solving block database problem
//    QVDRecordsT coins_info = UTXOHandler::searchInSpendableCoins(
//      {{"ut_coin", block_used_coins, "IN"}},
//      {"ut_ref_creation_date"});

        if coins_info.len() > 0
        {
            for a_coin in coins_info
            {
                let ut_coin: CCoinCodeT = a_coin["ut_coin"].clone();
                spendable_coins.push(ut_coin.clone());
                used_coins_dict.insert(
                    ut_coin.clone(),
                    Coin {
                        m_creation_date: a_coin["ut_creation_date"].clone(),
                        m_ref_creation_date: a_coin["ut_ref_creation_date"].clone(),
                        m_coin_code: a_coin["ut_coin"].clone(),
                        m_coin_owner: a_coin["ut_o_address"].clone(),
                        ut_visible_by: a_coin["ut_visible_by"].clone(),
                        m_coin_value: a_coin["ut_o_value"].parse::<CMPAIValueT>().unwrap(),
                    });
                // the block creation Date MUST be at least 12 hours after the creation date of reflocs
                let cycle_by_min = application().get_cycle_by_minutes();
                if block.m_block_creation_date < application().minutes_after(cycle_by_min, &a_coin["ut_ref_creation_date"])
                {
                    msg = format!(
                        "The creation of coin( {}) is after usage in the Block {}!",
                        cutils::short_coin_code(&ut_coin),
                        block.get_block_identifier()
                    );
                    dlog(
                        &msg,
                        constants::Modules::Trx,
                        constants::SecLevel::Error);
                    block_overview.m_msg = msg;
                    return block_overview;
                }
            }
        }
    }

    dlog(
        &format!("Block {} has {} maturated Inputs: {:?}",
                 block.get_block_identifier(),
                 spendable_coins.len(),
                 spendable_coins
        ),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    // all inputs which are not in spendable coins, potentialy can be invalid
    let block_not_matured_coins: VString = cutils::array_diff(&block_used_coins, &spendable_coins);
    if block_not_matured_coins.len() > 0
    {
        dlog(
            &format!(
                "Missed matured coins in table trx_coins at {} block {} missed({:#?}) inputs! probably is cloned transaction",
                application().get_now_sss(),
                block.get_block_identifier(),
                block_not_matured_coins,
            ),
            constants::Modules::Sec,
            constants::SecLevel::Error);
    }

    block_overview.m_status = true;
    block_overview.m_supported_p4p = supported_p4p;
    block_overview.m_block_used_coins = block_used_coins.clone();
    block_overview.m_map_coin_to_spender_doc = map_coin_to_spender_doc;
    block_overview.m_used_coins_dict = used_coins_dict;
    block_overview.m_block_not_matured_coins = block_not_matured_coins;
    return block_overview;
}

//old_name_was considerInvalidCoins
pub fn consider_invalid_coins(
    block_hash: &CBlockHashT,
    block_creation_date: &CDateT,
    block_used_coins: &VString,
    used_coins_dict: &HashMap<CCoinCodeT, Coin>,
    maybe_invalid_coins: &VString,
    map_coin_to_spender_doc: &QSDicT) -> (bool, QV2DicT, HashMap<CCoinCodeT, Coin>, bool, SpendCoinsList)
{
    let msg: String;
    let mut maybe_invalid_coins = maybe_invalid_coins.clone();
    let mut used_coins_dict = used_coins_dict.clone();
    let mut invalid_coins_dict: QV2DicT = HashMap::new();  // it contains invalid coins historical creation info

    // retrieve all spent coins in last 5 days
    let mut coins_in_spent_table: SpendCoinsList = SpentCoinsHandler::make_spent_coins_dict(block_used_coins);
    if coins_in_spent_table.m_coins_dict.keys().len() > 0
    {
        // the inputs which are already spent are invalid coins too
        maybe_invalid_coins = cutils::array_add(
            &maybe_invalid_coins,
            &coins_in_spent_table
                .m_coins_dict
                .keys()
                .cloned()
                .collect::<VString>());
        maybe_invalid_coins = cutils::array_unique(&maybe_invalid_coins);
    }

    if maybe_invalid_coins.len() > 0
    {
        dlog(
            &format!("Maybe invalid coins (either because of not matured or already spend): {:#?}", maybe_invalid_coins),
            constants::Modules::Trx,
            constants::SecLevel::Error);
        invalid_coins_dict = get_coins_generation_info_via_sql(&maybe_invalid_coins);
        dlog(
            &format!("invalid Coins Dict: {:?}", invalid_coins_dict),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        // controll if all potentially invalid coins, have coin creation record in DAG history
        if invalid_coins_dict.keys().len() != maybe_invalid_coins.len()
        {
            dlog(
                &format!("The block uses some un-existed inputs. may be machine is not synced. block({})", cutils::hash8c(block_hash)),
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, invalid_coins_dict, used_coins_dict.clone(), false, coins_in_spent_table);
        }

        // * control if invalidity is because of using really unmatured outputs(which will be matured in next hours)?
        // * if yes drop block
        for a_coin in invalid_coins_dict.keys()
        {
            let now_ = application().now();
            let is_matured = application().is_matured(
                &invalid_coins_dict[a_coin]["coinGenDocType"].to_string(),
                &invalid_coins_dict[a_coin]["coinGenCreationDate"].to_string(),
                &now_);
            if !is_matured
            {
                dlog(
                    &format!("The block uses at least one un-maturated input: block({}) coin({})",
                             cutils::hash8c(block_hash), a_coin),
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, invalid_coins_dict, used_coins_dict.clone(), false, coins_in_spent_table);
            }
        }
    }

    for an_invalid_coin_code in invalid_coins_dict.keys()
    {
        // append also invalid refs to used coins dict
        let the_coin: Coin = Coin {
            m_coin_code: an_invalid_coin_code.clone(),
            m_creation_date: "".to_string(),
            m_ref_creation_date: invalid_coins_dict[an_invalid_coin_code]["coinGenCreationDate"].clone(),
            m_coin_owner: invalid_coins_dict[an_invalid_coin_code]["coinGenOutputAddress"].clone(),
            ut_visible_by: "".to_string(),
            m_coin_value: invalid_coins_dict[an_invalid_coin_code]["coinGenOutputValue"].parse::<CMPAIValueT>().unwrap(),
        };
        used_coins_dict.insert(an_invalid_coin_code.clone(), the_coin);

        // * adding to spend-input-dictionary the invalid coins in current block too
        // * in order to having a complete history & order of entire spent coins of the block
        if !coins_in_spent_table.m_coins_dict.contains_key(an_invalid_coin_code)
        {
            coins_in_spent_table.m_coins_dict.insert(an_invalid_coin_code.clone(), vec![]);// < SpendCoinInfo * > {};
        }

        let mut tmp_dict = coins_in_spent_table.m_coins_dict[an_invalid_coin_code].clone();
        let tmp_elm: SpendCoinInfo = SpendCoinInfo {
            m_spend_date: block_creation_date.clone(),
            m_spend_block: block_hash.clone(),
            m_spend_document: map_coin_to_spender_doc[an_invalid_coin_code].clone(),
            m_spend_order: 0,
        };
        tmp_dict.push(tmp_elm);

        coins_in_spent_table.m_coins_dict.insert(an_invalid_coin_code.clone(), tmp_dict);
    }


    // all spent_loc must exist in invalid_coins_dict
    let tmp1: VString = invalid_coins_dict.keys().cloned().collect();
    let tmp2: VString = coins_in_spent_table.m_coins_dict.keys().cloned().collect::<VString>();
    if (tmp1.len() != tmp2.len()) ||
        (cutils::array_diff(&tmp1, &tmp2).len() > 0) ||
        (cutils::array_diff(&tmp2, &tmp1).len() > 0)
    {
        msg = format!(
            "Finding invalidation's messed up block({}) maybe Invalid Inputs, invalid Coins Dict: {:?} coins In Spent Table: {:?}",
            cutils::hash8c(block_hash),
            invalid_coins_dict,
            coins_in_spent_table);
        dlog(
            &msg,
            constants::Modules::Sec,
            constants::SecLevel::Error);
        return (false, invalid_coins_dict.clone(), used_coins_dict, false, coins_in_spent_table);
    }

    let mut is_sus_block = false;
    if invalid_coins_dict.keys().len() > 0
    {
        msg = format!("Some transaction inputs in block({}) are not valid, these are duplicated inputs: {:?}",
                      cutils::hash8c(block_hash), invalid_coins_dict);
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        is_sus_block = true;
    }

    // apllying machine-POV-order to coinsInSpentTable as an order-attr
    let coins_code = coins_in_spent_table.m_coins_dict.keys().cloned().collect::<VString>();
    for a_coin_code in &coins_code
    {
        //looping on orders
        let mut coins_vec = coins_in_spent_table.m_coins_dict[a_coin_code].clone();
        for inx in 0..coins_vec.len() as COutputIndexT
        {
            coins_vec[inx as usize].m_spend_order = inx;
        }
        coins_in_spent_table.m_coins_dict.insert(a_coin_code.clone(), coins_vec);
    }


    return (
        true,
        invalid_coins_dict,
        used_coins_dict,
        is_sus_block,
        coins_in_spent_table);
}

//old_name_was validateTransactions
pub fn validate_transactions(block: &Block, stage: &str) ->
(bool /* status */, bool /* is_sus_block */, String/* msg */, SpendCoinsList /* double_spends */)
{
    let msg: String;

    if block.m_block_ext_info.len() == 0
    {
        msg = format!("Missed ext Info for Block! {}, {}", block.get_block_identifier(), block.safe_stringify_block(true));
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, false, msg, SpendCoinsList::new());
    }

    let mut block_overview: BlockOverview = prepare_block_overview(block);
    if !block_overview.m_status
    { return (false, false, block_overview.m_msg, SpendCoinsList::new()); }

    let maybe_invalid_coins: VString = block_overview.m_block_not_matured_coins;

    let mut sum_remotes: CMPAIValueT = 0;
    let mut treasury_incomes: CMPAIValueT = 0;
    let mut backer_incomes: CMPAIValueT = 0;

    // let remoteBlockDPCostBacker = 0;
    for doc_inx in 0..block.get_docs_count() as CDocIndexT
    {
        let a_doc: &Document = &block.get_documents()[doc_inx as usize];

        // do validate only transactions
        if !a_doc.is_basic_transaction() &&
            !a_doc.is_dp_cost_payment()
        { continue; }


        // DPCOst payment control
        if a_doc.get_doc_type() == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT.to_string()
        {
            let (status, treasury_incomes_, backer_incomes_) = retrieve_dp_cost_info(
                a_doc,
                &block.m_block_backer);
            if !status
            { return (false, false, "Failed in calculation of retrieve-DPCost-Info".to_string(), SpendCoinsList::new()); }
            treasury_incomes = treasury_incomes_;
            backer_incomes = backer_incomes_;
        }

        let mut trx_stated_dp_cost: CMPAIValueT = 0;
        if block_overview.m_supported_p4p.contains(&a_doc.get_doc_hash())
        {
            dlog(
                &format!("The trx is supported by p4p trx. Block {} trx {}",
                         block.get_block_identifier(),
                         a_doc.get_doc_identifier()
                ),
                constants::Modules::Trx,
                constants::SecLevel::Info);
            // so we do not need to control trx fee, because it is already payed
        } else if [constants::document_types::DATA_AND_PROCESS_COST_PAYMENT.to_string()].contains(&a_doc.get_doc_type())
        {
            // this kind of documents do not need to have trx-fee
        } else {
            if !constants::SUPPORTS_CLONED_TRANSACTION && (a_doc.get_dpis().len() > 1)
            {
                msg = format!("The network still do not accept Cloned transactions!");
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, false, msg, SpendCoinsList::new());
            }

            for a_dpi_index in a_doc.get_dpis()
            {
                if a_doc.get_outputs()[*a_dpi_index as usize].m_address == block.m_block_backer
                {
                    trx_stated_dp_cost = a_doc.get_outputs()[*a_dpi_index as usize].m_amount;
                }
            }

            if trx_stated_dp_cost == 0
            {
                msg = format!(
                    "At least one trx hasn't backer fee! Block {} Trx {}",
                    block.get_block_identifier(),
                    a_doc.get_doc_identifier()
                );
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, false, msg, SpendCoinsList::new());
            }

            if trx_stated_dp_cost < get_transaction_minimum_fee(&block.get_creation_date())
            {
                msg = format!(
                    "The backer fee is less than Minimum acceptable fee!! Block {} Trx {} trx_stated_dp_cost({}) < minimum fee({})",
                    block.get_block_identifier(),
                    a_doc.get_doc_identifier(),
                    trx_stated_dp_cost,
                    get_transaction_minimum_fee(&block.get_creation_date())
                );
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, false, msg, SpendCoinsList::new());
            }

            let (status, locally_recalculate_trx_dp_cost) = a_doc.calc_doc_data_and_process_cost(
                stage,
                &block.get_creation_date(),
                0);
            if !status
            { return (false, false, "Failed in calc-Doc-Data-And-Process-Cost".to_string(), SpendCoinsList::new()); }

            dlog(
                &format!("Compare costs(remote: {} local: {}) doc {}  Block {}",
                         cutils::sep_num_3(trx_stated_dp_cost as CMPAISValueT),
                         cutils::sep_num_3(locally_recalculate_trx_dp_cost as CMPAISValueT),
                         a_doc.get_doc_identifier(),
                         block.get_block_identifier()
                ),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);

            if trx_stated_dp_cost < locally_recalculate_trx_dp_cost
            {
                msg = format!(
                    "Miss-calculated document length: {} The backer fee is less than network values! Block {} Trx {} trx_stated_dp_cost({}) < network minimum fee({}) nano-PAIs",
                    a_doc.safe_stringify_doc(true),
                    block.get_block_identifier(),
                    a_doc.get_doc_identifier(),
                    cutils::sep_num_3(trx_stated_dp_cost as CMPAISValueT),
                    cutils::sep_num_3(locally_recalculate_trx_dp_cost as CMPAISValueT)
                );
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return (false, false, msg, SpendCoinsList::new());
            }
        }

        sum_remotes += trx_stated_dp_cost;
    }

    dlog(
        &format!(
            "Backer Fees Sum = {} PAIs for Block {} ",
            cutils::sep_num_3(sum_remotes as CMPAISValueT),
            block.get_block_identifier()
        ),
        constants::Modules::App,
        constants::SecLevel::Info);

    // control if block total trx fees are valid
    let block_fix_cost = get_block_fix_cost(&block.get_creation_date());
    let before_block_tax = (sum_remotes as f64 * constants::BACKER_PERCENT_OF_BLOCK_FEE) / 100.0;
    let recalc_remote_backer_fee: CMPAIValueT = before_block_tax as CMPAIValueT - block_fix_cost;
    if recalc_remote_backer_fee != backer_incomes
    {
        msg = format!(
            "The locally calculated backer fee is not what remote is! local({}) remote({}) nano-PAIs, Block {}",
            cutils::sep_num_3(recalc_remote_backer_fee as CMPAISValueT),
            cutils::sep_num_3(backer_incomes as CMPAISValueT),
            block.get_block_identifier()
        );
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, false, msg, SpendCoinsList::new());
    }

    let locally_recalculate_block_treasury_income: CMPAIValueT = sum_remotes - recalc_remote_backer_fee;
    if locally_recalculate_block_treasury_income != treasury_incomes
    {
        msg = format!(
            "The locally calculated treasury is not what remote is! Block {} locally-recalculate-block-treasury-income({}) treasury-incomes({}) nano-PAIs",
            block.get_block_identifier(),
            cutils::sep_num_3(locally_recalculate_block_treasury_income as CMPAISValueT),
            cutils::sep_num_3(treasury_incomes as CMPAISValueT)
        );
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, false, msg, SpendCoinsList::new());
    }

    let scuds: GRecordsT = HashMap::new();
    if constants::SUPER_CONTROL_COINS_DOUBLE_SPENDING
    {
        // * after being sure about secure and proper functionality of code, we can cut this control in next months
        // * finding the block(s) which are used these coins and already are registered in DAG
        let (status, _scuds) = SpentCoinsHandler::find_coins_spend_locations(&block_overview.m_block_used_coins);
        if !status
        {
            return (false, false, "Failed in find-Coins-Spend-Locations".to_string(), SpendCoinsList::new());
        }

        if scuds.keys().len() > 0
        {
            msg = format!(
                "SCUDS: SUPER-CONTROL-COINS-DOUBLE-SPENDING found some double-spending with block {} SCUDS.spendsDict: {:?}",
                block.get_block_identifier(), scuds
            );
            dlog(
                &msg,
                constants::Modules::Sec,
                constants::SecLevel::Error);
            return (false, false, msg, SpendCoinsList::new());
        }
    }

    if constants::SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING
    {
        // * most paranoidic and pessimistic control of input validation
        // * for now I put this double-control to also quality control of the previous-controls.
        // * this control is too costly, so it must be removed or optimized ASAP
        let (validate_status, validate_msg, _coins_track) = tracking_back_the_coins(
            block,
            &vec![],
            &vec![]);
        if !validate_status
        {
            msg = format!(
                "SuperValidate, block {} error message: {}",
                block.get_block_identifier(),
                validate_msg
            );
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Error);
            return (false, false, msg, SpendCoinsList::new());
        } else {
            dlog(
                &format!("SuperValidate, block {} inputs have confirmed path going back to coinbase", block.get_block_identifier()),
                constants::Modules::Trx,
                constants::SecLevel::Info);
        }
    }

    let (
        status2,
        invalid_coins_dict,
        used_coins_dict_,
        is_sus_block,
        double_spends) =
        consider_invalid_coins(
            &block.get_block_hash(),
            &block.m_block_creation_date,
            &block_overview.m_block_used_coins,
            &block_overview.m_used_coins_dict,
            &maybe_invalid_coins,
            &block_overview.m_map_coin_to_spender_doc);
    if !status2
    { return (false, false, "Failed in consider-Invalid-Coins".to_string(), double_spends); }
    block_overview.m_used_coins_dict = used_coins_dict_;

    let equation_control_res = validate_equation(
        block,
        &block_overview.m_used_coins_dict,
        &invalid_coins_dict);
    if !equation_control_res
    { return (false, false, "Failed in validate-Equation".to_string(), double_spends); }

    // * control coin visibility in DAG history by going back throught ancestors
    // * since the block can contains only UTXOs which are already took palce in her hsitory
    // * in oder words, they are in block's sibility
    if !is_sus_block && !constants::SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING
    {
        let is_visible = control_coins_visibility_in_graph_history(
            &block_overview.m_block_used_coins,
            &block.m_block_ancestors,
            &block.get_block_hash());
        if !is_visible
        { return (false, false, "Failed in control-Coins-Visibility-In-Graph-History".to_string(), double_spends); }
    }


    return (true, is_sus_block, "valid".to_string(), double_spends);
}

//old_name_was appendTransactions
pub fn append_transactions(
    block: &mut Block,
    transient_block_info: &mut TransientBlockInfo)
    -> (bool /* creating block status */, bool /* should empty buffer */, String /* msg */)
{
    return fetch_buffered_transactions(block, transient_block_info);
}

