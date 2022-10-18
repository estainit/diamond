use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CCoinCodeT, CDocHashT, CMPAIValueT, COutputIndexT, QV2DicT, VString};
use crate::lib::dag::dag::{get_coins_generation_info_via_sql, retrieve_blocks_in_which_a_coin_have_been_produced};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{BlockAlterTreasuryIncome, BlockDPCostBacker, BlockDPCostTreasury, BlockTreasuryLog, CoinImportDataContainer, SingleTrxDPCost, SusInputDetection, ValidityCheck};
use crate::lib::dag::normal_block::rejected_transactions_handler::add_to_rejected_transactions;
use crate::lib::services::dna::dna_handler::get_an_address_shares;
use crate::lib::transactions::basic_transactions::coins::coins_handler::{CoinInfo, CoinDetails};
use crate::lib::transactions::basic_transactions::coins::suspect_trx_handler::{check_doc_validity, get_sus_info_by_block_hash, get_sus_info_by_doc_hash, retrieve_voter_percentages};

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

//old_name_was analyzeATransactionCoins
pub fn analyze_a_transaction_coins(
    block_inspect_container: &mut CoinImportDataContainer,
    block: &Block,
    doc: &Document)
{
    let doc_hash = doc.get_doc_hash();


    // * find DPCost payments, to not importing in UTXOs
    // * and also in case of doublespending removing these incomes from blockTotalTrxFee & treauryTotalTrxFeeIncome

    for inx in doc.get_dpis()
    {
        let an_out = &doc.get_outputs()[*inx as usize];
        if (doc.m_doc_type != constants::document_types::DATA_AND_PROCESS_COST_PAYMENT)
            && (an_out.m_address == block.get_block_backer())
        {
            block_inspect_container.m_a_single_trx_dp_cost.insert(doc_hash.clone(), SingleTrxDPCost {
                m_coin: cutils::pack_coin_code(&doc_hash, *inx),
                m_address: an_out.m_address.clone(),
                m_value: an_out.m_amount,
                m_ref_creation_date: block.m_block_creation_date.clone(),
            });
        }
    }

    block_inspect_container.m_transactions_detection.insert(doc_hash.clone(), "Normal".to_string());


    if constants::OUTPUT_TIME_LOCK_IS_ENABLED
    {

        //    // retrieve latest redeemTimes(if exist)
        //    let { docInputs, docMaxRedeem } = outputTimeLockHandler.getDocInputsAndMaxRedeem({
        //        doc,
        //        docExtInfo
        //    });
        //    let isOutputTimeLockedRelatedDoc = outputTimeLockHandler.isTimeLockRelated(docInputs);
        //    if (isOutputTimeLockedRelatedDoc)
        //        block_inspect_container.oTimeLockedRelatedDocs[doc.hash] = outputTimeLockHandler.isTimeLockRelated(docInputs);
        //    clog.trx.info(`doc(${utils.hash6c(doc.hash)}) is time Locked Related Doc(${isOutputTimeLockedRelatedDoc}) inputs(${docInputs.map(x => iutils.shortCoinRef(x))})`);
        //    console.log(`doc(${utils.hash6c(doc.hash)}) is time Locked Related Doc(${isOutputTimeLockedRelatedDoc}) inputs(${docInputs.map(x => iutils.shortCoinRef(x))})`);
        //    if ((0 < docMaxRedeem) || isOutputTimeLockedRelatedDoc) {
        //        // save redeem info to apply it on right time, and return
        //        let refLocBlocks = get_coins_generation_info_via_sql(docInputs);
        //        for (let refLoc of docInputs) {
        //            block_inspect_container.timeLockedDocs.push({
        //                blockHash: block.blockHash,
        //                docHash: doc.hash,
        //                pureHash: trxHashHandler.getPureHash(doc),
        //                refLoc,
        //                doc,
        //                redeemTime: utils.minutesAfter(docMaxRedeem + iConsts.getCycleByMinutes(), block.creation Date),
        //                docMaxRedeem,
        //                cloneCode: block.cycle,
        //                refCreation Date: refLocBlocks[refLoc].coinGenCreation Date
        //            });
        //        }
        //        let redeemTime = utils.minutesAfter(docMaxRedeem + iConsts.getCycleByMinutes(), block.creation Date);
        //        console.log(`doc(${utils.hash6c(doc.hash)}) created On(${block.creation Date}) MaxRedeem: ${utils.stringify(docMaxRedeem)} redeemTime(${redeemTime})`);
        //        clog.trx.info(`doc(${utils.hash6c(doc.hash)}) created On(${block.creation Date}) MaxRedeem: ${utils.stringify(docMaxRedeem)} redeemTime(${redeemTime})`);

        //        // do nothing more with this document
        //        block_inspect_container.transactionsDetection[doc.hash] = 'timeLocked';
        //        return;
        //    }
    }

    if !block_inspect_container.m_block_is_sus_case
    {
        block_inspect_container.m_can_import_normally = true;
        extract_doc_importable_coins(
            block_inspect_container,
            block,
            doc);

        return;
    }

    // control if there is atleast one susVote for this document?
    let (all_coins_are_valid, mut raw_votes) = get_sus_info_by_doc_hash(&doc_hash);
    if all_coins_are_valid
    {
        dlog(
            &format!("doc({}) of block{}, is not sus (even if block is blockIsSusCase) allCoinsOfDocAreValid",
                     cutils::hash8c(&doc_hash),
                     block.get_block_identifier(),
            ),
            constants::Modules::Trx,
            constants::SecLevel::Info);

        block_inspect_container.m_can_import_normally = true;
        block_inspect_container.m_transactions_detection.insert(doc_hash.clone(), "There is no sus vote for doc".to_string());
        extract_doc_importable_coins(
            block_inspect_container,
            block,
            doc);

        return;
    }

    let (_status, updated_raw_votes) = retrieve_voter_percentages(&mut raw_votes);
    block_inspect_container.m_raw_votes = updated_raw_votes;

    // analyze votes and decide about dobiuos transactions
    check_doc_validity(
        block_inspect_container,
        &doc_hash,
        true,
        true);
//  block_inspect_container.m_transactions_validity_check[doc_hash] = validity;

    block_inspect_container.m_can_import_normally = false;
    if block_inspect_container.m_transactions_validity_check[&doc_hash].m_cloned == "cloned".to_string()
    {
        // the document/transaction is cloned, so can import outputs regularly
        dlog(
            &format!(
                "Recognized a cloned trx{} in block{}",
                doc.get_doc_identifier(),
                block.get_block_identifier()
            ),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        block_inspect_container.m_can_import_normally = true;
    }

    if (block_inspect_container
        .m_transactions_validity_check[&doc_hash]
        .m_sus_vote_res.keys().len() == 1)

        && (block_inspect_container
        .m_transactions_validity_check[&doc_hash]
        .m_sus_vote_res[
        &block_inspect_container
            .m_transactions_validity_check[&doc_hash]
            .m_sus_vote_res.keys().cloned().collect::<VString>()[0]
        ].m_valid == true)
    {
        dlog(
            &format!(
                "Only one sus-input exist and it is valid trx({}) in block({}) ",
                cutils::hash8c(&doc_hash),
                block.get_block_identifier()
            ),
            constants::Modules::Trx,
            constants::SecLevel::Info);
        block_inspect_container.m_can_import_normally = true;
        block_inspect_container.m_transactions_detection.insert(
            doc_hash.clone(),
            "Only one sus-input exist and it is valid".to_string());
    }

    if block_inspect_container.m_can_import_normally
    {
        // so doc can be imported normally
        extract_doc_importable_coins(
            block_inspect_container,
            block,
            doc);
        return;
    }

    // transaction needs special treatment.
    // no out put of this transaction will be imported to UTXOs
    // depends on vote result, transaction inputs can be
    // 1. donated to treasury,
    // 2. restored in UTXOs,
    // 3. denied
    do_sus_treatments(
        block_inspect_container,
        block,
        doc);
    return;
}

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
    // whenever comunity riched 29 percent of voting power, so they will not more hu's voting in susBlock consideration
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

