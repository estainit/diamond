use serde_json::json;
use crate::{application, constants, cutils, dlog, get_value, machine};
use crate::lib::block::block_types::block::Block;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CDateT, CMPAISValueT, CMPAIValueT, JSonArray, VString};
use crate::lib::dag::dag::set_coins_import_status;
use crate::lib::dag::dag_walk_through::get_all_descendants;
use crate::lib::dag::normal_block::import_coins::analyze_block_used_coins::analyze_block_used_coins;
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::CoinImportDataContainer;
use crate::lib::dag::normal_block::normal_coins_handler::retrieve_proper_blocks;
use crate::lib::k_v_handler::upsert_kvalue;
use crate::lib::services::treasury::treasury_handler::{donate_transaction_input, insert_income};
use crate::lib::transactions::basic_transactions::coins::coins_handler::{add_new_coin, refresh_visibility};

//old_name_was importNormalBlockUTXOs
pub fn import_normal_block_coins(c_date: &CDateT)
{
    dlog(
        &format!("Importing Normal block Coins at {}", c_date),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    let block_rows = retrieve_proper_blocks(c_date);
    if block_rows.len() == 0
    {
        dlog(
            &format!("There is no importable normal block for time({})", c_date),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        return;
    }

    let mut block_inspect_container: CoinImportDataContainer = CoinImportDataContainer::new();
    // let block: Block;


    for a_block_row in block_rows
    {
        block_inspect_container.reset();
        // drop(&block);

        let (_status, _sf_ver, serialized_block_body) = unwrap_safed_content_for_db(&a_block_row["b_body"].to_string());
        let (status, block) = Block::load_block_by_serialized_content(&serialized_block_body);
        if !status
        {
            let msg: String = format!(
                "Malformed recorded block({})!",
                a_block_row["b_hash"]);
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Fatal);
            panic!("{}", msg);
        }
        dlog(
            &format!(
                "Extract matured coins(NormalBlock) on c_date({}) from block({}) created on({})",
                c_date,
                cutils::hash8c(&a_block_row["b_hash"].to_string()),
                block.m_block_creation_date),
            constants::Modules::Trx,
            constants::SecLevel::Info);

        analyze_block_used_coins(&mut block_inspect_container, &block);

        block_inspect_container.m_dp_cost_coin_codes = vec![];

        for a_key in block_inspect_container.m_a_single_trx_dp_cost.keys()
        {
            block_inspect_container.m_dp_cost_coin_codes.push(block_inspect_container.m_a_single_trx_dp_cost[a_key].m_coin.clone());
        }

        if block_inspect_container.m_must_not_import_trx_outputs.len() > 0
        {
            block_inspect_container.m_must_not_import_trx_outputs.sort();
        }
        block_inspect_container.m_must_not_import_trx_outputs.dedup();
        // Vec<CDocHashT>::iterator last = std::unique(block_inspect_container.m_must_not_import_trx_outputs.begin(), block_inspect_container.m_must_not_import_trx_outputs.end());
        // block_inspect_container.m_must_not_import_trx_outputs.erase(last, block_inspect_container.m_must_not_import_trx_outputs.end());
        dlog(
            &format!(
                "Block-inspect-container{} block_inspect_container: {:?}",
                block.get_block_identifier(),
                block_inspect_container),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);


        if machine().is_in_sync_process(false)
        {
            // SUS BLOCKS WHICH NEEDED VOTES TO BE IMPORTED AHAED (WNVTBIA)
            let current_wnvtbia = get_value("SusBlockWNVTBIA");
            let mut js_current_wnvtbia: JSonArray = json!([]);
            if current_wnvtbia == ""
            {
                let ser = cutils::controlled_json_stringify(&json!([]));
                upsert_kvalue("SusBlockWNVTBIA", &ser, false);
            } else {
                let (_status, js_current_wnvtbia_) = cutils::controlled_str_to_json(&current_wnvtbia);
                js_current_wnvtbia = js_current_wnvtbia_;
            }
            if !js_current_wnvtbia[block.get_block_hash()].is_null()
            {
                if !block_inspect_container.m_block_is_sus_case
                {
                    // * during insert to parsing q, machine recognized the block is suspicious and must has some FVotes
                    // * and now machine recognized the sus votes still not being considered
                    // * so returns back inorder to giving more time to machine to insert upcoming sus votes in few later seconds
                    dlog(
                        &format!(
                            "Can not import block coins because parsingQ reorganization was sus and now there is no vote! \
                                     Block{} blockIsSusCase {} block_inspect_container: {:?}",
                            block.get_block_identifier(),
                            block_inspect_container.m_block_is_sus_case,
                            block_inspect_container
                        ),
                        constants::Modules::Trx,
                        constants::SecLevel::Warning);
                    continue;
                }
            }
        }

        if block_inspect_container.m_does_enough_sus_votes_exist == "notEnoughSusVotesExist".to_string()
        {
            // log block import report
            //  logNormalBlockUTXOsImport.logImport({
            //  blockHash: block.blockHash,
            //  block_inspect_container: _.clone(block_inspect_container)
            //  });
            continue;
        }

        // find all descendant of current block(if exist)
        let (_status, w_blocks_descendants, _validity_percentage) =
            get_all_descendants(&block.get_block_hash(), false);


        // donate double spended funds(if exist)
        for an_entry in &block_inspect_container.m_block_treasury_logs
        {
            donate_transaction_input(
                &an_entry.m_title,
                &an_entry.m_cat,
                &an_entry.m_descriptions,
                &block.m_block_creation_date,
                an_entry.m_value,
                &block.get_block_hash(),
                &an_entry.m_coin,
            );
        }


        // calculate if Block Trx Fee must be modified
        let mut to_cut: CMPAIValueT = 0;
        let mut to_cut_from_treasury_fee: CMPAIValueT = 0;
        let mut to_cut_from_backer_fee: CMPAIValueT = 0;
        for doc_hash in &block_inspect_container.m_must_not_import_trx_outputs
        {
            to_cut += block_inspect_container.m_a_single_trx_dp_cost[doc_hash].m_value; // cut the DPCost of rejected/donated transaction from block incomes
        }
        if to_cut > 0
        {
            to_cut_from_backer_fee = cutils::c_floor((to_cut as f64 * constants::BACKER_PERCENT_OF_BLOCK_FEE) / 100.0) as CMPAIValueT;// - get_block_fix_cost();
            to_cut_from_treasury_fee = cutils::c_floor((to_cut - to_cut_from_backer_fee) as f64) as CMPAIValueT;
        }
        block_inspect_container.m_to_cut_from_backer_fee = to_cut_from_backer_fee;
        block_inspect_container.m_to_cut_from_treasury_fee = to_cut_from_treasury_fee;

        if block_inspect_container.m_rejected_transactions.len() > 0
        {
            // listener.doCallSync('SPSH_block_has_double_spend_input', { block, block_inspect_container });
        }

        // import block DPCost Backer & Treasury
        block_inspect_container.m_block_dp_cost_backer_final = block_inspect_container.m_block_dp_cost_backer.m_value - block_inspect_container.m_to_cut_from_backer_fee;
        block_inspect_container.m_block_dp_cost_treasury_final = block_inspect_container.m_block_dp_cost_treasury.m_value - block_inspect_container.m_to_cut_from_treasury_fee;

        if block_inspect_container.m_block_dp_cost_backer_final < 0
        {
            block_inspect_container.m_block_dp_cost_treasury_final += block_inspect_container.m_block_dp_cost_backer_final; // to cover get_block_fix_cost()
        }

        block_inspect_container.m_block_has_income =
            (block_inspect_container.m_block_dp_cost_backer_final > 0)
                && (block_inspect_container.m_block_dp_cost_treasury_final > 0);


        if block_inspect_container.m_block_has_income
        {

            // import backer's income
            dlog(
                &format!(
                    "Importing Normal block Coins(Backer) Block{}",
                    block.get_block_identifier()),
                constants::Modules::Trx,
                constants::SecLevel::Info);

            for a_w_block in &w_blocks_descendants
            {
                add_new_coin(
                    &a_w_block["b_creation_date"].to_string(),
                    &block_inspect_container.m_block_dp_cost_backer.m_coin,
                    &a_w_block["b_hash"].to_string(),
                    &block_inspect_container.m_block_dp_cost_backer.m_address,
                    block_inspect_container.m_block_dp_cost_backer_final,
                    &block.m_block_creation_date);
            }

            // import blockDPCost_Treasury
            let mut title: String = block_inspect_container.m_block_dp_cost_treasury.m_title.clone();

            if block_inspect_container.m_must_not_import_trx_outputs.len() > 0
            {
                // cut fees because of rejected transactions or ...
                let mut tmp: VString = vec![];
                for elm in &block_inspect_container.m_must_not_import_trx_outputs
                { tmp.push(cutils::hash8c(&elm)); }
                title = format!(
                    "{} - rejected TRXs({:?}) = sum({}) ",
                    title,
                    tmp,
                    cutils::nano_pai_to_pai(block_inspect_container.m_to_cut_from_treasury_fee as CMPAISValueT));
            }

            insert_income(
                &title,
                &block_inspect_container.m_block_dp_cost_treasury.m_cat,
                &block_inspect_container.m_block_dp_cost_treasury.m_descriptions,
                &block.m_block_creation_date,
                block_inspect_container.m_block_dp_cost_treasury_final,
                &block.get_block_hash(),
                &block_inspect_container.m_block_dp_cost_treasury.m_coin);

            //       // import free-docs costs payments to treasury
            //       FreeDocument::importCostsToTreasury(block, block_inspect_container);
            //
            //       // import Ballot costs payments to treasury
            //       BallotDocument::importCostsToTreasury(block, block_inspect_container);
            //
            //       // import Polling costs payments to treasury
            //       PollingDocument::importCostsToTreasury(block, block_inspect_container);
            //
            //       // import request for adm polling costs payments to treasury
            //       AdministrativePollingDocument::importCostsToTreasury(block, block_inspect_container);
            //
            // //      // TODO: remove to
            // //      // // import request for relaese reserved coins costs payments to treasury
            // //      // block_inspect_container.reqRelResCostStatus = reqRelResCostsHandler.importReqRelResCost({ block, block_inspect_container });
            //
            //       // import proposal costs payments to treasury
            //       ProposalDocument::importCostsToTreasury(block, block_inspect_container);  //importProposalsCost
            //
            //       // import FleNS costs(register, binding,...) payments to treasury
            //       INameRegDocument::importCostsToTreasury(block, block_inspect_container);  // importRegCost
            //       INameBindDocument::importCostsToTreasury(block, block_inspect_container);  // importBindingCost
            //       // IName Msg Document::importCostsToTreasury(block, block_inspect_container);  // importRegCost
            //
            //
            //       // import pledge costs payments to treasury
            //       PledgeDocument::importCostsToTreasury(block, block_inspect_container);
            //
            //       // import close pledge costs payments to treasury
            //       ClosePledgeDocument::importCostsToTreasury(block, block_inspect_container);


            // import normal UTXOs
            for a_coin in &block_inspect_container.m_importable_coins
            {
                // remove Ceased transaction's DPCost, if they are in a same block with related P4P transaction
                // or if the transaction is in some other block which is created by backers which are not listed in dPIs list of transaction
                if block_inspect_container.m_cut_ceased_trx_from_coins.contains(&a_coin.m_coin_code)
                { continue; }

                if block_inspect_container.m_dp_cost_coin_codes.contains(&a_coin.m_coin_code)
                { continue; }

                // looping on all descendants of current block, to be sure all descendant can see their UTXOs in their history
                dlog(
                    &format!(
                        "Final Importing Normal block Coins {:?} Block{}",
                        a_coin,
                        block.get_block_identifier()
                    ),
                    constants::Modules::Trx,
                    constants::SecLevel::TmpDebug);
                for a_w_block in &w_blocks_descendants
                {
                    add_new_coin(
                        &a_w_block["b_creation_date"].to_string(),
                        &a_coin.m_coin_code,
                        &a_w_block["b_hash"].to_string(),
                        &a_coin.m_coin_owner,
                        a_coin.m_coin_value,
                        &a_coin.m_creation_date);  // refCreationDate:
                }
            }
        }

        // restoring UTXOs of rejected transactions
        for a_coin in &block_inspect_container.m_to_be_restored_coins
        {
            dlog(
                &format!(
                    "A to Be Restored coin: {:?}", a_coin),
                constants::Modules::Trx,
                constants::SecLevel::Warning);

            // looping on all descendants of current block, to be sure all descendant can see their utxo in their history
            dlog(
                &format!(
                    "Importing Normal block Coins(restored) Block{}", block.get_block_identifier()),
                constants::Modules::Trx,
                constants::SecLevel::Info);

            for a_w_block in &w_blocks_descendants
            {
                add_new_coin(
                    &a_w_block["b_creation_date"],
                    &a_coin.m_cd_code,
                    &a_w_block["b_hash"],
                    &a_coin.m_cd_owner,
                    a_coin.m_cd_amount,
                    &a_coin.m_cd_creation_date);  // refCreationDate:
            }
        }

        // log block import report
        //    logNormalBlockUTXOsImport.logImport({
        //      blockHash: block.blockHash,
        //      block_inspect_container: _.clone(block_inspect_container)
        //    });


        // update coin imported
        set_coins_import_status(&block.get_block_hash(), &constants::YES.to_string());
    }

    dlog(
        &format!(
            "Block-inspect-container Final Result: {:?}", block_inspect_container),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    drop(&block_inspect_container);

    // finally refresh coins visibilities
    let now_ = application().now();
    refresh_visibility(&now_);
}

