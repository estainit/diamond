use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CCoinCodeT, CDocHashT, CMPAIValueT, COutputIndexT, QV2DicT, VString};
use crate::lib::dag::dag::{get_coins_generation_info_via_sql, retrieve_blocks_in_which_a_coin_have_been_produced};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{BlockAlterTreasuryIncome, BlockDPCostBacker, BlockDPCostTreasury, BlockTreasuryLog, CoinImportDataContainer, SusInputDetection, ValidityCheck};
use crate::lib::dag::normal_block::rejected_transactions_handler::add_to_rejected_transactions;
use crate::lib::transactions::basic_transactions::coins::coins_handler::{CoinInfo, CoinDetails};
use crate::lib::transactions::basic_transactions::coins::suspect_trx_handler::{retrieve_voter_percentages};

// const bool OUTPUT_TIMELOCK_IS_ENABLED = false;    //TODO: develope, teset and release this feature ASAP

//old_name_was extractDocImportableUTXOs
pub fn extract_doc_importable_coins(
    block_inspect_container: &mut CoinImportDataContainer,
    block: &Block,
    doc: &Document)
{
    dlog(
        &format!("Importing coins from block{} doc{}",
                 block.get_block_identifier(),
                 doc.get_doc_identifier()
        ),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    if doc.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT
    {
        // the block fee payment transaction always has to have no input and 2 outputs.
        // 1. TP_DP
        // 2. backer fee
        block_inspect_container.m_block_dp_cost_treasury = BlockDPCostTreasury {
            m_cat: "TP_DP".to_string(),
            m_title: format!("TP_DP Backing fee block{}", block.get_block_identifier()),
            m_descriptions: format!("Backing fee block{}", block.get_block_identifier()),
            m_coin: cutils::pack_coin_code(&doc.get_doc_hash(), 0),
            m_value: doc.get_outputs()[0].m_amount,
        };


        block_inspect_container.m_block_dp_cost_backer = BlockDPCostBacker {
            m_coin: cutils::pack_coin_code(&doc.get_doc_hash(), 1),
            m_address: doc.get_outputs()[1].m_address.clone(),
            m_value: doc.get_outputs()[1].m_amount,
        };
        return;
    }

    if doc.get_outputs().len() == 0
    {
        return;
    }

    for output_index in 0..doc.get_outputs().len()
    {
        let an_output = &doc.get_outputs()[output_index];
        let mut new_coin: CCoinCodeT = "".to_string();

        // exclude backerfee from all documents outputs, except the integrated one in which there is also treasury income
        if (constants::TREASURY_PAYMENTS.contains(&an_output.m_address.as_str()))
            && (doc.m_doc_type != constants::document_types::DATA_AND_PROCESS_COST_PAYMENT)
        {
            if !block_inspect_container.m_block_alter_treasury_incomes.contains_key(&an_output.m_address)
            {
                block_inspect_container.m_block_alter_treasury_incomes.insert(an_output.m_address.clone(), vec![]);
            }
            dlog(
                &format!("Going to insert a block Alter Treasury Incomes: {}",
                         an_output.m_address),
                constants::Modules::App,
                constants::SecLevel::Info);

            let mut tmp = block_inspect_container.m_block_alter_treasury_incomes[&an_output.m_address].clone();
            tmp.push(
                BlockAlterTreasuryIncome {
                    m_trx_hash: doc.get_doc_hash(),
                    m_coin: cutils::pack_coin_code(&doc.get_doc_hash(), output_index as COutputIndexT),
                    m_value: an_output.m_amount,
                });
            block_inspect_container.m_block_alter_treasury_incomes.insert(an_output.m_address.clone(), tmp);
            dlog(
                &format!("Block-inspect-container.blockAlterTreasuryIncomes {:?}",
                         block_inspect_container.m_block_alter_treasury_incomes),
                constants::Modules::Trx,
                constants::SecLevel::TmpDebug);
        } else {
            new_coin = cutils::pack_coin_code(&doc.get_doc_hash(), output_index as COutputIndexT);
        }


        // exclude backerfee(s), the one intended for this current backer and potentially other cloned backer's fee
        if
        (doc.m_doc_type != constants::document_types::DATA_AND_PROCESS_COST_PAYMENT)
            && (doc.get_dpis().len() > 0)
            && doc.get_dpis().contains(&(output_index as COutputIndexT))
        // && (anOutput[0] == block.backer)
        {
            new_coin = "".to_string();  //cutCeasedTrxFromUTXOs
        }

        if new_coin != ""
        {
            block_inspect_container.m_importable_coins.push(CoinInfo {
                m_coin_code: new_coin,
                m_creation_date: "".to_string(),
                m_ref_creation_date: block.get_creation_date(),
                m_coin_owner: an_output.m_address.clone(),
                m_visible_by: "".to_string(),
                m_coin_value: an_output.m_amount,
            });
        }
    }

    return;
}

//old_name_was doSusTreatments
pub fn do_sus_treatments(
    block_inspect_container: &mut CoinImportDataContainer,
    block: &Block,
    doc: &Document)
{
    let doc_hash: CDocHashT = doc.get_doc_hash();
    let considered_sus_coin_codes =
        block_inspect_container
            .m_transactions_validity_check[&doc_hash]
            .m_sus_vote_res
            .keys()
            .cloned()
            .collect::<VString>();

    let validity: &ValidityCheck = &block_inspect_container.m_transactions_validity_check[&doc_hash];
    dlog(
        &format!(
            "Validity response for block{} doc({}) validity: {:?}",
            block.get_block_identifier(),
            cutils::hash8c(&doc_hash),
            validity
        ),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    for the_coin in &considered_sus_coin_codes
    {
        if validity.m_sus_vote_res[the_coin].m_valid == true
        {
            // this input is valid, but by rejecting whole transaction and restoring inputs the input will remain un-spended.
//      block_inspect_container.m_to_be_restored_coins.push(the_coin);
            block_inspect_container.m_sus_inputs_detection.push(
                SusInputDetection {
                    m_coin: the_coin.clone(),
                    m_detection: "valid input of considered sus coins!!!".to_string(),
                });
            continue;
        }

        if validity.m_sus_vote_res[the_coin].m_action == "reject".to_string()
        {
            // even one rejection or donation of inputs is enough to cut trxFee dfrom blockTrxFee & bloockTreasuryFee
            block_inspect_container.m_must_not_import_trx_outputs.push(doc_hash.clone());
            block_inspect_container.m_transactions_detection.insert(doc_hash.clone(), "reject".to_string());

            dlog(
                &format!(
                    "SusVotes: Reject coin({}) of transaction({}) in block{} because of susVotes",
                    cutils::short_coin_code(the_coin),
                    cutils::hash8c(&doc_hash),
                    block.get_block_identifier()
                ),
                constants::Modules::Trx,
                constants::SecLevel::Warning);

            block_inspect_container.m_sus_inputs_detection.push(
                SusInputDetection {
                    m_coin: the_coin.clone(),
                    m_detection: "Already spent input".to_string(),
                });

            if !block_inspect_container.m_rejected_transactions.contains_key(&doc_hash)
            {
                block_inspect_container.m_rejected_transactions.insert(doc_hash.clone(), vec![]);// Vec < CCoinCodeT > {};
            }

            let mut tmp = block_inspect_container.m_rejected_transactions[&doc_hash].clone();
            tmp.push(the_coin.clone());
            block_inspect_container.m_rejected_transactions.insert(doc_hash.clone(), tmp);

            add_to_rejected_transactions(
                &block.get_block_hash(),
                &doc_hash,
                the_coin);

            continue;
        }

        if validity.m_sus_vote_res[the_coin].m_action == "donate".to_string()
        {
            // even one rejection or donation of inputs is enough to cut trxFee dfrom blockTrxFee & bloockTreasuryFee
            block_inspect_container.m_must_not_import_trx_outputs.push(doc_hash.clone());

            dlog(
                &format!(
                    "SusVotes: Donating coin({}) of transaction({}) in block{} because of susVotes",
                    cutils::short_coin_code(the_coin),
                    cutils::hash8c(&doc_hash),
                    block.get_block_identifier()
                ),
                constants::Modules::Trx,
                constants::SecLevel::Warning);

            // donate conflicted input, and since do not consider transaction, so the other inputs remain untouched
            let donate_coins_blocks: Vec<CoinDetails> = retrieve_blocks_in_which_a_coin_have_been_produced(the_coin);

            dlog(
                &format!(
                    "Donate Transaction Input. blocks by the_coin:({}) of transaction({}) in block{} because of susVotes: {:?}",
                    cutils::short_coin_code(the_coin),
                    cutils::hash8c(&doc_hash),
                    block.get_block_identifier(),
                    donate_coins_blocks
                ),
                constants::Modules::Trx,
                constants::SecLevel::Warning);
            // big FIXME: for cloning transactions issue
            block_inspect_container.m_block_treasury_logs.push(BlockTreasuryLog {
                m_title: "Donate because of DOUBLE-SPENDING".to_string(),
                m_cat: "TP_DONATE_DOUBLE_SPEND".to_string(),
                m_descriptions: format!("Pay to treasury because of trx conflict in block{} transaction({})", block.get_block_identifier(), cutils::hash8c(&doc_hash)),
                m_coin: the_coin.clone(),
                m_value: donate_coins_blocks[0].m_cd_amount,
                m_donate_coins_blocks: donate_coins_blocks,
            });

            block_inspect_container.m_sus_inputs_detection.push(SusInputDetection {
                m_coin: the_coin.clone(),
                m_detection: "Donate input".to_string(),
            });

            if !block_inspect_container.m_rejected_transactions.contains_key(&doc_hash)
            {
                block_inspect_container.m_rejected_transactions.insert(doc_hash.clone(), vec![]);
            }
            let mut tmp = block_inspect_container.m_rejected_transactions[&doc_hash].clone();
            tmp.push(the_coin.clone());
            block_inspect_container.m_rejected_transactions.insert(doc_hash.clone(), tmp);

            // record rejected transaction
            add_to_rejected_transactions(
                &block.get_block_hash(),
                &doc_hash,
                the_coin);
        } else {
            panic!("Strange situation in sus voting");
        }
    }

    // the entire UTXOs of a block during adding block to DAG are removed from trx_utxos
    // so here we have to resotore used inocent UTXOs of this rejected-transaction or donated-transaction into spendable UTXOs,
    // (except conflicted the_coins)
    for input in doc.get_inputs()
    {
        let a_coin = input.get_coin_code();
        if !considered_sus_coin_codes.contains(&a_coin)
        {
            let ref_loc_block: QV2DicT = get_coins_generation_info_via_sql(&vec![a_coin.clone()]);
            let tmp_coin = CoinDetails {
                m_cd_code: a_coin.clone(),
                m_cd_owner: ref_loc_block[&a_coin]["coinGenOutputAddress"].to_string(),
                m_cd_amount: ref_loc_block[&a_coin]["coinGenOutputValue"].parse::<CMPAIValueT>().unwrap(),
                m_cd_creation_date: ref_loc_block[&a_coin]["coinGenCreationDate"].clone(),
                m_cd_block_hash: block.get_block_hash(),
                m_cd_doc_index: 0,
                m_cd_doc_hash: "".to_string(),
                m_cd_output_index: 0,
                m_cd_cycle: ref_loc_block[&a_coin]["coinGenCycle"].clone(),
            };
            block_inspect_container.m_to_be_restored_coins.push(tmp_coin.clone());
            block_inspect_container.m_sus_inputs_detection.push(SusInputDetection {
                m_coin: a_coin.clone(),
                m_detection: "Valid input".to_string(),
            });
        }
    }

    return;
}