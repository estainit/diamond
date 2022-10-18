use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::dag::normal_block::import_coins::analyze_a_transaction_coins::analyze_a_transaction_coins;
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::CoinImportDataContainer;
use crate::lib::services::dna::dna_handler::get_an_address_shares;
use crate::lib::transactions::basic_transactions::coins::suspect_trx_handler::get_sus_info_by_block_hash;

//old_name_was analyzeBlockUsedCoins
pub fn analyze_block_used_coins(
    block_inspect_container: &mut CoinImportDataContainer,
    block: &Block)
{
    // Coin Import Module Common Data Container

    // retrieve ext-info
    let (_status_ext_block_info, _stat_ext_exist, _b_ext_info) = block.get_block_exts_infos();

    //  if (bExtInfoRes.bExtInfo == null)
    //  {
    //    msg = `missed bExtInfo5 (${utils.hash16c(block.blockHash)})`;
    //    clog.sec.error(msg);
    //    return { err: true, msg }
    //  }
    //  let bExtInfo = bExtInfoRes.bExtInfo

    // FIXME: is it possible to code reach here with a normal block while there is no entry in sus records?
    // in this case normal block's output will be recorded as spendable outputs!!!!
    let (has_sus_records, votes_dict) =
        get_sus_info_by_block_hash(&block.get_block_hash());  // susVotesByBlockHash =
    block_inspect_container.m_block_is_sus_case = has_sus_records;
    block_inspect_container.m_votes_dict = votes_dict;

    dlog(
        &format!(
            "Extract UTXOs from block({}) block is sus case({:?})",
            cutils::hash8c(&block.get_block_hash()),
            block_inspect_container.m_block_is_sus_case),
        constants::Modules::Trx,
        constants::SecLevel::Info);

    // control shares
    let (_share_count, hu_percent) =
        get_an_address_shares(&constants::HU_SHARE_ADDRESS.to_string(), &block.m_block_creation_date);
    // whenever community reached 29 percent of voting power, so they will not more HU's voting in susBlock consideration
    if (hu_percent < 71.0) && (100.0 - hu_percent < constants::MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK)
    {
        block_inspect_container.m_minimum_floating_vote = 100.0 - hu_percent;
    }

    if block_inspect_container.m_block_is_sus_case
    {
        dlog(
            &format!(
                "Block-inspect-container.susVotesByBlockHash: {:?}",
                block_inspect_container),
            constants::Modules::Sec,
            constants::SecLevel::Error);

        if !block_inspect_container.m_votes_dict.contains_key(&block.get_block_hash())
        {
            dlog(
                &format!(
                    "The sus block{} hasn't even one vote! so has to wait to more susVotes to be inserted",
                    block.get_block_identifier()),
                constants::Modules::Trx,
                constants::SecLevel::Error);
            block_inspect_container.m_does_enough_sus_votes_exist = "notEnoughSusVotesExist".to_string();
            return;
        }

        block_inspect_container.m_current_votes_percentage = block_inspect_container.m_votes_dict[&block.get_block_hash()].m_sum_percent;
        if block_inspect_container.m_current_votes_percentage < block_inspect_container.m_minimum_floating_vote
        {
            // the machine has to wait to enought susVotes to be inserted
            dlog(
                &format!(
                    "The sus block{} has {} percents, so the machine has to wait to more susVotes to be inserted",
                    block.get_block_identifier(),
                    block_inspect_container.m_votes_dict[&block.get_block_hash()].m_sum_percent),
                constants::Modules::Trx,
                constants::SecLevel::Info);
            block_inspect_container.m_does_enough_sus_votes_exist = "notEnoughSusVotesExist".to_string();
            return;
        }

        block_inspect_container.m_does_enough_sus_votes_exist = "Yes".to_string();
        dlog(
            &format!(
                "The block{} discovered as an suspicious, cloned or P4P transaction",
                block.get_block_identifier()
            ),
            constants::Modules::Trx,
            constants::SecLevel::Warning);
    }

    // retrieve p4p supported transactions
    for a_doc in block.get_documents()
    {
        if a_doc.m_doc_class == constants::trx_classes::P4P
        {
            block_inspect_container.m_p4p_docs.push(a_doc.clone());
        }
    }
    // retrieve org ceased backer fee outputs
    if block_inspect_container.m_p4p_docs.len() > 0
    {
        for a_doc in &block_inspect_container.m_p4p_docs
        {
            for out_inx in a_doc.get_dpis()
            {
                if a_doc.get_outputs()[*out_inx as usize].m_address != block.get_block_backer()
                {
                    // * this output of original ceased transaction must not be imported in UTXOs
                    // * because of taking place in this block
                    // * cut From Importable UTXOs Because OF Referenced Ceased Trx
                    block_inspect_container.m_cut_ceased_trx_from_coins.push(
                        cutils::pack_coin_code(&a_doc.get_doc_hash(), out_inx.clone()));
                }
            }
        }
    }


    // looping on documents in a block
    for doc_inx in 0..block.get_docs_count()
    {
        let a_doc = &block.get_documents()[doc_inx as usize];

        if a_doc.get_doc_ref() != ""
        {
            if a_doc.is_basic_transaction()
            {
                block_inspect_container.m_trx_u_dict.insert(a_doc.get_doc_hash(), a_doc.clone());
                block_inspect_container.m_map_u_trx_hash_to_trx_ref.insert(a_doc.get_doc_hash(), a_doc.get_doc_ref());
                block_inspect_container.m_map_u_trx_ref_to_trx_hash.insert(a_doc.get_doc_ref(), a_doc.get_doc_hash());
            } else {
                block_inspect_container.m_map_u_referencer_to_referenced.insert(a_doc.get_doc_hash(), a_doc.get_doc_ref());
                block_inspect_container.m_map_u_referenced_to_referencer.insert(a_doc.get_doc_ref(), a_doc.get_doc_hash());
            }
        }

        if !a_doc.is_basic_transaction() && !a_doc.is_dp_cost_payment()
        { continue; }

//    let docExtInfo = _.has(bExtInfo, docInx) ? bExtInfo[docInx] : null;
        analyze_a_transaction_coins(
            block_inspect_container,
            block,
            a_doc);
    }

    // * since each doc type need different treatment, so maybe it is not good to have a general controller.
    // * here we can only remove not-payed-docs from blockAlterTreasuryIncomes
    // * control if all documents costs are paying by a transaction?
    dlog(
        &format!(
            "Block-inspect-container.blockAlterTreasuryIncomes333: {:?}",
            block_inspect_container.m_block_alter_treasury_incomes),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    // for a_doc_type in block_inspect_container.m_block_alter_treasury_incomes.keys()
    // {
    //     for a_doc in &block_inspect_container.m_block_alter_treasury_incomes[a_doc_type]
    //     {}
    // }

    return;
}

